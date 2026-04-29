use std::fmt::Display;

use axum::extract::{Json, State};
use futures_util::StreamExt;
use log::{info, warn};
use sqlx::{SqliteTransaction, query, query_as};

use crate::ReactorMode;
use crate::advisor::{Advice, AdvisedAction, Advisor, PastAction, PastEvent, Snapshot};
use crate::components::reactor::{IntactReactorSnapshot, ReactorId, ReactorName, ReactorSnapshot};
use crate::components::turbine::{IntactTurbineSnapshot, TurbineSnapshot};
use crate::time::{IngameDateTime, IrlDateTime};
use crate::{Result, app::AppState};

#[derive(Debug, quicktype::Quicktype, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[quicktype(namespace = "server")]
pub(crate) struct Request {
    reactor_name: ReactorName,
    reactor_state: ReactorSnapshot,
    turbine_state: TurbineSnapshot,
    timestamp: IngameDateTime,
}

#[derive(Debug, quicktype::Quicktype, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[quicktype(namespace = "server")]
pub(crate) struct Response {
    reactor_name: ReactorName,
    #[serde(skip_serializing_if = "Option::is_none")]
    advice: Option<Advice>,
}

pub(crate) async fn route(
    app_state: State<AppState<impl Advisor>>,
    req: Json<Request>,
) -> Result<Json<Response>> {
    let State(app_state) = app_state;
    let Json(Request {
        reactor_name,
        reactor_state,
        timestamp: ingame_timestamp,
        turbine_state,
    }) = req;
    let irl_timestamp = IrlDateTime::now();

    let mut txn = app_state.db_pool.begin_with("BEGIN IMMEDIATE").await?;

    info!("processing request for {reactor_name}");

    info!("getting reactor id");
    let reactor_id = get_reactor_id(&mut txn, &reactor_name).await?;

    info!("recording request event");
    let event_id = register_event(&mut txn, reactor_id, irl_timestamp, ingame_timestamp).await?;

    info!("recording system state");
    store_system_state(&mut txn, event_id, &reactor_state, &turbine_state).await?;

    txn.commit().await?;
    let mut txn = app_state.db_pool.begin_with("BEGIN IMMEDIATE").await?;

    let advice = match reactor_state {
        ReactorSnapshot::Destroyed => None,
        ReactorSnapshot::Intact(_) => {
            info!("collecting reactor states");
            let snapshots =
                get_system_snapshots(&mut txn, reactor_id, app_state.snapshot_window_limit).await?;

            info!("getting past actions...");
            let past_actions =
                get_past_actions(&mut txn, reactor_id, app_state.snapshot_window_limit).await?;

            info!("collating history data...");
            let history = collate_history(snapshots, past_actions);

            info!("getting advice...");
            let advice = app_state.advisor.advise(history).await?;

            info!("recording advice...");
            record_advice(&mut txn, event_id, &advice).await?;

            Some(advice)
        }
    };
    txn.commit().await?;

    info!("returning response");
    Ok(Json(Response {
        reactor_name,
        advice,
    }))
}

async fn get_reactor_id(txn: &mut SqliteTransaction<'_>, name: &ReactorName) -> Result<ReactorId> {
    struct Row {
        id: i64,
    }
    let id_query = query_as!(
        Row,
        "
            SELECT id
            FROM reactor
            WHERE name = ?
        ",
        name,
    );
    let id = id_query.fetch_optional(&mut **txn).await?;
    if let Some(Row { id }) = id {
        return Ok(ReactorId::from(id));
    }

    let name_insertion_query = query!(
        "
            INSERT INTO reactor (name)
            VALUES (?)
        ",
        name
    );
    let info = name_insertion_query.execute(&mut **txn).await?;
    Ok(ReactorId::from(info.last_insert_rowid()))
}

async fn get_system_snapshots(
    txn: &mut SqliteTransaction<'_>,
    reactor_id: ReactorId,
    snapshot_window_limit: u16,
) -> Result<Vec<Snapshot>> {
    struct Row {
        id: EventId,
        mode: Option<i64>,
        temperature: Option<f64>,
        coolant_filled: Option<f64>,
        heated_coolant_filled: Option<f64>,
        fuel_filled: Option<f64>,
        waste_filled: Option<f64>,
        actual_burn_rate: Option<f64>,
        target_burn_rate: Option<i64>,
        damage_percent: Option<f64>,
        heating_rate: Option<f64>,
        boil_efficiency: Option<f64>,
        ingame_timestamp: IngameDateTime,
        stored_kinetic_energy: Option<f64>,
        energy_production_rate: Option<f64>,
    }
    let snapshots_query = query_as!(
        Row,
        "
            SELECT
                event.id,
                snapshot.mode,
                snapshot.temperature,
                snapshot.coolant_filled,
                snapshot.heated_coolant_filled,
                snapshot.fuel_filled,
                snapshot.waste_filled,
                snapshot.actual_burn_rate,
                snapshot.target_burn_rate,
                snapshot.damage_percent,
                snapshot.heating_rate,
                snapshot.boil_efficiency,
                snapshot.stored_kinetic_energy,
                snapshot.energy_production_rate,
                event.ingame_timestamp
            FROM snapshot
            JOIN event
            ON snapshot.event_id = event.id
            WHERE event.reactor_id = ?
            ORDER BY event.irl_timestamp DESC
            LIMIT ?
        ",
        reactor_id,
        snapshot_window_limit,
    );

    let mut ret = Vec::new();
    let mut snapshot_rows = snapshots_query.fetch(&mut **txn);
    while let Some(row) = snapshot_rows.next().await {
        let snapshot = match row? {
            Row {
                id: _,
                mode: Some(mode),
                temperature: Some(temperature),
                coolant_filled: Some(coolant_filled),
                heated_coolant_filled: Some(heated_coolant_filled),
                fuel_filled: Some(fuel_filled),
                waste_filled: Some(waste_filled),
                actual_burn_rate: Some(actual_burn_rate),
                target_burn_rate: Some(target_burn_rate),
                damage_percent: Some(damage_percent),
                heating_rate: Some(heating_rate),
                boil_efficiency: Some(boil_efficiency),
                stored_kinetic_energy: Some(stored_kinetic_energy),
                energy_production_rate: Some(energy_production_rate),
                ingame_timestamp,
            } => {
                let mode = ReactorMode::try_from(mode)?;
                let actual_burn_rate = actual_burn_rate.into();
                let target_burn_rate = target_burn_rate.into();
                Snapshot {
                    timestamp: ingame_timestamp,
                    reactor: ReactorSnapshot::Intact(IntactReactorSnapshot {
                        mode,
                        temperature,
                        coolant_filled,
                        heated_coolant_filled,
                        fuel_filled,
                        waste_filled,
                        actual_burn_rate,
                        target_burn_rate,
                        damage_percent,
                        heating_rate,
                        boil_efficiency,
                    }),
                    turbine: TurbineSnapshot::Intact(IntactTurbineSnapshot {
                        stored_kinetic_energy,
                        energy_production_rate,
                    }),
                }
            }
            Row {
                id: _,
                mode: None,
                temperature: None,
                coolant_filled: None,
                heated_coolant_filled: None,
                fuel_filled: None,
                waste_filled: None,
                actual_burn_rate: None,
                target_burn_rate: None,
                damage_percent: None,
                heating_rate: None,
                boil_efficiency: None,
                stored_kinetic_energy: None,
                energy_production_rate: None,
                ingame_timestamp: timestamp,
            } => Snapshot {
                timestamp,
                reactor: ReactorSnapshot::Destroyed,
                turbine: TurbineSnapshot::Destroyed,
            },
            Row { id, .. } => {
                warn!("event {id} has an invalid associated reactor state");
                continue;
            }
        };
        ret.push(snapshot);
    }
    Ok(ret)
}

async fn get_past_actions(
    txn: &mut SqliteTransaction<'_>,
    reactor_id: ReactorId,
    snapshot_window_limit: u16,
) -> Result<Vec<PastAction>> {
    struct Row {
        id: i64,
        timestamp: IngameDateTime,
        action: i64,
        new_target_burn_rate: Option<i64>,
    }
    let past_actions_query = query_as!(
        Row,
        "
            SELECT
                event.id,
                event.ingame_timestamp AS timestamp,
                advice.action,
                advice.new_target_burn_rate
            FROM advice
            JOIN event
            ON advice.event_id = event.id
            WHERE event.reactor_id = ?
            ORDER BY event.irl_timestamp DESC
            LIMIT ?
        ",
        reactor_id,
        snapshot_window_limit,
    );

    let mut ret = Vec::with_capacity(snapshot_window_limit.into());
    let mut rows = past_actions_query.fetch(&mut **txn);
    while let Some(row) = rows.next().await {
        let past_action = match row? {
            Row {
                id: _,
                timestamp,
                action: 0,
                new_target_burn_rate: None,
            } => PastAction {
                timestamp,
                action: AdvisedAction::NoAction,
            },
            Row {
                id: _,
                timestamp,
                action: 1,
                new_target_burn_rate: None,
            } => PastAction {
                timestamp,
                action: AdvisedAction::Scram,
            },
            Row {
                id: _,
                timestamp,
                action: 2,
                new_target_burn_rate: Some(new_target_burn_rate),
            } => {
                let new_target_burn_rate = new_target_burn_rate.into();
                PastAction {
                    timestamp,
                    action: AdvisedAction::SetBurnRate {
                        new_target_burn_rate,
                    },
                }
            }
            Row { id, .. } => {
                warn!("event {id} has invalid associated advice");
                continue;
            }
        };
        ret.push(past_action);
    }
    Ok(ret)
}

fn collate_history(snapshots: Vec<Snapshot>, past_actions: Vec<PastAction>) -> Vec<PastEvent> {
    let mut ret: Vec<_> = snapshots
        .into_iter()
        .map(PastEvent::Snapshot)
        .chain(past_actions.into_iter().map(PastEvent::Action))
        .collect();
    ret.sort_by(|a, b| a.timestamp().cmp(b.timestamp()));
    ret
}

#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub(crate) struct EventId(i64);

impl From<i64> for EventId {
    fn from(inner: i64) -> Self {
        Self(inner)
    }
}

impl Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        inner.fmt(f)
    }
}

async fn register_event(
    txn: &mut SqliteTransaction<'_>,
    reactor_id: ReactorId,
    irl_timestamp: IrlDateTime,
    ingame_timestamp: IngameDateTime,
) -> Result<EventId> {
    let irl_timestamp = irl_timestamp.unix_timestamp();
    let event_insertion_query = query!(
        "
            INSERT INTO event (reactor_id, irl_timestamp, ingame_timestamp)
            VALUES (?, ?, ?)
        ",
        reactor_id,
        irl_timestamp,
        ingame_timestamp,
    );
    let id = event_insertion_query
        .execute(&mut **txn)
        .await?
        .last_insert_rowid();
    Ok(EventId(id))
}

async fn store_system_state(
    txn: &mut SqliteTransaction<'_>,
    event_id: EventId,
    reactor_snapshot: &ReactorSnapshot,
    turbine_snapshot: &TurbineSnapshot,
) -> Result<()> {
    let intact = matches!(
        (reactor_snapshot, turbine_snapshot),
        (ReactorSnapshot::Intact(_), TurbineSnapshot::Intact(_))
    );
    match (reactor_snapshot, turbine_snapshot) {
        (ReactorSnapshot::Destroyed, _) | (_, TurbineSnapshot::Destroyed) => {
            let state_insertion_query = query!(
                "
                    INSERT INTO snapshot (event_id, intact) VALUES (?, ?)
                ",
                event_id,
                intact,
            );
            state_insertion_query.execute(&mut **txn).await?;
        }
        (
            ReactorSnapshot::Intact(intact_reactor_state),
            TurbineSnapshot::Intact(intact_turbine_state),
        ) => {
            let IntactReactorSnapshot {
                mode,
                temperature,
                coolant_filled,
                heated_coolant_filled,
                fuel_filled,
                waste_filled,
                actual_burn_rate,
                target_burn_rate,
                damage_percent,
                heating_rate,
                boil_efficiency,
            } = intact_reactor_state;
            let IntactTurbineSnapshot {
                stored_kinetic_energy,
                energy_production_rate,
            } = intact_turbine_state;
            let state_insertion_query = query!(
                "
                    INSERT INTO snapshot (
                        event_id,
                        intact,
                        mode,
                        temperature,
                        coolant_filled,
                        heated_coolant_filled,
                        fuel_filled,
                        waste_filled,
                        actual_burn_rate,
                        target_burn_rate,
                        damage_percent,
                        heating_rate,
                        boil_efficiency,
                        stored_kinetic_energy,
                        energy_production_rate
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ",
                event_id,
                intact,
                mode,
                temperature,
                coolant_filled,
                heated_coolant_filled,
                fuel_filled,
                waste_filled,
                actual_burn_rate,
                target_burn_rate,
                damage_percent,
                heating_rate,
                boil_efficiency,
                stored_kinetic_energy,
                energy_production_rate,
            );
            state_insertion_query.execute(&mut **txn).await?;
        }
    }
    Ok(())
}

async fn record_advice(
    txn: &mut SqliteTransaction<'_>,
    event_id: EventId,
    advice: &Advice,
) -> Result<()> {
    let Advice { action, reasoning } = &advice;
    let (advised_action_repr, new_target_burn_rate) = match action {
        AdvisedAction::NoAction => (0, None),
        AdvisedAction::Scram => (1, None),
        AdvisedAction::SetBurnRate {
            new_target_burn_rate,
        } => (2, Some(new_target_burn_rate)),
    };
    let advice_insertion_query = query!(
        "
            INSERT INTO advice (event_id, action, reasoning, new_target_burn_rate)
            VALUES (?, ?, ?, ?)
        ",
        event_id,
        advised_action_repr,
        reasoning,
        new_target_burn_rate,
    );
    advice_insertion_query.execute(&mut **txn).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use quicktype::Quicktype;

    use super::*;

    #[test]
    fn test_request_quicktype_def() {
        assert_eq!("server.Request", Request::type_name().to_string());
        assert_json_snapshot!(Request::type_spec().to_string());
    }

    #[test]
    fn test_response_quicktype_def() {
        assert_eq!("server.Response", Response::type_name().to_string());
        assert_json_snapshot!(Response::type_spec().to_string());
    }
}

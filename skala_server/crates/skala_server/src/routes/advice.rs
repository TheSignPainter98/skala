use axum::extract::{Json, State};
use log::info;
use sqlx::{SqliteTransaction, query, query_as};
use time::UtcDateTime;

use crate::advisor::{Advice, AdvisedAction, Advisor};
use crate::reactor::{IntactReactorState, ReactorId, ReactorName, ReactorState};
use crate::{Result, app::AppState};

#[derive(Debug, quicktype::Quicktype, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[quicktype(namespace = "server")]
pub(crate) struct Request {
    reactor_name: ReactorName,
    reactor_state: ReactorState,
    timestamp: IngameDateTime,
}

#[derive(Debug, quicktype::Quicktype, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
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
    }) = req;
    let mut txn = app_state.db_pool.begin_with("BEGIN IMMEDIATE").await?;

    info!("processing request for {reactor_name}");

    info!("getting reactor id");
    let reactor_id = get_reactor_id(&mut txn, &reactor_name).await?;

    info!("recording request event");
    let irl_timestamp = UtcDateTime::now();
    let event_id = register_event(&mut txn, reactor_id, irl_timestamp, ingame_timestamp).await?;

    info!("recording reactor state");
    store_reactor_state(&mut txn, event_id, &reactor_state).await?;

    // TODO(kcza): fetch reactor state history, pass as context to the advisor.

    let advice = match reactor_state {
        ReactorState::Destroyed => None,
        ReactorState::Intact(intact_reactor_state) => {
            info!("getting advice...");
            let advice = app_state.advisor.advise(intact_reactor_state).await?;

            info!("recording advice...");
            record_advice(&mut txn, event_id, &advice).await?;

            Some(advice)
        }
    };

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

#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub(crate) struct EventId(i64);

async fn register_event(
    txn: &mut SqliteTransaction<'_>,
    reactor_id: ReactorId,
    irl_timestamp: UtcDateTime,
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

#[derive(Debug, serde::Deserialize, sqlx::Type)]
#[sqlx(transparent)]
struct IngameDateTime(String);

async fn store_reactor_state(
    txn: &mut SqliteTransaction<'_>,
    event_id: EventId,
    reactor_state: &ReactorState,
) -> Result<()> {
    let intact = matches!(reactor_state, ReactorState::Intact(_));
    match reactor_state {
        ReactorState::Destroyed => {
            let state_insertion_query = query!(
                "
                    INSERT INTO reactor_state (event_id, intact) VALUES (?, ?)
                ",
                event_id,
                intact,
            );
            state_insertion_query.execute(&mut **txn).await?;
        }
        ReactorState::Intact(intact_reactor_state) => {
            let IntactReactorState {
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
            let state_insertion_query = query!(
                "
                    INSERT INTO reactor_state (
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
                        boil_efficiency
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    let advised_action_repr = match action {
        AdvisedAction::NoAction => 0,
        AdvisedAction::Scram => 1,
    };
    let advice_insertion_query = query!(
        "
            INSERT INTO advice (event_id, action, reasoning)
            VALUES (?, ?, ?)
        ",
        event_id,
        advised_action_repr,
        reasoning,
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
    fn test_foo() {
        // TODO(kcza): plumb the namespace!
        assert_eq!("server.Response", Response::type_name().to_string());
        assert_json_snapshot!(Response::type_spec().to_string());
    }
}

use axum::extract::{Json, State};
use log::{error, info};
use sqlx::{query, query_as};

use crate::advisor::{Advice, AdvisedAction, Advisor};
use crate::reactor::{ReactorName, ReactorState};
use crate::{Result, app::AppState};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Request {
    reactor_name: ReactorName,
    reactor_state: ReactorState,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct Response {
    reactor_name: ReactorName,
    advice: Advice,
}

pub(crate) async fn route(
    app_state: State<AppState<impl Advisor>>,
    req: Json<Request>,
) -> Result<Json<Response>> {
    let State(app_state) = app_state;
    let Json(Request {
        reactor_name,
        reactor_state,
    }) = req;

    info!("processing request for {reactor_name}");

    info!("getting reactor id");
    let reactor_id = {
        struct Row {
            id: i64,
        }
        let reactor_id_query = query_as!(
            Row,
            "
                SELECT id
                FROM reactor
                WHERE name = ?
            ",
            reactor_name,
        );
        let reactor_id = reactor_id_query.fetch_optional(&app_state.db_pool).await?;
        match reactor_id {
            Some(Row { id }) => id,
            None => {
                let reactor_name_insertion_query = query!(
                    "
                        INSERT INTO reactor (name)
                        VALUES (?)
                    ",
                    reactor_name
                );
                let info = reactor_name_insertion_query
                    .execute(&app_state.db_pool)
                    .await?;
                info.last_insert_rowid()
            }
        }
    };

    info!("recording reactor state");
    let ReactorState {
        status,
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
    } = reactor_state;
    let initial_advice_insertion_query = query!(
        "
            INSERT INTO advice (
                reactor_id,
                reactor_status,
                reactor_temperature,
                reactor_coolant_filled,
                reactor_heated_coolant_filled,
                reactor_fuel_filled,
                reactor_waste_filled,
                reactor_actual_burn_rate,
                reactor_target_burn_rate,
                reactor_damage_percent,
                reactor_heating_rate,
                reactor_boil_efficiency
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        reactor_id,
        status,
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
    let info = initial_advice_insertion_query
        .execute(&app_state.db_pool)
        .await?;
    let advice_id = info.last_insert_rowid();

    info!("getting advice...");
    // let advice_result = get_advice(&app_state, reactor_state).await;
    let advice_result = app_state.advisor.advise(reactor_state).await;
    let advice = match advice_result {
        Ok(advice) => {
            info!("recording advice");
            let Advice { action, reasoning } = &advice;
            let advised_action_repr = match action {
                AdvisedAction::NoAction => 0,
                AdvisedAction::Scram => 1,
            };
            let advice_insertion_query = query!(
                "
                    UPDATE advice
                    SET
                        status = 1,
                        advised_action = ?,
                        advised_action_reasoning = ?
                    WHERE id = ?
                ",
                advised_action_repr,
                reasoning,
                advice_id,
            );
            advice_insertion_query.execute(&app_state.db_pool).await?;
            advice
        }
        Err(err) => {
            error!("could not get advice: {err}");
            let advice_insertion_query = query!(
                "
                    UPDATE advice
                    SET status = 2
                    WHERE id = ?
                ",
                advice_id,
            );
            advice_insertion_query.execute(&app_state.db_pool).await?;
            return Err(err);
        }
    };

    info!("returning response");
    Ok(Json(Response {
        reactor_name,
        advice,
    }))
}

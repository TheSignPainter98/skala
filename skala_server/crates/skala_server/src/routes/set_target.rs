use axum::extract::{Query, State};
use log::info;
use sqlx::query;

use crate::advisor::Advisor;
use crate::components::reactor::ReactorName;
use crate::routes::common::{get_reactor_id, register_event};
use crate::time::{IngameDateTime, IrlDateTime};
use crate::{Result, app::AppState};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Request {
    reactor_name: ReactorName,
    rate: f64,
}

pub(crate) async fn route(
    app_state: State<AppState<impl Advisor>>,
    req: Query<Request>,
) -> Result<String> {
    let State(app_state) = app_state;
    let Query(Request { reactor_name, rate }) = req;

    let mut txn = app_state.db_pool.begin_with("BEGIN IMMEDIATE").await?;

    info!("setting production target for {reactor_name}");
    let reactor_id = get_reactor_id(&mut txn, &reactor_name).await?;
    let event_id = register_event(
        &mut txn,
        reactor_id,
        IrlDateTime::now(),
        IngameDateTime::from("(IRL)".to_owned()),
    )
    .await?;

    let target_insertion_query = query!(
        "
            INSERT INTO production_target (event_id, rate)
            VALUES (?, ?)
        ",
        event_id,
        rate,
    );
    target_insertion_query.execute(&mut *txn).await?;

    txn.commit().await?;

    Ok(format!("Target energy production rate set to {rate}"))
}

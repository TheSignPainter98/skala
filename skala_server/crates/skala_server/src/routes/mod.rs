mod advice;

use axum::Router;
use axum::routing::{get, post};

use crate::advisor::Advisor;
use crate::app::AppState;

pub(crate) fn register<A: Advisor + 'static>(app: Router<AppState<A>>) -> Router<AppState<A>> {
    app.route("/", get(|| async { ">:3" }))
        .route("/advice", post(self::advice::route))
}

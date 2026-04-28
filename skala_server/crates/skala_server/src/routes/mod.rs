mod advice;
mod health_check;
mod info;

use axum::Router;
use axum::routing::{get, post};

use crate::advisor::Advisor;
use crate::app::AppState;

pub(crate) fn register<A: Advisor + 'static>(app: Router<AppState<A>>) -> Router<AppState<A>> {
    app.route("/", get(self::health_check::route))
        .route("/advice", post(self::advice::route))
        .route("/info", get(self::info::route))
}

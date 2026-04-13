mod advice;

use axum::Router;
use axum::routing::{get, post};

use crate::app::AppState;

pub(crate) fn register(app: Router<AppState>) -> Router<AppState> {
    app.route("/", get(|| async { ">:3" }))
        .route("/advice", post(self::advice::route))
}

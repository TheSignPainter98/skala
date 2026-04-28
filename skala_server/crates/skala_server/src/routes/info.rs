use axum::Json;

#[derive(Debug, serde::Serialize)]
pub(crate) struct Response {
    version: &'static str,
}

pub(crate) async fn route() -> Json<Response> {
    Json(Response {
        version: env!("CARGO_PKG_VERSION"),
    })
}

use axum::body::{self, Body};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use log::error;

pub(crate) async fn log_error_response(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let response = next.run(req).await;
    let status = response.status();

    if status.is_success() {
        return response;
    }

    let (parts, body) = response.into_parts();
    let bytes = match body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => {
            error!("HTTP {status} response for {method} {uri} body could not be read: {err}");
            return Response::from_parts(parts, Body::empty());
        }
    };

    let response_body = String::from_utf8_lossy(&bytes);
    error!("HTTP {status} response for {method} {uri}: {response_body}");
    Response::from_parts(parts, Body::from(bytes))
}

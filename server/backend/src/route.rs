use axum::Router;
use axum::extract::OriginalUri;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;

use tower_http::compression::CompressionLayer;

use tracing::warn;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", get(home))
    .fallback(error_404)
    .layer(CompressionLayer::new().br(true).gzip(true))
}

async fn home() -> impl IntoResponse {
  (StatusCode::OK, "klaxon")
}

async fn error_404(OriginalUri(uri): OriginalUri) -> impl IntoResponse {
  warn!("unable to find resource: {uri}");
  (StatusCode::NOT_FOUND, "not found")
}

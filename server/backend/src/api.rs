use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;

use serde_json::json;

use tower_http::compression::CompressionLayer;

use tracing::error;

use crate::cluster;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/api/health", get(health::check))
    .route("/api/health/nodes", get(health::nodes))
    .route("/api/health/pods", get(health::pods))
    .route("/api/health/summary", get(health::summary))
    .layer(CompressionLayer::new().br(true).gzip(true))
}

#[derive(thiserror::Error, Debug)]
enum Error {
  #[error("cluster error - {0}")]
  Cluster(#[from] cluster::Error),
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    match &self {
      Error::Cluster(_) => {
        error!("internal error - {}", self);
        let body = Json(json!({ "error": self.to_string() }));
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
      }
    }
  }
}

mod health {
  use super::*;

  pub async fn check(State(state): State<AppState>) -> Result<Response, Error> {
    let summary = cluster::summary(&state.client).await?;
    let status = if summary.healthy {
      StatusCode::OK
    } else {
      StatusCode::SERVICE_UNAVAILABLE
    };
    Ok((status, Json(json!({ "healthy": summary.healthy }))).into_response())
  }

  pub async fn nodes(
    State(state): State<AppState>,
  ) -> Result<Json<Vec<cluster::NodeStatus>>, Error> {
    let nodes = cluster::list_nodes(&state.client).await?;
    Ok(Json(nodes))
  }

  pub async fn pods(State(state): State<AppState>) -> Result<Json<Vec<cluster::PodStatus>>, Error> {
    let pods = cluster::list_pods(&state.client).await?;
    Ok(Json(pods))
  }

  pub async fn summary(
    State(state): State<AppState>,
  ) -> Result<Json<cluster::ClusterSummary>, Error> {
    let summary = cluster::summary(&state.client).await?;
    Ok(Json(summary))
  }
}

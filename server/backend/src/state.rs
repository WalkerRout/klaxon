use kube::Client;

use tracing::{info, warn};

use crate::cluster;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("cluster error - {0}")]
  Cluster(#[from] cluster::Error),
}

#[derive(Clone)]
pub struct AppState {
  pub client: Client,
}

impl AppState {
  pub async fn new() -> Result<Self, Error> {
    info!("creating kubernetes client...");
    let client = cluster::connect().await?;

    info!("attempting cluster reachability check...");
    match cluster::ping(&client).await {
      Ok(()) => info!("cluster reachable"),
      Err(e) => warn!("cluster check skipped (api may be unavailable) - {}", e),
    }

    Ok(Self { client })
  }
}

use chrono::{DateTime, Utc};

use k8s_openapi::api::core::v1::{Node, Pod};

use kube::Client;
use kube::api::{Api, ListParams};

use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("kube error - {0}")]
  Kube(#[from] kube::Error),
}

pub async fn connect() -> Result<Client, Error> {
  let client = Client::try_default().await?;
  Ok(client)
}

pub async fn ping(client: &Client) -> Result<(), Error> {
  let api: Api<Node> = Api::all(client.clone());
  api.list(&ListParams::default().limit(1)).await?;
  Ok(())
}

// nodes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
  pub name: String,
  pub ready: bool,
  pub reason: Option<String>,
  pub last_heartbeat: Option<DateTime<Utc>>,
}

pub async fn list_nodes(client: &Client) -> Result<Vec<NodeStatus>, Error> {
  let api: Api<Node> = Api::all(client.clone());
  let nodes = api.list(&ListParams::default()).await?;

  let statuses = nodes
    .into_iter()
    .map(|node| {
      let name = node.metadata.name.unwrap_or_default();
      let condition = node
        .status
        .as_ref()
        .and_then(|s| s.conditions.as_ref())
        .and_then(|cs| cs.iter().find(|c| c.type_ == "Ready"))
        .cloned();

      let ready = condition
        .as_ref()
        .map(|c| c.status == "True")
        .unwrap_or(false);
      let reason = condition.as_ref().and_then(|c| c.reason.clone());
      let last_heartbeat = condition
        .as_ref()
        .and_then(|c| c.last_heartbeat_time.as_ref().map(|t| t.0));

      NodeStatus {
        name,
        ready,
        reason,
        last_heartbeat,
      }
    })
    .collect();

  Ok(statuses)
}

// pods

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodStatus {
  pub namespace: String,
  pub name: String,
  pub phase: String,
  pub ready: bool,
  pub restart_count: i32,
}

pub async fn list_pods(client: &Client) -> Result<Vec<PodStatus>, Error> {
  let api: Api<Pod> = Api::all(client.clone());
  let pods = api.list(&ListParams::default()).await?;

  let statuses = pods
    .into_iter()
    .map(|pod| {
      let namespace = pod.metadata.namespace.clone().unwrap_or_default();
      let name = pod.metadata.name.clone().unwrap_or_default();
      let phase = pod
        .status
        .as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());
      let ready = pod
        .status
        .as_ref()
        .and_then(|s| s.container_statuses.as_ref())
        .map(|cs| cs.iter().all(|c| c.ready))
        .unwrap_or(false);
      let restart_count = pod
        .status
        .as_ref()
        .and_then(|s| s.container_statuses.as_ref())
        .map(|cs| cs.iter().map(|c| c.restart_count).sum())
        .unwrap_or(0);

      PodStatus {
        namespace,
        name,
        phase,
        ready,
        restart_count,
      }
    })
    .collect();

  Ok(statuses)
}

// summary

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSummary {
  pub healthy: bool,
  pub nodes: Vec<NodeStatus>,
  pub pods: Vec<PodStatus>,
}

pub async fn summary(client: &Client) -> Result<ClusterSummary, Error> {
  let nodes = list_nodes(client).await?;
  let pods = list_pods(client).await?;

  let nodes_ok = nodes.iter().all(|n| n.ready);
  let pods_ok = pods
    .iter()
    .all(|p| p.phase == "Running" || p.phase == "Succeeded");

  Ok(ClusterSummary {
    healthy: nodes_ok && pods_ok,
    nodes,
    pods,
  })
}

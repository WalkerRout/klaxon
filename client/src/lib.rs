#![no_std]

extern crate alloc;

use core::marker::PhantomData;

use alloc::string::String;
use alloc::vec::Vec;

use chrono::{DateTime, Utc};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Error {
  Decode(serde_json::Error),
}

impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Self {
    Self::Decode(e)
  }
}

pub struct Endpoint<T> {
  pub path: &'static str,
  _t: PhantomData<fn() -> T>,
}

impl<T> Endpoint<T> {
  const fn new(path: &'static str) -> Self {
    Self {
      path,
      _t: PhantomData,
    }
  }

  pub fn url(&self, base: &str) -> String {
    let mut url = String::with_capacity(base.len() + self.path.len());
    url.push_str(base);
    url.push_str(self.path);
    url
  }
}

impl<T> Endpoint<T>
where
  T: DeserializeOwned,
{
  pub fn parse(&self, body: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(body).map_err(Error::from)
  }
}

// health

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
  pub healthy: bool,
}

pub const HEALTH: Endpoint<Health> = Endpoint::new("/api/health");

// nodes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
  pub name: String,
  pub ready: bool,
  pub reason: Option<String>,
  pub last_heartbeat: Option<DateTime<Utc>>,
}

pub const NODES: Endpoint<Vec<NodeStatus>> = Endpoint::new("/api/health/nodes");

// pods

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodStatus {
  pub namespace: String,
  pub name: String,
  pub phase: String,
  pub ready: bool,
  pub restart_count: i32,
}

pub const PODS: Endpoint<Vec<PodStatus>> = Endpoint::new("/api/health/pods");

// summary

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSummary {
  pub healthy: bool,
  pub nodes: Vec<NodeStatus>,
  pub pods: Vec<PodStatus>,
}

pub const SUMMARY: Endpoint<ClusterSummary> = Endpoint::new("/api/health/summary");

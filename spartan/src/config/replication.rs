use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Default amount of seconds between replication jobs
const fn default_replication_timer() -> u64 {
    180
}

#[derive(Serialize, Deserialize)]
pub struct Primary {
    pub destination: Vec<SocketAddr>,

    #[serde(default = "default_replication_timer")]
    pub replication_timer: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Replica {
    pub host: SocketAddr,
}

#[derive(Serialize, Deserialize)]
pub enum Replication {
    Primary(Primary),
    Replica(Replica),
}

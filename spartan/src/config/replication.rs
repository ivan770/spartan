use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Default amount of seconds between replication jobs
const fn default_replication_timer() -> u64 {
    180
}

/// Default amount of seconds between replica command restart tries
const fn default_replica_try_timer() -> u64 {
    5
}

#[derive(Serialize, Deserialize)]
pub struct Primary {
    pub destination: Box<[SocketAddr]>,

    #[serde(default = "default_replication_timer")]
    pub replication_timer: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Replica {
    pub host: SocketAddr,

    #[serde(default = "default_replica_try_timer")]
    pub try_timer: u64,
}

#[derive(Serialize, Deserialize)]
pub enum Replication {
    Primary(Primary),
    Replica(Replica),
}

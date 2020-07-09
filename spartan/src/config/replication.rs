use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
pub struct Primary {
    destination: Vec<SocketAddr>,
}

#[derive(Serialize, Deserialize)]
pub struct Replica {
    host: SocketAddr,
}

#[derive(Serialize, Deserialize)]
pub enum Replication {
    Primary(Primary),
    Replica(Replica),
}

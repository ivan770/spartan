use actix_raft::AppData;
use serde::{Deserialize, Serialize};
use spartan_lib::core::{message::Message, payload::Identifiable};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub queue: String,
    pub action: Action,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Action {
    Push(Message),
    Pop,
    Requeue(<Message as Identifiable>::Id),
    Delete(<Message as Identifiable>::Id),
    Gc,
    Clear,
}

impl AppData for LogEntry {}

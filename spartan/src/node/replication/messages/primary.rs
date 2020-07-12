use crate::node::replication::event::Event;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub enum Request<'a> {
    Ping,
    AskIndex,
    SendRange(Box<[(&'a u64, &'a Event)]>),
}

#[derive(Deserialize)]
pub enum Response {
    Pong,
    RecvIndex(Vec<(Box<str>, u64)>),
    RecvRange,
}

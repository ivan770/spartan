use crate::node::replication::event::Event;
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Serialize, Deserialize)]
pub enum PrimaryRequest<'a> {
    Ping,
    AskIndex,
    SendRange(
        Cow<'a, str>,
        Box<[(MaybeOwned<'a, u64>, MaybeOwned<'a, Event>)]>,
    ),
}

#[derive(Serialize, Deserialize)]
pub enum ReplicaRequest<'a> {
    Pong,
    RecvIndex(Box<[(Box<str>, u64)]>),
    RecvRange,
    QueueNotFound(Cow<'a, str>),
}

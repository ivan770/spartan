use crate::node::replication::event::Event;
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub enum PrimaryRequest<'a> {
    Ping,
    AskIndex,
    SendRange(
        Cow<'a, str>,
        Box<[(MaybeOwned<'a, u64>, MaybeOwned<'a, Event>)]>,
    ),
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub enum ReplicaRequest<'a> {
    Pong,
    RecvIndex(Box<[(Box<str>, u64)]>),
    RecvRange,
    QueueNotFound(Cow<'a, str>),
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub enum Request<'a> {
    Primary(PrimaryRequest<'a>),
    Replica(ReplicaRequest<'a>),
}

#[cfg(test)]
impl PartialEq for Request<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Request::Primary(a), Request::Primary(b)) => {
                match (a, b) {
                    (PrimaryRequest::Ping, PrimaryRequest::Ping) => true,
                    _ => false
                }
            }
            _ => false
        }
    }
}

impl<'a> Request<'a> {
    pub fn get_primary(self) -> Option<PrimaryRequest<'a>> {
        match self {
            Request::Primary(r) => Some(r),
            Request::Replica(_) => None,
        }
    }

    pub fn get_replica(self) -> Option<ReplicaRequest<'a>> {
        match self {
            Request::Replica(r) => Some(r),
            Request::Primary(_) => None,
        }
    }
}

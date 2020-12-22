use std::borrow::Cow;

use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};

use crate::node::event::Event;

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum PrimaryRequest<'c, 'r> {
    Ping,
    AskIndex,
    SendRange(
        Cow<'c, str>,
        Box<[(MaybeOwned<'r, u64>, MaybeOwned<'r, Event<'r>>)]>,
    ),
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum ReplicaRequest<'c> {
    Pong(Cow<'static, str>),
    RecvIndex(Box<[(Cow<'c, str>, u64)]>),
    RecvRange,
    QueueNotFound(Cow<'c, str>),
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum Request<'c, 'r> {
    Primary(PrimaryRequest<'c, 'r>),
    Replica(ReplicaRequest<'c>),
}

impl<'c, 'r> Request<'c, 'r> {
    pub fn get_primary(self) -> Option<PrimaryRequest<'c, 'r>> {
        match self {
            Request::Primary(r) => Some(r),
            Request::Replica(_) => None,
        }
    }

    pub fn get_replica(self) -> Option<ReplicaRequest<'c>> {
        match self {
            Request::Replica(r) => Some(r),
            Request::Primary(_) => None,
        }
    }
}

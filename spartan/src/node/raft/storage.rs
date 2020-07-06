use super::{entry::LogEntry, error::ReplicationError};
use actix::{Actor, Context, Handler};
use actix_raft::{
    messages::Entry,
    storage::{AppendEntryToLog, GetLogEntries},
    AppData,
};
use std::{collections::BTreeMap, ops::Bound::Included, sync::Arc};
use crate::node::Manager;

impl Actor for Manager<'static> {
    type Context = Context<Self>;
}

// impl RaftStorage<LogEntry, ReplicationResponse, ReplicationError> for Manager<'static> {
//     type Actor = Self;
//     type Context = Context<Self>;
// }

impl Handler<GetLogEntries<LogEntry, ReplicationError>> for Manager<'static> {
    type Result = Result<Vec<Entry<LogEntry>>, ReplicationError>;

    fn handle(
        &mut self,
        msg: GetLogEntries<LogEntry, ReplicationError>,
        _: &mut Self::Context,
    ) -> Self::Result {
        Ok(self
            .replication_log
            .range((Included(msg.start), Included(msg.stop)))
            .map(|(_, value)| value)
            .cloned()
            .collect())
    }
}

impl Handler<AppendEntryToLog<LogEntry, ReplicationError>> for Manager<'static> {
    type Result = Result<(), ReplicationError>;

    fn handle(
        &mut self,
        msg: AppendEntryToLog<LogEntry, ReplicationError>,
        _: &mut Self::Context,
    ) -> Self::Result {
        let entry = msg.entry;

        self.replication_log.insert(
            entry.index,
            Arc::try_unwrap(entry).map_err(|_| ReplicationError::ExistingEntryReferences)?,
        );

        Ok(())
    }
}

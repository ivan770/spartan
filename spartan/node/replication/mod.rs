//! # General schema
//!
//! ```
//! +-----------+          +---------------+
//! |           |          |               |
//! | Ping      +----><----+ Pong          |
//! |           |          |               |
//! | AskIndex  +----><----+ RecvIndex     |
//! |           |          |               |
//! | SendRange +----><--+-+ RecvRange     |
//! |           |        ^ |               |
//! +-----------+        +-+ QueueNotFound |
//!                        |               |
//!                        +---------------+
//! ```
//!
//! This schema describes order, in which messages are sent between primary and replica nodes.
//!
//! Primary is on the left and always sends messages first, while replica node is on the right, and only responds to them.
//!
//! # Messages
//!
//! ## Ping and Pong
//!
//! These messages are used to check, if all replicas are still online, and if their and primary node versions are same.
//!
//! While just the TCP ping can be used, it's better to check if replica has the same replication protocol as primary.
//!
//! ## `AskIndex` and `RecvIndex`
//!
//! After we check replica health, we need to ask about last received index of each queue.
//!
//! Replica sends back `RecvIndex` message, that contains data about available queues and their indexes.
//!
//! ## `SendRange` and `RecvRange` (and probably `QueueNotFound`)
//!
//! With `queue = index` array of each replica available, primary node takes new events from event log of each queue, and sends it to replica.
//!
//! Replica responds with either `RecvRange` in case if everything is OK, or with `QueueNotFound` in case if primary node sent queue, that doesn't exist.

/// Replication storage (event log)
pub mod storage;

/// Replication TCP messages
pub mod message;

/// Primary node
pub mod primary;

/// Replica node
pub mod replica;

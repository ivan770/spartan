use serde::{Deserialize, Serialize};
use spartan_lib::core::{message::Message, payload::Identifiable};

#[derive(Serialize, Deserialize)]
pub enum Event {
    Push(Message),
    Pop,
    Requeue(<Message as Identifiable>::Id),
    Delete(<Message as Identifiable>::Id),
    Gc,
    Clear,
}

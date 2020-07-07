use serde::{Serialize, Deserialize};
use spartan_lib::core::{payload::Identifiable, message::Message};

#[derive(Serialize, Deserialize)]
pub enum Event {
    Push(Message),
    Pop,
    Requeue(<Message as Identifiable>::Id),
    Delete(<Message as Identifiable>::Id),
    Gc,
    Clear,
}

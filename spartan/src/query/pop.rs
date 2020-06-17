use serde::Serialize;
use spartan_lib::core::message::Message;

#[derive(Serialize, new)]
pub struct PopResponse<'message> {
    message: &'message Message,
}

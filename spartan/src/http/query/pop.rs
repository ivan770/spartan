use serde::Serialize;
use spartan_lib::core::message::Message;

#[derive(Serialize)]
pub struct PopResponse<'message> {
    message: &'message Message,
}

impl<'message> PopResponse<'message> {
    pub fn new(message: &'message Message) -> Self {
        PopResponse {
            message
        }
    }
}

#[cfg(test)]
#[derive(serde::Deserialize)]
pub struct TestPopResponse {
    pub message: Message,
}

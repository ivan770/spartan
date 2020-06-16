use super::Query;
use serde::Serialize;
use spartan_lib::core::message::Message;

#[derive(Serialize, new)]
pub struct PopResponse<'message> {
    message: &'message Message,
}

impl Query for PopResponse<'_> {}

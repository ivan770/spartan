use super::Query;
use serde::Serialize;
use spartan_lib::core::message::Message;

#[derive(Serialize, new)]
pub struct PopResponse<'a> {
    message: &'a Message,
}

impl Query for PopResponse<'_> {}

use crate::{
    node::QueueExtractor,
    query::{pop::PopResponse, Error, Response},
    respond, Request,
};
use spartan_lib::core::dispatcher::StatusAwareDispatcher;
use tide::{Result, StatusCode};

pub async fn pop(request: Request) -> Result {
    let mut queue = respond!(QueueExtractor::new(&request).extract().await);
    let message = respond!(queue
        .pop()
        .ok_or_else(|| Error::new(StatusCode::NotFound, "Queue is empty")));
    Ok(PopResponse::new(message).respond())
}

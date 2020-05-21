use crate::{
    node::QueueExtractor,
    query::{clear::ClearResponse, Response},
    respond, Request,
};
use spartan_lib::core::dispatcher::simple::SimpleDispatcher;
use tide::Result;

pub async fn clear(request: Request) -> Result {
    let mut queue = respond!(QueueExtractor::new(&request).extract().await);
    queue.clear();
    Ok(ClearResponse::new().respond())
}

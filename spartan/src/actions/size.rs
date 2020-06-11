use crate::{
    node::QueueExtractor,
    query::{size::SizeResponse, Response},
    respond, Request,
};
use spartan_lib::core::dispatcher::simple::SimpleDispatcher;
use tide::Result;

/// Get queue size.
///
/// Doesn't require any input, returns queue size.
pub async fn size(request: Request) -> Result {
    let queue = respond!(QueueExtractor::new(&request).extract().await);
    Ok(SizeResponse::new(queue.size()).respond())
}

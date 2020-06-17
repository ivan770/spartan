use super::QueueError;
use crate::{node::Manager, query::pop::PopResponse};
use actix_web::{
    web::{Data, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::StatusAwareDispatcher;

/// Pop message from queue.
///
/// Doesn't require any input, returns reserved message.
/// After reserving message, you either need to return it to queue, or delete it.
/// Messages that are not returned after timeout are deleted by GC.
pub async fn pop(manager: Data<Manager>, queue: Path<(String,)>) -> Result<HttpResponse> {
    let mut queue = manager.queue(&queue.0).await?;
    let message = queue.pop().ok_or_else(|| QueueError::NoMessageAvailable)?;
    Ok(HttpResponse::Ok().json(PopResponse::new(message)))
}

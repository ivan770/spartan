use super::QueueError;
use crate::{node::Manager, query::requeue::RequeueRequest};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::StatusAwareDispatcher;

/// Requeues message back to queue.
///
/// Requires ID of message being requeued, returns empty response.
/// Message try counter is incremented.
pub async fn requeue(
    request: Json<RequeueRequest>,
    manager: Data<Manager<'_>>,
    queue: Path<(String,)>,
) -> Result<HttpResponse> {
    let mut queue = manager.queue(&queue.0).await?;
    queue
        .requeue(request.id)
        .ok_or_else(|| QueueError::NoMessageAvailable)?;
    Ok(HttpResponse::Ok().json(()))
}

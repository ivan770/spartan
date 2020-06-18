use super::QueueError;
use crate::{
    node::Manager,
    query::delete::{DeleteRequest, DeleteResponse},
};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::simple::PositionBasedDelete;

/// Delete message from queue.
///
/// Requires ID of message being deleted, returns deleted message.
pub async fn delete(
    request: Json<DeleteRequest>,
    manager: Data<Manager<'_>>,
    queue: Path<(String,)>,
) -> Result<HttpResponse> {
    let mut queue = manager.queue(&queue.0).await?;
    let message = queue
        .delete(request.id)
        .ok_or_else(|| QueueError::NoMessageAvailable)?;
    Ok(HttpResponse::Ok().json(DeleteResponse::new(message)))
}

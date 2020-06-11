use crate::{
    node::QueueExtractor,
    query::{
        delete::{DeleteRequest, DeleteResponse},
        Error, Response,
    },
    respond, Request,
};
use spartan_lib::core::dispatcher::simple::PositionBasedDelete;
use tide::{Result, StatusCode};

/// Delete message from queue.
///
/// Requires ID of message being deleted, returns deleted message.
pub async fn delete(mut request: Request) -> Result {
    let json: DeleteRequest = respond!(request.body_json().await.map_err(|_| Error::bad_request()));
    let mut queue = respond!(QueueExtractor::new(&request).extract().await);
    let message = respond!(queue
        .delete(json.id)
        .ok_or_else(|| Error::new(StatusCode::NotFound, "Message not found")));
    Ok(DeleteResponse::new(message).respond())
}

use crate::{
    node::QueueExtractor,
    query::{
        requeue::{RequeueRequest, RequeueResponse},
        Error, Response,
    },
    respond, Request,
};
use spartan_lib::core::dispatcher::StatusAwareDispatcher;
use tide::{Result, StatusCode};

/// Requeues message back to queue.
///
/// Requires ID of message being requeued, returns empty response.
/// Message try counter is incremented.
pub async fn requeue(mut request: Request) -> Result {
    let json: RequeueRequest =
        respond!(request.body_json().await.map_err(|_| Error::bad_request()));
    let mut queue = respond!(QueueExtractor::new(&request).extract().await);
    respond!(queue
        .requeue(json.id)
        .ok_or_else(|| Error::new(StatusCode::NotFound, "Message not found")));
    Ok(RequeueResponse::new().respond())
}

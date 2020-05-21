use crate::{
    node::QueueExtractor,
    query::{
        delete::{DeleteRequest, DeleteResponse},
        Error, Response,
    },
    respond, Request,
};
use spartan_lib::core::dispatcher::simple::PositionBasedDelete;
use tide::{StatusCode, Result};

pub async fn delete(mut request: Request) -> Result {
    let json: DeleteRequest = respond!(request.body_json().await.map_err(|_| Error::bad_request()));
    let mut queue = respond!(QueueExtractor::new(&request).extract().await);
    respond!(queue
        .delete(json.id)
        .ok_or_else(|| Error::new(StatusCode::NotFound, "Message not found")));
    Ok(DeleteResponse::new().respond())
}

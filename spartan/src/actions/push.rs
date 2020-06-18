use crate::{node::Manager, query::push::PushRequest};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Result,
};
use spartan_lib::core::{
    dispatcher::SimpleDispatcher,
    message::{builder::MessageBuilder, Message},
};

/// Push message to queue.
///
/// Requires message body. Offset, max tries, timeout, delay are optional.
/// Returns empty response.
pub async fn push(
    request: Json<PushRequest>,
    manager: Data<Manager<'_>>,
    queue: Path<(String,)>,
) -> Result<HttpResponse> {
    let mut queue = manager.queue(&queue.0).await?;
    queue.push(apply_builder(&request));
    Ok(HttpResponse::Ok().json(()))
}

/// Compose message from push request.
pub fn apply_builder(request: &PushRequest) -> Message {
    let mut builder = MessageBuilder::default().body(request.body.clone());

    if let Some(offset) = request.offset {
        builder = builder.offset(offset);
    };

    if let Some(max_tries) = request.max_tries {
        builder = builder.max_tries(max_tries);
    };

    if let Some(timeout) = request.timeout {
        builder = builder.timeout(timeout);
    };

    if let Some(delay) = request.delay {
        builder = builder.delay(delay);
    };

    builder.compose().expect("No message body provided")
}

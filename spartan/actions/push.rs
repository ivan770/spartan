use std::convert::TryInto;

use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Result,
};
use maybe_owned::MaybeOwned;
use spartan_lib::core::{dispatcher::SimpleDispatcher, message::Message};

use crate::{
    actions::QueueError,
    http::query::push::PushRequest,
    node::{event::Event, Manager},
};

/// Push message to queue.
///
/// Requires message body. Offset, max tries, timeout, delay are optional.
///
/// Returns empty response.
pub async fn push(
    request: Json<PushRequest>,
    manager: Data<Manager<'_>>,
    Path((name,)): Path<(String,)>,
) -> Result<HttpResponse> {
    let queue = manager.queue(&name)?;
    let message: Message = request
        .into_inner()
        .try_into()
        .map_err(QueueError::MessageCompose)?;

    queue
        .log_event(&name, &manager, Event::Push(MaybeOwned::Borrowed(&message)))
        .await?;

    queue.database().await.push(message);
    Ok(HttpResponse::Ok().json(()))
}

#[cfg(test)]
mod tests {
    use actix_web::{
        test::{init_service, read_response, read_response_json},
        web::Bytes,
    };

    use crate::{
        http::query::{pop::TestPopResponse, push::PushRequest},
        init_application, test_request,
        utils::testing::CONFIG,
    };

    #[actix_rt::test]
    async fn test_push() {
        use spartan_lib::core::payload::Dispatchable;

        let mut app = init_service(init_application!(&CONFIG)).await;

        read_response(
            &mut app,
            test_request!(
                post,
                "/test",
                &PushRequest {
                    body: String::from("Hello, world").into_boxed_str(),
                    ..Default::default()
                }
            ),
        )
        .await;

        let pop: TestPopResponse = read_response_json(&mut app, test_request!(get, "/test")).await;
        assert_eq!(pop.message.body(), "Hello, world");
    }

    #[actix_rt::test]
    async fn test_delayed_push() {
        let mut app = init_service(init_application!(&CONFIG)).await;

        read_response(
            &mut app,
            test_request!(
                post,
                "/test",
                &PushRequest {
                    body: String::from("Hello, world").into_boxed_str(),
                    delay: Some(900),
                    ..Default::default()
                }
            ),
        )
        .await;

        let pop = read_response(&mut app, test_request!(get, "/test")).await;
        assert_eq!(pop, Bytes::from_static(b"No message available"));
    }
}

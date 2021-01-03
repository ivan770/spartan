use std::{convert::TryInto, sync::Arc};

use maybe_owned::MaybeOwned;
use spartan_lib::core::{dispatcher::SimpleDispatcher, message::Message};
use warp::reply::{json, Json};

use crate::{
    actions::{QueueError, Result},
    http::query::push::PushRequest,
    node::{event::Event, Manager},
};

/// Push message to queue.
///
/// Requires message body. Offset, max tries, timeout, delay are optional.
///
/// Returns empty response.
pub async fn push(manager: Arc<Manager<'_>>, name: String, request: PushRequest) -> Result<Json> {
    let queue = manager.queue(&name)?;
    let message: Message = request.try_into().map_err(QueueError::MessageCompose)?;

    queue
        .log_event(&name, &manager, Event::Push(MaybeOwned::Borrowed(&message)))
        .await?;

    queue.database().await.push(message);

    Ok(json(&()))
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::{
        http::query::{pop::test_response::TestPopResponse, push::PushRequest},
        init_application, test_json_request, test_request,
        utils::testing::CONFIG,
    };

    #[tokio::test]
    async fn test_push() {
        let app = init_application!(&CONFIG);

        test_request!(
            app,
            "POST",
            "/test",
            &PushRequest {
                body: String::from("Hello, world").into_boxed_str(),
                ..Default::default()
            }
        )
        .await;

        let pop: TestPopResponse = test_json_request!(app, "GET", "/test");

        assert_eq!(&*pop.body, "Hello, world");
    }

    #[tokio::test]
    async fn test_delayed_push() {
        let app = init_application!(&CONFIG);

        test_request!(
            app,
            "POST",
            "/test",
            &PushRequest {
                body: String::from("Hello, world").into_boxed_str(),
                delay: Some(900),
                ..Default::default()
            }
        )
        .await;

        let pop = test_request!(app, "GET", "/test").await;
        assert_eq!(*pop.body(), Bytes::from_static(b"No message available"));
    }
}

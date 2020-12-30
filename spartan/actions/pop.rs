use std::sync::Arc;

use spartan_lib::core::dispatcher::StatusAwareDispatcher;
use warp::reply::{json, Json};

use crate::{
    actions::{QueueError, Result},
    http::query::pop::PopResponse,
    node::{event::Event, Manager},
};

/// Pop message from queue.
///
/// Doesn't require any input, returns reserved message.
///
/// After reserving message, you either need to return it to queue, or delete it.
///
/// Messages that are not returned after timeout are deleted by GC.
pub async fn pop(manager: Arc<Manager<'_>>, name: String) -> Result<Json> {
    let queue = manager.queue(&name)?;

    queue.log_event(&name, &manager, Event::Pop).await?;

    let mut database = queue.database().await;
    let message = database.pop().ok_or(QueueError::NoMessageAvailable)?;

    Ok(json(&PopResponse::from(message)))
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
    async fn test_empty_pop() {
        let app = init_application!(&CONFIG);
        let pop = test_request!(app, "GET", "/test").await;
        assert_eq!(*pop.body(), Bytes::from_static(b"No message available"));
    }

    #[tokio::test]
    async fn test_message_pop() {
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
}

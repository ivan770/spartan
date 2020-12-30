use std::sync::Arc;

use spartan_lib::core::dispatcher::StatusAwareDispatcher;
use warp::reply::{json, Json};

use crate::{
    actions::{QueueError, Result},
    http::query::requeue::RequeueRequest,
    node::{event::Event, Manager},
};

/// Requeues message back to queue.
///
/// Requires ID of message being requeued, returns empty response.
///
/// Message try counter is incremented.
pub async fn requeue(
    manager: Arc<Manager<'_>>,
    name: String,
    request: RequeueRequest,
) -> Result<Json> {
    let queue = manager.queue(&name)?;

    queue
        .log_event(&name, &manager, Event::Requeue(request.id))
        .await?;

    queue
        .database()
        .await
        .requeue(request.id)
        .ok_or(QueueError::MessageNotFound)?;

    Ok(json(&()))
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use uuid::Uuid;

    use crate::{
        http::query::{
            pop::test_response::TestPopResponse, push::PushRequest, requeue::RequeueRequest,
        },
        init_application, test_json_request, test_request,
        utils::testing::CONFIG,
    };

    #[tokio::test]
    async fn test_empty_requeue() {
        let app = init_application!(&CONFIG);
        let resp = test_request!(
            app,
            "POST",
            "/test/requeue",
            &RequeueRequest { id: Uuid::new_v4() }
        )
        .await;

        assert_eq!(*resp.body(), Bytes::from_static(b"Message not found"));
    }

    #[tokio::test]
    async fn test_message_requeue() {
        let app = init_application!(&CONFIG);

        test_request!(
            app,
            "POST",
            "/test",
            &PushRequest {
                body: String::from("Hello, world").into_boxed_str(),
                max_tries: Some(2),
                ..Default::default()
            }
        )
        .await;

        let first_pop: TestPopResponse = test_json_request!(app, "GET", "/test");

        test_request!(
            app,
            "POST",
            "/test/requeue",
            &RequeueRequest { id: first_pop.id }
        )
        .await;

        let second_pop: TestPopResponse = test_json_request!(app, "GET", "/test");

        assert_eq!(first_pop.id, second_pop.id);

        test_request!(
            app,
            "POST",
            "/test/requeue",
            &RequeueRequest { id: second_pop.id }
        )
        .await;

        let third_pop = test_request!(app, "GET", "/test").await;

        assert_eq!(
            *third_pop.body(),
            Bytes::from_static(b"No message available")
        );
    }
}

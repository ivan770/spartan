use std::sync::Arc;

use spartan_lib::core::dispatcher::PositionBasedDelete;
use warp::reply::{json, Json};

use crate::{
    actions::{QueueError, Result},
    http::query::delete::{DeleteRequest, DeleteResponse},
    node::{event::Event, Manager},
};

/// Delete message from queue.
///
/// Requires ID of message being deleted, returns deleted message.
pub async fn delete(
    manager: Arc<Manager<'_>>,
    name: String,
    request: DeleteRequest,
) -> Result<Json> {
    let queue = manager.queue(&name)?;

    queue
        .log_event(&name, &manager, Event::Delete(request.id))
        .await?;

    let message = queue
        .database()
        .await
        .delete(request.id)
        .ok_or(QueueError::MessageNotFound)?;

    Ok(json(&DeleteResponse::from(message)))
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use spartan_lib::{core::payload::Identifiable, uuid::Uuid};

    use crate::{
        http::query::{
            delete::{DeleteRequest, DeleteResponse},
            pop::test_response::TestPopResponse,
            push::PushRequest,
            size::SizeResponse,
        },
        init_application, test_json_request, test_request,
        utils::testing::CONFIG,
    };

    #[tokio::test]
    async fn test_empty_delete() {
        let app = init_application!(&CONFIG);
        let resp = test_request!(
            app,
            "DELETE",
            "/test",
            &DeleteRequest { id: Uuid::new_v4() }
        )
        .await;
        assert_eq!(*resp.body(), Bytes::from_static(b"Message not found"));
    }

    #[tokio::test]
    async fn test_message_delete() {
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

        let size: SizeResponse = test_json_request!(app, "GET", "/test/size");

        assert_eq!(size.size, 1);

        let delete: DeleteResponse =
            test_json_request!(app, "DELETE", "/test", &DeleteRequest { id: pop.id });

        assert_eq!(delete.message.id(), pop.id);

        let size: SizeResponse = test_json_request!(app, "GET", "/test/size");

        assert_eq!(size.size, 0);
    }
}

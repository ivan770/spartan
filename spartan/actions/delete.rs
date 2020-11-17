use super::QueueError;
use crate::{
    http::query::delete::{DeleteRequest, DeleteResponse},
    node::Manager,
};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::PositionBasedDelete;

use crate::node::event::Event;

/// Delete message from queue.
///
/// Requires ID of message being deleted, returns deleted message.
pub async fn delete(
    request: Json<DeleteRequest>,
    manager: Data<Manager<'_>>,
    Path((name,)): Path<(String,)>,
) -> Result<HttpResponse> {
    let queue = manager.queue(&name)?;

    queue
        .log_event(&name, &manager, Event::Delete(request.id))
        .await?;

    let message = queue
        .database()
        .await
        .delete(request.id)
        .ok_or(QueueError::MessageNotFound)?;

    Ok(HttpResponse::Ok().json(DeleteResponse::new(message)))
}

#[cfg(test)]
mod tests {
    use crate::{
        http::query::{
            delete::{DeleteRequest, DeleteResponse},
            pop::TestPopResponse,
            push::PushRequest,
            size::SizeResponse,
        },
        init_application, test_request,
        utils::testing::CONFIG,
    };
    use actix_web::{
        test::{init_service, read_response, read_response_json},
        web::Bytes,
    };
    use spartan_lib::{core::payload::Identifiable, uuid::Uuid};

    #[actix_rt::test]
    async fn test_empty_delete() {
        let mut app = init_service(init_application!(&CONFIG)).await;
        let resp = read_response(
            &mut app,
            test_request!(delete, "/test", &DeleteRequest { id: Uuid::new_v4() }),
        )
        .await;
        assert_eq!(resp, Bytes::from_static(b"Message not found"));
    }

    #[actix_rt::test]
    async fn test_message_delete() {
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

        let size: SizeResponse =
            read_response_json(&mut app, test_request!(get, "/test/size")).await;
        assert_eq!(size.size, 1);

        let delete: DeleteResponse = read_response_json(
            &mut app,
            test_request!(
                delete,
                "/test",
                &DeleteRequest {
                    id: pop.message.id()
                }
            ),
        )
        .await;

        assert_eq!(delete.message.id(), pop.message.id());

        let size: SizeResponse =
            read_response_json(&mut app, test_request!(get, "/test/size")).await;
        assert_eq!(size.size, 0);
    }
}

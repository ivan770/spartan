use super::QueueError;
use crate::{http::query::requeue::RequeueRequest, node::Manager};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::StatusAwareDispatcher;

/// Requeues message back to queue.
///
/// Requires ID of message being requeued, returns empty response.
/// Message try counter is incremented.
pub async fn requeue(
    request: Json<RequeueRequest>,
    manager: Data<Manager<'_>>,
    queue: Path<(String,)>,
) -> Result<HttpResponse> {
    let mut queue = manager.queue(&queue.0).await?;
    queue
        .requeue(request.id)
        .ok_or_else(|| QueueError::MessageNotFound)?;
    Ok(HttpResponse::Ok().json(()))
}

#[cfg(test)]
mod tests {
    use crate::{
        http::query::{pop::TestPopResponse, push::PushRequest, requeue::RequeueRequest},
        init_application, test_request,
        utils::testing::CONFIG,
    };
    use actix_web::{
        test::{init_service, read_response, read_response_json},
        web::Bytes,
    };
    use uuid::Uuid;

    #[actix_rt::test]
    async fn test_empty_requeue() {
        let mut app = init_service(init_application!(&CONFIG)).await;
        let resp = read_response(
            &mut app,
            test_request!(
                post,
                "/test/requeue",
                &RequeueRequest { id: Uuid::new_v4() }
            ),
        )
        .await;
        assert_eq!(resp, Bytes::from_static(b"Message not found"));
    }

    #[actix_rt::test]
    async fn test_message_requeue() {
        let mut app = init_service(init_application!(&CONFIG)).await;

        read_response(
            &mut app,
            test_request!(
                post,
                "/test",
                &PushRequest {
                    body: String::from("Hello, world"),
                    max_tries: Some(2),
                    ..Default::default()
                }
            ),
        )
        .await;

        let first_pop: TestPopResponse =
            read_response_json(&mut app, test_request!(get, "/test")).await;

        read_response(
            &mut app,
            test_request!(
                post,
                "/test/requeue",
                &RequeueRequest {
                    id: first_pop.message.id
                }
            ),
        )
        .await;

        let second_pop: TestPopResponse =
            read_response_json(&mut app, test_request!(get, "/test")).await;

        assert_eq!(first_pop.message.id, second_pop.message.id);

        read_response(
            &mut app,
            test_request!(
                post,
                "/test/requeue",
                &RequeueRequest {
                    id: second_pop.message.id
                }
            ),
        )
        .await;

        let third_pop = read_response(&mut app, test_request!(get, "/test")).await;

        assert_eq!(third_pop, Bytes::from_static(b"No message available"));
    }
}

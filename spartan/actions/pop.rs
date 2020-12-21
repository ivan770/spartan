use actix_web::{
    web::{Data, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::StatusAwareDispatcher;

use super::QueueError;
use crate::{
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
pub async fn pop(
    manager: Data<Manager<'_>>,
    Path((name,)): Path<(String,)>,
) -> Result<HttpResponse> {
    let queue = manager.queue(&name)?;

    queue.log_event(&name, &manager, Event::Pop).await?;

    let mut database = queue.database().await;
    let message = database.pop().ok_or(QueueError::NoMessageAvailable)?;

    Ok(HttpResponse::Ok().json(PopResponse::from(message)))
}

#[cfg(test)]
mod tests {
    use actix_web::{
        test::{init_service, read_response, read_response_json},
        web::Bytes,
    };

    use crate::{
        http::query::{pop::test_response::TestPopResponse, push::PushRequest},
        init_application, test_request,
        utils::testing::CONFIG,
    };

    #[actix_rt::test]
    async fn test_empty_pop() {
        let mut app = init_service(init_application!(&CONFIG)).await;
        let pop = read_response(&mut app, test_request!(get, "/test")).await;
        assert_eq!(pop, Bytes::from_static(b"No message available"));
    }

    #[actix_rt::test]
    async fn test_message_pop() {
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
        assert_eq!(&*pop.body, "Hello, world");
    }
}

use crate::{node::Manager, query::size::SizeResponse};
use actix_web::{
    web::{Data, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::simple::SimpleDispatcher;

/// Get queue size.
///
/// Doesn't require any input, returns queue size.
pub async fn size(manager: Data<Manager<'_>>, queue: Path<(String,)>) -> Result<HttpResponse> {
    let queue = manager.queue(&queue.0).await?;
    Ok(HttpResponse::Ok().json(SizeResponse::new(queue.size())))
}

#[cfg(test)]
mod tests {
    use crate::{
        init_application,
        query::{push::PushRequest, size::SizeResponse},
        test_request,
        utils::testing::CONFIG
    };
    use actix_web::test::{init_service, read_response, read_response_json};

    #[actix_rt::test]
    async fn test_size() {
        let mut app = init_service(init_application!(&CONFIG)).await;

        let size: SizeResponse =
            read_response_json(&mut app, test_request!(get, "/test/size")).await;
        assert_eq!(size.size, 0);

        read_response(
            &mut app,
            test_request!(
                post,
                "/test",
                &PushRequest {
                    body: String::from("Hello, world"),
                    ..Default::default()
                }
            ),
        )
        .await;

        let size: SizeResponse =
            read_response_json(&mut app, test_request!(get, "/test/size")).await;
        assert_eq!(size.size, 1);
    }
}

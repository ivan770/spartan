use crate::{http::query::size::SizeResponse, node::Manager};
use actix_web::{
    web::{Data, Path},
    HttpResponse, Result,
};
use spartan_lib::core::dispatcher::simple::SimpleDispatcher;

/// Get queue size.
///
/// Doesn't require any input, returns queue size.
pub async fn size(
    manager: Data<Manager<'_>>,
    Path((name,)): Path<(String,)>,
) -> Result<HttpResponse> {
    let queue = manager.queue(&name)?.database().await;

    Ok(HttpResponse::Ok().json(SizeResponse::new(queue.size())))
}

#[cfg(test)]
mod tests {
    use crate::{
        http::query::{push::PushRequest, size::SizeResponse},
        init_application, test_request,
        utils::testing::CONFIG,
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
                    body: String::from("Hello, world").into_boxed_str(),
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

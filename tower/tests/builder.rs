#![cfg(all(feature = "buffer", feature = "limit", feature = "retry"))]

use futures_util::{future::Ready, pin_mut};
use std::time::Duration;
use tower::buffer::BufferLayer;
use tower::builder::ServiceBuilder;
use tower::limit::{concurrency::ConcurrencyLimitLayer, rate::RateLimitLayer};
use tower::retry::{Policy, RetryLayer};
use tower::util::ServiceExt;
use tower_service::*;
use tower_test::{assert_request_eq, mock};

#[tokio::test]
async fn builder_service() {
    let (service, handle) = mock::pair();
    pin_mut!(handle);

    let policy = MockPolicy;
    let mut client = ServiceBuilder::new()
        .layer(BufferLayer::new(5))
        .layer(ConcurrencyLimitLayer::new(5))
        .layer(RateLimitLayer::new(5, Duration::from_secs(1)))
        .layer(RetryLayer::new(policy))
        .layer(BufferLayer::new(5))
        .service(service);

    // allow a request through
    handle.allow(1);

    let fut = client.ready_and().await.unwrap().call("hello");
    assert_request_eq!(handle, "hello").send_response("world");
    assert_eq!(fut.await.unwrap(), "world");
}

#[derive(Debug, Clone)]
struct MockPolicy;

impl<E> Policy<&'static str, &'static str, E> for MockPolicy
where
    E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
{
    type Future = Ready<Self>;

    fn retry(
        &self,
        _req: &&'static str,
        _result: Result<&&'static str, &E>,
    ) -> Option<Self::Future> {
        None
    }

    fn clone_request(&self, req: &&'static str) -> Option<&'static str> {
        Some(req.clone())
    }
}

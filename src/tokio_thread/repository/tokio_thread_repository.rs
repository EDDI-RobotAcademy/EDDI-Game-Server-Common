use std::future::Future;
use async_trait::async_trait;

#[async_trait]
pub trait TokioThreadRepository {
    async fn spawn(&self, id: usize, fut: impl Future<Output = ()> + Send + 'static);
}

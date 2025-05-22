use futures::future::BoxFuture;
use async_trait::async_trait;

#[async_trait]
pub trait TokioThreadRepository: Send + Sync {
    async fn spawn(&self, id: usize, fut: BoxFuture<'static, ()>);
}

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use async_trait::async_trait;
use futures::future::{BoxFuture, FutureExt};

use crate::tokio_thread::entity::tokio_thread_entity::TokioThreadEntity;
use crate::tokio_thread::repository::tokio_thread_repository::TokioThreadRepository;

pub struct TokioThreadRepositoryImpl {
    threadsHashMap: Arc<RwLock<HashMap<usize, TokioThreadEntity>>>,
}

impl TokioThreadRepositoryImpl {
    pub fn new() -> Self {
        Self {
            threadsHashMap: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TokioThreadRepository for TokioThreadRepositoryImpl {
    async fn spawn(&self, id: usize, fut: BoxFuture<'static, ()>) {
        let handle: JoinHandle<()> = tokio::spawn(fut);
        let entity = TokioThreadEntity::new(id, handle);
        let mut threadsHashMap = self.threadsHashMap.write().await;
        threadsHashMap.insert(id, entity);
        println!("[Repo] Spawned thread with id {}", id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_spawn_stores_thread_entity() {
        let repo = TokioThreadRepositoryImpl::new();

        let id = 42;
        repo.spawn(id, async move {
            println!("[Thread {}] Running...", id);
            sleep(Duration::from_millis(100)).await;
            println!("[Thread {}] Finished.", id);
        }.boxed()).await;

        let threadsHashMap = repo.threadsHashMap.read().await;
        assert!(threadsHashMap.contains_key(&id));
        assert!(threadsHashMap.get(&id).unwrap().is_running);
        println!("[TEST] Thread with id {} is running and stored", id);
    }
}

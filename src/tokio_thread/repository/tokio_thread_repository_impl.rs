use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use async_trait::async_trait;
use std::future::Future;

use crate::tokio_thread::entity::tokio_thread_entity::TokioThreadEntity;

#[async_trait]
pub trait TokioThreadRepository {
    async fn spawn(&self, id: usize, fut: impl Future<Output = ()> + Send + 'static);
}

pub struct TokioThreadRepositoryImpl {
    threads: Arc<RwLock<HashMap<usize, TokioThreadEntity>>>,
}

impl TokioThreadRepositoryImpl {
    pub fn new() -> Self {
        Self {
            threads: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TokioThreadRepository for TokioThreadRepositoryImpl {
    async fn spawn(&self, id: usize, fut: impl Future<Output = ()> + Send + 'static) {
        let handle: JoinHandle<()> = tokio::spawn(fut);
        let entity = TokioThreadEntity::new(id, handle);
        let mut threads = self.threads.write().await;
        threads.insert(id, entity);
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
        }).await;

        // 내부 저장소에 있는지 확인
        let threads = repo.threads.read().await;
        assert!(threads.contains_key(&id));
        assert!(threads.get(&id).unwrap().is_running);
        println!("[TEST] Thread with id {} is running and stored", id);
    }
}

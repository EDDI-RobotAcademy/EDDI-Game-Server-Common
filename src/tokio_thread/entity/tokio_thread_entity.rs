use tokio::task::JoinHandle;

pub struct TokioThreadEntity {
    pub id: usize,
    pub handle: Option<JoinHandle<()>>,
    pub is_running: bool,
}

impl TokioThreadEntity {
    pub fn new(id: usize, handle: JoinHandle<()>) -> Self {
        Self {
            id,
            handle: Some(handle),
            is_running: true,
        }
    }

    pub fn stop(&mut self) {
        self.is_running = false;
        self.handle = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_thread_entity_creation_and_stop() {
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            // 임시 작업 생성
            let handle = tokio::spawn(async {
                // 잠깐 대기
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            });

            let mut entity = TokioThreadEntity::new(1, handle);

            assert_eq!(entity.id, 1);
            assert!(entity.is_running);
            assert!(entity.handle.is_some());

            entity.stop();

            assert!(!entity.is_running);
            assert!(entity.handle.is_none());
        });
    }
}
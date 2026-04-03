use tokio::sync::{mpsc, Mutex};
use std::collections::VecDeque;
use crate::sync::task::SyncTask;

pub struct SyncTaskQueue {
    sender: mpsc::Sender<SyncTask>,
    receiver: Mutex<mpsc::Receiver<SyncTask>>,
    capacity: usize,
}

impl SyncTaskQueue {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: Mutex::new(receiver),
            capacity,
        }
    }

    pub async fn submit(&self, task: SyncTask) -> Result<(), QueueError> {
        match self.sender.try_send(task) {
            Ok(_) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                Err(QueueError::QueueFull)
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(QueueError::QueueClosed)
            }
        }
    }

    pub async fn submit_blocking(&self, task: SyncTask) -> Result<(), QueueError> {
        self.sender.send(task).await
            .map_err(|_| QueueError::QueueClosed)
    }

    pub async fn next(&self) -> Option<SyncTask> {
        let mut receiver = self.receiver.lock().await;
        receiver.recv().await
    }

    pub async fn try_next(&self) -> Option<SyncTask> {
        let mut receiver = self.receiver.lock().await;
        match receiver.try_recv() {
            Ok(task) => Some(task),
            Err(_) => None,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn close(&self) {
        self.sender.closed();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("Queue is full")]
    QueueFull,
    #[error("Queue is closed")]
    QueueClosed,
}

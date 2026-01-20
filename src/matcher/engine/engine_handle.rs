use tokio::sync::mpsc;

use crate::matcher::engine::engine_event::EngineEvent;

pub struct EngineHandle {
    tx: mpsc::Sender<EngineEvent>,
}

impl EngineHandle {
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<EngineEvent>) {
        let (tx, rx) = mpsc::channel(capacity);
        (Self { tx }, rx)
    }

    pub async fn send(&self, event: EngineEvent) {
        let _ = self.tx.send(event).await;
    }
}

use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Context {
    shutdown_tx: broadcast::Sender<()>,
}

impl Context {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        Self { shutdown_tx }
    }

    pub fn build(self) -> Arc<Context> {
        Arc::new(self)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

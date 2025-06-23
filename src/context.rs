use std::sync::Arc;
use tokio::sync::broadcast;

// Shared context
#[derive(Clone, Debug)]
pub struct Context {
    inner: Arc<ContextInner>,
}

// Internal shared context
#[derive(Debug)]
struct ContextInner {
    shutdown_tx: broadcast::Sender<()>,
}

impl Context {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        Self { inner: Arc::new(ContextInner { shutdown_tx }) }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.inner.shutdown_tx.subscribe()
    }

    pub fn shutdown(&self) {
        let _ = self.inner.shutdown_tx.send(());
    }
}

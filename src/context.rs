use crate::{Config, CFG};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

// Shared context
#[derive(Clone, Debug)]
pub struct Context(Arc<ContextInner>);

// Internal shared context
#[derive(Debug)]
struct ContextInner {
    shutdown_tx: broadcast::Sender<()>,
    cfg: Arc<RwLock<Config>>,
}

impl Context {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        let cfg = CFG.clone();
        Self(Arc::new(ContextInner { shutdown_tx, cfg }))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.0.shutdown_tx.subscribe()
    }

    pub fn shutdown(&self) {
        let _ = self.0.shutdown_tx.send(());
    }

    pub async fn config(&self) -> Config {
        self.0.cfg.read().await.clone()
    }
}

use crate::{Context, Error};
use std::sync::Arc;
use tracing::info;

pub struct MqttServer;

impl MqttServer {
    pub async fn start(ctx: Arc<Context>) -> Result<(), Error> {
        info!("MqttServer starting...");

        info!("MqttServer shutdown");
        Ok(())
    }
}

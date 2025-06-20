use crate::Error;
use tokio::sync::broadcast::Receiver;
use tracing::info;

pub struct MqttServer;
impl MqttServer {
    pub async fn start(mut shutdown: Receiver<()>) -> Result<(), Error> {
        info!("MqttServer starting...");
        info!("MqttServer shutdown");
        Ok(())
    }
}

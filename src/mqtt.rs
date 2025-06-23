use crate::{Config, Context, Error};
use tracing::info;

pub struct MqttServer {
    ctx: Context,
}

impl MqttServer {
    pub async fn start(ctx: Context) -> Result<(), Error> {
        info!("MqttServer starting...");

        let server = Self { ctx };
        server.run().await;
        let config = Config::get().listener;
        println!("{:?}", config);

        info!("MqttServer shutdown");
        Ok(())
    }

    async fn run(self) {}
}

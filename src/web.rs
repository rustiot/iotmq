use crate::Context;
use crate::Error;
use crate::{api, Config};
use axum::{Extension, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

// Web configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Web {
    #[serde(default = "Web::default_port")]
    pub port: u16,
}

impl Web {
    fn default_port() -> u16 {
        8888
    }
}

pub struct WebServer;

impl WebServer {
    fn routes(ctx: Context) -> Router {
        Router::new()
            .nest("/api", api::routes().layer(Extension(ctx)))
            .nest_service("/static", ServeDir::new("./web/dist/static"))
            .fallback_service(ServeFile::new("./web/dist/index.html"))
    }

    pub async fn start(ctx: Context) -> Result<(), Error> {
        let config = Config::get().web;
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), config.port);

        info!("WebServer[{}] starting...", addr);

        let ln = TcpListener::bind(addr).await?;
        axum::serve(ln, Self::routes(ctx.clone()))
            .with_graceful_shutdown(async move {
                let _ = ctx.subscribe().recv().await;
            })
            .await?;

        info!("WebServer shutdown");
        Ok(())
    }
}

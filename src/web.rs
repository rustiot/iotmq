use crate::api;
use crate::Context;
use crate::Error;
use axum::{Extension, Router};
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

// Web configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Web {
    #[serde(default = "Web::default_addr")]
    pub addr: String,
}

impl Web {
    fn default_addr() -> String {
        "0.0.0.0:8888".into()
    }
}

pub struct WebServer;

impl WebServer {
    fn routes(ctx: Context) -> Router {
        let routes = Router::new().nest("/api", api::routes().layer(Extension(ctx)));
        if cfg!(debug_assertions) {
            routes
                .nest_service("/static", ServeDir::new("./src/web/dist/static"))
                .fallback_service(ServeFile::new("./src/web/dist/index.html"))
        } else {
            routes
                .nest_service("/static", ServeDir::new("./web/static"))
                .fallback_service(ServeFile::new("./web/index.html"))
        }
    }

    pub async fn start(ctx: Context) -> Result<(), Error> {
        let config = ctx.config().await.web;

        info!("WebServer[{}] starting...", config.addr);

        let ln = TcpListener::bind(config.addr).await?;
        axum::serve(ln, Self::routes(ctx.clone()))
            .with_graceful_shutdown(async move {
                let _ = ctx.subscribe().recv().await;
            })
            .await?;

        info!("WebServer shutdown");

        Ok(())
    }
}

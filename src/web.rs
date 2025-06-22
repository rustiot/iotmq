use crate::Config;
use crate::Context;
use crate::Error;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Router;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
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
    #[inline]
    fn default_port() -> u16 {
        8080
    }
}

pub struct WebServer;

impl WebServer {
    fn routes() -> Router {
        Router::new()
            .nest("/api", Self::api_routes())
            .nest_service("/static", ServeDir::new("./web/dist/static"))
            .fallback_service(ServeFile::new("./web/dist/index.html"))
    }

    fn api_routes() -> Router {
        Router::new().fallback(not_found)
    }

    pub async fn start(ctx: Arc<Context>) -> Result<(), Error> {
        let config = Config::get().web;
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), config.port);

        info!("WebServer[{}] starting...", addr);

        let ln = TcpListener::bind(addr).await?;
        axum::serve(ln, Self::routes())
            .with_graceful_shutdown(async move {
                let _ = ctx.subscribe().recv().await;
            })
            .await?;

        info!("WebServer shutdown");
        Ok(())
    }
}

// 404 Not Found
async fn not_found() -> impl IntoResponse {
    let html = format!(
        r#"
        <html>
        <head><title>404 Not Found</title></head>
        <body>
        <center><h1>404 Not Found</h1></center>
        <hr><center>{} {}</center>
        </body>
        </html>"#,
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    (StatusCode::NOT_FOUND, Html(html))
}

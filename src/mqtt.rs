pub mod codec;
mod types;
pub mod v3;
pub mod v5;

pub use codec::Codec;
use types::*;

use crate::{Client, Context, Error, ListenerConfig, Protocol};
use anyhow::anyhow;
use async_tungstenite::tokio::accept_hdr_async;
use async_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use async_tungstenite::tungstenite::http::HeaderValue;
use futures::future::join_all;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info};

#[derive(Debug)]
struct Listener {
    protocol: Protocol,
    listener: TcpListener,
    config: Arc<ListenerConfig>,
}

impl Listener {
    async fn tcp(&self, ctx: Context) -> Result<(), Error> {
        info!("MqttServer TCP listening on {}", self.config.addr);

        let mut shutdown = ctx.subscribe();
        loop {
            tokio::select! {
                res = self.listener.accept() => {
                    let stream = match res {
                        Ok((stream,addr)) => {
                            debug!("TCP accepted new connection from {}", addr);
                            stream
                        }
                        Err(_) => continue
                    };

                    tokio::spawn(async move {
                        Client::new(stream);
                    });
                }
                _ = shutdown.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn tls(&self, ctx: Context) -> Result<(), Error> {
        let acceptor = self.acceptor()?;

        info!("MqttServer TLS listening on {}", self.config.addr);

        let mut shutdown_rx = ctx.subscribe();
        loop {
            tokio::select! {
                res = self.listener.accept() => {
                    let stream = match res {
                        Ok((stream,addr)) => {
                            debug!("TLS accepted new connection from {}", addr);
                            stream
                        }
                        Err(_) => continue
                    };

                    let _stream = match acceptor.accept(stream).await {
                        Ok(stream) => stream,
                        Err(_) => continue
                    };

                    //
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn ws(&self, ctx: Context) -> Result<(), Error> {
        info!("MqttServer WS listening on {}", self.config.addr);

        let mut shutdown_rx = ctx.subscribe();
        loop {
            tokio::select! {
                res = self.listener.accept() => {
                    let stream = match res {
                        Ok((stream,addr)) => {
                            debug!("WS accepted new connection from {}", addr);
                            stream
                        }
                        Err(_) => continue
                    };

                    let _stream = match accept_hdr_async(stream, ws_callback).await {
                        Ok(stream) => stream,
                        Err(_) => continue
                    };

                    //
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn wss(&self, ctx: Context) -> Result<(), Error> {
        let acceptor = self.acceptor()?;

        info!("MqttServer WSS listening on {}", self.config.addr);

        let mut shutdown_rx = ctx.subscribe();
        loop {
            tokio::select! {
                res = self.listener.accept() => {
                    let stream = match res {
                        Ok((stream,addr)) => {
                            debug!("WSS accepted new connection from {}", addr);
                            stream
                        }
                        Err(_) => continue
                    };

                     let stream = match acceptor.accept(stream).await {
                        Ok(stream) => stream,
                        Err(_) => continue
                    };

                    let _stream = match accept_hdr_async(stream, ws_callback).await {
                        Ok(stream) => stream,
                        Err(_) => continue
                    };

                    //
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }

    fn acceptor(&self) -> Result<TlsAcceptor, Error> {
        let key_file =
            self.config.key.as_ref().ok_or(anyhow!("{:?} key is not set", self.protocol))?;
        let key = PrivateKeyDer::from_pem_file(key_file)
            .map_err(|e| anyhow!("{:?} key[{:?}]: {:?}", self.protocol, key_file, e))?;

        let cert_file =
            self.config.cert.as_ref().ok_or(anyhow!("{:?} cert is not set", self.protocol))?;
        let certs = CertificateDer::pem_file_iter(cert_file)
            .map_err(|e| anyhow!("{:?} cert[{:?}]: {:?}", self.protocol, cert_file, e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow!(e))?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| anyhow!(e))?;
        let acceptor = TlsAcceptor::from(Arc::new(config));
        Ok(acceptor)
    }
}

fn ws_callback(request: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
    let err = "Sec-WebSocket-Protocol header missing".to_string();
    let protocol =
        request.headers().get("Sec-WebSocket-Protocol").ok_or(ErrorResponse::new(Some(err)))?;
    if protocol != "mqtt" {
        let err = format!("Sec-WebSocket-Protocol: {:?} is not supported", protocol);
        return Err(ErrorResponse::new(Some(err)));
    }
    response.headers_mut().insert("sec-websocket-protocol", HeaderValue::from_static("mqtt"));
    Ok(response)
}

pub struct MqttServer {
    ctx: Context,
    listeners: Vec<Listener>,
}

impl MqttServer {
    pub async fn start(ctx: Context) -> Result<(), Error> {
        info!("MqttServer starting...");

        let cfg = ctx.config().await;
        let mut listeners = Vec::new();

        for (protocol, config) in cfg.listeners {
            let listener = TcpListener::bind(config.addr).await?;
            listeners.push(Listener { protocol, listener, config: Arc::new(config) });
        }

        let server = Self { ctx, listeners };
        server.run().await;

        info!("MqttServer shutdown");
        Ok(())
    }

    async fn run(self) {
        join_all(self.listeners.into_iter().map(|listener| {
            let ctx = self.ctx.clone();
            tokio::spawn(async move {
                let result = match listener.protocol {
                    Protocol::Tcp => listener.tcp(ctx).await,
                    Protocol::Tls => listener.tls(ctx).await,
                    Protocol::Ws => listener.ws(ctx).await,
                    Protocol::Wss => listener.wss(ctx).await,
                };
                if let Err(e) = result {
                    error!("{}", e);
                }
            })
        }))
        .await;
    }
}

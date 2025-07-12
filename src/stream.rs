use crate::protocol::{v3, v5, version, Codec, ConnAck, Error, Packet, Version};
use crate::Session;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

pub struct Stream<S> {
    io: Framed<S, Codec>,
    addr: SocketAddr,
    version: Version,
}

impl<S> Stream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn new(io: S, addr: SocketAddr) -> Self {
        Self { io: Framed::new(io, Codec::Version(version::Codec)), addr, version: Version::V5 }
    }

    // Handshake
    pub async fn handshake(io: S, addr: SocketAddr) -> Result<Session, Error> {
        let mut stream = Self::new(io, addr);

        // Get Version
        stream.version = stream.version().await?;

        // Receive Connect Package
        let (packet, size) = stream.recv().await?;
        println!("{:?} {}", packet, size);

        let packet =
            Packet::ConnAck(ConnAck { session_present: false, reason_code: 0, properties: None });
        stream.send(packet).await?;

        Ok(Session::new())
    }

    // Get Version
    async fn version(&mut self) -> Result<Version, Error> {
        match self.recv().await? {
            (Packet::Version(version), _) => {
                match version {
                    Version::V3 => *self.io.codec_mut() = Codec::V3(v3::Codec),
                    Version::V5 => *self.io.codec_mut() = Codec::V5(v5::Codec),
                }
                Ok(version)
            }
            _ => Err(Error::MalformedPacket),
        }
    }

    // Receive Packet
    async fn recv(&mut self) -> Result<(Packet, u32), Error> {
        match self.io.next().await {
            Some(Ok(packet)) => Ok(packet),
            Some(Err(e)) => Err(e),
            None => Err(Error::Disconnect(self.addr.to_string())),
        }
    }

    // Send Packet
    async fn send(&mut self, packet: Packet) -> Result<(), Error> {
        self.io.send(packet).await
    }
}

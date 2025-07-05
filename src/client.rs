use crate::mqtt::{v5, Codec};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

pub struct Client<S> {
    io: Framed<S, Codec>,
}

impl<S> Client<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(io: S) -> Self {
        Self { io: Framed::new(io, Codec::V5(v5::Codec)) }
    }
}

use crate::mqtt::{v3, v5};
use tokio_util::bytes::BytesMut;
use tokio_util::codec::Decoder;

#[derive(Debug)]
pub enum Codec {
    V3(v3::Codec),
    V5(v5::Codec),
}

#[derive(Debug)]
pub enum Packet {
    V3(v3::Packet),
    V5(v5::Packet),
}

impl Decoder for Codec {
    type Item = Packet;
    type Error = std::io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        return Ok(None);
    }
}

use crate::mqtt::{v3, v5};
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

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
        Ok(None)
    }
}

impl Encoder<Packet> for Codec {
    type Error = std::io::Error;
    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

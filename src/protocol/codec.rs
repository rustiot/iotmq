use super::{v3, v5, version, Error, Packet};
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

pub enum Codec {
    V3(v3::Codec),
    V5(v5::Codec),
    Version(version::Codec),
}

impl Decoder for Codec {
    type Item = (Packet, u32);
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(match self {
            Codec::V3(codec) => codec.decode(src)?,
            Codec::V5(codec) => codec.decode(src)?,
            Codec::Version(codec) => codec.decode(src)?,
        })
    }
}

impl Encoder<Packet> for Codec {
    type Error = Error;
    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(match self {
            Codec::V3(codec) => codec.encode(item, dst)?,
            Codec::V5(codec) => codec.encode(item, dst)?,
            _ => (),
        })
    }
}

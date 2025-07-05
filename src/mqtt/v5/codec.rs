use crate::mqtt::v5::Packet;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::Decoder;

#[derive(Debug)]
pub struct Codec;

impl Decoder for Codec {
    type Item = Packet;
    type Error = std::io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        print!("{}", src.len());
        return Ok(None);
    }
}

use super::connect;
use crate::protocol::{decode_len, Error, Packet, PacketType};
use bytes::Buf;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

pub struct Codec;

impl Decoder for Codec {
    type Item = (Packet, u32);
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check minimum length
        if src.len() < 2 {
            return Ok(None);
        }

        // Decode remaining length
        let bytes = src.as_ref();
        let packet_type = bytes[0] >> 4;
        let (bytes, packet_size) = match decode_len(&bytes[1..])? {
            Some((len, len_len)) => {
                let packet_size = 1 + len_len + len;
                if src.len() < packet_size {
                    src.reserve(packet_size);
                    return Ok(None);
                }
                src.advance(len_len + 1);
                let bytes = src.split_to(len).freeze();
                (bytes, packet_size as u32)
            }
            None => return Ok(None),
        };

        // Decode packet
        let packet_type = PacketType::try_from(packet_type).map_err(|_| Error::MalformedPacket)?;
        let packet = match packet_type {
            PacketType::Connect => Packet::Connect(connect::decode(bytes)?),
            _ => unreachable!(),
        };

        Ok(Some((packet, packet_size)))
    }
}

impl Encoder<Packet> for Codec {
    type Error = Error;
    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        println!("{item:?}");
        Ok(())
    }
}

use super::{decode_len, decode_string, Error, Packet};
use bytes::{Buf, Bytes};
use std::fmt::format;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::Decoder;

#[derive(Debug)]
pub enum Version {
    V3,
    V5,
}

pub struct Codec;

impl Decoder for Codec {
    type Item = (Packet, u32);
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check minimum length
        if src.len() < 2 {
            return Ok(None);
        }

        // Check connect package type
        let mut bytes = src.as_ref();
        if bytes[0] != 0x10 {
            return Err(Error::MalformedPacket);
        }

        // Decode remaining length
        match decode_len(&bytes[1..])? {
            Some((_, len_len)) => {
                bytes.advance(len_len + 1);
                if bytes.len() < 9 {
                    return Ok(None);
                }

                // Decode protocol
                let mut bytes = Bytes::copy_from_slice(bytes);
                let protocol = decode_string(&mut bytes)?;
                if protocol != "MQTT" && protocol != "MQIsdp" {
                    return Err(Error::ProtocolError(format!("[protocol: {}]", protocol)));
                }

                // Decode Level
                match bytes[0] {
                    3 => Ok(Some((Packet::Version(Version::V3), 0))),
                    4 => Ok(Some((Packet::Version(Version::V3), 0))),
                    5 => Ok(Some((Packet::Version(Version::V5), 0))),
                    level => Err(Error::UnsupportedProtocolVersion(format!("{}", level))),
                }
            }
            None => Ok(None),
        }
    }
}

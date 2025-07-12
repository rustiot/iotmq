mod codec;
pub mod v3;
pub mod v5;
pub mod version;

pub use codec::*;
pub use version::Version;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use num_enum::TryFromPrimitive;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Malformed packet")]
    MalformedPacket,
    #[error("Invalid protocol: {0}")]
    ProtocolError(String),
    #[error("Unsupported protocol version: {0}")]
    UnsupportedProtocolVersion(String),
    #[error("Client disconnect: {0}")]
    Disconnect(String),
    #[error("Length too big")]
    LenTooLong,
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce,
    ExactlyOnce,
}

impl Default for QoS {
    fn default() -> Self {
        Self::AtMostOnce
    }
}

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum PacketType {
    Reserved = 0,
    Connect,
    ConnAck,
    Publish,
    PubAck,
    PubRec,
    PubRel,
    PubComp,
    Subscribe,
    SubAck,
    Unsubscribe,
    UnsubAck,
    PingReq,
    PingResp,
    Disconnect,
    Auth,
}

// MQTT Packet
#[derive(Debug)]
pub enum Packet {
    Version(Version),
    Connect(Connect),
    ConnAck(ConnAck),
    // Publish(Publish),
    // PubAck(PubAck),
    // PubRec(PubRec),
    // PubRel(PubRel),
    // PubComp(PubComp),
    // Subscribe(Subscribe),
    // SubAck(SubAck),
    // Unsubscribe(Unsubscribe),
    // UnsubAck(UnsubAck),
    // PingReq,
    // PingResp,
    // Disconnect(Disconnect),
    // Auth(Auth)
}

// CONNECT Packet
#[derive(Debug, Default)]
pub struct Connect {
    pub protocol_name: String,
    pub protocol_level: String,
    pub username_flag: bool,
    pub password_flag: bool,
    pub will_retain: bool,
    pub will_qos: QoS,
    pub will_flag: bool,
    pub clean_start: bool,
    pub keepalive: u16,
    pub properties: Option<v5::ConnectProperties>,
    pub client_id: String,
    pub will_properties: Option<v5::WillProperties>,
    pub will_topic: String,
    pub will_payload: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Property {
    PayloadFormatIndicator = 0x01,
    MessageExpiryInterval = 0x02,
    ContentType = 0x03,
    ResponseTopic = 0x08,
    CorrelationData = 0x09,
    SubIdentifier = 0x0B,
    SessionExpiryInterval = 0x11,
    AssignedClientIdentifier = 0x12,
    ServerKeepAlive = 0x13,
    AuthMethod = 0x15,
    AuthData = 0x16,
    RequestProblemInfo = 0x17,
    WillDelayInterval = 0x18,
    RequestResponseInfo = 0x19,
    ResponseInfo = 0x1A,
    ServerReference = 0x1C,
    ReasonString = 0x1F,
    ReceiveMaximum = 0x21,
    TopicAliasMaximum = 0x22,
    TopicAlias = 0x23,
    MaximumQoS = 0x24,
    RetainAvailable = 0x25,
    UserProperty = 0x26,
    MaxPacketSize = 0x27,
    WildcardSubAvailable = 0x28,
    SubIdentifierAvailable = 0x29,
    SharedSubAvailable = 0x2A,
}

#[derive(Debug, Default)]
pub struct ConnAck {
    pub session_present: bool,
    pub reason_code: u8,
    pub properties: Option<v5::ConnAckProperties>,
}

// Protocol level
struct Level(u8);
impl Level {
    fn str(self) -> &'static str {
        match self {
            Level(3) => "3.1",
            Level(4) => "3.1.1",
            _ => "5.0",
        }
    }
}

// Decode length
pub fn decode_len(src: &[u8]) -> Result<Option<(usize, usize)>, Error> {
    let mut len = 0;
    let mut len_len = 0;
    let mut shift = 0;
    for byte in src {
        len_len += 1;
        len |= ((byte & 0x7F) as usize) << shift;
        if byte & 0x80 == 0 {
            return Ok(Some((len, len_len)));
        }
        shift += 7;
        if shift > 21 {
            return Err(Error::MalformedPacket);
        }
    }
    Ok(None)
}

// Encode length
fn encode_len(dst: &mut BytesMut, mut len: usize) -> Result<(), Error> {
    if len > 268_435_455 {
        return Err(Error::LenTooLong);
    }

    loop {
        let mut byte = (len % 128) as u8;
        len /= 128;
        if len > 0 {
            byte |= 0x80;
        }
        dst.put_u8(byte);
        if len == 0 {
            break;
        }
    }
    Ok(())
}

// Length size
fn len_len(mut len: usize) -> usize {
    let mut len_len = 1;
    while len >= 128 {
        len /= 128;
        len_len += 1;
    }
    len_len
}

// Decode string
fn decode_string(src: &mut Bytes) -> Result<String, Error> {
    let len = src.get_u16() as usize;
    let bytes = src.split_to(len);
    let str = String::from_utf8(bytes.to_vec()).map_err(|e| Error::ProtocolError(e.to_string()))?;
    Ok(str)
}

// Encode string
fn encode_string(dst: &mut BytesMut, str: &str) {
    dst.put_u16(str.len() as u16);
    dst.extend_from_slice(str.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_len_len() {
        assert_eq!(len_len(0), 1);
        assert_eq!(len_len(128), 2);
        assert_eq!(len_len(16384), 3);
        assert_eq!(len_len(2097152), 4);
    }

    #[test]
    fn test_encode_len() {
        let mut buf = BytesMut::new();
        assert!(matches!(encode_len(&mut buf, 268_435_456), Err(Error::LenTooLong)));

        let mut buf = BytesMut::new();
        assert!(encode_len(&mut buf, 0).is_ok());
        assert_eq!(buf[..], [0x00]);

        let mut buf = BytesMut::new();
        assert!(encode_len(&mut buf, 1).is_ok());
        assert_eq!(buf[..], [0x01]);

        let mut buf = BytesMut::new();
        assert!(encode_len(&mut buf, 128).is_ok());
        assert_eq!(buf[..], [0x80, 0x01]);

        let mut buf = BytesMut::new();
        assert!(encode_len(&mut buf, 268_435_455).is_ok());
        assert_eq!(buf[..], [0xFF, 0xFF, 0xFF, 0x7F]);
    }

    #[test]
    fn test_decode_len() {
        let src = &[];
        assert!(matches!(decode_len(src), Ok(None)));

        let src = &[0x01];
        assert!(matches!(decode_len(src), Ok(Some((1, 1)))));

        let src = &[0x80, 0x01];
        assert!(matches!(decode_len(src), Ok(Some((128, 2)))));

        let src = &[0xFF, 0xFF, 0xFF, 0x7F];
        assert!(matches!(decode_len(src), Ok(Some((268_435_455, 4)))));

        let src = &[0xFF, 0xFF, 0xFF, 0xFF];
        assert!(matches!(decode_len(src), Err(Error::MalformedPacket)));
    }
}

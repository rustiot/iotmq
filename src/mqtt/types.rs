use num_enum::TryFromPrimitive;

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum ProtocolLevel {
    V31 = 3,
    V311,
    V5,
}
impl Default for ProtocolLevel {
    fn default() -> Self {
        Self::V5
    }
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

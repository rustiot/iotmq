mod codec;
mod connect;

pub use codec::Codec;
use connect::Connect;

#[derive(Debug)]
pub enum Packet {
    Connect(Connect),
    // ConnAck(ConnAck),
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

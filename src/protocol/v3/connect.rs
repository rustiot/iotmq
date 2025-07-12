use crate::protocol::{decode_len, decode_string, Connect, Error, Level, Property, QoS};
use bytes::{Buf, Bytes};

pub fn decode(mut src: Bytes) -> Result<Connect, Error> {
    let mut connect = Connect { ..Default::default() };

    // Protocol
    connect.protocol_name = decode_string(&mut src)?;
    let level = src.get_u8();
    connect.protocol_level = Level(level).str().to_string();

    // Connect Flags
    let connect_flags = src.get_u8();
    connect.username_flag = connect_flags & 0x80 > 0;
    connect.password_flag = connect_flags & 0x40 > 0;
    connect.will_retain = connect_flags & 0x20 > 0;
    let qos = (connect_flags & 0x18) >> 3;
    connect.will_qos =
        QoS::try_from(qos).map_err(|_| Error::ProtocolError(format!("[QoS: {}]", qos)))?;
    connect.will_flag = connect_flags & 0x04 > 0;
    connect.clean_start = connect_flags & 0x02 > 0;

    // Keep Alive
    connect.keepalive = src.get_u16();

    // Client ID
    connect.client_id = decode_string(&mut src)?;

    // Will
    if connect.will_flag {
        connect.will_topic = decode_string(&mut src)?;
        connect.will_payload = decode_string(&mut src)?;
    }

    // User Name
    if connect.username_flag {
        connect.username = Some(decode_string(&mut src)?);
    }

    // Password
    if connect.password_flag {
        connect.password = Some(decode_string(&mut src)?);
    }

    Ok(connect)
}

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

    // Properties
    connect.properties = ConnectProperties::decode(&mut src)?;

    // Client ID
    connect.client_id = decode_string(&mut src)?;

    // Will
    if connect.will_flag {
        connect.will_properties = WillProperties::decode(&mut src)?;
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

#[derive(Debug, Default)]
pub struct ConnectProperties {
    pub session_expiry_interval: Option<u32>,
    pub receive_max: Option<u16>,
    pub max_packet_size: Option<u32>,
    pub topic_alias_max: Option<u16>,
    pub request_response_info: Option<u8>,
    pub request_problem_info: Option<u8>,
    pub user_property: Vec<(String, String)>,
    pub auth_method: Option<String>,
    pub auth_data: Option<Vec<u8>>,
}

impl ConnectProperties {
    fn new() -> Self {
        Self { ..Default::default() }
    }

    fn decode(src: &mut Bytes) -> Result<Option<Self>, Error> {
        let bytes = src.as_ref();
        let (len, len_len) = match decode_len(bytes)? {
            Some(len) => len,
            None => return Err(Error::MalformedPacket),
        };
        src.advance(len_len);
        if len == 0 {
            return Ok(None);
        }

        let mut src = src.split_to(len);
        let mut prop = Self::new();

        loop {
            if !src.has_remaining() {
                return Ok(Some(prop));
            }

            let id = src.get_u8();
            let property = Property::try_from(id).map_err(|_| Error::MalformedPacket)?;
            match property {
                Property::SessionExpiryInterval => {
                    prop.session_expiry_interval = Some(src.get_u32());
                }

                Property::ReceiveMaximum => {
                    prop.receive_max = Some(src.get_u16());
                }

                Property::MaxPacketSize => {
                    prop.max_packet_size = Some(src.get_u32());
                }

                Property::TopicAliasMaximum => {
                    prop.topic_alias_max = Some(src.get_u16());
                }

                Property::RequestResponseInfo => {
                    prop.request_response_info = Some(src.get_u8());
                }

                Property::RequestProblemInfo => {
                    prop.request_problem_info = Some(src.get_u8());
                }

                Property::UserProperty => {
                    let k = decode_string(&mut src)?;
                    let v = decode_string(&mut src)?;
                    prop.user_property.push((k, v));
                }

                Property::AuthMethod => {
                    prop.auth_method = Some(decode_string(&mut src)?);
                }

                Property::AuthData => {
                    let len = src.get_u16() as usize;
                    let read = src.split_to(len);
                    prop.auth_data = Some(read.to_vec());
                }
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct WillProperties {
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Vec<u8>>,
    pub will_delay_interval: Option<u32>,
    pub message_expiry_interval: Option<u32>,
    pub payload_format_indicator: Option<u8>,
    pub user_property: Vec<(String, String)>,
}

impl WillProperties {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    fn decode(src: &mut Bytes) -> Result<Option<Self>, Error> {
        let bytes = src.as_ref();
        let (len, len_len) = match decode_len(bytes)? {
            Some(len) => len,
            None => return Err(Error::MalformedPacket),
        };
        src.advance(len_len);
        if len == 0 {
            return Ok(None);
        }

        let mut src = src.split_to(len);
        let mut prop = Self::new();

        loop {
            if !src.has_remaining() {
                return Ok(Some(prop));
            }

            let id = src.get_u8();
            let property = Property::try_from(id).map_err(|_| Error::MalformedPacket)?;
            match property {
                Property::ContentType => {
                    prop.content_type = Some(decode_string(&mut src)?);
                }

                Property::ResponseTopic => {
                    prop.response_topic = Some(decode_string(&mut src)?);
                }

                Property::CorrelationData => {
                    let len = src.get_u16() as usize;
                    let read = src.split_to(len);
                    prop.correlation_data = Some(read.to_vec())
                }

                Property::WillDelayInterval => {
                    prop.will_delay_interval = Some(src.get_u32());
                }

                Property::MessageExpiryInterval => {
                    prop.message_expiry_interval = Some(src.get_u32());
                }

                Property::PayloadFormatIndicator => {
                    prop.payload_format_indicator = Some(src.get_u8());
                }

                Property::UserProperty => {
                    let k = decode_string(&mut src)?;
                    let v = decode_string(&mut src)?;
                    prop.user_property.push((k, v));
                }
                _ => unreachable!(),
            }
        }
    }
}

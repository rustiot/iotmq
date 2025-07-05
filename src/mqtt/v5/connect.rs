use crate::mqtt::{ProtocolLevel, QoS};

// CONNECT Packet
#[derive(Debug, Default)]
pub struct Connect {
    pub protocol_name: String,
    pub protocol_level: ProtocolLevel,
    pub username_flag: bool,
    pub password_flag: bool,
    pub will_flag: bool,
    pub clean_start: bool,
    pub keepalive: u16,
    pub properties: Option<Properties>,
    pub client_id: String,
    pub will: Option<Will>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Default)]
pub struct Properties {
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

#[derive(Debug, Default)]
pub struct Will {
    pub qos: QoS,
    pub retain: bool,
    pub topic: String,
    pub payload: String,

    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Vec<u8>>,
    pub will_delay_interval: Option<u32>,
    pub message_expiry_interval: Option<u32>,
    pub payload_format_indicator: Option<u8>,
    pub user_property: Vec<(String, String)>,
}

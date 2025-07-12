use crate::protocol::{encode_len, encode_string, len_len, ConnAck, Error, PacketType, Property};
use bytes::{BufMut, BytesMut};

pub fn encode(packet: ConnAck, dst: &mut BytesMut) -> Result<(), Error> {
    let mut prop_len = 0;
    if let Some(ref prop) = packet.properties {
        prop_len = prop.len();
    }

    let len = 2 + len_len(prop_len) + prop_len;
    dst.put_u8((PacketType::ConnAck as u8) << 4);
    encode_len(dst, len)?;
    dst.put_u8(packet.session_present as u8);
    dst.put_u8(packet.reason_code);

    encode_len(dst, prop_len)?;
    if let Some(prop) = packet.properties {
        prop.encode(dst);
    }

    Ok(())
}

#[derive(Debug, Default)]
pub struct ConnAckProperties {
    pub session_expiry_interval: Option<u32>,
    pub assigned_client_identifier: Option<String>,
    pub server_keep_alive: Option<u16>,
    pub auth_method: Option<String>,
    pub auth_data: Option<Vec<u8>>,
    pub response_info: Option<String>,
    pub server_reference: Option<String>,
    pub reason_string: Option<String>,
    pub receive_maximum: Option<u16>,
    pub topic_alias_max: Option<u16>,
    pub maximum_qos: Option<u8>,
    pub retain_available: Option<u8>,
    pub user_property: Vec<(String, String)>,
    pub max_packet_size: Option<u32>,
    pub wildcard_sub_available: Option<u8>,
    pub sub_identifier_available: Option<u8>,
    pub shared_sub_available: Option<u8>,
}

impl ConnAckProperties {
    pub fn encode(self, dst: &mut BytesMut) {
        if let Some(session_expiry_interval) = self.session_expiry_interval {
            dst.put_u8(Property::SessionExpiryInterval as u8);
            dst.put_u32(session_expiry_interval);
        }

        if let Some(assigned_client_identifier) = self.assigned_client_identifier {
            dst.put_u8(Property::AssignedClientIdentifier as u8);
            encode_string(dst, &assigned_client_identifier);
        }

        if let Some(server_keep_alive) = self.server_keep_alive {
            dst.put_u8(Property::ServerKeepAlive as u8);
            dst.put_u16(server_keep_alive);
        }

        if let Some(auth_method) = self.auth_method {
            dst.put_u8(Property::AuthMethod as u8);
            encode_string(dst, &auth_method);
        }

        if let Some(auth_data) = self.auth_data {
            dst.put_u8(Property::AuthData as u8);
            dst.put_u16(auth_data.len() as u16);
            dst.extend_from_slice(&auth_data);
        }

        if let Some(response_info) = self.response_info {
            dst.put_u8(Property::ResponseInfo as u8);
            encode_string(dst, &response_info);
        }

        if let Some(server_reference) = self.server_reference {
            dst.put_u8(Property::ServerReference as u8);
            encode_string(dst, &server_reference);
        }

        if let Some(reason_string) = self.reason_string {
            dst.put_u8(Property::ReasonString as u8);
            encode_string(dst, &reason_string);
        }

        if let Some(receive_maximum) = self.receive_maximum {
            dst.put_u8(Property::ReceiveMaximum as u8);
            dst.put_u16(receive_maximum);
        }

        if let Some(topic_alias_max) = self.topic_alias_max {
            dst.put_u8(Property::TopicAliasMaximum as u8);
            dst.put_u16(topic_alias_max);
        }

        if let Some(maximum_qos) = self.maximum_qos {
            dst.put_u8(Property::MaximumQoS as u8);
            dst.put_u8(maximum_qos);
        }

        if let Some(retain_available) = self.retain_available {
            dst.put_u8(Property::RetainAvailable as u8);
            dst.put_u8(retain_available);
        }

        for (k, v) in self.user_property.iter() {
            dst.put_u8(Property::UserProperty as u8);
            encode_string(dst, k);
            encode_string(dst, v);
        }

        if let Some(max_packet_size) = self.max_packet_size {
            dst.put_u8(Property::MaxPacketSize as u8);
            dst.put_u32(max_packet_size);
        }

        if let Some(wildcard_sub_available) = self.wildcard_sub_available {
            dst.put_u8(Property::WildcardSubAvailable as u8);
            dst.put_u8(wildcard_sub_available);
        }

        if let Some(sub_identifier_available) = self.sub_identifier_available {
            dst.put_u8(Property::SubIdentifierAvailable as u8);
            dst.put_u8(sub_identifier_available);
        }

        if let Some(shared_sub_available) = self.shared_sub_available {
            dst.put_u8(Property::SharedSubAvailable as u8);
            dst.put_u8(shared_sub_available);
        }
    }

    pub fn len(&self) -> usize {
        let mut len = 0;

        if self.session_expiry_interval.is_some() {
            len += 1 + 4;
        }

        if let Some(ref assigned_client_identifier) = self.assigned_client_identifier {
            len += 1 + 2 + assigned_client_identifier.len();
        }

        if self.server_keep_alive.is_some() {
            len += 1 + 2;
        }

        if let Some(ref auth_method) = self.auth_method {
            len += 1 + 2 + auth_method.len();
        }

        if let Some(ref auth_data) = self.auth_data {
            len += 1 + 2 + auth_data.len();
        }

        if let Some(ref response_info) = self.response_info {
            len += 1 + 2 + response_info.len();
        }

        if let Some(ref server_reference) = self.server_reference {
            len += 1 + 2 + server_reference.len();
        }

        if let Some(ref reason_string) = self.reason_string {
            len += 1 + 2 + reason_string.len();
        }

        if self.receive_maximum.is_some() {
            len += 1 + 2;
        }

        if self.topic_alias_max.is_some() {
            len += 1 + 2;
        }

        if self.maximum_qos.is_some() {
            len += 1 + 1;
        }

        if self.retain_available.is_some() {
            len += 1 + 1;
        }

        for (k, v) in self.user_property.iter() {
            len += 1 + 2 + k.len() + 2 + v.len();
        }

        if self.max_packet_size.is_some() {
            len += 1 + 4;
        }

        if self.wildcard_sub_available.is_some() {
            len += 1 + 1;
        }

        if self.sub_identifier_available.is_some() {
            len += 1 + 1;
        }

        if self.shared_sub_available.is_some() {
            len += 1 + 1;
        }

        len
    }
}

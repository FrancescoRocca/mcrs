use bytes::{Buf, BytesMut};

use uuid::Uuid;

use crate::utils;

#[derive(Clone)]
pub enum ClientIntent {
    Status,
    Login,
    Transfer,
    Error,
    None,
}

impl ClientIntent {
    pub fn as_str(&self) -> &str {
        match self {
            ClientIntent::Status => "Status",
            ClientIntent::Login => "Login",
            ClientIntent::Transfer => "Transfer",
            ClientIntent::Error => "Error",
            ClientIntent::None => "None",
        }
    }
}

pub enum Packet {
    Handshake {
        id: u32,
        protocol_version: u32,
        server_address: String,
        server_port: u16,
        intent: ClientIntent,
        length: usize,
    },
    Status,
    Ping {
        id: u32,
        data: BytesMut,
        length: usize,
    },
    Login {
        id: u32,
        name: String,
        uuid: Uuid,
        length: usize,
    },
    None,
}

impl Packet {
    pub async fn parse(data: &mut BytesMut, intent: &ClientIntent) -> Self {
        utils::print_hex(data, data.len());

        let (packet_length, packet_id) = Packet::parse_headers(data);

        match intent {
            ClientIntent::None => {
                /* Handshake (first packet) */
                Packet::parse_handshake(data, packet_length as usize, packet_id)
            }
            ClientIntent::Status => {
                if data.is_empty() {
                    /* Ping */
                    Packet::Ping {
                        id: packet_id,
                        data: data.clone(),
                        length: packet_length as usize,
                    }
                } else {
                    /* Status */
                    Packet::Status
                }
            }
            ClientIntent::Login => {
                /* Login */
                Packet::parse_login(data, packet_length as usize, packet_id)
            }
            ClientIntent::Transfer => Packet::None,
            ClientIntent::Error => Packet::None,
        }
    }

    fn parse_headers(data: &mut BytesMut) -> (u32, u32) {
        let packet_length = utils::read_varint(data);
        let packet_id = utils::read_varint(data);

        (packet_length, packet_id)
    }

    fn parse_handshake(data: &mut BytesMut, length: usize, id: u32) -> Packet {
        let protocol_version = utils::read_varint(data);

        let address_len = utils::read_varint(data);

        let addr_bytes = data.split_to(address_len as usize);
        let server_address = String::from_utf8_lossy(&addr_bytes).to_string();
        let server_port = u16::from_be_bytes([data.get_u8(), data.get_u8()]);

        let intent = utils::read_varint(data);
        let intent = match intent {
            0x01 => ClientIntent::Status,
            0x02 => ClientIntent::Login,
            0x03 => ClientIntent::Transfer,
            _ => ClientIntent::Error,
        };

        Packet::Handshake {
            id,
            protocol_version,
            server_address,
            server_port,
            intent,
            length,
        }
    }

    fn parse_login(data: &mut BytesMut, length: usize, id: u32) -> Packet {
        let name_len = utils::read_varint(data);
        let name_bytes = data.split_to(name_len as usize);
        let name = String::from_utf8_lossy(&name_bytes).to_string();

        let uuid_bytes = data.split_to(16);
        let uuid = Uuid::from_slice(&uuid_bytes).unwrap();

        Packet::Login {
            id,
            name,
            uuid,
            length,
        }
    }
}

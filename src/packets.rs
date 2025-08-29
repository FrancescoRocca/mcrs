use bytes::{Buf, BufMut, BytesMut};
use tokio::io::AsyncWriteExt;

use std::io::Error;
use uuid::Uuid;

use crate::{
    json,
    server::MinecraftConnection,
    utils::{self, write_varint},
};

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
    Ping,
    Login {
        id: u32,
        name: String,
        uuid: Uuid,
        length: usize,
    },
    None,
}

impl Packet {
    pub async fn parse(conn: &mut MinecraftConnection) -> Self {
        print_hex(&conn.buffer, conn.buffer.len());

        let (packet_length, packet_id) = parse_headers(&mut conn.buffer);

        match packet_id {
            0x00 => {
                /* Status Request */
                if packet_length == 1 {
                    if let Err(e) = send_status(conn).await {
                        eprintln!("Error while sending status request: {}", e);
                    }

                    return Packet::Status;
                }

                match conn.intent {
                    ClientIntent::None => {
                        /* Handshake */
                        parse_handshake(
                            &mut conn.buffer,
                            packet_id,
                            packet_length as usize,
                            &mut conn.protocol_version,
                            &mut conn.intent,
                        )
                    }
                    ClientIntent::Login => {
                        /* Login */
                        parse_login(&mut conn.buffer, packet_id, packet_length as usize)
                    }
                    _ => Packet::None,
                }
            }
            0x01 => {
                match conn.intent {
                    ClientIntent::Status => {
                        /* Ping */
                        if let Err(e) = send_ping(conn).await {
                            eprintln!("Error while sending ping: {}", e);
                            return Packet::None;
                        }

                        return Packet::Ping;
                    }
                    _ => {
                        eprintln!("Not implemented.");
                    }
                }

                Packet::None
            }
            _ => {
                println!("[debug] Not implemented");
                Packet::None
            }
        }
    }
}

fn print_hex(data: &[u8], length: usize) {
    println!("[debug] Packet:");
    for b in data.iter().take(length) {
        print!("{:#x} ", b);
    }
    println!();
}

fn parse_headers(data: &mut BytesMut) -> (u32, u32) {
    let packet_length = utils::read_varint(data);
    let packet_id = utils::read_varint(data);

    (packet_length, packet_id)
}

fn parse_handshake(
    data: &mut BytesMut,
    id: u32,
    length: usize,
    cprotocol_version: &mut u32,
    next_intent: &mut ClientIntent,
) -> Packet {
    let protocol_version = utils::read_varint(data);
    *cprotocol_version = protocol_version;

    let address_len = utils::read_varint(data);

    let mut addr_bytes: Vec<u8> = Vec::new();

    for _ in 0..address_len {
        addr_bytes.push(data.get_u8());
    }

    let server_address = std::str::from_utf8(&addr_bytes).unwrap_or("").to_string();
    let server_port = u16::from_be_bytes([data.get_u8(), data.get_u8()]);

    let intent = utils::read_varint(data);
    let intent = match intent {
        0x01 => ClientIntent::Status,
        0x02 => ClientIntent::Login,
        0x03 => ClientIntent::Transfer,
        _ => ClientIntent::Error,
    };

    *next_intent = intent.clone();

    Packet::Handshake {
        id,
        protocol_version,
        server_address,
        server_port,
        intent,
        length,
    }
}

fn parse_login(data: &mut BytesMut, id: u32, length: usize) -> Packet {
    let name_len = utils::read_varint(data);
    let mut name_bytes: Vec<u8> = Vec::new();

    for _ in 0..name_len {
        name_bytes.push(data.get_u8());
    }

    let name = std::str::from_utf8(&name_bytes).unwrap_or("").to_string();

    let mut uuid_bytes: Vec<u8> = Vec::new();
    for _ in 0..16 {
        uuid_bytes.push(data.get_u8());
    }
    let uuid = Uuid::from_bytes(uuid_bytes.try_into().unwrap());

    Packet::Login {
        id,
        name,
        uuid,
        length,
    }
}

pub async fn send_ping(conn: &mut MinecraftConnection) -> Result<(), Error> {
    let packet_size = utils::write_varint(1 + 8);

    let mut out = BytesMut::with_capacity(16);
    out.put_slice(&packet_size);
    out.put_u8(0x01);
    out.put_slice(&conn.buffer[..8]);

    conn.connection.write_all_buf(&mut out).await?;
    conn.buffer.clear();

    Ok(())
}

pub async fn send_status(conn: &mut MinecraftConnection) -> Result<(), Error> {
    let status = json::Status::new("1.21.8", conn.protocol_version, 20, 0, "Hello, World!");
    let status_json = status.json();

    let packet_len = utils::write_varint(
        utils::varint_size(status_json.len() as u32) + 1 + status_json.len() as u32,
    );
    let status_len = utils::write_varint(status_json.len() as u32);

    conn.connection.write_all(&packet_len).await?;
    conn.connection.write_all(&[0x00]).await?;
    conn.connection.write_all(&status_len).await?;
    conn.connection.write_all(status_json.as_bytes()).await?;

    Ok(())
}

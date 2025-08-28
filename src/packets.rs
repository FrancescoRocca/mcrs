use std::io::Write;

use crate::{json, server::MinecraftConnection, utils};

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
    None,
}

impl Packet {
    pub fn parse(conn: &mut MinecraftConnection) -> Self {
        let start = conn.bytes_read;
        let available = conn.length - start;
        if available == 0 {
            return Packet::None;
        }
        let buf = &conn.buffer[start..start + available];

        print_hex(buf, buf.len());

        /* Packet length */
        let (packet_length, len_len) = utils::read_varint(buf);

        /* Packet id */
        let (packet_id, id_len) = utils::read_varint(&buf[len_len..]);
        let advance = len_len + packet_length as usize;

        match packet_id {
            0x00 => {
                /* Status Request */
                if packet_length == 1 {
                    conn.bytes_read += advance;
                    send_status(conn);

                    return Packet::Status;
                }

                /* Handshake */
                let payload_start = len_len + id_len;
                let payload_len = packet_length as usize - id_len;
                let payload = &buf[payload_start..payload_start + payload_len];

                let packet = parse_handshake(
                    payload,
                    packet_id,
                    packet_length as usize,
                    &mut conn.protocol_version,
                    &mut conn.intent,
                );
                conn.bytes_read += advance;

                packet
            }
            0x01 => {
                match conn.intent {
                    ClientIntent::Status => {
                        /* Ping */
                        let packet_start = start;
                        send_ping(conn, packet_start, advance);
                        conn.bytes_read += advance;

                        return Packet::Ping;
                    }
                    _ => {
                        conn.bytes_read += advance;
                    }
                }

                Packet::None
            }
            _ => {
                conn.bytes_read += advance;
                println!("[debug] Not implemented");
                Packet::None
            }
        }
    }
}

fn print_hex(data: &[u8], length: usize) {
    println!("[debug] Packet:");
    for i in 0..length {
        print!("{:#x} ", data[i]);
    }
    println!();
}

fn parse_handshake(
    data: &[u8],
    id: u32,
    length: usize,
    cprotocol_version: &mut u32,
    next_intent: &mut ClientIntent,
) -> Packet {
    let mut offset = 0;

    let (protocol_version, off) = utils::read_varint(data);
    *cprotocol_version = protocol_version;
    offset += off;

    let (address_len, off) = utils::read_varint(&data[offset..]);
    offset += off;
    let mut server_address = String::new();
    for x in 0..address_len {
        server_address.push(data[offset + (x as usize)] as char);
    }
    offset += address_len as usize;

    let mut server_port: u16 = (data[offset] as u16) << 8;
    offset = offset + 1;
    server_port |= data[offset] as u16;
    offset += 1;

    let (intent, _) = utils::read_varint(&data[offset..]);
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

pub fn send_ping(conn: &mut MinecraftConnection, packet_start: usize, packet_len: usize) {
    let end = packet_start + packet_len;
    let slice = &conn.buffer[packet_start..end];
    conn.connection.write_all(slice).unwrap();
}

pub fn send_status(conn: &mut MinecraftConnection) {
    println!("[debug] Sending response...");
    let status = json::Status::new("1.21.8", conn.protocol_version, 20, 0, "Hello, World!");
    let status_json = status.json();

    let packet_len = utils::write_varint(
        utils::varint_size(status_json.len() as u32) + 1 + status_json.len() as u32,
    );
    conn.connection.write_all(&packet_len).unwrap();
    conn.connection.write_all(&[0x00]).unwrap();
    let status_len = utils::write_varint(status_json.len() as u32);
    conn.connection.write_all(&status_len).unwrap();
    conn.connection.write_all(status_json.as_bytes()).unwrap();
}

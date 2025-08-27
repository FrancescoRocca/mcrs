use std::io::Write;

use crate::{json, server::MinecraftConnection};

#[derive(Clone)]
pub enum ClientIntent {
    Status = 0,
    Login = 1,
    Transfer = 2,
    Error = 4,
}

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

#[derive(Clone)]
pub struct Packet {
    id: u32,
    protocol_version: u32,
    server_address: String,
    server_port: u16,
    intent: ClientIntent,
    length: usize,
}

impl Packet {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    pub fn server_address(&self) -> &String {
        &self.server_address
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    pub fn intent(&self) -> &str {
        match self.intent {
            ClientIntent::Status => "Status",
            ClientIntent::Login => "Login",
            ClientIntent::Transfer => "Transfer",
            ClientIntent::Error => "Error",
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn new() -> Self {
        Self {
            id: 0,
            protocol_version: 0,
            server_address: String::new(),
            server_port: 0,
            intent: ClientIntent::Error,
            length: 0,
        }
    }

    pub fn parse(conn: &mut MinecraftConnection) -> Option<Self> {
        print_hex(&conn.buffer, conn.length);
        let mut offset = 0;

        let (packet_length, off) = read_varint(&conn.buffer);
        println!("[debug] Packet len: {packet_length}");
        offset += off;

        let (packet_id, off) = read_varint(&conn.buffer[offset..]);
        println!("[debug] Packet id: {:#x}", packet_id);
        offset += off;

        match packet_id {
            0x00 => {
                /* Status Request */
                if packet_length == 1 {
                    println!("[debug] Status Request");
                    conn.bytes_read += packet_length as usize + 1;

                    println!("[debug] Sending response...");
                    let status = json::Status::new(
                        "1.21.8",
                        conn.last_packet.protocol_version(),
                        20,
                        0,
                        "Hello, World!",
                    );
                    let status_json = status.json();

                    let packet_len = write_varint(
                        varint_size(status_json.len() as u32) + 1 + status_json.len() as u32,
                    );
                    conn.connection.write_all(&packet_len).unwrap();
                    conn.connection.write_all(&[0x00]).unwrap();
                    let status_len = write_varint(status_json.len() as u32);
                    conn.connection.write_all(&status_len).unwrap();
                    conn.connection.write_all(status_json.as_bytes()).unwrap();

                    return None;
                }

                /* Handshake */
                println!("[debug] Handshake");
                let mut packet = parse_handshake(&conn.buffer[offset..], packet_id);
                packet.length = packet_length as usize;
                conn.bytes_read += packet_length as usize + 1;
                conn.last_packet = packet.clone();

                Some(packet)
            }
            0x01 => {
                /* Handle Login */
                None
            }
            _ => {
                println!("[debug] Not implemented");
                Some(Packet {
                    id: 0,
                    protocol_version: 0,
                    server_address: String::from("undefined"),
                    server_port: 25565,
                    intent: ClientIntent::Error,
                    length: 0,
                })
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

fn read_varint(data: &[u8]) -> (u32, usize) {
    let mut value: u32 = 0;
    let mut pos: u32 = 0;
    let mut offset = 0;

    loop {
        let byte = data[offset];
        offset += 1;

        value |= ((byte & SEGMENT_BITS) as u32) << pos;

        if (byte & CONTINUE_BIT) == 0 {
            break;
        }

        pos += 7;
    }

    (value, offset)
}

fn varint_size(mut value: u32) -> u32 {
    let mut size = 0;

    loop {
        value >>= 7;
        size += 1;

        if value == 0 {
            break;
        }
    }

    size
}

fn write_varint(mut value: u32) -> Vec<u8> {
    let mut buffer = Vec::new();

    loop {
        let mut tmp: u8 = (value & SEGMENT_BITS as u32) as u8;
        value >>= 7;

        if value != 0 {
            tmp |= CONTINUE_BIT;
        }

        buffer.push(tmp);

        if value == 0 {
            break;
        }
    }

    buffer
}

fn parse_handshake(data: &[u8], id: u32) -> Packet {
    let mut offset = 0;

    let (protocol_version, off) = read_varint(data);
    offset += off;

    let (address_len, off) = read_varint(&data[offset..]);
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

    let (intent, _) = read_varint(&data[offset..]);
    let intent = match intent {
        0x01 => ClientIntent::Status,
        0x02 => ClientIntent::Login,
        0x03 => ClientIntent::Transfer,
        _ => ClientIntent::Error,
    };

    Packet {
        id,
        protocol_version,
        server_address,
        server_port,
        intent,
        length: 0,
    }
}

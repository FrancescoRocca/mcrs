use std::io::Write;

use crate::{json, server::MinecraftConnection, utils};

#[derive(Clone)]
pub enum ClientIntent {
    Status = 0,
    Login = 1,
    Transfer = 2,
    Error = 4,
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
    // Status?
    Ping {
        id: u32,
        payload: Vec<u8>,
    },
}

impl Packet {
    pub fn parse(conn: &mut MinecraftConnection) -> Option<Self> {
        print_hex(&conn.buffer, conn.length);
        let mut offset = 0;

        /* Packet length */
        let (packet_length, off) = utils::read_varint(&conn.buffer);
        println!("[debug] Packet len: {packet_length}");
        offset += off;

        /* Packet id */
        let (packet_id, off) = utils::read_varint(&conn.buffer[offset..]);
        println!("[debug] Packet id: {:#x}", packet_id);
        offset += off;

        match packet_id {
            0x00 => {
                /* Status Request */
                if packet_length == 1 {
                    conn.bytes_read += packet_length as usize + 1;
                    println!("[debug] Status Request ({} bytes)", conn.bytes_read);
                    send_status(conn);

                    return None;
                }

                /* Handshake */
                let packet = parse_handshake(&conn.buffer[offset..], packet_id);
                //packet.length = packet_length as usize;
                conn.bytes_read += packet_length as usize + 1;
                println!("[debug] Handshake ({} bytes)", conn.bytes_read);
                //conn.last_packet = packet;

                Some(packet)
            }
            0x01 => {
                /*match conn.last_packet.intent {
                    ClientIntent::Status => {
                        /* Ping */
                        conn.bytes_read += conn.length;
                        println!("[debug] Ping ({} bytes)", conn.bytes_read);
                        send_ping(conn);
                    }
                    _ => {}
                }*/

                None
            }
            _ => {
                println!("[debug] Not implemented");
                None
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

fn parse_handshake(data: &[u8], id: u32) -> Packet {
    let mut offset = 0;

    let (protocol_version, off) = utils::read_varint(data);
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

    Packet::Handshake {
        id,
        protocol_version,
        server_address,
        server_port,
        intent,
        length: 0,
    }
}

pub fn send_ping(conn: &mut MinecraftConnection) {
    conn.connection
        .write_all(&conn.buffer[..conn.bytes_read])
        .unwrap();
}

pub fn send_status(conn: &mut MinecraftConnection) {
    println!("[debug] Sending response...");
    let status = json::Status::new("1.21.8", 771 /* TODO fix */, 20, 0, "Hello, World!");
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

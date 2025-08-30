use crate::json;
use crate::packets::Packet;
use crate::server::MinecraftConnection;
use crate::utils;
use bytes::{BufMut, BytesMut};
use std::io::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn handle_packet(
    conn: &mut MinecraftConnection,
    packet: &mut Packet,
) -> Result<(), Error> {
    match packet {
        Packet::Status => {
            println!("Status Request");
            send_status(&mut conn.connection, conn.protocol_version).await
        }
        Packet::Ping { data, .. } => {
            println!("Ping Request");
            send_ping(&mut conn.connection, data).await
        }
        Packet::Login { .. } => {
            // TODO
            println!("Login Request");
            Ok(())
        }
        _ => Err(Error::other("Handle packet not implemented")),
    }
}

async fn send_ping(conn: &mut TcpStream, data: &mut BytesMut) -> Result<(), Error> {
    let packet_size = utils::write_varint(1 + 8);

    let mut out = BytesMut::with_capacity(16);
    out.put_slice(&packet_size);
    out.put_u8(0x01);
    out.put_slice(&data[..8]);

    conn.write_all_buf(&mut out).await?;
    data.clear();

    Ok(())
}

async fn send_status(conn: &mut TcpStream, protocol_version: u32) -> Result<(), Error> {
    let status = json::Status::new("1.21.8", protocol_version, 20, 0, "Hello, World!");
    let status_json = status.json();

    let packet_len = utils::write_varint(
        utils::varint_size(status_json.len() as u32) + 1 + status_json.len() as u32,
    );
    let status_len = utils::write_varint(status_json.len() as u32);

    conn.write_all(&packet_len).await?;
    conn.write_all(&[0x00]).await?;
    conn.write_all(&status_len).await?;
    conn.write_all(status_json.as_bytes()).await?;

    Ok(())
}

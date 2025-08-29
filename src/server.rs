use bytes::BytesMut;
use std::io::Result;

use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use crate::packets::{ClientIntent, Packet};

pub struct MinecraftConnection {
    pub connection: TcpStream,
    pub buffer: BytesMut,
    pub protocol_version: u32,
    pub intent: ClientIntent,
}

impl MinecraftConnection {
    pub fn new(connection: TcpStream) -> Self {
        Self {
            connection,
            buffer: BytesMut::with_capacity(1024),
            protocol_version: 0,
            intent: ClientIntent::None,
        }
    }

    pub async fn next_packet(&mut self) -> Packet {
        if self.buffer.len() == 0 {
            return Packet::None;
        }
        Packet::parse(self).await
    }
}

pub struct MinecraftServer {
    host: String,
    port: String,
}

impl MinecraftServer {
    pub fn new(host: &str, port: &str) -> Self {
        Self {
            host: host.to_string(),
            port: port.to_string(),
        }
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> &String {
        &self.port
    }

    async fn handle_client_data(mc: &mut MinecraftConnection) {
        loop {
            let n = match mc.connection.read_buf(&mut mc.buffer).await {
                Ok(0) => {
                    println!("Client disconnected");
                    return;
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    return;
                }
            };

            println!("Read {} bytes", n);
            loop {
                match mc.next_packet().await {
                    Packet::Handshake {
                        id,
                        protocol_version,
                        server_address,
                        server_port,
                        intent,
                        length,
                    } => {
                        println!(
                            "id: {}, proto: {}, {}:{}, intent: {}, len: {}",
                            id,
                            protocol_version,
                            server_address,
                            server_port,
                            intent.as_str(),
                            length
                        );
                    }
                    Packet::Login {
                        id,
                        name,
                        uuid,
                        length,
                    } => {
                        println!("id: {} name: {} uuid: {} len: {}", id, name, uuid, length);
                    }
                    _ => {}
                };

                if mc.buffer.is_empty() {
                    break;
                }
            }
        }
    }

    #[tokio::main]
    pub async fn run(&mut self) -> Result<()> {
        let addr: String = format!("{}:{}", self.host, self.port).parse().unwrap();
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("New client: {}", addr);
            let mut mc = MinecraftConnection::new(socket);

            tokio::spawn(async move {
                MinecraftServer::handle_client_data(&mut mc).await;
            });
        }
    }
}

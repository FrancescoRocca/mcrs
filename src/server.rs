use std::io::Result;

use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use crate::packets::{ClientIntent, Packet};

pub struct MinecraftConnection {
    pub connection: TcpStream,
    pub buffer: Vec<u8>,
    pub length: usize,
    pub bytes_read: usize,
    pub protocol_version: u32,
    pub intent: ClientIntent,
}

impl MinecraftConnection {
    pub fn new(connection: TcpStream) -> Self {
        Self {
            connection,
            buffer: vec![0u8; 1024],
            length: 0,
            bytes_read: 0,
            protocol_version: 0,
            intent: ClientIntent::None,
        }
    }

    pub async fn next_packet(&mut self) -> Packet {
        if self.length == 0 {
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
            let n = match mc.connection.read(&mut mc.buffer).await {
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
            mc.length = n;
            let packet = mc.next_packet().await;
            println!("Bytes read: {}", mc.bytes_read);
            match packet {
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
                _ => {}
            }

            if mc.bytes_read == n {
                mc.bytes_read = 0;
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

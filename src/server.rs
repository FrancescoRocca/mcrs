use bytes::BytesMut;
use std::io::Result;

use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use crate::packets::{ClientIntent, Packet};
use crate::responses;

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
        if self.buffer.is_empty() {
            return Packet::None;
        }
        Packet::parse(&mut self.buffer, &self.intent).await
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

    async fn handle_client_data(conn: &mut MinecraftConnection) {
        loop {
            let n = match conn.connection.read_buf(&mut conn.buffer).await {
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
                let mut packet = conn.next_packet().await;
                match responses::handle_packet(conn, &mut packet).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("{}", e)
                    }
                }

                if conn.buffer.is_empty() {
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

use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    os::fd::AsRawFd,
};

use crate::packets::{self};

pub struct Server {
    host: String,
    port: String,
}

impl Server {
    pub fn new(host: String, port: String) -> Self {
        Server { host, port }
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> &String {
        &self.port
    }

    fn handle_client(&self, mut stream: TcpStream) {
        let mut buffer: Vec<u8> = vec![0u8; 1024];
        let n = stream.read(&mut buffer).unwrap();
        println!("Read {n} bytes");
        let packet = packets::Packet::parse(&mut stream, &buffer[..n]);

        println!(
            "{}:{} (proto v:{}, intent: {})",
            packet.server_address(),
            packet.server_port(),
            packet.protocol_version(),
            packet.intent()
        );
    }

    pub fn run(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).unwrap();

        /* Accept connections and process them */
        loop {
            match listener.accept() {
                Ok((socket, addr)) => {
                    let client_fd = socket.as_raw_fd();
                    println!("Socket: {}, Addr: {}", client_fd, addr);
                    self.handle_client(socket);
                }
                Err(e) => {
                    println!("Couldn't get client: {e:?}");
                }
            }
        }
    }
}

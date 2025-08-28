use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read};

use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use crate::packets::{ClientIntent, Packet};

const SERVER: Token = Token(0);

pub struct MinecraftConnection {
    pub connection: TcpStream,
    pub buffer: Vec<u8>,
    pub length: usize,
    pub bytes_read: usize,
    pub intent: ClientIntent,
}

impl MinecraftConnection {
    pub fn new(connection: TcpStream) -> Self {
        Self {
            connection,
            buffer: vec![0u8; 1024],
            length: 0,
            bytes_read: 0,
            intent: ClientIntent::None,
        }
    }

    pub fn read_data(&mut self) -> Result<usize, String> {
        loop {
            match self.connection.read(&mut self.buffer) {
                Ok(0) => {
                    return Err("Connection closed".to_string());
                }
                Ok(n) => {
                    return Ok(n);
                }
                Err(e) => {
                    if e.kind() == ErrorKind::Interrupted {
                        continue;
                    }

                    if e.kind() == ErrorKind::WouldBlock {
                        return Ok(0);
                    }

                    return Err(e.to_string());
                }
            }
        }
    }

    pub fn next_packet(&mut self) -> Packet {
        if self.length == 0 {
            return Packet::None;
        }
        Packet::parse(self)
    }
}

pub struct MinecraftServer {
    host: String,
    port: String,
    server: TcpListener,
    poll: Poll,
    connections: HashMap<Token, MinecraftConnection>,
    /* A counter is ok for now, using fd would be better */
    unique_token: usize,
}

impl MinecraftServer {
    pub fn new(host: &str, port: &str) -> Self {
        let addr = format!("{}:{}", host, port).parse().unwrap();
        /* TODO: improve error handling */
        Self {
            host: host.to_string(),
            port: port.to_string(),
            server: TcpListener::bind(addr).unwrap(),
            poll: Poll::new().unwrap(),
            connections: HashMap::new(),
            unique_token: 1,
        }
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> &String {
        &self.port
    }

    fn handle_client_data(minecraft_connection: &mut MinecraftConnection) -> Result<(), Error> {
        match minecraft_connection.read_data() {
            Ok(n) => {
                minecraft_connection.length = n;

                let packet = minecraft_connection.next_packet();
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
                            "id: {:#x}, proto: {}, {}:{}, intent: {}, len: {}",
                            id,
                            protocol_version,
                            server_address,
                            server_port,
                            intent.as_str(),
                            length
                        );
                    }
                    Packet::None => {
                        println!("Packet empty");
                    }
                }
                Ok(())
            }
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
        }
    }

    fn handle_new_client(&mut self) {
        let (mut connection, addr) = self.server.accept().unwrap();
        println!("New client: {}", addr);

        let token = Token(self.unique_token);
        self.unique_token += 1;

        self.poll
            .registry()
            .register(&mut connection, token, Interest::READABLE)
            .unwrap();

        let minecraft_connection = MinecraftConnection::new(connection);

        self.connections.insert(token, minecraft_connection);
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        let mut events = Events::with_capacity(128);

        /* Start listening for incoming connections */
        self.poll
            .registry()
            .register(&mut self.server, SERVER, Interest::READABLE)?;

        /* Accept connections and process them */
        loop {
            self.poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                self.handle_event(event);
            }
        }
    }

    fn handle_event(&mut self, event: &Event) {
        match event.token() {
            SERVER => {
                /* Accept connection */
                self.handle_new_client();
            }
            token => {
                /* Handle client data */
                let should_remove = {
                    let conn = self.connections.get_mut(&token).unwrap();
                    MinecraftServer::handle_client_data(conn).is_err()
                };

                if should_remove {
                    println!("Removing connection for token {:?}", token);
                    if let Some(mut conn) = self.connections.remove(&token) {
                        self.poll
                            .registry()
                            .deregister(&mut conn.connection)
                            .unwrap();
                    }
                }
            }
        }
    }
}

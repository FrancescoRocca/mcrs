use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read};

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use crate::packets::Packet;

const SERVER: Token = Token(0);

pub struct MinecraftConnection {
    pub connection: TcpStream,
    pub buffer: Vec<u8>,
    pub length: usize,
    pub bytes_read: usize,
    pub last_packet: Packet,
}

impl MinecraftConnection {
    pub fn new(connection: TcpStream) -> Self {
        Self {
            connection,
            buffer: vec![0u8; 1024],
            length: 0,
            bytes_read: 0,
            last_packet: Packet::new(),
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

    pub fn next_packet(&mut self) -> Option<Packet> {
        if self.length == 0 {
            return None;
        }
        Packet::parse(self)
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

    fn handle_client_data(
        &self,
        minecraft_connection: &mut MinecraftConnection,
    ) -> Result<(), Error> {
        match minecraft_connection.read_data() {
            Ok(0) => {
                // No data read
                Ok(())
            }
            Ok(n) => {
                minecraft_connection.length = n;

                let packet = minecraft_connection.next_packet();
                match packet {
                    Some(packet) => {
                        println!("Packet len: {}, id: {}", packet.length(), packet.id());
                        Ok(())
                    }
                    None => {
                        println!("Packet empty");
                        Ok(())
                    }
                }
            }
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
        }
    }

    fn handle_new_client(
        &self,
        server: &mut TcpListener,
        poll: &mut Poll,
        connections: &mut HashMap<Token, MinecraftConnection>,
        unique_token: &mut usize,
    ) {
        let (mut connection, addr) = server.accept().unwrap();
        println!("New client: {}", addr);

        let token = Token(*unique_token);
        *unique_token += 1;

        poll.registry()
            .register(&mut connection, token, Interest::READABLE)
            .unwrap();

        let minecraft_connection = MinecraftConnection::new(connection);

        connections.insert(token, minecraft_connection);
    }

    pub fn run(&self) -> std::io::Result<()> {
        /* Create a new poll instance */
        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(128);

        let addr = format!("{}:{}", self.host, self.port).parse().unwrap();
        let mut server = TcpListener::bind(addr)?;

        /* Start listening for incoming connections */
        poll.registry()
            .register(&mut server, SERVER, Interest::READABLE)?;

        /* A counter is ok for now, using fd would be better */
        let mut unique_token = 1;
        let mut connections: HashMap<Token, MinecraftConnection> = HashMap::new();

        /* Accept connections and process them */
        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        /* Accept connection */
                        self.handle_new_client(
                            &mut server,
                            &mut poll,
                            &mut connections,
                            &mut unique_token,
                        );
                    }
                    token => {
                        // Handle client data
                        let should_remove = {
                            let conn = connections.get_mut(&token).unwrap();
                            self.handle_client_data(conn).is_err()
                        };

                        if should_remove {
                            println!("Removing connection for token {:?}", token);
                            if let Some(mut conn) = connections.remove(&token) {
                                poll.registry().deregister(&mut conn.connection).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}

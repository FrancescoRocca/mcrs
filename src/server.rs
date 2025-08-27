use std::collections::HashMap;
use std::io::Read;

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use crate::packets::{self};

const SERVER: Token = Token(0);

pub struct Server {
    host: String,
    port: String,
}

impl Server {
    pub fn new(host: &str, port: &str) -> Self {
        Server {
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

    fn handle_client_data(&self, stream: &mut TcpStream) {
        let mut buffer: Vec<u8> = vec![0u8; 1024];
        let n = stream.read(&mut buffer).unwrap();
        println!("Read {n} bytes");
        let packet = packets::Packet::parse(stream, &buffer[..n]);

        println!(
            "{}:{} (proto v:{}, intent: {})",
            packet.server_address(),
            packet.server_port(),
            packet.protocol_version(),
            packet.intent()
        );
    }

    fn handle_new_client(
        &self,
        server: &mut TcpListener,
        poll: &mut Poll,
        connections: &mut HashMap<Token, TcpStream>,
        unique_token: &mut usize,
    ) {
        let (mut connection, addr) = server.accept().unwrap();
        println!("New client: {}", addr);

        let token = Token(*unique_token);
        *unique_token += 1;

        poll.registry()
            .register(&mut connection, token, Interest::READABLE)
            .unwrap();

        connections.insert(token, connection);
    }

    pub fn run(&self) {
        /* Create a new poll instance */
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);

        let addr = format!("{}:{}", self.host, self.port).parse().unwrap();
        let mut server = TcpListener::bind(addr).unwrap();

        /* Start listening for incoming connections */
        poll.registry()
            .register(&mut server, SERVER, Interest::READABLE)
            .unwrap();

        /* A counter is ok for now, using fd would be better */
        let mut unique_token = 1;
        let mut connections: HashMap<Token, TcpStream> = HashMap::new();

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
                        /* Handle client data */
                        let conn = connections.get_mut(&token).unwrap();
                        self.handle_client_data(conn);
                    }
                }
            }
        }
    }
}

pub enum ClientIntent {
    Status,
    Login,
    Transfer,
    Error,
}

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

pub struct Packet {
    protocol_version: u32,
    server_address: String,
    server_port: u16,
    intent: ClientIntent,
}

impl Packet {
    pub fn parse(data: &[u8]) -> Self {
        print_hex(data);
        let mut offset = 0;

        let (length, off) = read_varint(data);
        println!("[debug] Packet len: {length}");
        offset += off;

        let (packet_id, off) = read_varint(&data[offset..]);
        println!("[debug] Packet id: {:#x}", packet_id);
        offset += off;

        match packet_id {
            0x00 => {
                println!("[debug] Handshake");
                parse_handshake(&data[offset..])
            }
            _ => {
                println!("[debug] Not implemented");
                Packet {
                    protocol_version: 0,
                    server_address: String::from("undefined"),
                    server_port: 25565,
                    intent: ClientIntent::Error,
                }
            }
        }
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
}

fn print_hex(data: &[u8]) {
    println!("[debug] Packet:");
    for b in data {
        print!("{:#x} ", *b);
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

fn parse_handshake(data: &[u8]) -> Packet {
    let mut offset = 0;

    let (protocol_version, off) = read_varint(data);
    offset += off;
    println!("Proto v: {protocol_version}");

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
        protocol_version,
        server_address,
        server_port,
        intent,
    }
}

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

pub fn read_varint(data: &[u8]) -> (u32, usize) {
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

pub fn varint_size(mut value: u32) -> u32 {
    let mut size = 0;

    loop {
        value >>= 7;
        size += 1;

        if value == 0 {
            break;
        }
    }

    size
}

pub fn write_varint(mut value: u32) -> Vec<u8> {
    let mut buffer = Vec::new();

    loop {
        let mut tmp: u8 = (value & SEGMENT_BITS as u32) as u8;
        value >>= 7;

        if value != 0 {
            tmp |= CONTINUE_BIT;
        }

        buffer.push(tmp);

        if value == 0 {
            break;
        }
    }

    buffer
}

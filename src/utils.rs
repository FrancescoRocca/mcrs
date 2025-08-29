use bytes::{Buf, BytesMut};

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

/**
 * Returns the value of a varint
 */
pub fn read_varint(data: &mut BytesMut) -> u32 {
    let mut value: u32 = 0;
    let mut pos: u32 = 0;

    loop {
        let byte = data.get_u8();
        value |= ((byte & SEGMENT_BITS) as u32) << pos;

        if (byte & CONTINUE_BIT) == 0 {
            break;
        }

        pos += 7;
    }

    value
}

/**
 * Returns the size of a value in varint style
 */
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

/**
 * Returns the value in varint style
 */
pub fn write_varint(mut value: u32) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();

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

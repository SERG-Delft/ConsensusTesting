use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Formatter;

use crate::deserialization::blob_iterator::BlobIterator;

pub struct Vector256<'a> {
    blob: &'a [u8],
}

impl<'b> Vector256<'b> {
    pub fn parse(blob: &'b mut BlobIterator) -> Self {
        let byte_1 = blob.next_byte();
        let size = match byte_1 {
            0..=192 => {
                byte_1 as usize
            }
            193..=240 => {
                let byte_2 = blob.next_byte();
                193 + (((byte_1 as usize) - 193) * 256) + (byte_2 as usize)
            }
            241..=254 => {
                let byte_2 = blob.next_byte();
                let byte_3 = blob.next_byte();
                12481 + (((byte_1 as usize) - 241) * 65536) + ((byte_2 as usize) * 256) + (byte_3 as usize)
            }
            _ => { panic!("size out of range!") }
        };
        Vector256 { blob: blob.next_n_bytes(size) }
    }
}

impl fmt::Display for Vector256<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Vector256<{}>", self.blob.iter().map(|b| format!("{:02X}", b)).collect::<String>())
    }
}

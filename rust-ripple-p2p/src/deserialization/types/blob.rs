use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Formatter;

use crate::deserialization::blob_iterator::BlobIterator;

pub struct Blob<'a> {
    blob: &'a [u8],
}

impl<'b> Blob<'b> {
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
        Blob { blob: blob.next_n_bytes(size) }
    }
}

impl fmt::Display for Blob<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Blob<{}>", self.blob.iter().map(|b| format!("{:02X}", b)).collect::<String>())
    }
}

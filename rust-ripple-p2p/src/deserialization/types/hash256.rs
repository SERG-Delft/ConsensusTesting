use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Formatter;

use crate::deserialization::blob_iterator::BlobIterator;

pub struct Hash256 {
    hash: [u8; 32],
}

impl Hash256 {
    pub fn parse(blob: &mut BlobIterator) -> Self {
        Hash256 { hash: <[u8; 32]>::try_from(blob.next_n_bytes(32)).unwrap() }
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Hash256<{}>", self.hash.iter().map(|b| format!("{:02X}", b)).collect::<String>())
    }
}

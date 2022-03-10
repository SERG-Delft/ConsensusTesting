use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Formatter;

use crate::deserialization::blob_iterator::BlobIterator;

pub struct AccountID {
    id: [u8; 20],
}

impl AccountID {
    pub fn parse(blob: &mut BlobIterator) -> Self {
        blob.next_byte();
        AccountID { id: <[u8; 20]>::try_from(blob.next_n_bytes(20)).unwrap() }
    }
}

impl fmt::Display for AccountID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "AccountID<{}>", self.id.iter().map(|b| format!("{:02X}", b)).collect::<String>())
    }
}

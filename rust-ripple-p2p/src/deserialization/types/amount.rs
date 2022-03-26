use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;

use crate::deserialization::blob_iterator::BlobIterator;

pub struct Amount {
    pub amount: u64,
}

impl Amount {
    pub fn parse(blob: &mut BlobIterator) -> Self {
        let is_xrp: bool = (blob.peek() & 0b1000_0000) == 0;
        if !is_xrp { panic!("cannot parse issued currency") }
        Amount { amount: (u64::from_be_bytes(blob.next_n_bytes(8).try_into().unwrap())) ^ 0x4000000000000000 }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Amount<{}>", self.amount)
    }
}

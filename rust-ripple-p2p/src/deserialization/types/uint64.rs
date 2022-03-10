use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;

use crate::deserialization::blob_iterator::BlobIterator;

pub struct UInt64 {
    value: u64,
}

impl UInt64 {
    pub fn parse(blob: &mut BlobIterator) -> Self {
        UInt64 { value: u64::from_be_bytes(blob.next_n_bytes(8).try_into().unwrap()) }
    }
}

impl fmt::Display for UInt64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UInt64<{}>", self.value)
    }
}

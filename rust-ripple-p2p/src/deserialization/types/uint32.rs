use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;

use crate::deserialization::blob_iterator::BlobIterator;

pub struct UInt32 {
    value: u32,
}

impl UInt32 {
    pub fn parse(blob: &mut BlobIterator) -> Self {
        UInt32 { value: u32::from_be_bytes(blob.next_n_bytes(4).try_into().unwrap()) }
    }
}

impl fmt::Display for UInt32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UInt32<{}>", self.value)
    }
}

use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;

use crate::deserialization::blob_iterator::BlobIterator;

pub struct UInt16 {
    value: u16,
}

impl UInt16 {
    pub fn parse(blob: &mut BlobIterator) -> Self {
        UInt16 { value: u16::from_be_bytes(blob.next_n_bytes(2).try_into().unwrap()) }
    }
}

impl fmt::Display for UInt16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UInt16<{}>", self.value)
    }
}

use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;

use crate::deserialization::blob_iterator::BlobIterator;

pub struct UInt8 {
    pub value: u8,
}

impl UInt8 {
    #[allow(unused)]
    pub fn parse(blob: &mut BlobIterator) -> Self {
        UInt8 { value: u8::from_be_bytes(blob.next_n_bytes(1).try_into().unwrap()) }
    }
}

impl fmt::Display for UInt8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UInt8<{}>", self.value)
    }
}

use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;

pub struct UInt32<'a> {
    pub value: &'a mut [u8],
}

impl fmt::Display for UInt32<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UInt32<{}>", u32::from_be_bytes((*self.value).try_into().unwrap()))
    }
}

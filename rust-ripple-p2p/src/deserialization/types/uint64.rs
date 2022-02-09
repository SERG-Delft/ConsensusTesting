use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;

pub struct UInt64<'a> {
    pub value: &'a mut [u8],
}

impl fmt::Display for UInt64<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UInt64<{}>", u64::from_be_bytes((*self.value).try_into().unwrap()))
    }
}

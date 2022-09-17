use std::fmt;
use std::fmt::Formatter;

pub struct UInt64 {
    pub value: u64,
}

impl fmt::Display for UInt64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UInt64<{}>", self.value)
    }
}

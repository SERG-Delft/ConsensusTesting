use std::fmt;
use std::fmt::Formatter;

pub struct UInt8 {
    pub value: u8,
}

impl fmt::Display for UInt8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UInt8<{}>", self.value)
    }
}

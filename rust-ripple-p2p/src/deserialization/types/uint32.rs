use std::fmt;
use std::fmt::Formatter;

pub struct UInt32 {
    pub value: u32,
}

impl fmt::Display for UInt32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UInt32<{}>", self.value)
    }
}

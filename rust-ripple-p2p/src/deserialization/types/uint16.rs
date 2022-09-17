use std::fmt;
use std::fmt::Formatter;

pub struct UInt16 {
    pub value: u16,
}

impl fmt::Display for UInt16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UInt16<{}>", self.value)
    }
}

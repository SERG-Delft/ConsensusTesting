use std::fmt;
use std::fmt::Formatter;

pub struct Vector256<'a> {
    pub blob: &'a [u8],
}

impl fmt::Display for Vector256<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Vector256<{}>", self.blob.iter().map(|b| format!("{:02X}", b)).collect::<String>())
    }
}

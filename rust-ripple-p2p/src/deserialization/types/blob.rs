use std::fmt;
use std::fmt::Formatter;

pub struct Blob<'a> {
    pub blob: &'a [u8],
}

impl fmt::Display for Blob<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Blob<{}>", self.blob.iter().map(|b| format!("{:02X}", b)).collect::<String>())
    }
}

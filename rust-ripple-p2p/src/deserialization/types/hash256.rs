use std::fmt;
use std::fmt::Formatter;

pub struct Hash256 {
    pub hash: [u8; 32],
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hash256<{}>",
            self.hash
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<String>()
        )
    }
}

use std::fmt;
use std::fmt::Formatter;

pub struct Amount {
    pub amount: u64,
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Amount<{}>", self.amount)
    }
}

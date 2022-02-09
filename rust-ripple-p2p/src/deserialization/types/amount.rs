use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;

pub struct Amount<'a> {
    pub amount: &'a mut [u8],
}

impl fmt::Display for Amount<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Amount<{}XRP>", u64::from_be_bytes((*self.amount).try_into().unwrap()))
    }
}

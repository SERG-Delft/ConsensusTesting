use std::fmt;
use std::fmt::Formatter;

pub struct AccountID<'a> {
    pub id: &'a mut [u8; 20],
}

impl fmt::Display for AccountID<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "AccountID<{}>", self.id.iter().map(|b| format!("{:02X}", b)).collect::<String>())
    }
}

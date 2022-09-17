use std::fmt;
use std::fmt::Formatter;

pub struct AccountID {
    pub id: [u8; 20],
}

impl fmt::Display for AccountID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccountID<{}>",
            ripple_address_codec::encode_account_id(&self.id)
        )
    }
}

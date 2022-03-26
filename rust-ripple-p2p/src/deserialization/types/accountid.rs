use std::convert::TryFrom;
use std::fmt;
use std::fmt::Formatter;

use crate::deserialization::blob_iterator::BlobIterator;

pub struct AccountID {
    pub id: [u8; 20],
}

impl AccountID {
    pub fn parse(blob: &mut BlobIterator) -> Self {
        blob.next_byte();
        AccountID { id: <[u8; 20]>::try_from(blob.next_n_bytes(20)).unwrap() }
    }

    pub fn base_58_check(&self) -> String {
        ripple_address_codec::encode_account_id(&self.id)
    }
}

impl fmt::Display for AccountID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "AccountID<{}>", ripple_address_codec::encode_account_id(&self.id))
    }
}

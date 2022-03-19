pub use accountid::AccountID;
pub use amount::Amount;
pub use blob::Blob;
pub use hash256::Hash256;
pub use uint16::UInt16;
pub use uint32::UInt32;
pub use uint64::UInt64;
pub use uint8::UInt8;
pub use vector256::Vector256;

mod accountid;
mod amount;
mod blob;
mod hash256;
mod uint16;
mod uint32;
mod uint64;
mod uint8;
mod vector256;

pub struct SerializationTypeValue {
    pub(crate) field: SerializationField,
    pub(crate) type_name: String,
}

#[allow(unused)]
pub enum SerializationField {
    U8(UInt8),
    U16(UInt16),
    U32(UInt32),
    U64(UInt64),
    H256(Hash256),
    Amount(Amount),
    Blob(String),
    AccountId(AccountID),
    Vec256(String),
}
use crate::deserialization::types::{UInt16, UInt32,UInt64, Amount, Hash256, Blob, AccountID, Vector256};

pub enum ValueType<'a> {
    UInt16(UInt16<'a>),
    UInt32(UInt32<'a>),
    UInt64(UInt64<'a>),
    Amount(Amount<'a>),
    Hash256(Hash256<'a>),
    Blob(Blob<'a>),
    AccountID(AccountID<'a>),
    Vector256(Vector256<'a>)
}

pub struct FieldTuple<'a> {
    pub field_name: String,
    pub value: ValueType<'a>
}

impl FieldTuple<'_> {
    pub fn new<'a>(field_name: String, value: ValueType<'a>) -> FieldTuple<'a> {
        FieldTuple { field_name, value }
    }
}

pub struct Transaction<'a> {
    pub field_id: String,
    pub field_code: String,
    pub fields_list: Vec<FieldTuple<'a>>
}

impl Transaction<'_> {
    pub fn new<'a>(field_id: String, field_code: String, fields_list: Vec<FieldTuple<'a>>) -> Transaction<'a> {
        Transaction { field_id, field_code, fields_list }
    }
}
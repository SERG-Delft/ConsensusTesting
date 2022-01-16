use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use types::*;

use crate::deserialization::blob_iterator::BlobIterator;

mod blob_iterator;
mod types;

pub fn deserialize(blob: &[u8]) {
    println!("{}", blob.iter().map(|b| format!("{:02X}", b)).collect::<String>());
    let mut blob_iterator = BlobIterator::new(blob);

    while blob_iterator.has_next() {
        let res = get_type_field_code(&mut blob_iterator);
        let field_type = decode_type_code(res.0);
        if field_type == "NotPresent" {
            break;
        }
        let type_name = decode_field_code(field_type, res.1);
        if type_name == "UNKNOWN".to_string() {
            break;
        }
        // println!("type code: {}, field code: {:?}", field_type, type_name);

        match field_type {
            "UInt16" => {
                let field = UInt16::parse(&mut blob_iterator);
                println!("- {}: {}", type_name, field)
            }
            "UInt32" => {
                let field = UInt32::parse(&mut blob_iterator);
                println!("- {}: {}", type_name, field)
            }
            "UInt64" => {
                let field = UInt64::parse(&mut blob_iterator);
                println!("- {}: {}", type_name, field)
            }
            "Hash256" => {
                let field = Hash256::parse(&mut blob_iterator);
                println!("- {}: {}", type_name, field)
            }
            "Amount" => {
                let field = Amount::parse(&mut blob_iterator);
                println!("- {}: {}", type_name, field)
            }
            "Blob" => {
                let field = Blob::parse(&mut blob_iterator);
                println!("- {}: {}", type_name, field)
            }
            "AccountID" => {
                let field = AccountID::parse(&mut blob_iterator);
                println!("- {}: {}", type_name, field)
            }
            _ => { break; }
            // _ => { panic!("unknown field type {}...", field_type) }
        }
    }
}

fn decode_type_code(type_code: u8) -> &'static str {
    return match type_code {
        0 => { "NotPresent" }
        1 => { "UInt16" }
        2 => { "UInt32" }
        3 => { "UInt64" }
        5 => { "Hash256" }
        6 => { "Amount" }
        7 => { "Blob" }
        8 => { "AccountID" }
        14 => { "STObject" }
        _ => { "UNKNOWN" }
        // _ => { panic!("unknown type code: {}", type_code) }
    };
}

fn decode_field_code(field_type: &str, field_code: u8) -> String {
    let fields = read_from_file();
    let current_key = FieldType { nth: field_code, type_field: field_type.to_string() };
    return match fields.get(&current_key) {
        Some(field) => { field.field_name.to_string() }
        None => { "UNKNOWN".to_string() }
    };
    // return match (field_type, field_code) {
    //     ("UInt16", 2) => { "TransactionType" }
    //     ("UInt32", 2) => { "Flags" }
    //     ("UInt32", 4) => { "Sequence" }
    //     ("UInt32", 6) => { "LedgerSequence" }
    //     ("UInt32", 9) => { "SigningTime" }
    //     ("UInt32", 27) => { "LastLedgerSequence" }
    //     ("UInt32", 31) => { "ReserveBase" }
    //     ("UInt32", 32) => { "ReserveIncrement" }
    //     ("UInt64", 10) => { "Cookie" }
    //     ("UInt64", 11) => { "ServerVersion" }
    //     ("Hash256", 1) => { "LedgerHash" }
    //     ("Hash256", 23) => { "ConsensusHash" }
    //     ("Hash256", 25) => { "ValidatedHash" }
    //     ("Amount", 1) => { "Amount" }
    //     ("Amount", 6) => { "LowLimit" }
    //     ("Amount", 8) => { "Fee" }
    //     ("Blob", 3) => { "SigningPubKey" }
    //     ("Blob", 4) => { "TxnSignature" }
    //     ("Blob", 6) => { "Signature" }
    //     ("AccountID", 1) => { "Account" }
    //     ("AccountID", 3) => { "Destination" }
    //     _ => { "UNKNOWN" }
    //     // _ => { panic!("unknown field code: {} for type {}", field_code, field_type) }
    // };
}

///
///
/// # Arguments
///
/// * `blob`:
///
/// returns: (type code as u8, field code as u8)
///
/// # Examples
///
/// ```
///
/// ```
fn get_type_field_code(blob: &mut BlobIterator) -> (u8, u8) {
    let first_byte = blob.next_byte();
    let low_bits = first_byte & 0b0000_1111;
    let high_bits = (first_byte & 0b1111_0000) >> 4;

    return match (high_bits == 0, low_bits == 0) {
        (true, true) => { (blob.next_byte(), blob.next_byte()) }
        (false, true) => { (high_bits, blob.next_byte()) }
        (true, false) => { (blob.next_byte(), low_bits) }
        (false, false) => { (high_bits, low_bits) }
    };
}


///
///
///
/// # Fields
///
///
///
pub fn read_from_file() -> HashMap<FieldType, FieldInformation> {
    let mut data = String::new();
    let mut file = File::open("src/deserialization/definitions.json").expect("Getting the file did not work.");
    file.read_to_string(&mut data).expect("Reading from file did not work.");

    let all_values: serde_json::Value = serde_json::from_str(&*data).expect("Parsing the data did not work.");
    // get only the fields
    let fields = serde_json::json!(all_values["FIELDS"]);

    // Hashmap with all the fields (key: nth + type)
    let mut all_fields = HashMap::new();

    for field in fields.as_array().unwrap() {
        // the array of each field in the JSON
        let current_field = field[1].as_object().unwrap();
        // key: nth + type
        let current_key = FieldType { nth: current_field["nth"].as_u64().unwrap() as u8, type_field: current_field["type"].to_string() };
        let current_value = FieldInformation {
            // field name
            field_name: field[0].to_string(),
            // isVLEncoded
            is_vl_encoded: current_field["isVLEncoded"].as_bool().unwrap(),
            // isSerialized
            is_serialized: current_field["isSerialized"].as_bool().unwrap(),
            // isSigningField
            is_signing_field: current_field["isSigningField"].as_bool().unwrap()
        };
        all_fields.insert(current_key, current_value);
    }

    return all_fields
}

#[derive(PartialEq, Eq, Hash)]
pub struct FieldType {

    pub nth: u8,
    pub type_field: String

}

impl FieldType {

    pub fn new(nth: u8, type_field: String) -> Self {
        FieldType { nth, type_field }
    }

}

pub struct FieldInformation {

    pub field_name: String,
    pub is_vl_encoded: bool,
    pub is_serialized: bool,
    pub is_signing_field: bool

}

impl FieldInformation {

    pub fn new(field_name: String, is_vl_encoded: bool, is_serialized: bool, is_signing_field: bool) -> Self {
        FieldInformation{ field_name, is_vl_encoded, is_serialized, is_signing_field }
    }

}

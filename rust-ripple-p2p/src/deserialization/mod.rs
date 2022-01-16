use types::*;

use crate::deserialization::blob_iterator::BlobIterator;

mod blob_iterator;
mod definitions_fields;
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
        if type_name == "UNKNOWN" {
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

fn decode_field_code(field_type: &str, field_code: u8) -> &'static str {
    return match (field_type, field_code) {
        ("UInt16", 2) => { "TransactionType" }
        ("UInt32", 2) => { "Flags" }
        ("UInt32", 4) => { "Sequence" }
        ("UInt32", 6) => { "LedgerSequence" }
        ("UInt32", 9) => { "SigningTime" }
        ("UInt32", 27) => { "LastLedgerSequence" }
        ("UInt32", 31) => { "ReserveBase" }
        ("UInt32", 32) => { "ReserveIncrement" }
        ("UInt64", 10) => { "Cookie" }
        ("UInt64", 11) => { "ServerVersion" }
        ("Hash256", 1) => { "LedgerHash" }
        ("Hash256", 23) => { "ConsensusHash" }
        ("Hash256", 25) => { "ValidatedHash" }
        ("Amount", 1) => { "Amount" }
        ("Amount", 6) => { "LowLimit" }
        ("Amount", 8) => { "Fee" }
        ("Blob", 3) => { "SigningPubKey" }
        ("Blob", 4) => { "TxnSignature" }
        ("Blob", 6) => { "Signature" }
        ("AccountID", 1) => { "Account" }
        ("AccountID", 3) => { "Destination" }
        _ => { "UNKNOWN" }
        // _ => { panic!("unknown field code: {} for type {}", field_code, field_type) }
    };
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

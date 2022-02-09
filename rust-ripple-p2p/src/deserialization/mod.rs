use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use protobuf::Message;

use crate::deserialization::blob_parser::BlobParser;
use crate::message_handler::RippleMessageObject;
use crate::protos::ripple::{TMLedgerInfoType, TMLedgerNode};

mod blob_parser;
mod types;

pub fn deserialize(message: &mut RippleMessageObject, from: usize, to: usize) -> Vec<u8> {
    match message {
        RippleMessageObject::TMTransaction(transaction) => {
            parse_canonical_binary_format(transaction.mut_rawTransaction(), from, to);
            transaction.write_to_bytes().unwrap()
        }
        RippleMessageObject::TMValidation(validation) => {
            parse_canonical_binary_format(validation.mut_validation(), from, to);
            validation.write_to_bytes().unwrap()
        }
        RippleMessageObject::TMLedgerData(ledger_data) => {
            let parse = match ledger_data.get_field_type() {
                TMLedgerInfoType::liBASE => { parse_ledger_base_nodes }
                TMLedgerInfoType::liTX_NODE => { parse_ledger_transaction_nodes }
                TMLedgerInfoType::liAS_NODE => { parse_ledger_account_state_nodes }
                TMLedgerInfoType::liTS_CANDIDATE => { parse_ledger_ts_candidate_nodes }
            };
            parse(ledger_data.mut_nodes());
            ledger_data.write_to_bytes().unwrap()
        }
        _ => panic!("not handled")
    }
}

fn parse_ledger_base_nodes(nodes: &mut [TMLedgerNode]) {
    let mut header_node = nodes.get(0).expect("has to have at least one node");
    println!("#{:?} node id", header_node.get_nodeid());
    // let mut blob = BlobParser::new(nodes.get(0).expect("has to have at least one node").mut_nodedata());
    // println!("- Seq: {}", UInt32::parse(&mut blob));
    // println!("- Drops: {}", UInt64::parse(&mut blob));
    // println!("- Parent Hash: {}", Hash256::parse(&mut blob));
    // println!("- Tx Hash: {}", Hash256::parse(&mut blob));
    // println!("- Account Hash: {}", Hash256::parse(&mut blob));
    // println!("- Parent Close Time: {}", UInt32::parse(&mut blob));
    // println!("- Close Time: {}", UInt32::parse(&mut blob));
    // println!("- Close Time Resolution: {}", blob.read_uint8());
    // println!("- Close Flags: {}", blob.read_uint8());
    // println!("- Hash: {}", Hash256::parse(&mut blob)); not included
    // if blob.has_next() {
    //     panic!("more to parse... {}", blob.len())
    // }
    if nodes.len() > 1 {
        parse_ledger_node_from_wire(nodes.get_mut(1).unwrap())
    }
    if nodes.len() > 2 {
        parse_ledger_node_from_wire(nodes.get_mut(1).unwrap())
    }
    if nodes.len() > 3 {
        panic!("more to parse...")
    }
}

fn parse_ledger_transaction_nodes(nodes: &mut [TMLedgerNode]) {
    for node in nodes {
        parse_ledger_node_from_wire(node)
    }
}

fn parse_ledger_account_state_nodes(nodes: &mut [TMLedgerNode]) {
    for node in nodes {
        parse_ledger_node_from_wire(node)
    }
}

fn parse_ledger_ts_candidate_nodes(nodes: &mut [TMLedgerNode]) {
    for node in nodes {
        parse_ledger_node_from_wire(node)
    }
}

fn parse_ledger_node_from_wire(node: &mut TMLedgerNode) {
    println!("#{:?} node id", node.get_nodeid());
    let mut blob = BlobParser::new(node.mut_nodedata());
    match blob.last_byte() {
        0 => { // wireTypeTransaction
            parse_canonical_binary_format_with_iterator(blob, 0, 0)
        }
        1 => { // wireTypeAccountState
            println!("- Hash: {:?}", blob.last_n_bytes(32));
            parse_canonical_binary_format_with_iterator(blob, 0, 0)
        }
        2 => { // wireTypeInner
            panic!("not implemented")
        }
        3 => { // wireTypeCompressedInner
            // while blob.has_next() {
            //     // let hash = blob.read_hash256();
            //     // let branch = blob.read_uint8();
            //     // println!("- branch #{} has {}", branch, hash)
            // }
        }
        4 => { // wireTypeTransactionWithMeta
            // println!("- Hash: {}", Hash256::parse(&mut BlobIterator::new(blob.last_n_bytes(32))));
            println!("- More unparsed data available! format unknow...")
        }
        _ => { panic!("unknown wire type") }
    }
}

fn parse_canonical_binary_format(blob: &mut [u8], from: usize, to: usize) {
    let iterator = BlobParser::new(blob);
    parse_canonical_binary_format_with_iterator(iterator, from, to)
}

fn parse_canonical_binary_format_with_iterator(mut blob: BlobParser, from: usize, to: usize) {
    while blob.has_next() {
        let (type_code, field_code) = type_field_code(&mut blob);
        let field_type = decode_type_code(type_code);
        let type_name = decode_field_code(field_type, field_code);
        match field_type {
            "UInt16" => {
                let field = blob.read_uint16();
                println!("- {}: {}", type_name, field)
            }
            "UInt32" => {
                let field = blob.read_uint32();
                println!("- {}: {}", type_name, field);
                if type_name == "Flags" && from == 0 {
                    // if type_name == "Sequence" {
                    field.value[3] = 1;
                }
            }
            "UInt64" => {
                let field = blob.read_uint64();
                println!("- {}: {}", type_name, field)
            }
            "Hash256" => {
                let field = blob.read_hash256();
                println!("- {}: {}", type_name, field)
            }
            "Amount" => {
                let field = blob.read_amount();
                println!("- {}: {}", type_name, field)
            }
            "Blob" => {
                let field = blob.read_blob();
                println!("- {}: {}", type_name, field)
            }
            "AccountID" => {
                let field = blob.read_account_id();
                println!("- {}: {}", type_name, field)
            }
            "Vector256" => {
                let field = blob.read_vector256();
                println!("- {}: {}", type_name, field)
            }
            _ => { panic!("unknown field type {}...", field_type) }
        }
    }
}

fn type_field_code(blob: &mut BlobParser) -> (u8, u8) {
    let first_byte = blob.next_byte();

    let low_bits = first_byte & 0b0000_1111;
    let high_bits = (first_byte & 0b1111_0000) >> 4;

    match (high_bits == 0, low_bits == 0) {
        (true, true) => { (blob.next_byte(), blob.next_byte()) }
        (false, true) => { (high_bits, blob.next_byte()) }
        (true, false) => { (blob.next_byte(), low_bits) }
        (false, false) => { (high_bits, low_bits) }
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
        19 => { "Vector256" }
        _ => { panic!("unknown type code: {}", type_code) }
    };
}

fn decode_field_code(field_type: &str, field_code: u8) -> String {
    let fields = read_from_file();
    let current_key = FieldType { nth: field_code, type_field: field_type.to_string() };
    let result = match fields.get(&current_key) {
        Some(field) => { field.field_name.to_string() }
        None => { "Unknown".to_string() }
    };
    // if the key is not in the fields in definitions.json
    if result.eq("Unknown") {
        return match (field_type, field_code) {
            ("UInt64", 10) => { "Cookie".to_string() }
            ("Hash256", 25) => { "ValidatedHash".to_string() }
            _ => { "Unknown".to_string() }
        };
    }
    return result;
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

    // hashmap with all the fields (key: nth + type)
    let mut all_fields = HashMap::new();

    for field in fields.as_array().unwrap() {
        // the array of each field in the JSON
        let current_field = field[1].as_object().unwrap();
        // key: nth + type
        let current_key = FieldType {
            nth: current_field["nth"].as_u64().unwrap() as u8,
            type_field: current_field["type"]
                .to_string().replace("\"", ""),
        };
        let current_value = FieldInformation {
            // field name
            field_name: field[0].to_string().replace("\"", ""),
            // isVLEncoded
            is_vl_encoded: current_field["isVLEncoded"].as_bool().unwrap(),
            // isSerialized
            is_serialized: current_field["isSerialized"].as_bool().unwrap(),
            // isSigningField
            is_signing_field: current_field["isSigningField"].as_bool().unwrap(),
        };
        all_fields.insert(current_key, current_value);
    }

    all_fields
}

#[derive(PartialEq, Eq, Hash)]
pub struct FieldType {
    pub nth: u8,
    pub type_field: String,
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
    pub is_signing_field: bool,
}

impl FieldInformation {
    pub fn new(field_name: String, is_vl_encoded: bool, is_serialized: bool, is_signing_field: bool) -> Self {
        FieldInformation { field_name, is_vl_encoded, is_serialized, is_signing_field }
    }
}

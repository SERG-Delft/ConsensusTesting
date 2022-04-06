use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use json::{JsonValue, object};
use rippled_binary_codec::serialize::serialize_tx;
use serde_json::Value;

use types::*;

use crate::deserialization::blob_iterator::BlobIterator;
use crate::message_handler::{ParsedValidation, RippleMessageObject};
use crate::protos::ripple::{TMLedgerInfoType, TMLedgerNode};

mod blob_iterator;
mod types;

#[allow(unused)]
pub fn deserialize(message: &RippleMessageObject) {
    match message {
        RippleMessageObject::TMTransaction(transaction) => {
            parse_canonical_binary_format(transaction.get_rawTransaction());
        }
        RippleMessageObject::TMValidation(validation) => {
            parse_canonical_binary_format(validation.get_validation());
        }
        RippleMessageObject::TMLedgerData(ledger_data) => {
            let parse = match ledger_data.get_field_type() {
                TMLedgerInfoType::liBASE => { parse_ledger_base_nodes }
                TMLedgerInfoType::liTX_NODE => { parse_ledger_transaction_nodes }
                TMLedgerInfoType::liAS_NODE => { parse_ledger_account_state_nodes }
                TMLedgerInfoType::liTS_CANDIDATE => { parse_ledger_ts_candidate_nodes }
            };
            parse(ledger_data.get_nodes())
        }
        _ => {}
    }
}

pub fn deserialize_validation(blob: &[u8]) -> ParsedValidation {
    let mut parsed_validation = ParsedValidation::default();
    let deserialization_result = parse_canonical_binary_format(blob);
    for field in deserialization_result {
        match field.type_name.as_str() {
            "Cookie" => parsed_validation.cookie = match field.field {
                SerializationField::U64(value) => value.value,
                _ => 0
            },
            "hash" => parsed_validation.hash = match field.field {
                SerializationField::H256(value) => format!("{:x?}", value.hash),
                _ => "".to_string()
            },
            "ConsensusHash" => parsed_validation.consensus_hash = match field.field {
                SerializationField::H256(value) => format!("{:x?}", value.hash),
                _ => "".to_string()
            },
            "ValidatedHash" => parsed_validation.validated_hash = match field.field {
                SerializationField::H256(value) => format!("{:x?}", value.hash),
                _ => "".to_string()
            },
            "SigningPubKey" => parsed_validation.signing_pub_key = match field.field {
                SerializationField::Blob(value) => value,
                _ => "".to_string()
            },
            "Signature" => parsed_validation.signature = match field.field {
                SerializationField::Blob(value) => value,
                _ => "".to_string()
            },
            "Flags" => parsed_validation.flags = match field.field {
                SerializationField::U32(value) => value.value,
                _ => 0
            },
            "LedgerSequence" => parsed_validation.ledger_sequence = match field.field {
                SerializationField::U32(value) => value.value,
                _ => 0
            },
            "SigningTime" => parsed_validation.signing_time = match field.field {
                SerializationField::U32(value) => value.value,
                _ => 0
            },
            _ => {}
        }
    }
    // println!("{:?}", parsed_validation);
    parsed_validation
}

fn parse_ledger_base_nodes(nodes: &[TMLedgerNode]) {
    let header_node = nodes.get(0).expect("has to have at least one node");
    println!("#{:?} node id", header_node.get_nodeid());
    let mut blob = BlobIterator::new(header_node.get_nodedata());
    println!("- Seq: {}", UInt32::parse(&mut blob));
    println!("- Drops: {}", UInt64::parse(&mut blob));
    println!("- Parent Hash: {}", Hash256::parse(&mut blob));
    println!("- Tx Hash: {}", Hash256::parse(&mut blob));
    println!("- Account Hash: {}", Hash256::parse(&mut blob));
    println!("- Parent Close Time: {}", UInt32::parse(&mut blob));
    println!("- Close Time: {}", UInt32::parse(&mut blob));
    println!("- Close Time Resolution: {}", UInt8::parse(&mut blob));
    println!("- Close Flags: {}", UInt8::parse(&mut blob));
    // println!("- Hash: {}", Hash256::parse(&mut blob)); not included
    if blob.has_next() {
        panic!("more to parse... {}", blob.len())
    }
    if nodes.len() > 1 {
        parse_ledger_node_from_wire(nodes.get(1).unwrap())
    }
    if nodes.len() > 2 {
        parse_ledger_node_from_wire(nodes.get(1).unwrap())
    }
    if nodes.len() > 3 {
        panic!("more to parse...")
    }
}

fn parse_ledger_transaction_nodes(nodes: &[TMLedgerNode]) {
    for node in nodes {
        parse_ledger_node_from_wire(node)
    }
}

fn parse_ledger_account_state_nodes(nodes: &[TMLedgerNode]) {
    for node in nodes {
        parse_ledger_node_from_wire(node)
    }
}

fn parse_ledger_ts_candidate_nodes(nodes: &[TMLedgerNode]) {
    for node in nodes {
        parse_ledger_node_from_wire(node)
    }
}

fn parse_ledger_node_from_wire(node: &TMLedgerNode) {
    println!("#{:?} node id", node.get_nodeid());
    let mut blob_iterator = BlobIterator::new(node.get_nodedata());
    match blob_iterator.last_byte() {
        0 => { // wireTypeTransaction
            parse_canonical_binary_format_with_iterator(blob_iterator);
        }
        1 => { // wireTypeAccountState
            println!("- Hash: {}", Hash256::parse(&mut BlobIterator::new(blob_iterator.last_n_bytes(32))));
            parse_canonical_binary_format_with_iterator(blob_iterator);
        }
        2 => { // wireTypeInner
            panic!("not implemented")
        }
        3 => { // wireTypeCompressedInner
            while blob_iterator.has_next() {
                let hash = Hash256::parse(&mut blob_iterator);
                let branch = UInt8::parse(&mut blob_iterator);
                println!("- branch #{} has {}", branch, hash)
            }
        }
        4 => { // wireTypeTransactionWithMeta
            println!("- Hash: {}", Hash256::parse(&mut BlobIterator::new(blob_iterator.last_n_bytes(32))));
            println!("- More unparsed data available!")
        }
        _ => { panic!("unknown wire type") }
    }
}

fn parse_canonical_binary_format(blob: &[u8]) -> Vec<SerializationTypeValue> {
    let iterator = BlobIterator::new(blob);
    parse_canonical_binary_format_with_iterator(iterator)
}

fn parse_canonical_binary_format_with_iterator(mut blob_iterator: BlobIterator) -> Vec<SerializationTypeValue> {
    let mut json = object!{};
    let mut contents = vec![];
    // println!("New validation");
    while blob_iterator.has_next() {
        let res = get_type_field_code(&mut blob_iterator);
        let field_type = decode_type_code(res.0);
        let type_name = decode_field_code(field_type, res.1);
        match field_type {
            "UInt16" => {
                let field = UInt16::parse(&mut blob_iterator);
                // println!("- {}: {}", type_name, field);
                json[&type_name] = field.value.into();
                contents.push(SerializationTypeValue { field: SerializationField::U16(field), type_name });
            }
            "UInt32" => {
                let field = UInt32::parse(&mut blob_iterator);
                // println!("- {}: {}", type_name, field);
                json[&type_name] = field.value.into();
                contents.push(SerializationTypeValue { field: SerializationField::U32(field), type_name });
            }
            "UInt64" => {
                let field = UInt64::parse(&mut blob_iterator);
                // println!("- {}: {}", type_name, field);
                json[&type_name] = field.value.into();
                contents.push(SerializationTypeValue { field: SerializationField::U64(field), type_name });
            }
            "Hash256" => {
                let field = Hash256::parse(&mut blob_iterator);
                println!("- {}: {}", type_name, field);
                json[&type_name] = format!("{:x?}", field.hash).into();
                contents.push(SerializationTypeValue { field: SerializationField::H256(field), type_name });
            }
            "Amount" => {
                let field = Amount::parse(&mut blob_iterator);
                println!("- {}: {}", &type_name, &field);
                json[&type_name] = field.amount.to_string().into();
                contents.push(SerializationTypeValue { field: SerializationField::Amount(field), type_name });
            }
            "Blob" => {
                let field = Blob::parse(&mut blob_iterator);
                // println!("- {}: {}", type_name, field);
                json[&type_name] = format!("{:x?}", field.blob).into();
                contents.push(SerializationTypeValue { field: SerializationField::Blob(format!("{:x?}", field.blob)), type_name });
            }
            "AccountID" => {
                let field = AccountID::parse(&mut blob_iterator);
                // println!("- {}: {}", type_name, field);
                json[&type_name] = field.base_58_check().into();
                contents.push(SerializationTypeValue { field: SerializationField::AccountId(field), type_name });
            }
            "Vector256" => {
                let field = Vector256::parse(&mut blob_iterator);
                // println!("- {}: {}", type_name, field);
                contents.push(SerializationTypeValue { field: SerializationField::Vec256(format!("{:X?}", field.blob)), type_name });
            }
            _ => { panic!("unknown field type {}...", field_type) }
        }
    }
    println!("{}", json);
    println!("{:?}", mutate_and_serialize_json(json));
    contents
}

fn mutate_and_serialize_json(mut deserialized_transaction: JsonValue) -> Vec<u8> {

    // mutate the amount
    let amount = deserialized_transaction["Amount"].as_str().unwrap();
    let mutated_amount = u64::from_str(amount).unwrap() + 100;
    deserialized_transaction["Amount"] = JsonValue::from(mutated_amount.to_string());

    // map the transaction type to its name (based on definitions.json)
    let transaction_types = read_transaction_types();
    let current_key = deserialized_transaction["TransactionType"].as_i64().unwrap();
    let transaction_type = match transaction_types.get(&current_key) {
        Some(transaction) => { transaction.to_string() }
        None => { "Invalid".to_string() }
    };
    deserialized_transaction["TransactionType"] = JsonValue::from(transaction_type);

    // map the signing public key
    let encoded_signing_key = hex::encode_upper(deserialized_transaction["SigningPubKey"].as_str().unwrap());
    deserialized_transaction["SigningPubKey"] = JsonValue::from(encoded_signing_key);

    // map the txn signature
    let encoded_txt_signature = hex::encode_upper(deserialized_transaction["TxnSignature"].as_str().unwrap());
    deserialized_transaction["TxnSignature"] = JsonValue::from(encoded_txt_signature);

    // println!("{}", deserialized_transaction["SigningPubKey"]);
    // println!("{}", deserialized_transaction["TxnSignature"]);
    // println!("{}", deserialized_transaction.to_string());

    return match serialize_tx(deserialized_transaction.to_string(), false) {
        Some(string) => { serialize_tx(deserialized_transaction.to_string(), false).unwrap().as_bytes().to_vec() }
        None => { panic!("could not serialize") }
    }
}

fn mutate_transaction(mut contents: Vec<SerializationTypeValue>) -> Vec<SerializationTypeValue> {

    let mutated_contents = contents.into_iter().
        map(|x| {
            match x {
                SerializationTypeValue { field, type_name } if (type_name == String::from("Amount")) => {

                    let mutated_field = match field {
                        SerializationField::Amount(current_amount) => {
                            let mutated_amount = current_amount.amount + 100;
                            SerializationField::Amount(Amount{ amount: mutated_amount })
                        }
                        _ => { field }
                    };

                    SerializationTypeValue { field: mutated_field, type_name: String::from("Amount") }
                }
                _ => { x }
            }
        })
        .collect();

    mutated_contents
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
/// # Transaction types
///
///
///
pub fn read_transaction_types() -> HashMap<i64, String> {
    let mut data = String::new();
    let mut file = File::open("src/deserialization/definitions.json").expect("Getting the file did not work.");
    file.read_to_string(&mut data).expect("Reading from file did not work.");

    let all_values: serde_json::Value = serde_json::from_str(&*data).expect("Parsing the data did not work.");
    // get only the transactions
    let transactions = serde_json::json!(all_values["TRANSACTION_TYPES"]);

    let mut all_transactions = HashMap::new();

    for transaction in transactions.as_object().unwrap() {
        let (field_1, field_2) = transaction;
        let current_key = field_2.as_i64().unwrap();
        let current_value = field_1.to_string();
        all_transactions.insert(current_key, current_value);
    }

    all_transactions
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
        let current_key = FieldType { nth: current_field["nth"].as_u64().unwrap() as u8, type_field: current_field["type"]
            .to_string().replace("\"", "") };
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
    #[allow(unused)]
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
    #[allow(unused)]
    pub fn new(field_name: String, is_vl_encoded: bool, is_serialized: bool, is_signing_field: bool) -> Self {
        FieldInformation { field_name, is_vl_encoded, is_serialized, is_signing_field }
    }
}

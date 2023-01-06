use json::{object, JsonValue};
use lazy_static::lazy_static;
use nom::branch::alt;
use nom::bytes::complete::take;
use nom::combinator::{fail, map, rest, success, value, verify};
use nom::error::Error;
use nom::multi::{length_value, many0};
use nom::number::complete::{be_u16, be_u32, be_u64, be_u8};
use nom::sequence::{pair, preceded, tuple};
use nom::{bits, IResult};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;

lazy_static! {
    static ref MAPPING: HashMap<FieldType, FieldInformation> = read_from_file();
}

fn decode_type_code(type_code: u8) -> &'static str {
    match type_code {
        0 => "NotPresent",
        1 => "UInt16",
        2 => "UInt32",
        3 => "UInt64",
        5 => "Hash256",
        6 => "Amount",
        7 => "Blob",
        8 => "AccountID",
        14 => "STObject",
        19 => "Vector256",
        _ => "Unknown",
    }
}

fn decode_field_code(field_type: &str, field_code: u8) -> String {
    let fields = &MAPPING;
    let current_key = FieldType {
        nth: field_code,
        type_field: field_type.to_string(),
    };
    let result = match fields.get(&current_key) {
        Some(field) => field.field_name.to_string(),
        None => "Unknown".to_string(),
    };
    // if the key is not in the fields in definitions.json
    if result.eq("Unknown") {
        return match (field_type, field_code) {
            ("UInt64", 10) => "Cookie".to_string(),
            ("Hash256", 25) => "ValidatedHash".to_string(),
            _ => "Unknown".to_string(),
        };
    }
    result
}

fn field_id(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
    // let next_byte = &;
    let (input, (high, low)) = bits::<_, _, Error<(&[u8], usize)>, _, _>(pair(
        bits::complete::take(4usize),
        bits::complete::take(4usize),
    ))(input)?;
    match (high != 0, low != 0) {
        (true, true) => value((high, low), success(0))(input),
        (true, false) => pair(value(high, success(0)), map(take(1usize), |x: &[u8]| x[0]))(input),
        (false, true) => pair(map(take(1usize), |x: &[u8]| x[0]), value(low, success(0)))(input),
        (false, false) => pair(
            map(take(1usize), |x: &[u8]| x[0]),
            map(take(1usize), |x: &[u8]| x[0]),
        )(input),
    }
}

fn parse_fail(input: &[u8]) -> IResult<&[u8], JsonValue> {
    map(fail, |n: u16| JsonValue::Number(n.into()))(input)
}

fn parse_uint16(input: &[u8]) -> IResult<&[u8], JsonValue> {
    map(be_u16, |n: u16| JsonValue::Number(n.into()))(input)
}

fn parse_uint32(input: &[u8]) -> IResult<&[u8], JsonValue> {
    map(be_u32, |n: u32| JsonValue::Number(n.into()))(input)
}

fn parse_uint64(input: &[u8]) -> IResult<&[u8], JsonValue> {
    map(be_u64, |n: u64| JsonValue::Number(n.into()))(input)
}

fn parse_hash256(input: &[u8]) -> IResult<&[u8], JsonValue> {
    map(take(32usize), |hash: &[u8]| {
        JsonValue::String(hex::encode(hash))
    })(input)
}

fn parse_length_1_byte(input: &[u8]) -> IResult<&[u8], u32> {
    map(verify(be_u8, |n: &u8| *n <= 192), |n: u8| n as u32)(input)
}

fn parse_length_2_bytes(input: &[u8]) -> IResult<&[u8], u32> {
    let formula = |(x, y): (u8, u8)| 193 + (((x as u32) - 193) * 256) + (y as u32);
    map(pair(verify(be_u8, |n: &u8| *n <= 240), be_u8), formula)(input)
}

fn parse_length_3_bytes(input: &[u8]) -> IResult<&[u8], u32> {
    let formula = |(x, y, z): (u8, u8, u8)| {
        12481 + (((x as u32) - 241) * 65536) + ((y as u32) * 256) + (z as u32)
    };
    map(tuple((be_u8, be_u8, be_u8)), formula)(input)
}

fn parse_length(input: &[u8]) -> IResult<&[u8], u32> {
    alt((
        parse_length_1_byte,
        parse_length_2_bytes,
        parse_length_3_bytes,
    ))(input)
}

fn parse_blob(input: &[u8]) -> IResult<&[u8], JsonValue> {
    length_value(
        parse_length,
        map(rest, |x: &[u8]| JsonValue::String(hex::encode(x))),
    )(input)
}

fn parse_amount(input: &[u8]) -> IResult<&[u8], JsonValue> {
    map(be_u64, |n: u64| {
        JsonValue::String((n ^ 0x4000000000000000).to_string())
    })(input)
}

fn parse_account_id(input: &[u8]) -> IResult<&[u8], JsonValue> {
    map(preceded(take(1usize), take(20usize)), |address: &[u8]| {
        JsonValue::String(ripple_address_codec::encode_account_id(
            <&[u8; 20]>::try_from(address).unwrap(),
        ))
    })(input)
}

fn parse_field(input: &[u8]) -> IResult<&[u8], (String, JsonValue)> {
    let (input, (type_code, field_code)) = field_id(input)?;
    let type_str = decode_type_code(type_code);
    let field_name = decode_field_code(type_str, field_code);
    pair(
        value(field_name, success(0)),
        match type_str {
            "UInt16" => parse_uint16,
            "UInt32" => parse_uint32,
            "UInt64" => parse_uint64,
            "Hash256" => parse_hash256,
            "Blob" => parse_blob,
            "Amount" => parse_amount,
            "AccountID" => parse_account_id,
            _ => parse_fail,
        },
    )(input)
}

pub fn parse(input: &[u8]) -> IResult<&[u8], JsonValue> {
    let (input, values) = many0(parse_field)(input)?;
    let mut json = object! {};
    for (k, v) in values {
        json[k] = v;
    }
    Ok((input, json))
}

fn read_from_file() -> HashMap<FieldType, FieldInformation> {
    let mut data = String::new();
    let mut file = File::open("serialize/src/deserialization/definitions.json")
        .expect("Getting the file did not work.");
    file.read_to_string(&mut data)
        .expect("Reading from file did not work.");

    let all_values: serde_json::Value =
        serde_json::from_str(&data).expect("Parsing the data did not work.");
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
            type_field: current_field["type"].to_string().replace('\"', ""),
        };
        let current_value = FieldInformation {
            // field name
            field_name: field[0].to_string().replace('\"', ""),
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
    pub fn new(
        field_name: String,
        is_vl_encoded: bool,
        is_serialized: bool,
        is_signing_field: bool,
    ) -> Self {
        FieldInformation {
            field_name,
            is_vl_encoded,
            is_serialized,
            is_signing_field,
        }
    }
}

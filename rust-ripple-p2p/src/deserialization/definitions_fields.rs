use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use httparse::Response;
use serde_json::from_str;
use serde_json::{Result, Value};

pub struct DefinitionFields {

}

impl DefinitionFields {

    fn read_from_file(&self) -> HashMap<FieldType, FieldInformation> {
        let mut data = String::new();
        let mut f = File::open("/definitions.json")?;
        f.read_to_string(&mut data)?;
        
        let all_values: serde_json::Value = serde_json::from_str(&*data)?;
        // get only the fields
        let fields = json! all_values["FIELDS"];

        // Hashmap with all the fields (key: nth + type)
        let mut all_fields = HashMap::new();

        for field in fields {
            // the array of each field in the JSON
            let current_field = field[1];
            // key: nth + type
            let current_key = FieldType { nth: current_field[0], type_field: current_field[4].to_string() };
            let current_value = FieldInformation {
                // field name
                field_name: field[0],
                // isVLEncoded
                is_VL_encoded: current_field[1],
                // isSerialized
                is_serialized: current_field[2],
                // isSigningField
                is_signing_field: current_field[3]
            };
            all_fields.insert(current_key, current_value);
        }

        return all_fields
    }

}

pub struct FieldType {

    pub nth: i16,
    pub type_field: String

}

impl FieldType {

    pub fn new(nth: i16, type_field: String) -> Self {
        FieldType { nth, type_field }
    }

}

pub struct FieldInformation {

    pub field_name: String,
    pub is_VL_encoded: bool,
    pub is_serialized: bool,
    pub is_signing_field: bool

}

impl FieldInformation {

    pub fn new(field_name: String, is_VL_encoded: bool, is_serialized: bool, is_signing_field: bool) -> Self {
        FieldInformation{ field_name, is_VL_encoded, is_serialized, is_signing_field }
    }

}

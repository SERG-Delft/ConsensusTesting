use std::collections::HashMap;
use serde_json::from_str;

pub struct DefinitionFields {

}

impl DefinitionFields {

    fn compute_map_for_types(&self) -> HashMap<String, i16> {
        let mut types = HashMap::new();

        types.insert("Validation".to_string(), 10003);
        types.insert("Done".to_string(), -1);
        types.insert("Hash128".to_string(), 4);
        types.insert("Blob".to_string(), 7);
        types.insert("AccountID".to_string(), 8);
        types.insert("Amount".to_string(), 6);
        types.insert("Hash256".to_string(), 5);
        types.insert("UInt8".to_string(), 16);
        types.insert("Vector256".to_string(), 19);
        types.insert("STObject".to_string(), 14);
        types.insert("Unknown".to_string(), -2);
        types.insert("Transaction".to_string(), 10001);
        types.insert("Hash160".to_string(), 17);
        types.insert("PathSet".to_string(), 18);
        types.insert("LedgerEntry".to_string(), 10002);
        types.insert("UInt16".to_string(), 1);
        types.insert("NotPresent".to_string(), 0);
        types.insert("UInt64".to_string(), 3);
        types.insert("UInt32".to_string(), 2);
        types.insert("STArray".to_string(), 15);

        return types
    }

    fn compute_map_for_ledger_types(&self) -> HashMap<String, i16> {
        let mut types = HashMap::new();

        types.insert("Any".to_string(), -3);
        types.insert("Child".to_string(), -2);
        types.insert("Invalid".to_string(), -1);
        types.insert("AccountRoot".to_string(), 97);
        types.insert("DirectoryNode".to_string(), 100);
        types.insert("RippleState".to_string(), 114);
        types.insert("Ticket".to_string(), 84);
        types.insert("SignerList".to_string(), 83);
        types.insert("Offer".to_string(), 111);
        types.insert("LedgerHashes".to_string(), 104);
        types.insert("Amendments".to_string(), 102);
        types.insert("FeeSettings".to_string(), 115);
        types.insert("Escrow".to_string(), 117);
        types.insert("PayChannel".to_string(), 120);
        types.insert("DepositPreauth".to_string(), 112);
        types.insert("Check".to_string(), 67);
        types.insert("Nickname".to_string(), 110);
        types.insert("Contract".to_string(), 99);
        types.insert("GeneratorMap".to_string(), 103);
        types.insert("NegativeUNL".to_string(), 78);

        return types
    }

    fn compute_map_for_fields(&self) -> HashMap<String, FieldType> {
        let mut fields = HashMap::new();

        // Add all field types
        fields.insert("Generic".to_string(), FieldType{nth: 0, is_vl_encoded: false, is_serialized: false, is_signing_field: false, type_field: "Unknown".to_string()});
        fields.insert("Invalid".to_string(), FieldType{nth: -1, is_vl_encoded: false, is_serialized: false, is_signing_field: false, type_field: "Unknown".to_string()});
        fields.insert("LedgerEntryType".to_string(), FieldType{nth: 1, is_vl_encoded: false, is_serialized: true, is_signing_field: true, type_field: "UInt16".to_string()});
        fields.insert("TransactionType".to_string(), FieldType{nth: 2, is_vl_encoded: false, is_serialized: true, is_signing_field: true, type_field: "UInt16".to_string()});
        fields.insert("SignerWeight".to_string(), FieldType{nth: 3, is_vl_encoded: false, is_serialized: true, is_signing_field: true, type_field: "UInt16".to_string()});
        fields.insert("Flags".to_string(), FieldType{nth: 2, is_vl_encoded: false, is_serialized: true, is_signing_field: true, type_field: "UInt32".to_string()});

        return fields
    }

    pub fn get_fields(&self, type_code: i16, field_code: i16)-> (String, String) {
        let mut type_name = String::from("Invalid");
        let mut field_name = String::from("Invalid");

        let mut types = self.compute_map_for_types();

        let mut fields = self.compute_map_for_fields();

        // Also check for ledger types
        for (key, value) in types {
            if type_code == value {
                type_name = key;
            }
        }

        for (key, value) in fields {
            if field_code == value.nth {
                let the_type_string = value.type_field;
                if the_type_string == type_name.to_string() {
                    field_name = key;
                }
            }
        }

        return (type_name.to_string(), field_name.to_string())
    }

}

pub struct FieldType {
    pub nth: i16,
    pub is_vl_encoded: bool,
    pub is_serialized: bool,
    pub is_signing_field: bool,
    pub type_field: String
}

impl FieldType {

    pub fn new(nth: i16, is_vl_encoded: bool, is_serialized: bool, is_signing_field: bool, type_field: String) -> Self {
        FieldType { nth, is_vl_encoded, is_serialized, is_signing_field, type_field }
    }

}

use crate::protos::ripple::{TMManifest, TMPing, TMCluster};
use protobuf::ProtobufError;

pub fn handle_message(message_type: u16, payload: &[u8]) {
    let proto_message: impl Default = match message_type {
        2 => protobuf::Message::parse_from_bytes(&payload).unwrap(): TMManifest,
        3 => protobuf::Message::parse_from_bytes(&payload).unwrap(): TMPing,
        5 => protobuf::Message::parse_from_bytes(&payload).unwrap(): TMCluster,
        // 5 => "mtCLUSTER",
        // 15 => "mtENDPOINTS",
        // 30 => "mtTRANSACTION",
        // 31 => "mtGET_LEDGER",
        // 32 => "mtLEDGER_DATA",
        // 33 => "mtPROPOSE_LEDGER",
        // 34 => "mtSTATUS_CHANGE",
        // 35 => "mtHAVE_SET",
        // 41 => "mtVALIDATION",
        // 42 => "mtGET_OBJECTS",
        // 50 => "mtGET_SHARD_INFO",
        // 51 => "mtSHARD_INFO",
        // 52 => "mtGET_PEER_SHARD_INFO",
        // 53 => "mtPEER_SHARD_INFO",
        // 54 => "mtVALIDATORLIST",
        _ => TMManifest::new()
    };
}
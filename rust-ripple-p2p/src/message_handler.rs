// use crate::protos::ripple::{TMManifest, TMPing, TMCluster};
// use protobuf::ProtobufError;

// pub fn invoke_protocol_message<T: protobuf::Message + ?Sized>(message_type: u16, payload: &[u8]) -> Box<T> {
//     let proto_message: Box<T> = match message_type {
//         2 => parse_message::<TMManifest>(&payload),
//         3 => Box::<T>::new(protobuf::Message::parse_from_bytes(&payload).unwrap()),
//         5 => Box::<T>::new(protobuf::Message::parse_from_bytes(&payload).unwrap()),
//         // 5 => "mtCLUSTER",
//         // 15 => "mtENDPOINTS",
//         // 30 => "mtTRANSACTION",
//         // 31 => "mtGET_LEDGER",
//         // 32 => "mtLEDGER_DATA",
//         // 33 => "mtPROPOSE_LEDGER",
//         // 34 => "mtSTATUS_CHANGE",
//         // 35 => "mtHAVE_SET",
//         // 41 => "mtVALIDATION",
//         // 42 => "mtGET_OBJECTS",
//         // 50 => "mtGET_SHARD_INFO",
//         // 51 => "mtSHARD_INFO",
//         // 52 => "mtGET_PEER_SHARD_INFO",
//         // 53 => "mtPEER_SHARD_INFO",
//         // 54 => "mtVALIDATORLIST",
//         _ => panic!("Unknown message")
//     };
//     return proto_message
// }
//
// pub fn parse_message<T: protobuf::Message + ?Sized>(payload: &[u8]) -> Box<T> {
//     return Box::<T>::new(protobuf::Message::parse_from_bytes(&payload).unwrap())
// }
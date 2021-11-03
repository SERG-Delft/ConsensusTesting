use std::fmt::{Debug, Display, Formatter};
use crate::protos::ripple::{TMManifest, TMPing, TMCluster, TMEndpoints, TMTransaction, TMGetLedger, TMLedgerData, TMProposeSet, TMStatusChange, TMHaveTransactionSet, TMValidation, TMGetObjectByHash, TMGetShardInfo, TMShardInfo, TMGetPeerShardInfo, TMPeerShardInfo, TMValidatorList, TMPing_pingType};
use protobuf::{Message, ProtobufEnum};
use openssl::ssl::SslStream;
use std::net::TcpStream;
use byteorder::{BigEndian, ByteOrder};
use std::io::Write;
use log::debug;
use hex;
use serde_json;

/// Deserialize message
pub fn invoke_protocol_message(message_type: u16, payload: &[u8]) -> RippleMessageObject {
    let proto_message: RippleMessageObject = match message_type {
        2 => RippleMessageObject::TMManifest(parse_message::<TMManifest>(&payload)),
        3 => RippleMessageObject::TMPing(parse_message::<TMPing>(&payload)),
        5 => RippleMessageObject::TMCluster(parse_message::<TMCluster>(&payload)),
        15 => RippleMessageObject::TMEndpoints(parse_message::<TMEndpoints>(&payload)),
        30 => RippleMessageObject::TMTransaction(parse_message::<TMTransaction>(&payload)),
        31 => RippleMessageObject::TMGetLedger(parse_message::<TMGetLedger>(&payload)),
        32 => RippleMessageObject::TMLedgerData(parse_message::<TMLedgerData>(&payload)),
        33 => RippleMessageObject::TMProposeSet(parse_message::<TMProposeSet>(&payload)),
        34 => RippleMessageObject::TMStatusChange(parse_message::<TMStatusChange>(&payload)),
        35 => RippleMessageObject::TMHaveTransactionSet(parse_message::<TMHaveTransactionSet>(&payload)),
        41 => RippleMessageObject::TMValidation(parse_message::<TMValidation>(&payload)),
        42 => RippleMessageObject::TMGetObjectByHash(parse_message::<TMGetObjectByHash>(&payload)),
        50 => RippleMessageObject::TMGetShardInfo(parse_message::<TMGetShardInfo>(&payload)),
        51 => RippleMessageObject::TMShardInfo(parse_message::<TMShardInfo>(&payload)),
        52 => RippleMessageObject::TMGetPeerShardInfo(parse_message::<TMGetPeerShardInfo>(&payload)),
        53 => RippleMessageObject::TMPeerShardInfo(parse_message::<TMPeerShardInfo>(&payload)),
        54 => RippleMessageObject::TMValidatorList(parse_message::<TMValidatorList>(&payload)),
        _ => panic!("Unknown message {}", message_type)
    };
    return proto_message
}

pub fn parse_message<T: protobuf::Message>(payload: &[u8]) -> T {
    return protobuf::Message::parse_from_bytes(&payload).unwrap()
}

#[derive(Debug)]
pub enum RippleMessageObject {
    TMManifest(TMManifest),
    TMPing(TMPing),
    TMCluster(TMCluster),
    TMEndpoints(TMEndpoints),
    TMTransaction(TMTransaction),
    TMGetLedger(TMGetLedger),
    TMLedgerData(TMLedgerData),
    TMProposeSet(TMProposeSet),
    TMStatusChange(TMStatusChange),
    TMHaveTransactionSet(TMHaveTransactionSet),
    TMValidation(TMValidation),
    TMGetObjectByHash(TMGetObjectByHash),
    TMGetShardInfo(TMGetShardInfo),
    TMShardInfo(TMShardInfo),
    TMGetPeerShardInfo(TMGetPeerShardInfo),
    TMPeerShardInfo(TMPeerShardInfo),
    TMValidatorList(TMValidatorList)
}

impl Display for RippleMessageObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (name, string) = match self {
            RippleMessageObject::TMManifest(manifest) => ("Manifest", serde_json::to_string(manifest).unwrap()),
            RippleMessageObject::TMPing(ping) => ("Ping", serde_json::to_string(ping).unwrap()),
            RippleMessageObject::TMCluster(cluster) => ("Cluster", serde_json::to_string(cluster).unwrap()),
            RippleMessageObject::TMEndpoints(endpoints) => ("Endpoints", serde_json::to_string(endpoints).unwrap()),
            RippleMessageObject::TMTransaction(transaction) => ("Transaction", serde_json::to_string(transaction).unwrap()),
            RippleMessageObject::TMGetLedger(get_ledger) => ("GetLedger", serde_json::to_string(get_ledger).unwrap()),
            RippleMessageObject::TMLedgerData(ledger_data) => ("LedgerData", serde_json::to_string(ledger_data).unwrap()),
            RippleMessageObject::TMProposeSet(propose_set) => ("ProposeSet", serde_json::to_string(propose_set).unwrap()),
            RippleMessageObject::TMStatusChange(status_change) => ("StatusChange", serde_json::to_string(status_change).unwrap()),
            RippleMessageObject::TMHaveTransactionSet(have_transaction_set) => ("HaveTransactionSet", serde_json::to_string(have_transaction_set).unwrap()),
            RippleMessageObject::TMValidation(validation) => ("Validation", serde_json::to_string(validation).unwrap()),
            RippleMessageObject::TMGetObjectByHash(get_object_by_hash) => ("GetObjectByHash", serde_json::to_string(get_object_by_hash).unwrap()),
            RippleMessageObject::TMGetShardInfo(get_shard_info) => ("GetShardInfo", serde_json::to_string(get_shard_info).unwrap()),
            RippleMessageObject::TMShardInfo(shard_info) => ("ShardInfo", serde_json::to_string(shard_info).unwrap()),
            RippleMessageObject::TMGetPeerShardInfo(get_peer_shard_info) => ("GetPeerShardInfo", serde_json::to_string(get_peer_shard_info).unwrap()),
            RippleMessageObject::TMPeerShardInfo(peer_shard_info) => ("PeerShardInfo", serde_json::to_string(peer_shard_info).unwrap()),
            RippleMessageObject::TMValidatorList(validator_list) => ("ValidatorList", serde_json::to_string(validator_list).unwrap()),
        };
        write!(f, "{}: {}", name, string)
    }
}

impl Display for TMManifest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.descriptor().name(), hex::encode_upper(self.get_stobject()))
    }
}

impl Display for TMTransaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}, {}, {}, {}", self.descriptor().name(), self.get_status(), self.get_receiveTimestamp(), self.get_deferred(), hex::encode_upper(self.get_rawTransaction()))
    }
}

// impl Display for TMGetLedger {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}: {}, {:?}, {:?}, {}, {}, {}", self.descriptor().name(), self.get_ledgerSeq(), self.get_itype(), self.get_ltype(), hex::encode_upper(self.get_ledgerHash()), self.get_nodeIDs().)
//     }
// }


// fn return_pong(mut pong: Box<TMPing>, ssl_stream: &mut SslStream<TcpStream>) {
//     let message_size: usize = (pong.compute_size() + 6) as usize;
//     pong.set_field_type(TMPing_pingType::ptPONG);
//     let mut write_vec = vec![0; message_size];
//     let write_bytes: &mut [u8] = write_vec.as_mut_slice();
//     BigEndian::write_u32(&mut write_bytes[0..4], (message_size - 6) as u32);
//     BigEndian::write_u16(&mut write_bytes[4..6], 3);
//     write_bytes[6..message_size].clone_from_slice(&*pong.write_to_bytes().unwrap());
//     match ssl_stream.write_all(write_bytes) {
//         Ok(_) => debug!("Pong successful"),
//         Err(err) => debug!("Pong error occurred: {:?}", err)
//     };
// }
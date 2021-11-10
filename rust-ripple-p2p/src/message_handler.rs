use log::error;
use std::fmt::{Debug, Display, Formatter};
use protobuf::Message;
use crate::protos::ripple::{TMManifest, TMPing, TMCluster, TMEndpoints, TMTransaction, TMGetLedger, TMLedgerData, TMProposeSet, TMStatusChange, TMHaveTransactionSet, TMValidation, TMGetObjectByHash, TMGetShardInfo, TMShardInfo, TMGetPeerShardInfo, TMPeerShardInfo, TMValidatorList};
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

impl RippleMessageObject {
    pub fn from_str(variant: &str, json: &str) -> Vec<u8> {
        match variant {
            "Manifest" => serde_json::from_str::<TMManifest>(json).unwrap().write_to_bytes().unwrap(),
            "Ping" => serde_json::from_str::<TMPing>(json).unwrap().write_to_bytes().unwrap(),
            "Cluster" => serde_json::from_str::<TMCluster>(json).unwrap().write_to_bytes().unwrap(),
            "Endpoints" => serde_json::from_str::<TMEndpoints>(json).unwrap().write_to_bytes().unwrap(),
            "Transaction" => serde_json::from_str::<TMTransaction>(json).unwrap().write_to_bytes().unwrap(),
            "GetLedger" => serde_json::from_str::<TMGetLedger>(json).unwrap().write_to_bytes().unwrap(),
            "LedgerData" => serde_json::from_str::<TMLedgerData>(json).unwrap().write_to_bytes().unwrap(),
            "ProposeSet" => serde_json::from_str::<TMProposeSet>(json).unwrap().write_to_bytes().unwrap(),
            "StatusChange" => serde_json::from_str::<TMStatusChange>(json).unwrap().write_to_bytes().unwrap(),
            "HaveTransactionSet" => serde_json::from_str::<TMHaveTransactionSet>(json).unwrap().write_to_bytes().unwrap(),
            "Validation" => serde_json::from_str::<TMValidation>(json).unwrap().write_to_bytes().unwrap(),
            "GetObjectByHash" => serde_json::from_str::<TMGetObjectByHash>(json).unwrap().write_to_bytes().unwrap(),
            "GetShardInfo" => serde_json::from_str::<TMGetShardInfo>(json).unwrap().write_to_bytes().unwrap(),
            "ShardInfo" => serde_json::from_str::<TMShardInfo>(json).unwrap().write_to_bytes().unwrap(),
            "GetPeerShardInfo" => serde_json::from_str::<TMGetPeerShardInfo>(json).unwrap().write_to_bytes().unwrap(),
            "PeerShardInfo" => serde_json::from_str::<TMPeerShardInfo>(json).unwrap().write_to_bytes().unwrap(),
            "ValidatorList" => serde_json::from_str::<TMValidatorList>(json).unwrap().write_to_bytes().unwrap(),
            _ => error!()
        }
    }
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

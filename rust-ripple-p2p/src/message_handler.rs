use crate::protos::ripple::{
    TMCluster, TMEndpoints, TMGetLedger, TMGetObjectByHash, TMGetPeerShardInfo, TMGetShardInfo,
    TMHaveTransactionSet, TMLedgerData, TMManifest, TMPeerShardInfo, TMPing, TMProposeSet,
    TMShardInfo, TMStatusChange, TMTransaction, TMValidation, TMValidatorList,
};
use byteorder::{BigEndian, ByteOrder};
use openssl::sha::sha256;
use serde_json;
use std::fmt::{Debug, Display, Formatter};
// use crate::deserialization::{deserialize_validation};

pub fn from_bytes(bytes: &[u8]) -> RippleMessageObject {
    invoke_protocol_message(BigEndian::read_u16(&bytes[4..6]), &bytes[6..])
}

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
        35 => RippleMessageObject::TMHaveTransactionSet(parse_message::<TMHaveTransactionSet>(
            &payload,
        )),
        41 => RippleMessageObject::TMValidation(parse_message::<TMValidation>(&payload)),
        42 => RippleMessageObject::TMGetObjectByHash(parse_message::<TMGetObjectByHash>(&payload)),
        50 => RippleMessageObject::TMGetShardInfo(parse_message::<TMGetShardInfo>(&payload)),
        51 => RippleMessageObject::TMShardInfo(parse_message::<TMShardInfo>(&payload)),
        52 => {
            RippleMessageObject::TMGetPeerShardInfo(parse_message::<TMGetPeerShardInfo>(&payload))
        }
        53 => RippleMessageObject::TMPeerShardInfo(parse_message::<TMPeerShardInfo>(&payload)),
        54 => RippleMessageObject::TMValidatorList(parse_message::<TMValidatorList>(&payload)),
        _ => panic!("Unknown message {}", message_type),
    };
    return proto_message;
}

pub fn parse_message<T: protobuf::Message>(payload: &[u8]) -> T {
    return protobuf::Message::parse_from_bytes(&payload).unwrap();
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
    TMValidatorList(TMValidatorList),
}

impl RippleMessageObject {
    pub fn node_pub_key(&self) -> Option<String> {
        match self {
            RippleMessageObject::TMProposeSet(propose_set) => {
                let type_prefixed_key = [&[28u8], propose_set.get_nodePubKey()].concat();
                let checksum = sha256(&sha256(&type_prefixed_key));
                let propose_key = [&type_prefixed_key, &checksum[..4]].concat();
                let node_key = bs58::encode(propose_key.clone())
                    .with_alphabet(bs58::Alphabet::RIPPLE)
                    .into_string();
                Some(node_key)
            }
            _ => None,
        }
    }
}

impl Display for RippleMessageObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (name, string) = match self {
            RippleMessageObject::TMManifest(manifest) => {
                ("Manifest", serde_json::to_string(manifest).unwrap())
            }
            RippleMessageObject::TMPing(ping) => ("Ping", serde_json::to_string(ping).unwrap()),
            RippleMessageObject::TMCluster(cluster) => {
                ("Cluster", serde_json::to_string(cluster).unwrap())
            }
            RippleMessageObject::TMEndpoints(endpoints) => {
                ("Endpoints", serde_json::to_string(endpoints).unwrap())
            }
            RippleMessageObject::TMTransaction(transaction) => {
                ("Transaction", serde_json::to_string(transaction).unwrap())
            }
            RippleMessageObject::TMGetLedger(get_ledger) => {
                ("GetLedger", serde_json::to_string(get_ledger).unwrap())
            }
            RippleMessageObject::TMLedgerData(ledger_data) => {
                ("LedgerData", serde_json::to_string(ledger_data).unwrap())
            }
            RippleMessageObject::TMProposeSet(propose_set) => {
                ("ProposeSet", serde_json::to_string(propose_set).unwrap())
            }
            RippleMessageObject::TMStatusChange(status_change) => (
                "StatusChange",
                serde_json::to_string(status_change).unwrap(),
            ),
            RippleMessageObject::TMHaveTransactionSet(have_transaction_set) => (
                "HaveTransactionSet",
                serde_json::to_string(have_transaction_set).unwrap(),
            ),
            RippleMessageObject::TMValidation(validation) => {
                ("Validation", serde_json::to_string(validation).unwrap())
            }
            RippleMessageObject::TMGetObjectByHash(get_object_by_hash) => (
                "GetObjectByHash",
                serde_json::to_string(get_object_by_hash).unwrap(),
            ),
            RippleMessageObject::TMGetShardInfo(get_shard_info) => (
                "GetShardInfo",
                serde_json::to_string(get_shard_info).unwrap(),
            ),
            RippleMessageObject::TMShardInfo(shard_info) => {
                ("ShardInfo", serde_json::to_string(shard_info).unwrap())
            }
            RippleMessageObject::TMGetPeerShardInfo(get_peer_shard_info) => (
                "GetPeerShardInfo",
                serde_json::to_string(get_peer_shard_info).unwrap(),
            ),
            RippleMessageObject::TMPeerShardInfo(peer_shard_info) => (
                "PeerShardInfo",
                serde_json::to_string(peer_shard_info).unwrap(),
            ),
            RippleMessageObject::TMValidatorList(validator_list) => (
                "ValidatorList",
                serde_json::to_string(validator_list).unwrap(),
            ),
        };
        write!(f, "{}: {}", name, string)
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ParsedValidation {
    pub ledger_sequence: u32,
    pub validated_hash: String,
    pub hash: String,
    pub consensus_hash: String,
    pub cookie: u64,
    pub signing_pub_key: String,
    pub signature: String,
    pub flags: u32,
    pub signing_time: u32,
}

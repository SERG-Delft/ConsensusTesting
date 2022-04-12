use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use byteorder::{BigEndian, ByteOrder};
use crate::protos::ripple::{TMManifest, TMPing, TMCluster, TMEndpoints, TMTransaction, TMGetLedger, TMLedgerData, TMProposeSet, TMStatusChange, TMHaveTransactionSet, TMValidation, TMGetObjectByHash, TMGetShardInfo, TMShardInfo, TMGetPeerShardInfo, TMPeerShardInfo, TMValidatorList};
use serde_json;
use crate::deserialization::{deserialize_validation};

/// Deserialize message
pub fn parse_protocol_message(message_type: u16, payload: &[u8]) -> RippleMessageObject {
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

pub fn write_message<T: protobuf::Message>(message_type: u16, message: T) -> Vec<u8> {
    let payload = match message.write_to_bytes() {
        Ok(res) => res,
        Err(err) => panic!("Writing protobuf message to bytes failed: {}", err)
    };
    let mut payload_size_buf = [0;4];
    BigEndian::write_u32(&mut payload_size_buf, payload.len() as u32);
    let mut message_type_buf = [0;2];
    BigEndian::write_u16(&mut message_type_buf, message_type);
    payload_size_buf.iter().copied().chain(message_type_buf.iter().copied()).chain(payload.into_iter()).collect()
}

pub fn rmo_to_bytes(rmo: RippleMessageObject) -> Vec<u8> {
    match rmo {
        RippleMessageObject::TMManifest(manifest) => write_message(2, manifest),
        RippleMessageObject::TMPing(ping) => write_message(3, ping),
        RippleMessageObject::TMCluster(cluster) => write_message(5, cluster),
        RippleMessageObject::TMEndpoints(endpoints) => write_message(15, endpoints),
        RippleMessageObject::TMTransaction(transaction) => write_message(30, transaction),
        RippleMessageObject::TMGetLedger(get_ledger) => write_message(31, get_ledger),
        RippleMessageObject::TMLedgerData(ledger_data) => write_message(32, ledger_data),
        RippleMessageObject::TMProposeSet(propose_set) => write_message(33, propose_set),
        RippleMessageObject::TMStatusChange(status_change) => write_message(34, status_change),
        RippleMessageObject::TMHaveTransactionSet(have_transaction_set) => write_message(35, have_transaction_set),
        RippleMessageObject::TMValidation(validation) => write_message(41, validation),
        RippleMessageObject::TMGetObjectByHash(get_object_by_hash) => write_message(42, get_object_by_hash),
        RippleMessageObject::TMGetShardInfo(get_shard_info) => write_message(50, get_shard_info),
        RippleMessageObject::TMShardInfo(shard_info) => write_message(51, shard_info),
        RippleMessageObject::TMGetPeerShardInfo(get_peer_shard_info) => write_message(52, get_peer_shard_info),
        RippleMessageObject::TMPeerShardInfo(peer_shard_info) => write_message(53, peer_shard_info),
        RippleMessageObject::TMValidatorList(validator_list) => write_message(54, validator_list),
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    pub fn message_type(&self) -> &'static str {
        match self {
            RippleMessageObject::TMManifest(_) => "Manifest",
            RippleMessageObject::TMPing(_) => "Ping",
            RippleMessageObject::TMCluster(_) => "Cluster",
            RippleMessageObject::TMEndpoints(_) => "Endpoints",
            RippleMessageObject::TMTransaction(_) => "Transaction",
            RippleMessageObject::TMGetLedger(_) => "GetLedger",
            RippleMessageObject::TMLedgerData(_) => "LedgerData",
            RippleMessageObject::TMProposeSet(_) => "ProposeSet",
            RippleMessageObject::TMStatusChange(_) => "StatusChange",
            RippleMessageObject::TMHaveTransactionSet(_) => "HaveTransactionSet",
            RippleMessageObject::TMValidation(_) => "Validation",
            RippleMessageObject::TMGetObjectByHash(_) => "GetObjectByHash",
            RippleMessageObject::TMGetShardInfo(_) => "GetShardInfo",
            RippleMessageObject::TMShardInfo(_) => "ShardInfo",
            RippleMessageObject::TMGetPeerShardInfo(_) => "GetPeerShardInfo",
            RippleMessageObject::TMPeerShardInfo(_) => "PeerShardInfo",
            RippleMessageObject::TMValidatorList(_) => "ValidatorList",
        }
    }
}

impl Hash for RippleMessageObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.message_type().hash(state);
    }
}

impl Default for RippleMessageObject {
    fn default() -> Self {
        RippleMessageObject::TMProposeSet(TMProposeSet::default())
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
            RippleMessageObject::TMValidation(validation) => {
                ("Validation", serde_json::to_string(&ParsedValidation::new(validation)).unwrap())
                // ("Validation", serde_json::to_string(validation).unwrap())
            },
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

impl ParsedValidation {
    pub fn new(validation: &TMValidation) -> Self {
        deserialize_validation(validation.get_validation().clone())
    }
}
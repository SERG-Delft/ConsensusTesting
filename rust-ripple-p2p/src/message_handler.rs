use protobuf::ProtobufError;
use serialize::ripple::{
    TMCluster, TMEndpoints, TMGetLedger, TMGetObjectByHash, TMGetPeerShardInfo, TMGetShardInfo,
    TMHaveTransactionSet, TMLedgerData, TMManifest, TMPeerShardInfo, TMPing, TMProposeSet,
    TMShardInfo, TMStatusChange, TMTransaction, TMValidation, TMValidatorList,
};
use serialize::RippleMessageObject;
use std::fmt::Debug;

pub fn from_bytes(bytes: &[u8]) -> Result<RippleMessageObject, ProtobufError> {
    invoke_protocol_message(
        u16::from_be_bytes(bytes[4..6].try_into().unwrap()),
        &bytes[6..],
    )
}

/// Deserialize message
pub fn invoke_protocol_message(
    message_type: u16,
    payload: &[u8],
) -> Result<RippleMessageObject, ProtobufError> {
    let proto_message: RippleMessageObject = match message_type {
        2 => RippleMessageObject::TMManifest(parse_message::<TMManifest>(payload)?),
        3 => RippleMessageObject::TMPing(parse_message::<TMPing>(payload)?),
        5 => RippleMessageObject::TMCluster(parse_message::<TMCluster>(payload)?),
        15 => RippleMessageObject::TMEndpoints(parse_message::<TMEndpoints>(payload)?),
        30 => RippleMessageObject::TMTransaction(parse_message::<TMTransaction>(payload)?),
        31 => RippleMessageObject::TMGetLedger(parse_message::<TMGetLedger>(payload)?),
        32 => RippleMessageObject::TMLedgerData(parse_message::<TMLedgerData>(payload)?),
        33 => RippleMessageObject::TMProposeSet(parse_message::<TMProposeSet>(payload)?),
        34 => RippleMessageObject::TMStatusChange(parse_message::<TMStatusChange>(payload)?),
        35 => RippleMessageObject::TMHaveTransactionSet(parse_message::<TMHaveTransactionSet>(
            payload,
        )?),
        41 => RippleMessageObject::TMValidation(parse_message::<TMValidation>(payload)?),
        42 => RippleMessageObject::TMGetObjectByHash(parse_message::<TMGetObjectByHash>(payload)?),
        50 => RippleMessageObject::TMGetShardInfo(parse_message::<TMGetShardInfo>(payload)?),
        51 => RippleMessageObject::TMShardInfo(parse_message::<TMShardInfo>(payload)?),
        52 => {
            RippleMessageObject::TMGetPeerShardInfo(parse_message::<TMGetPeerShardInfo>(payload)?)
        }
        53 => RippleMessageObject::TMPeerShardInfo(parse_message::<TMPeerShardInfo>(payload)?),
        54 => RippleMessageObject::TMValidatorList(parse_message::<TMValidatorList>(payload)?),
        _ => return Err(ProtobufError::WireError(protobuf::error::WireError::Other)),
    };
    Ok(proto_message)
}

pub fn parse_message<T: protobuf::Message>(payload: &[u8]) -> Result<T, ProtobufError> {
    protobuf::Message::parse_from_bytes(payload)
}

#[derive(Default, serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
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

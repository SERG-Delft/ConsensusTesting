use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use openssl::sha::sha256;
use parser::parse;

pub mod parser;

include!(concat!(env!("OUT_DIR"), "/mod.rs"));

use crate::ripple::{
    TMCluster, TMEndpoints, TMGetLedger, TMGetObjectByHash, TMGetPeerShardInfo, TMGetShardInfo,
    TMHaveTransactionSet, TMLedgerData, TMManifest, TMPeerShardInfo, TMPing, TMProposeSet,
    TMShardInfo, TMStatusChange, TMTransaction, TMValidation, TMValidatorList,
};

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

#[derive(Debug)]
pub struct RippleMessage {
    pub from_node: String,
    pub to_node: String,
    pub timestamp: DateTime<Utc>,
    pub message: RippleMessageObject,
}

impl RippleMessage {
    pub fn new(
        from_node: String,
        to_node: String,
        timestamp: DateTime<Utc>,
        message: RippleMessageObject,
    ) -> Box<Self> {
        Box::from(RippleMessage {
            from_node,
            to_node,
            timestamp,
            message,
        })
    }
}

impl RippleMessageObject {
    pub fn node_pub_key(&self) -> Option<String> {
        match self {
            RippleMessageObject::TMProposeSet(propose_set) => {
                let type_prefixed_key = [&[28u8], propose_set.get_nodePubKey()].concat();
                let checksum = sha256(&sha256(&type_prefixed_key));
                let propose_key = [&type_prefixed_key, &checksum[..4]].concat();
                let node_key = bs58::encode(propose_key)
                    .with_alphabet(bs58::Alphabet::RIPPLE)
                    .into_string();
                Some(node_key)
            }
            _ => None,
        }
    }
}

impl Display for RippleMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            RippleMessageObject::TMValidation(validation) => {
                let ripple_epoch =
                    DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap();
                let from_node_buf = &self.from_node;
                let to_node_buf = &self.to_node;
                let time_since = self
                    .timestamp
                    .signed_duration_since(ripple_epoch)
                    .num_seconds();
                // write!(f, "{}\n", hex::encode(validation.get_validation()));
                let parsed = parse(validation.get_validation());
                if parsed.is_err() {
                    return writeln!(f, "-- cannot parse validation");
                }
                let parsed = parsed.unwrap().1;

                // let pub_key = self.message.node_pub_key();
                let type_prefixed_key = [
                    &[28u8],
                    hex::decode(parsed["SigningPubKey"].as_str().unwrap())
                        .unwrap()
                        .as_slice(),
                ]
                .concat();
                let checksum = sha256(&sha256(&type_prefixed_key));
                let key = [&type_prefixed_key, &checksum[..4]].concat();
                let node_key = Some(
                    bs58::encode(key)
                        .with_alphabet(bs58::Alphabet::RIPPLE)
                        .into_string(),
                );
                let node = match node_key {
                    Some(ref key) => match key.as_str() {
                        "n9KkgT2SFxpQGic7peyokvkXcAmNLFob1AZXeErMFHxJ71q5MGaK" => "0",
                        "n9M6ouZU7cLwRHPiVZjgJdEgrVyx2uv9euZzepdb34wDoj1RP5uS" => "1",
                        "n9LJhBqLGTjPQa2KJtJmkHUubaHs1Y1ENYKZVmzZYhNb7GXh9m4j" => "2",
                        "n9KgN4axJo1WC3fjFoUSkJ4gtZX4Pk2jPZzGR5CE9ddo16ewAPjN" => "3",
                        "n9MsRMobdfpGvpXeGb3F6bm7WZbCiPrxzc1qBPP7wQox3NJzs5j2" => "4",
                        "n9JFX46v3d3WgQW8DJQeBwqTk8vaCR7LufApEy65J1eK4X7dZbR3" => "5",
                        "n9LFueHyYVJSyDArog2qtR42NixmeGxpaqFEFFp1xjxGU9aYRDZc" => "6",
                        _ => key.as_str(),
                    },
                    None => panic!("needs node key"),
                };
                writeln!(
                    f,
                    "-- {} [{}->{}] Validation {} validates {}",
                    time_since, from_node_buf, to_node_buf, node, parsed
                )
                .unwrap();
            }
            RippleMessageObject::TMProposeSet(proposal) => {
                let ripple_epoch =
                    DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap();
                let from_node_buf = &self.from_node;
                let to_node_buf = &self.to_node;
                let time_since = self
                    .timestamp
                    .signed_duration_since(ripple_epoch)
                    .num_seconds();
                let pub_key = self.message.node_pub_key();
                let node = match pub_key {
                    Some(ref key) => match key.as_str() {
                        "n9KkgT2SFxpQGic7peyokvkXcAmNLFob1AZXeErMFHxJ71q5MGaK" => "0",
                        "n9M6ouZU7cLwRHPiVZjgJdEgrVyx2uv9euZzepdb34wDoj1RP5uS" => "1",
                        "n9LJhBqLGTjPQa2KJtJmkHUubaHs1Y1ENYKZVmzZYhNb7GXh9m4j" => "2",
                        "n9KgN4axJo1WC3fjFoUSkJ4gtZX4Pk2jPZzGR5CE9ddo16ewAPjN" => "3",
                        "n9MsRMobdfpGvpXeGb3F6bm7WZbCiPrxzc1qBPP7wQox3NJzs5j2" => "4",
                        "n9JFX46v3d3WgQW8DJQeBwqTk8vaCR7LufApEy65J1eK4X7dZbR3" => "5",
                        "n9LFueHyYVJSyDArog2qtR42NixmeGxpaqFEFFp1xjxGU9aYRDZc" => "6",
                        _ => key.as_str(),
                    },
                    None => panic!("needs node key"),
                };
                writeln!(
                    f,
                    "-- {} [{}->{}] ProposeSet<{} proposes {}, seq={}, prev={}>",
                    time_since,
                    from_node_buf,
                    to_node_buf,
                    node,
                    hex::encode(&proposal.get_currentTxHash()[..2]),
                    proposal.get_proposeSeq(),
                    hex::encode(proposal.get_previousledger())
                )
                .unwrap();
            }
            _ => {}
        };

        let ripple_epoch = DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap();
        let from_node_buf = &self.from_node;
        let to_node_buf = &self.to_node;
        let time_since = self
            .timestamp
            .signed_duration_since(ripple_epoch)
            .num_seconds();
        let message_buf = self.message.to_string();
        writeln!(
            f,
            "{} {} [{}->{}] sent {}",
            time_since,
            self.message.node_pub_key().get_or_insert("".to_string()),
            from_node_buf,
            to_node_buf,
            message_buf
        )
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

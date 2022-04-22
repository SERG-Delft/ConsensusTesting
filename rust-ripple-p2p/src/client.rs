use websocket::{ClientBuilder, OwnedMessage, Message};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use serde_json::{json, Value};
use serde::{Serialize, Deserialize};
use std::thread::JoinHandle;
use log::*;

/// Client struct responsible for handling websocket connection to ripple node
/// Start a send and receive thread,
/// To send to node, use sender_channel
/// Handle received messages in receive loop
#[allow(unused)]
pub struct Client<'a> {
    peer: u16,
    pub sender_channel: Sender<Message<'a>>,
    send_loop: JoinHandle<()>,
    receive_loop: JoinHandle<()>
}

impl Client<'static> {
    pub fn new(
        peer: u16,
        connection: &str,
        subscription_collector_sender: Sender<PeerSubscriptionObject>,
        server_state_collector_sender: Sender<PeerServerStateObject>,
        test_harness_sender: Sender<(Transaction, String)>,
    ) -> Self {
        let client = ClientBuilder::new(connection)
            .unwrap()
            .connect_insecure()
            .unwrap();

        let (mut receiver, mut sender) = client.split().unwrap();

        let (tx, rx) = channel();

        let tx_1: Sender<Message> = tx.clone();

        let send_loop = thread::spawn(move || {
            loop {
                // Send loop
                let message = match rx.recv() {
                    Ok(m) => m,
                    Err(e) => {
                        trace!("Send Loop: {:?}", e);
                        return;
                    }
                };
                // Send the message
                match sender.send_message(&message) {
                    Ok(()) => {
                        trace!("Send Loop sent message: {:?}", message);
                        ()
                    },
                    Err(e) => {
                        error!("Send Loop: {:?}", e);
                        let _ = sender.send_message(&Message::close());
                        return;
                    }
                }
            }
        });

        let receive_loop = thread::spawn(move || {
            // Receive loop
            for message in receiver.incoming_messages() {
                let message = match message {
                    Ok(m) => m,
                    Err(e) => {
                        warn!("Receive Loop erred: {:?}", e);
                        let _ = tx_1.send(Message::from(OwnedMessage::Close(None)));
                        return;
                    }
                };
                match message {
                    OwnedMessage::Text(text) => {
                        match serde_json::from_str(text.as_str()) {
                            Ok(v) => {
                                let value: Value = v;
                                match value["id"].as_str() {
                                    Some("server_state") => {
                                        match serde_json::from_value::<ServerStateObject>(value["result"]["state"].clone()) {
                                            Ok(server_state_object) => {
                                                server_state_collector_sender.send(PeerServerStateObject::new(peer, server_state_object)).unwrap();
                                            },
                                            Err(_) => { println!("Could not parse peer{} server_state object: {}", peer, text); }
                                        }
                                    }
                                    Some("Test harness") => {
                                        let transaction = match serde_json::from_value::<Transaction>(value["result"]["tx_json"].clone()) {
                                            Ok(transaction) => transaction,
                                            Err(_) => {
                                                error!("peer{} Test harness: {}", peer, text.as_str());
                                                match serde_json::from_value::<Transaction>(value["request"]["tx_json"].clone()) {
                                                    Ok(transaction) => transaction,
                                                    Err(_) => panic!("Could not even parse request")
                                                }
                                            }
                                        };
                                        let engine_result = serde_json::from_value(value["result"]["engine_result"].clone()).unwrap_or_else(|_|"error".to_string());
                                        test_harness_sender.send((transaction, engine_result)).unwrap();
                                    }
                                    None => match serde_json::from_value::<SubscriptionObject>(value) {
                                        Ok(subscription_object) => {
                                            subscription_collector_sender.send(PeerSubscriptionObject::new(peer, subscription_object)).unwrap();
                                        },
                                        Err(_) => { warn!("Could not parse peer{} subscription object: {}", peer, text); }
                                    },
                                    _ => {}
                                }
                            },
                            _ => { warn!("Unknown client message from peer: {}", peer) }
                        }
                    },
                    _ => warn!("Receive Loop: {:?}", message)
                }
            }
        });

        // Start subscriptions
        Client::subscribe(&tx, "subscription", vec!["consensus", "ledger", "validations", "peer_status", "transactions_proposed", "server"]);

        Client {
            peer,
            sender_channel: tx,
            send_loop,
            receive_loop
        }
    }

    #[allow(unused)]
    pub fn start(self) {
        self.send_loop.join().unwrap();
        self.receive_loop.join().unwrap();
    }

    // Create a payment transaction
    #[allow(unused)]
    pub fn create_payment_transaction(
        amount: u32,
        destination_id: &str,
        sender_address: &str,
        sequence: Option<u32>,
        include_fee: bool
    ) -> Transaction
    {
        // Create payment object for payment to account
        let amount = if include_fee {
            amount * 10u32.pow(7) + 10
        } else {
            amount * 10u32.pow(7)
        };
        let payment = Payment {
            amount: amount.to_string(),
            destination: String::from(destination_id),
            destination_tag: None,
            invoice_id: None,
            send_max: None,
            deliver_min: None
        };

        // Create transaction object containing the payment
        Transaction {
            data: Some(payment),
            account: String::from(sender_address),
            transaction_type: TransactionType::Payment,
            fee: None,
            sequence,
            account_txn_id: None,
            flags: None,
            last_ledger_sequence: None,
            source_tag: None,
            signing_pub_key: None,
            txn_signature: None,
            date: None,
            hash: None,
        }
    }

    #[allow(unused)]
    pub fn ping(&mut self, id: &str) {
        let json = json!({
            "id": id,
            "command": "ping"
        });
        self.sender_channel.send(Message::text(json.to_string())).unwrap();
    }

    #[allow(unused)]
    pub fn ledger(tx: &Sender<Message>, id: &str) {
        let json = json!({
            "id": id,
            "command": "ledger",
            "ledger_index": "current",
            "full": true,
            "accounts": true,
            "transactions": true,
            "expand": true,
            "owner_funds": true
        });
        tx.send(Message::text(json.to_string())).unwrap();
    }

    // Sign and submit a transaction to the network
    #[allow(unused)]
    pub fn sign_and_submit(tx: &Sender<Message>, id: &str, transaction: &Transaction, secret: &str) {
        let json = json!({
            "id": id,
            "command": "submit",
            "tx_json": transaction,
            "secret": secret,
            "fee_mult_max": 10000000,
        });
        match transaction.sequence {
            Some(sequence) => trace!("Sending transaction: {}", sequence),
            None => {},
        }

        match tx.send(Message::text(json.to_string())) {
            Ok(_) => {}
            Err(_) => error!("Client closed!")
        }
    }

    pub fn subscribe(tx: &Sender<Message>, id: &str, streams: Vec<&str>) {
        let json = json!({
            "id": id,
            "command": "subscribe",
            "streams": streams
        });
        tx.send(Message::text(json.to_string())).unwrap();
    }

    #[allow(unused)]
    pub fn server_state(tx: &Sender<Message>) {
        let json = json!({
            "id": "server_state",
            "command": "server_state"
        });
        tx.send(Message::text(json.to_string())).unwrap();
    }
}

/// A transaction struct containing some, but not all, fields a ripple transaction can hold
/// Used for communication with node by serde (de)serialization
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct Transaction {
    #[serde(rename = "Data", skip_serializing_if = "Option::is_none", flatten)]
    pub data: Option<Payment>,
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "TransactionType")]
    pub transaction_type: TransactionType,
    #[serde(rename = "Fee", skip_serializing_if = "Option::is_none")]
    pub fee: Option<String>,
    #[serde(rename = "Sequence", skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
    #[serde(rename = "AccountTxnID", skip_serializing_if = "Option::is_none")]
    pub account_txn_id: Option<String>,
    #[serde(rename = "Flags", skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    #[serde(rename = "LastLedgerSequence", skip_serializing_if = "Option::is_none")]
    pub last_ledger_sequence: Option<u32>,
    #[serde(rename = "SourceTag", skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    #[serde(rename = "SigningPubKey", skip_serializing_if = "Option::is_none")]
    pub signing_pub_key: Option<String>,
    #[serde(rename = "TxnSignature", skip_serializing_if = "Option::is_none")]
    pub txn_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// The different transaction types
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Hash)]
pub enum TransactionType {
    Payment,
    OfferCreate,
    OfferCancel,
    TrustSet,
    AccountSet,
    AccountDelete,
    SetRegularKey,
    SignerListSet,
    EscrowCreate,
    EscrowFinish,
    EscrowCancel,
    PaymentChannelCreate,
    PaymentChannelFund,
    PaymentChannelClaim,
    DepositPreauth,
    EnableAmendment,
    SetFee,
    UNLModify
}

impl Default for TransactionType {
    fn default() -> Self { TransactionType::Payment }
}

/// Fields specific to a payment transaction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default, Hash)]
pub struct Payment  {
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "Destination")]
    pub destination: String,
    #[serde(rename = "DestinationTag", skip_serializing_if = "Option::is_none")]
    pub destination_tag: Option<u32>,
    #[serde(rename = "InvoiceID", skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<String>,
    #[serde(rename = "SendMax", skip_serializing_if = "Option::is_none")]
    pub send_max: Option<u32>,
    #[serde(rename = "DeliverMin", skip_serializing_if = "Option::is_none")]
    pub deliver_min: Option<u32>
}

/// A validated ledger struct received from the ledger subscription stream
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ValidatedLedger {
    pub fee_base: u32,
    pub fee_ref: u32,
    pub ledger_hash: String,
    pub ledger_index: u32,
    pub ledger_time: u32,
    #[serde(skip_serializing)]
    pub reserve_base: u32,
    #[serde(skip_serializing)]
    pub reserve_inc: u32,
    pub txn_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validated_ledgers: Option<String>
}

/// A validation message received by the node from some other node (or itself)
/// Received from the validations subscription stream
#[allow(unused)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReceivedValidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    amendments: Option<Vec<String>>,
    base_fee: Option<u32>,
    #[serde(skip_serializing)]
    flags: u32,
    full: bool,
    ledger_hash: String,
    ledger_index: String,
    load_fee: Option<u32>,
    master_key: Option<String>,
    #[serde(skip_serializing)]
    reserve_base: Option<u32>,
    #[serde(skip_serializing)]
    reserve_inc: Option<u32>,
    #[serde(skip_serializing)]
    signature: String,
    signing_time: u32,
    validation_public_key: String
}

/// A type of peer status event, sent when a peer of this node changes status
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PeerStatusEvent {
    #[serde(rename = "CLOSING_LEDGER")]
    ClosingLedger,
    #[serde(rename = "ACCEPTED_LEDGER")]
    AcceptedLedger,
    #[serde(rename = "SWITCHED_LEDGER")]
    SwitchedLedger,
    #[serde(rename = "LOST_SYNC")]
    LostSync
}

/// A peer status event, sent when a peer of this node changes status
/// Sent by the peer_status subscription stream
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerStatusChange {
    action: PeerStatusEvent,
    date: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    ledger_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ledger_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ledger_index_max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ledger_index_min: Option<u32>,
}

/// A consensus phase change done by this node
/// Sent by the consensus subscription stream
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConsensusChange {
    pub consensus: String
}

/// The status of the subscribed to server
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerStatus {
    base_fee: u32,
    load_base: u32,
    load_factor: u32,
    load_factor_fee_escalation: u32,
    load_factor_fee_queue: u32,
    load_factor_fee_reference: u32,
    load_factor_server: u32,
    server_status: String,
}

/// The different types of subscription objects
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SubscriptionObject {
    #[serde(rename = "ledgerClosed")]
    ValidatedLedger(ValidatedLedger),
    #[serde(rename = "validationReceived")]
    ReceivedValidation(ReceivedValidation),
    #[serde(rename = "peerStatusChange")]
    PeerStatusChange(PeerStatusChange),
    #[serde(rename = "consensusPhase")]
    ConsensusChange(ConsensusChange),
    #[serde(rename = "transaction")]
    Transaction(TransactionSubscription),
    #[serde(rename = "serverStatus")]
    ServerStatus(ServerStatus),
}

/// A transaction subscription object, received whenever a ledger is closed with this transaction.
/// The same transaction is received again with validated true, when the ledger containing the transaction is validated.
/// Sent by the transaction_proposed subscription stream.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionSubscription {
    engine_result: String,
    engine_result_code: u32,
    engine_result_message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ledger_current_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ledger_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ledger_index: Option<u32>,
    pub transaction: Transaction,
    pub validated: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Ledger {
    base_fee: u32,
    close_time: u32,
    hash: String,
    reserve_base: u32,
    reserve_inc: u32,
    seq: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LastClose {
    converge_time: u32,
    proposers: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StateDetails {
    pub duration_us: String,
    pub transitions: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StateAccounting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected: Option<StateDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disconnected: Option<StateDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full: Option<StateDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syncing: Option<StateDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<StateDetails>,
}

impl StateAccounting {
    pub fn diff(state: &State, accounting_before: &StateAccounting, accounting_after: &StateAccounting) -> (u32, u32) {
        match state {
            State::Connected => Self::diff_individual(&accounting_before.connected, &accounting_after.connected),
            State::Disconnected => Self::diff_individual(&accounting_before.connected, &accounting_after.connected),
            State::_Full => Self::diff_individual(&accounting_before.full, &accounting_after.full),
            State::Syncing => Self::diff_individual(&accounting_before.syncing, &accounting_after.syncing),
            State::Tracking => Self::diff_individual(&accounting_before.tracking, &accounting_after.tracking),
        }
    }

    pub fn diff_individual(detail_before: &Option<StateDetails>, detail_after: &Option<StateDetails>) -> (u32, u32) {
        match detail_after {
            Some(after) => match detail_before {
                Some(before) => (after.duration_us.parse::<u32>().unwrap() - before.duration_us.parse::<u32>().unwrap(), after.transitions - before.transitions),
                None => (after.duration_us.parse::<u32>().unwrap(), after.transitions)
            }
            None => (0, 0)
        }
    }
}

pub enum State {
    Connected,
    Disconnected,
    _Full,
    Syncing,
    Tracking,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Load {
    job_types: Vec<Job>,
    threads: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Job {
    job_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avg_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    peak_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    per_second: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_progress: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ServerStateObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    amendment_blocked: Option<bool>,
    build_version: String,
    complete_ledgers: String,
    closed_ledger: Option<Ledger>,
    io_latency_ms: u32,
    jq_trans_overflow: String,
    last_close: LastClose,
    load: Load,
    load_base: u32,
    load_factor: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_factor_fee_escalation: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_factor_fee_queue: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_factor_fee_reference: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_factor_server: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    peer_disconnects: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    peer_disconnects_resources: Option<String>,
    peers: u32,
    pubkey_node: String,
    pubkey_validator: String,
    server_state: String,
    server_state_duration_us: String,
    pub state_accounting: StateAccounting,
    time: String,
    uptime: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    validated_ledger: Option<Ledger>,
    validation_quorum: u32,
    validator_list_expires: u32,
}

/// A subscription object coupled to a peer.
pub struct PeerSubscriptionObject {
    pub peer: u16,
    pub subscription_object: SubscriptionObject
}

impl PeerSubscriptionObject {
    fn new(peer: u16, subscription_object: SubscriptionObject) -> Self {
        PeerSubscriptionObject { peer, subscription_object }
    }
}

/// A server info object coupled to a peer
pub struct PeerServerStateObject {
    pub peer: u16,
    pub server_state_object: ServerStateObject
}

impl PeerServerStateObject {
    fn new(peer: u16, server_state_object: ServerStateObject) -> Self {
        PeerServerStateObject{ peer, server_state_object }
    }
}

#[cfg(test)]
mod client_tests {
    use serde_json::Value;
    use crate::client::{SubscriptionObject, Transaction};

    #[test]
    fn parse_transaction_subscription_test() {
        let text = String::from("{\"engine_result\":\"tesSUCCESS\",\"engine_result_code\":0,\"engine_result_message\":\"The transaction was applied. Only final in a validated ledger.\",\"ledger_hash\":\"26CEAA70664693084A374B2997E87EB12D1835B658070336F2BB00956A7034B6\",\"ledger_index\":257,\"meta\":{\"AffectedNodes\":[{\"CreatedNode\":{\"LedgerEntryType\":\"FeeSettings\",\"LedgerIndex\":\"4BC50C9B0D8515D3EAAE1E74B29A95804346C491EE1A95BF25E4AAB854A6A651\",\"NewFields\":{\"BaseFee\":\"a\",\"ReferenceFeeUnits\":10,\"ReserveBase\":20000000,\"ReserveIncrement\":5000000}}}],\"TransactionIndex\":0,\"TransactionResult\":\"tesSUCCESS\"},\"status\":\"closed\",\"transaction\":{\"Account\":\"rrrrrrrrrrrrrrrrrrrrrhoLvTp\",\"BaseFee\":\"a\",\"Fee\":\"0\",\"LedgerSequence\":257,\"ReferenceFeeUnits\":10,\"ReserveBase\":20000000,\"ReserveIncrement\":5000000,\"Sequence\":0,\"SigningPubKey\":\"\",\"TransactionType\":\"SetFee\",\"date\":703267220,\"hash\":\"9CCE3C7AD8ABF51C3E2B36D5BA8C1197BD3CAD20AD1B60BB7D036147D870008E\"},\"type\":\"transaction\",\"validated\":true}");
        let v: Value = serde_json::from_str(text.as_str()).unwrap();
        let res = serde_json::from_value::<SubscriptionObject>(v);
        assert!(res.is_ok());
    }

    #[test]
    fn parse_transaction_submit_test() {
        let text = String::from("{\"id\":\"Test harness\",\"result\":{\"deprecated\":\"Signing support in the 'submit' command has been deprecated and will be removed in a future version of the server. Please migrate to a standalone signing tool.\",\"engine_result\":\"tesSUCCESS\",\"engine_result_code\":0,\"engine_result_message\":\"The transaction was applied. Only final in a validated ledger.\",\"tx_blob\":\"1200002280000000240000000161400000003B9ACA0068400000000000000A73210330E7FC9D56BB25D6893BA3F317AE5BCF33B3291BD63DB32654A313222F7FD02074473045022100D39D6D57D44805CDEF0AC773170694C92D078234FF8C22FC0573E4C95BCC3D1E02203CF47C8D855EECDD48A4CCF62BEC09FA5F854EA970D400A3F4CADBFB88B1574F8114B5F762798A53D543A014CAF8B297CFF8F2F937E883147CC7B086211F8C6ECD22D2104BC3AC06A25B900F\",\"tx_json\":{\"Account\":\"rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh\",\"Amount\":\"1000000000\",\"Destination\":\"rU48rTg9WhAA4kTFSRDZnfbuxKGqSU9You\",\"Fee\":\"10\",\"Flags\":2147483648,\"Sequence\":1,\"SigningPubKey\":\"0330E7FC9D56BB25D6893BA3F317AE5BCF33B3291BD63DB32654A313222F7FD020\",\"TransactionType\":\"Payment\",\"TxnSignature\":\"3045022100D39D6D57D44805CDEF0AC773170694C92D078234FF8C22FC0573E4C95BCC3D1E02203CF47C8D855EECDD48A4CCF62BEC09FA5F854EA970D400A3F4CADBFB88B1574F\",\"hash\":\"8406CADDE46381CC3D2D8F6A31AC4C3640583FC81741CD4D6DA51DF9C40DC00F\"}},\"status\":\"success\",\"type\":\"response\"}");
        let v: Value = serde_json::from_str(text.as_str()).unwrap();
        let payment = serde_json::from_value::<Transaction>(v["result"]["tx_json"].clone());
        let engine_result: String = serde_json::from_value(v["result"]["engine_result"].clone()).unwrap();
        assert_eq!(engine_result, "tesSUCCESS".to_string());
        println!("{:?}", payment);
        assert_eq!(payment.is_ok(), true);
    }
}

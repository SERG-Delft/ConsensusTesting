use websocket::{ClientBuilder, OwnedMessage, Message};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use serde_json::json;
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
    pub fn new(peer: u16, connection: &str, subscription_collector_sender: Sender<PeerSubscriptionObject>) -> Self {
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
                        debug!("Send Loop: {:?}", e);
                        return;
                    }
                };
                // Send the message
                match sender.send_message(&message) {
                    Ok(()) => {
                        debug!("Send Loop sent message: {:?}", message);
                        ()
                    },
                    Err(e) => {
                        debug!("Send Loop: {:?}", e);
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
                        debug!("Receive Loop erred: {:?}", e);
                        let _ = tx_1.send(Message::from(OwnedMessage::Close(None)));
                        return;
                    }
                };
                match message {
                    OwnedMessage::Text(text) => {
                        match serde_json::from_str::<SubscriptionObject>(text.as_str()) {
                            Ok(subscription_object) => {
                                subscription_collector_sender.send(PeerSubscriptionObject::new(peer, subscription_object)).unwrap()
                            },
                            Err(_) => { warn!("Could not parse: {}", text) }
                        }
                    },
                    _ => debug!("Receive Loop: {:?}", message)
                }
            }
        });

        // Start subscriptions
        Client::subscribe(&tx, format!("Ripple{} subscription", peer).as_str(), vec!["consensus", "ledger", "validations", "peer_status", "transactions_proposed"]);

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
    pub fn create_payment_transaction(amount: u32,
                                      destination_id: &str,
                                      sender_address: &str) -> Transaction {
        // Create payment object for payment to account
        let payment = Payment {
            amount,
            destination: String::from(destination_id),
            destination_tag: None,
            invoice_id: None,
            send_max: None,
            deliver_min: None
        };

        // Create transaction object containing the payment
        Transaction {
            account: String::from(sender_address),
            transaction_type: TransactionType::Payment,
            fee: None,
            sequence: None,
            account_txn_id: None,
            flags: None,
            last_ledger_sequence: None,
            source_tag: None,
            signing_pub_key: None,
            txn_signature: None,
            date: None,
            hash: None,
            data: Some(payment)
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
            "secret": secret
        });
        tx.send(Message::text(json.to_string())).unwrap();
    }

    pub fn subscribe(tx: &Sender<Message>, id: &str, streams: Vec<&str>) {
        let json = json!({
            "id": id,
            "command": "subscribe",
            "streams": streams
        });
        tx.send(Message::text(json.to_string())).unwrap();
    }
}

/// A transaction struct containing some, but not all, fields a ripple transaction can hold
/// Used for communication with node by serde (de)serialization
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Transaction {
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
    #[serde(rename = "Data", skip_serializing_if = "Option::is_none", flatten)]
    pub data: Option<Payment>
}

/// The different transaction types
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
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
    DepositPreauth
}

/// Fields specific to a payment transaction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Payment  {
    #[serde(rename = "Amount")]
    pub amount: u32,
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
    #[serde(skip_serializing)]
    pub fee_base: u32,
    #[serde(skip_serializing)]
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
#[derive(Serialize, Deserialize, Clone)]
pub struct ReceivedValidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    amendments: Option<Vec<String>>,
    #[serde(skip_serializing)]
    base_fee: Option<u32>,
    #[serde(skip_serializing)]
    flags: u32,
    full: bool,
    ledger_hash: String,
    ledger_index: String,
    #[serde(skip_serializing)]
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
#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Serialize, Deserialize, Clone)]
pub struct ConsensusChange {
    pub consensus: String
}

/// The different types of subscription objects
#[derive(Serialize, Deserialize, Clone)]
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
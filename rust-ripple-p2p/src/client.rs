use websocket::{ClientBuilder, OwnedMessage, Message};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::thread::JoinHandle;
use std::time::Duration;
use log::*;

const _NODE_PRIVATE_KEY: &str = "e55dc8f3741ac9668dbe858409e5d64f5ce88380f7228eccfe82b92b2c7848ba";
const _NODE_PUBLIC_KEY_BASE58: &str = "n9KAa2zVWjPHgfzsE3iZ8HAbzJtPrnoh4H2M2HgE7dfqtvyEb1KJ";
// Account and its keys to send transaction to
pub const _ACCOUNT_ID: &str = "rE4DHSdcXafD7DkpJuFCAvc3CvsgXHjmEJ";
const _MASTER_KEY: &str = "BUSY MARS SLED SNUG OBOE REID SUNK NEW GYM LAD LICE FEAT";
const _MASTER_SEED: &str = "saNSJMEBKisBr6phJtGXUcV85RBZ3";
const _MASTER_SEED_HEX: &str = "FDDE6A91607445E59C6F7CF07AF7B661";
const _PUBLIC_KEY_HEX: &str = "03137FF01C82A1CF507CC243EBF629A99F2256FA43BCB7A458F638AF9A5488CD87";
const _PUBLIC_KEY: &str = "aBQsqGF1HEduKrHrSVzNE5yeCTJTGgrsKgyjNLgabS2Rkq7CgZiq";

// Genesis account with initial supply of XRP
pub const _GENESIS_SEED: &str = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";
pub const _GENESIS_ADDRESS: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

const _AMOUNT: u32 = 2u32.pow(31);

#[allow(unused)]
pub struct Client<'a> {
    peer: u16,
    pub sender_channel: Sender<Message<'a>>,
    send_loop: JoinHandle<()>,
    receive_loop: tokio::task::JoinHandle<()>
}

impl Client<'static> {
    pub fn new(peer: u16, connection: &str, subscription_collector_sender: tokio::sync::mpsc::Sender<PeerSubscriptionObject>) -> Self {
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

        // println!("{}", connection);
        let sender_clone = tx.clone();
        let is_byzantine = connection.ends_with("8");
        let receive_loop = tokio::spawn(async move {
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
                    // Say what we received
                    OwnedMessage::Text(text) => {
                        match serde_json::from_str::<SubscriptionObject>(text.as_str()) {
                            Ok(subscription_object) => {
                                match &subscription_object {
                                    SubscriptionObject::ValidatedLedger(vl) => {
                                        if vl.ledger_index == 5 && is_byzantine {
                                            // thread::sleep(Duration::from_millis(450));
                                            // Client::sign_and_submit(
                                            //     &sender_clone,
                                            //     format!("Ripple TXN").as_str(),
                                            //     &Client::create_payment_transaction(200000000, _ACCOUNT_ID, _GENESIS_ADDRESS),
                                            //     _GENESIS_SEED
                                            // );
                                            // println!("submitted");
                                        }
                                    }
                                    _ => ()
                                }
                                subscription_collector_sender.send(PeerSubscriptionObject::new(peer, subscription_object)).await.unwrap()
                            },
                            Err(_) => { warn!("Could not parse") }
                        }
                    },
                    _ => debug!("Receive Loop: {:?}", message)
                }
            }
        });

        // Start subscription
        Client::subscribe(&tx, format!("Ripple{} subscription", peer).as_str(), vec!["consensus", "ledger", "validations", "peer_status"]);

        Client {
            peer,
            sender_channel: tx,
            send_loop,
            receive_loop
        }
    }

    #[allow(unused)]
    pub async fn start(self) {
        self.send_loop.join().unwrap();
        self.receive_loop.await.unwrap();
    }

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Transaction {
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "TransactionType")]
    pub transaction_type: TransactionType,
    #[serde(rename = "Fee", skip_serializing_if = "Option::is_none")]
    pub fee: Option<u32>,
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
    #[serde(rename = "Data", skip_serializing_if = "Option::is_none", flatten)]
    pub data: Option<Payment>
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[allow(unused)]
#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConsensusChange {
    consensus: String
}

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
    ConsensusChange(ConsensusChange)
}

#[derive(Debug)]
pub struct PeerSubscriptionObject {
    pub peer: u16,
    pub subscription_object: SubscriptionObject
}

impl PeerSubscriptionObject {
    fn new(peer: u16, subscription_object: SubscriptionObject) -> Self {
        PeerSubscriptionObject { peer, subscription_object }
    }
}
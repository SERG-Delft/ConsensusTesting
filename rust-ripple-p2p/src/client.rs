use futures::{SinkExt, StreamExt};
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::Message;

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

pub struct Client {
    #[allow(unused)]
    peer: u16,
    pub sender_channel: Sender<Message>,
    send_loop: tokio::task::JoinHandle<()>,
    receive_loop: tokio::task::JoinHandle<()>,
}

impl Client {
    pub async fn new(
        peer: u16,
        connection: String,
        subscription_collector_sender: tokio::sync::mpsc::Sender<PeerSubscriptionObject>,
    ) -> Self {
        let (client, _) = tokio_tungstenite::connect_async(&connection).await.unwrap();

        let (mut ws_tx, mut ws_rx) = client.split();

        let (tx, mut rx) = tokio::sync::mpsc::channel(16);

        let send_loop = tokio::spawn(async move {
            loop {
                // Send loop
                let message = match rx.recv().await {
                    Some(m) => m,
                    None => {
                        debug!("Send Loop: None()");
                        return;
                    }
                };
                // Send the message
                ws_tx.send(message).await.unwrap();
            }
        });

        let receive_loop = tokio::spawn(async move {
            // Receive loop
            while let Some(Ok(message)) = ws_rx.next().await {
                match serde_json::from_str::<SubscriptionObject>(message.to_text().unwrap()) {
                    Ok(subscription_object) => subscription_collector_sender
                        .send(PeerSubscriptionObject::new(peer, subscription_object))
                        .await
                        .unwrap(),
                    Err(_) => {
                        warn!("Could not parse")
                    }
                }
            }
        });

        // Start subscription
        Client::subscribe(
            &tx,
            format!("Ripple{} subscription", peer).as_str(),
            vec!["consensus", "ledger", "validations", "peer_status"],
        )
        .await;

        Client {
            peer,
            sender_channel: tx,
            send_loop,
            receive_loop,
        }
    }

    #[allow(unused)]
    pub async fn start(self) {
        self.send_loop.await.unwrap();
        self.receive_loop.await.unwrap();
    }

    #[allow(unused)]
    pub fn create_payment_transaction(
        amount: u32,
        destination_id: &str,
        sender_address: &str,
    ) -> Transaction {
        // Create payment object for payment to account
        let payment = Payment {
            amount,
            destination: String::from(destination_id),
            destination_tag: None,
            invoice_id: None,
            send_max: None,
            deliver_min: None,
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
            data: Some(payment),
        }
    }

    #[allow(unused)]
    pub async fn ping(&mut self, id: &str) {
        let json = json!({
            "id": id,
            "command": "ping"
        });
        self.sender_channel
            .send(Message::text(json.to_string()))
            .await
            .unwrap();
    }

    #[allow(unused)]
    pub async fn ledger(tx: &Sender<Message>, id: &str) {
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
        tx.send(Message::text(json.to_string())).await.unwrap();
    }

    #[allow(unused)]
    pub async fn sign_and_submit(
        tx: &Sender<Message>,
        id: &str,
        transaction: &Transaction,
        secret: &str,
    ) {
        let json = json!({
            "id": id,
            "command": "submit",
            "tx_json": transaction,
            "secret": secret
        });
        tx.send(Message::text(json.to_string())).await.unwrap();
    }

    pub async fn subscribe(tx: &Sender<Message>, id: &str, streams: Vec<&str>) {
        let json = json!({
            "id": id,
            "command": "subscribe",
            "streams": streams
        });
        tx.send(Message::text(json.to_string())).await.unwrap();
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
    pub data: Option<Payment>,
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
    DepositPreauth,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment {
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
    pub deliver_min: Option<u32>,
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
    pub validated_ledgers: Option<String>,
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
    validation_public_key: String,
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
    LostSync,
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
    consensus: String,
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
    ConsensusChange(ConsensusChange),
}

#[derive(Debug)]
pub struct PeerSubscriptionObject {
    pub peer: u16,
    pub subscription_object: SubscriptionObject,
}

impl PeerSubscriptionObject {
    fn new(peer: u16, subscription_object: SubscriptionObject) -> Self {
        PeerSubscriptionObject {
            peer,
            subscription_object,
        }
    }
}

use websocket::{ClientBuilder, OwnedMessage, Message};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::thread::JoinHandle;
use log::*;

#[allow(unused)]
pub struct Client<'a> {
    pub sender_channel: Sender<Message<'a>>,
    send_loop: JoinHandle<()>,
    receive_loop: JoinHandle<()>
}

impl Client<'static> {
    pub fn new(connection: &str) -> Self {
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
                        debug!("Receive Loop: {:?}", e);
                        let _ = tx_1.send(Message::from(OwnedMessage::Close(None)));
                        return;
                    }
                };
                match message {
                    // Say what we received
                    _ => debug!("Receive Loop: {:?}", message),
                }
            }
        });

        Client {
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

    pub fn sign_and_submit(tx: &Sender<Message>, id: &str, transaction: &Transaction, secret: &str) {
        let json = json!({
            "id": id,
            "command": "submit",
            "tx_json": transaction,
            "secret": secret
        });
        tx.send(Message::text(json.to_string())).unwrap();
    }
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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
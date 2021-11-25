use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use websocket::Message;
use crate::client::{Client, Transaction};

const _AMOUNT: u32 = 2u32.pow(31);
const _ACCOUNT_ID: &str = "rE4DHSdcXafD7DkpJuFCAvc3CvsgXHjmEJ";
const _GENESIS_ADDRESS: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
const _GENESIS_SEED: &str = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";

#[derive(Debug)]
pub struct TestHarness<'a> {
    pub transactions: Vec<TransactionTimed>,
    pub number_of_ledgers: usize,
    pub client_sender: Sender<Message<'a>>
}

impl TestHarness<'static> {
    pub fn parse_test_harness(client_sender: Sender<Message<'static>>) -> Self {
        let file = File::open("harness.txt").unwrap();
        let buf_reader = BufReader::new(file);
        let mut lines = buf_reader.lines().map(|l| l.unwrap());
        let number_of_ledgers: usize = lines.next().unwrap().parse::<usize>().expect("Number of ledgers not an int");
        let mut transactions = vec![];
        for line in lines {
            let transaction = Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS);
            let delay = Duration::from_millis(line.parse::<u64>().expect("Transaction delay needs to be of u64"));
            transactions.push(TransactionTimed { transaction, delay });
        }
        Self {
            number_of_ledgers,
            transactions,
            client_sender,
        }
    }

    pub fn schedule_transactions(self) {
        for transaction in self.transactions {
            Self::schedule_transaction(transaction, self.client_sender.clone());
        }
    }

    pub fn schedule_transaction(transaction: TransactionTimed, client_sender: Sender<Message<'static>>) {
        thread::spawn(move ||{
            thread::sleep(transaction.delay);
            Client::sign_and_submit(
                &client_sender,
                "Test harness",
                &transaction.transaction,
                _GENESIS_SEED
            );
        });
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct TransactionTimed {
    transaction: Transaction,
    delay: Duration,
}

#[cfg(test)]
mod harness_tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use crate::client::Client;
    use crate::test_harness::{_ACCOUNT_ID, _AMOUNT, _GENESIS_ADDRESS, TestHarness, TransactionTimed};

    #[test]
    fn parse_harness() {
        let number_of_ledgers = 3;
        let transaction1 = TransactionTimed {
            transaction: Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS),
            delay: Duration::from_millis(600)
        };
        let transaction2 = TransactionTimed {
            transaction: Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS),
            delay: Duration::from_millis(1350)
        };
        let transaction3 = TransactionTimed {
            transaction: Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS),
            delay: Duration::from_millis(6000)
        };
        let transactions = vec![transaction1, transaction2, transaction3];
        let (tx, rx) = channel();
        let expected_harness = TestHarness { transactions, number_of_ledgers, client_sender: tx.clone() };
        let actual_harness = TestHarness::parse_test_harness(tx.clone());
        assert_eq!(actual_harness.transactions, expected_harness.transactions);
        assert_eq!(actual_harness.number_of_ledgers, expected_harness.number_of_ledgers);
    }
}
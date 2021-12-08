use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use itertools::Itertools;
use log::debug;
use websocket::Message;
use crate::client::{Client, Transaction};
use crate::node_state::MutexNodeStates;

const _AMOUNT: u32 = 2u32.pow(31);
const _ACCOUNT_ID: &str = "rE4DHSdcXafD7DkpJuFCAvc3CvsgXHjmEJ";
const _GENESIS_ADDRESS: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
const _GENESIS_SEED: &str = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";

/// Struct containing transactions in the test harness.
/// Transactions are created based on the contents of "harness.txt".
/// Every line is one transaction where the number defines the time in ms since the start
/// of the harness to apply the transaction to the network.
#[derive(Debug)]
pub struct TestHarness<'a> {
    pub transactions: Vec<TransactionTimed>,
    pub client_senders: Vec<Sender<Message<'a>>>
}

impl TestHarness<'static> {

    // Parse the test harness file
    // execution_sequence is the sequence number of this test harness execution. Used for providing correct sequence numbers to transactions
    pub fn parse_test_harness(client_senders: Vec<Sender<Message<'static>>>, execution_sequence: usize) -> Self {
        let file = File::open("harness.txt").unwrap();
        let buf_reader = BufReader::new(file);
        let lines = buf_reader.lines().map(|l| l.unwrap()).collect_vec();
        let number_of_transactions = lines.len();
        let mut transactions = vec![];
        for (i, line) in lines.iter().enumerate() {
            let transaction_timed = Self::parse_transaction(execution_sequence * number_of_transactions + i, line);
            transactions.push(transaction_timed);
        }
        Self {
            transactions,
            client_senders,
        }
    }

    fn parse_transaction(sequence: usize, line: &String) -> TransactionTimed {
        let split = line.split_whitespace().collect_vec();
        let client_index = split[0].parse::<usize>().expect("Client index needs to of u32");
        let delay = Duration::from_millis(split[1].parse::<u64>().expect("Transaction delay needs to be of u64"));
        let transaction = Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS, Some((sequence + 1) as u32));
        // let transaction = Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS, None);
        TransactionTimed { transaction, delay, client_index }
    }

    // Schedule transactions in struct
    pub fn schedule_transactions(self, node_states: Arc<MutexNodeStates>) {
        let number_of_transactions = self.transactions.len();
        for transaction in self.transactions {
            let client_index = transaction.client_index.clone();
            Self::schedule_transaction(transaction, self.client_senders[client_index].clone());
        }
        // Wait for all transactions to have been validated
        while node_states.get_min_validated_transactions() < number_of_transactions {
            node_states.transactions_cvar.wait(&mut node_states.node_states.lock());
            debug!("{} out of {} transactions validated, max: {}", node_states.get_min_validated_transactions(), number_of_transactions, node_states.get_max_validated_transaction());
        }
        println!("Test harness over");
    }

    // Schedule a transaction according to its delay
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

/// A transaction coupled with its delay and client
#[derive(Eq, PartialEq, Debug)]
pub struct TransactionTimed {
    transaction: Transaction,
    delay: Duration,
    client_index: usize,
}

#[cfg(test)]
mod harness_tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use crate::client::Client;
    use crate::test_harness::{_ACCOUNT_ID, _AMOUNT, _GENESIS_ADDRESS, TestHarness, TransactionTimed};

    // This test fails without a "harness.txt" containing 600, 1350, 6000 on separate lines
    #[test]
    fn parse_harness() {
        let transaction1 = TransactionTimed {
            transaction: Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS, Some(1)),
            delay: Duration::from_millis(600),
            client_index: 0,
        };
        let transaction2 = TransactionTimed {
            transaction: Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS, Some(2)),
            delay: Duration::from_millis(1350),
            client_index: 0,
        };
        let transaction3 = TransactionTimed {
            transaction: Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS, Some(3)),
            delay: Duration::from_millis(6000),
            client_index: 0,
        };
        let transactions = vec![transaction1, transaction2, transaction3];
        let (tx, _) = channel();
        let expected_harness = TestHarness { transactions, client_senders: vec![tx.clone()] };
        let actual_harness = TestHarness::parse_test_harness(vec![tx.clone()], 0);
        assert_eq!(actual_harness.transactions, expected_harness.transactions);
    }
}
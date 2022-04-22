use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::{FromStr};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use itertools::{Itertools};
use log::{debug, error, warn};
use websocket::Message;
use crate::client::{Client, Transaction};
use crate::container_manager::AccountKeys;
use crate::node_state::MutexNodeStates;

const _AMOUNT: u32 = 2u32.pow(31);
const _ACCOUNT_ADDRESS: &str = "rE4DHSdcXafD7DkpJuFCAvc3CvsgXHjmEJ";
const _ACCOUNT_SEED: &str = "saNSJMEBKisBr6phJtGXUcV85RBZ3";
const _GENESIS_ADDRESS: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
const _GENESIS_SEED: &str = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";

/// Struct containing transactions in the test harness.
/// Transactions are created based on the contents of "harness.txt".
/// Every line is one transaction where the number defines the time in ms since the start
/// of the harness to apply the transaction to the network.
#[derive(Debug)]
pub struct TestHarness<'a> {
    pub transactions: Vec<TransactionTimed>,
    pub accounts: Vec<Account>,
    pub client_senders: Vec<Sender<Message<'a>>>,
    pub client_receiver: Receiver<(Transaction, String)>,
    pub succeeded_transactions: HashSet<Transaction>,
    pub transaction_results: Vec<TransactionResult>
}

impl TestHarness<'static> {
    // Parse the test harness file
    // execution_sequence is the sequence number of this test harness execution. Used for providing correct sequence numbers to transactions
    pub fn parse_test_harness(client_senders: Vec<Sender<Message<'static>>>, client_receiver: Receiver<(Transaction, String)>, file_name: Option<&str>) -> Self {
        let file = match file_name {
            None => File::open("harness.txt").unwrap(),
            Some(file_name) => File::open(file_name).unwrap(),
        };
        let buf_reader = BufReader::new(file);
        let lines = buf_reader.lines().map(|l| l.unwrap()).collect_vec();
        let (number_of_transactions, number_of_accounts, transaction_results): (usize, usize, Vec<TransactionResult>) = Self::parse_first_line(&lines[1]);
        let genesis_account = Account::new(
            AccountKeys { account_id: String::from("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"), master_seed: String::from("snoPBrXtMeMyMHUVTgbuqAfg1SUTb") },
            1
        );
        let mut accounts = vec![genesis_account];
        for _ in 0..number_of_accounts {
            let account = crate::container_manager::create_account();
            accounts.push(Account::new(account, 0));
        }
        let mut transactions = vec![];
        for i in 0..number_of_transactions {
            let transaction_timed = Self::parse_transaction(&lines[i+2], &mut accounts);
            transactions.push(transaction_timed);
        }
        Self {
            transactions,
            accounts,
            client_senders,
            client_receiver,
            succeeded_transactions: HashSet::new(),
            transaction_results
        }
    }

    fn parse_first_line(line: &str) -> (usize, usize, Vec<TransactionResult>) {
        let split = line.split(";").collect_vec();
        if let &[number_of_transaction, number_of_accounts, transaction_results] = &*split {
            let cleaned_transaction_results_string = transaction_results.replace(&['[', ']'], "");
            let indiv_tx_results = cleaned_transaction_results_string.split(",").collect_vec();
            let transaction_results_parsed = Self::parse_tx_results(indiv_tx_results);
            let number_of_transactions = usize::from_str(number_of_transaction).expect("Number of transactions is not usize");
            if transaction_results_parsed.iter().map(|res| res.transaction_indices.len()).sum::<usize>() != number_of_transactions {
                panic!("All transactions should be present in transaction result conditions");
            }
            (number_of_transactions, usize::from_str(number_of_accounts).unwrap(), transaction_results_parsed)
        } else {
            panic!("First line of harness.txt should contain 3 items ; separated")
        }
    }

    fn parse_tx_results(transactions: Vec<&str>) -> Vec<TransactionResult> {
        let mut results = vec![];
        for transaction in transactions {
            results.push(TransactionResult::new(transaction.split("|").map(|tx| usize::from_str(tx).unwrap()).collect_vec()))
        }
        results
    }

    fn parse_transaction(line: &str, accounts: &Vec<Account>) -> TransactionTimed {
        let split = line.split_whitespace().collect_vec();
        let client_index = split[0].parse::<usize>().expect("Client index needs to of u32");
        let delay = Duration::from_millis(split[1].parse::<u64>().expect("Transaction delay needs to be of u64"));
        let amount = split[2].parse::<u32>().expect("Amount needs to of u32");
        let from = split[3].parse::<usize>().expect("From account index needs to be of usize");
        let to = split[4].parse::<usize>().expect("To account index needs to be of usize");
        let transaction = Client::create_payment_transaction(amount, &accounts[to].account_keys.account_id, &accounts[from].account_keys.account_id, None);
        TransactionTimed { transaction, from, delay, client_index }
    }

    // Schedule transactions
    pub fn schedule_transactions(&mut self, node_states: Arc<MutexNodeStates>) {
        node_states.clear_transactions();
        let number_of_transactions = self.transactions.len();
        for transaction in &self.transactions {
            let client_index = transaction.client_index.clone();
            let sequence = match self.accounts[transaction.from].transaction_sequence {
                0 => None,
                _ => Some(self.accounts[transaction.from].sequence())
            };
            Self::schedule_transaction(transaction.transaction.clone(), sequence, self.accounts[transaction.from].account_keys.master_seed.clone(), transaction.delay, self.client_senders[client_index].clone());
        }
        for _ in 0..number_of_transactions {
            match self.client_receiver.recv() {
                Ok((transaction, status)) => self.handle_transaction_submission(transaction, &status),
                Err(err) => error!("Client sender hung up: {}", err)
            }
        }
        // Wait for all transactions to have been validated
        let mut min_validated_transactions = node_states.get_min_validated_transactions_idx(&self.transactions);
        while TransactionResult::check_transaction_results(&self.transaction_results, &min_validated_transactions) == false {
            // debug!("{} out of {} transactions validated, unvalidated idxs: {:?}", node_states.get_number_min_validated_transactions(), self.succeeded_transactions.len(),
            //     self.succeeded_transactions.difference(&min_validated_transactions)
            //         .map(|tx| Self::calc_tx_idx(&self.transactions, tx).unwrap())
            //         .collect::<Vec<usize>>()
            //     );
            node_states.transactions_cvar.wait(&mut node_states.node_states.lock());
            min_validated_transactions = node_states.get_min_validated_transactions_idx(&self.transactions);
        }
        self.succeeded_transactions.clear();
        println!("Test harness over");
    }

    // Schedule a transaction according to its delay
    pub fn schedule_transaction(mut transaction: Transaction, sequence: Option<u32>, secret: String, delay: Duration, client_sender: Sender<Message<'static>>) {
        transaction.sequence = sequence;
        thread::spawn(move ||{
            thread::sleep(delay);
            Client::sign_and_submit(
                &client_sender,
                "Test harness",
                &transaction,
                &secret
            );
        });
    }

    fn handle_transaction_submission(&mut self, transaction: Transaction, status: &str) {
        match status {
            "tesSUCCESS" => {
                debug!("Transaction successful: {}, transaction: {:?}", status, transaction);
                self.succeeded_transactions.insert(transaction);
            },
            "terPRE_SEQ" => {
                debug!("Transaction retried high sequence: {}, {}, transaction: {:?}", status, transaction.sequence.unwrap(), transaction);
                self.succeeded_transactions.insert(transaction);
            },
            "tefPAST_SEQ" => {
                debug!("Transaction failed low sequence: {}, {}, transaction: {:?}", status, transaction.sequence.unwrap(), transaction);
            },
            "error" => {
                error!("Transaction error, resubmitting...");
                let timed_tx = self.transactions.iter().find(|tx| tx.transaction == transaction).unwrap();
                Self::schedule_transaction(transaction.clone(), transaction.sequence, self.accounts[timed_tx.from].account_keys.master_seed.clone(), Duration::from_millis(200), self.client_senders[timed_tx.client_index].clone());
            }
            _ => warn!("Transaction had a result other than tesSUCCESS: {}, transaction: {:?}", status, transaction)
        }
    }

    pub fn create_accounts(&mut self) {
        for i in 1..self.accounts.len() {
            Self::schedule_transaction(
                Client::create_payment_transaction(
                    20,
                    &self.accounts[i].account_keys.account_id,
                    &self.accounts[0].account_keys.account_id,
                    None,
                ),
                Some(self.accounts[0].sequence()),
                self.accounts[0].account_keys.master_seed.clone(),
                Duration::ZERO,
                self.client_senders[0].clone(),
            )
        }
    }

    pub fn calc_tx_idx(timed_transactions: &Vec<TransactionTimed>, tx: &Transaction) -> Option<usize> {
        timed_transactions.iter().position(|x| x.transaction.account == tx.account && x.transaction.data.as_ref().unwrap().destination == tx.data.as_ref().unwrap().destination)
    }
}

/// A transaction coupled with its delay and client
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct TransactionTimed {
    pub transaction: Transaction,
    delay: Duration,
    client_index: usize,
    from: usize,
}

#[derive(Debug, Clone)]
pub struct Account {
    account_keys: AccountKeys,
    transaction_sequence: u32,
}

impl Account {
    pub fn new(account_keys: AccountKeys, transaction_sequence: u32) -> Self {
        Self {
            account_keys, transaction_sequence
        }
    }

    pub fn sequence(&mut self) -> u32 {
        let seq = self.transaction_sequence;
        self.transaction_sequence += 1;
        seq
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct TransactionResult {
    pub transaction_indices: Vec<usize>,
}

impl TransactionResult {
    pub fn new(transaction_indices: Vec<usize>) -> Self {
        Self { transaction_indices }
    }

    pub fn result_is_met(&self, validated_transactions: &Vec<usize>) -> bool {
        let matched_transactions = self.transaction_indices.iter().filter(|tx_idx| validated_transactions.contains(tx_idx)).collect_vec();
        match matched_transactions.len() {
            1 => {
                debug!("Transaction: {} has succeeded, {:?} have failed", matched_transactions[0], self.transaction_indices.iter().filter(|tx| tx != &matched_transactions[0]).collect_vec());
                true
            }
            0 => {
                false
            }
            _ => {
                error!("Test failed!, multiple transactions in {:?} have been validated, DOUBLE SPEND", matched_transactions);
                false
            }
        }
    }

    pub fn check_transaction_results(expected_results: &Vec<TransactionResult>, validated_transactions: &Vec<usize>) -> bool {
        expected_results.iter().map(|result| result.result_is_met(&validated_transactions)).all(|x| x)
    }
}

#[cfg(test)]
mod harness_tests {
    use std::collections::HashSet;
    use std::sync::Arc;
    #[allow(unused_imports)]
    use std::sync::mpsc::channel;
    use std::sync::mpsc::{Receiver};
    use std::thread;
    use std::time::Duration;
    use serde_json::json;
    use websocket::Message;
    use crate::client::{Client};
    use crate::container_manager::AccountKeys;
    use crate::node_state::{MutexNodeStates, NodeState, NodeStates};
    use crate::test_harness::{Account, TestHarness, TransactionResult, TransactionTimed};

    fn parse_harness() -> (TestHarness<'static>, TestHarness<'static>, Vec<Receiver<Message<'static>>>) {
        crate::container_manager::start_key_generator();
        let (tx_1, rx_1) = channel();
        let (tx_2, rx_2) = channel();
        let (_client_tx, client_rx) = channel();
        let (_expected_client_tx, expected_client_rx) = channel();
        let actual_harness = TestHarness::parse_test_harness(vec![tx_1.clone(), tx_2.clone()], client_rx, Some("harness_test.txt"));
        let accounts = actual_harness.accounts.clone();
        let transaction1 = TransactionTimed {
            transaction: Client::create_payment_transaction(80, &accounts[1].account_keys.account_id, &accounts[0].account_keys.account_id, None),
            delay: Duration::from_millis(0),
            client_index: 0,
            from: 0
        };
        let transaction2 = TransactionTimed {
            transaction: Client::create_payment_transaction(80, &accounts[2].account_keys.account_id, &accounts[1].account_keys.account_id, None),
            delay: Duration::from_millis(1000),
            client_index: 0,
            from: 1
        };
        let transaction3 = TransactionTimed {
            transaction: Client::create_payment_transaction(80, &accounts[3].account_keys.account_id, &accounts[1].account_keys.account_id, None),
            delay: Duration::from_millis(1000),
            client_index: 1,
            from: 1
        };
        let transactions = vec![transaction1, transaction2, transaction3];
        let expected_transaction_results = vec![TransactionResult::new(vec![0]), TransactionResult::new(vec![1,2])];
        let expected_harness = TestHarness { transactions, accounts, client_senders: vec![tx_1.clone(), tx_2.clone()], client_receiver: expected_client_rx, succeeded_transactions: HashSet::new(), transaction_results: expected_transaction_results };
        (actual_harness, expected_harness, vec![rx_1, rx_2])
    }

    #[test]
    fn test_transaction_scheduler() {
        let (mut actual_harness, mut expected_harness, receivers) = parse_harness();
        let node_states = Arc::new(MutexNodeStates::new(NodeStates::new(vec![NodeState::new(0), NodeState::new(1)])));
        assert_eq!(actual_harness.transactions, expected_harness.transactions);
        thread::spawn(move || actual_harness.schedule_transactions(node_states.clone()));
        thread::sleep(Duration::from_millis(200));
        let transaction_1 =  receivers[0].try_recv().unwrap();
        assert_eq!(transaction_1, sign_and_submit_message(&mut expected_harness, 0, true));
        thread::sleep(Duration::from_millis(2000));
        let transaction_2 =  receivers[0].try_recv().unwrap();
        let transaction_3 =  receivers[1].try_recv().unwrap();
        assert_eq!(transaction_2, sign_and_submit_message(&mut expected_harness, 1, false));
        assert_eq!(transaction_3, sign_and_submit_message(&mut expected_harness, 2, false));
    }

    #[test]
    fn test_sequence() {
        let mut account = Account::new(AccountKeys { account_id: "id".to_string(), master_seed: "secret".to_string()}, 0);
        assert_eq!(0, account.sequence());
        assert_eq!(1, account.sequence());
        assert_eq!(2, account.transaction_sequence);
        assert_eq!(2, account.sequence());
    }

    fn sign_and_submit_message(harness: &mut TestHarness, transaction_index: usize, fill_sequence: bool) -> Message<'static> {
        let timed_transaction = harness.transactions[transaction_index].clone();
        let mut transaction = timed_transaction.transaction.clone();
        transaction.sequence = if fill_sequence { Some(harness.accounts[timed_transaction.from].sequence()) } else { None };
        let secret = harness.accounts[timed_transaction.from].account_keys.master_seed.clone();
        let json = json!({
            "id": "Test harness",
            "command": "submit",
            "tx_json": transaction,
            "secret": &secret,
            "fee_mult_max": 10000000,
        });
        Message::text(json.to_string())
    }

    #[test]
    fn test_parse_first_line() {
        let (number_of_transaction, number_of_accounts, transaction_results) = TestHarness::parse_first_line("3;3;[0,1|2]");
        assert_eq!(number_of_transaction, 3);
        assert_eq!(number_of_accounts, 3);
        let expected_transaction_results = vec![TransactionResult::new(vec![0]), TransactionResult::new(vec![1,2])];
        assert_eq!(transaction_results, expected_transaction_results);
    }

    #[test]
    fn test_check_transaction_results() {
        let (_number_of_transaction, _number_of_accounts, transaction_results) = TestHarness::parse_first_line("3;3;[0,1|2]");
        let expected_false_multiple = TransactionResult::check_transaction_results(&transaction_results, &vec![0, 1, 2]);
        let expected_false_none = TransactionResult::check_transaction_results(&transaction_results, &vec![0]);
        let expected_false_one_none = TransactionResult::check_transaction_results(&transaction_results, &vec![]);
        let expected_true_1 = TransactionResult::check_transaction_results(&transaction_results, &vec![0, 1]);
        let expected_true_2 = TransactionResult::check_transaction_results(&transaction_results, &vec![0, 2]);
        assert_eq!(expected_false_multiple, false);
        assert_eq!(expected_false_none, false);
        assert_eq!(expected_false_one_none, false);
        assert_eq!(expected_true_1, true);
        assert_eq!(expected_true_2, true);
    }
}
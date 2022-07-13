use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::{FromStr};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use itertools::{Itertools};
use log::{debug, error, trace, warn};
use spin_sleep::SpinSleeper;
use websocket::Message;
use crate::client::{Client, Transaction};
use crate::container_manager::AccountKeys;
use crate::node_state::MutexNodeStates;
use crate::consensus_properties::ConsensusProperties;
use crate::failure_writer::ConsensusPropertyTypes;
use crate::test_harness::TestResult::{Failed, InProgress, Success};

const MAX_EVENTS_TEST: usize = 10000;

/// Struct containing transactions in the test harness.
/// Transactions are created based on the contents of "harness.txt".
/// Every line is one transaction where the number defines the time in ms since the start
/// of the harness to apply the transaction to the network.
#[derive(Debug)]
pub struct TestHarness<'a> {
    pub transactions: Vec<TransactionTimed>,
    pub accounts: Vec<Account>,
    pub starting_balances: Vec<(usize, u32)>,
    pub client_senders: Vec<Sender<Message<'a>>>,
    pub client_receiver: Receiver<(Transaction, String)>,
    pub balance_receiver: Receiver<u32>,
    pub succeeded_transactions: HashSet<Transaction>,
    pub unfunded_transactions: HashSet<Transaction>,
    pub transaction_results: Vec<TransactionResult>,
    pub failure_sender: Sender<Vec<ConsensusPropertyTypes>>,
}

impl TestHarness<'static> {
    // Parse the test harness file
    // execution_sequence is the sequence number of this test harness execution. Used for providing correct sequence numbers to transactions
    pub fn parse_test_harness(
        client_senders: Vec<Sender<Message<'static>>>,
        client_receiver: Receiver<(Transaction, String)>,
        balance_receiver: Receiver<u32>,
        failure_sender: Sender<Vec<ConsensusPropertyTypes>>,
        file_name: Option<&str>
    ) -> Self {
        let file = match file_name {
            None => File::open("harness.txt").unwrap(),
            Some(file_name) => File::open(file_name).unwrap(),
        };
        let buf_reader = BufReader::new(file);
        let lines = buf_reader.lines().map(|l| l.unwrap()).collect_vec();
        let (number_of_transactions, number_of_accounts, number_of_starting_balances, transaction_results): (usize, usize, usize, Vec<TransactionResult>) = Self::parse_first_line(&lines[1]);
        let genesis_account = Account::new(
            AccountKeys { account_id: String::from("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"), master_seed: String::from("snoPBrXtMeMyMHUVTgbuqAfg1SUTb") },
            1
        );
        let mut accounts = vec![genesis_account];
        for _ in 0..number_of_accounts {
            let account = crate::container_manager::create_account();
            accounts.push(Account::new(account, 0));
        }
        let mut starting_balances = vec![];
        for i in 0..number_of_starting_balances {
            starting_balances.push(Self::parse_starting_balance(&lines[i+2]))
        }
        let mut transactions = vec![];
        for i in 0..number_of_transactions {
            let subsequent_seq = transaction_results.iter().find(|tx| tx.transaction_indices.contains(&i)).unwrap().subsequent_seq;
            let transaction_timed = Self::parse_transaction(&lines[i+number_of_starting_balances+2], &mut accounts, subsequent_seq, number_of_transactions, i);
            transactions.push(transaction_timed);
        }
        Self {
            transactions,
            accounts,
            starting_balances,
            client_senders,
            client_receiver,
            balance_receiver,
            succeeded_transactions: HashSet::new(),
            unfunded_transactions: HashSet::new(),
            transaction_results,
            failure_sender
        }
    }

    fn parse_first_line(line: &str) -> (usize, usize, usize, Vec<TransactionResult>) {
        let split = line.split(";").collect_vec();
        if let &[number_of_transaction, number_of_accounts, number_of_starting_balances, transaction_results] = &*split {
            let cleaned_transaction_results_string = transaction_results.replace(&['[', ']'], "");
            let indiv_tx_results = cleaned_transaction_results_string.split(",").collect_vec();
            let transaction_results_parsed = Self::parse_tx_results(indiv_tx_results);
            let number_of_transactions = usize::from_str(number_of_transaction).expect("Number of transactions is not usize");
            let number_of_accounts = usize::from_str(number_of_accounts).expect("Number of accounts is not usize");
            let number_of_starting_balances = usize::from_str(number_of_starting_balances).expect("Number of starting balances is not usize");
            if transaction_results_parsed.iter().map(|res| res.transaction_indices.len()).sum::<usize>() != number_of_transactions {
                panic!("All transactions should be present in transaction result conditions");
            }
            (number_of_transactions, number_of_accounts, number_of_starting_balances, transaction_results_parsed)
        } else {
            panic!("First line of harness.txt should contain 3 items ; separated")
        }
    }

    fn parse_starting_balance(line: &str) -> (usize, u32) {
        let split = line.split_whitespace().collect_vec();
        let account_idx = split[0].parse::<usize>().expect("From account index needs to be of usize");
        let balance = split[1].parse::<u32>().expect("Balance needs to of u32");
        (account_idx, balance)
    }

    fn parse_tx_results(transactions: Vec<&str>) -> Vec<TransactionResult> {
        let mut results = vec![];
        for transaction in transactions {
            let (transaction, subsequent_seq) = transaction.split_at(transaction.len()-1);
            results.push(TransactionResult::new(transaction.split("|").map(|tx| usize::from_str(tx).unwrap()).collect_vec(), subsequent_seq == "y"))
        }
        results
    }

    fn parse_transaction(line: &str, accounts: &Vec<Account>, subsequent_seq: bool, number_of_transactions: usize, source_tag: usize) -> TransactionTimed {
        let split = line.split_whitespace().collect_vec();
        let client_index = split[0].parse::<usize>().expect("Client index needs to of u32");
        let delay = Duration::from_millis(split[1].parse::<u64>().expect("Transaction delay needs to be of u64"));
        let amount = split[2].parse::<u32>().expect("Amount needs to of u32");
        let from = split[3].parse::<usize>().expect("From account index needs to be of usize");
        let to = split[4].parse::<usize>().expect("To account index needs to be of usize");
        let include_fee: usize = (from == 0) as usize * number_of_transactions;
        let transaction = Client::create_payment_transaction(amount, &accounts[to].account_keys.account_id, &accounts[from].account_keys.account_id, None, include_fee, source_tag);
        TransactionTimed { transaction, from, delay, client_index, subsequent_seq }
    }

    pub(crate) fn setup_balances(&mut self, node_states: &Arc<MutexNodeStates>) {
        debug!("Setting up balances");
        node_states.clear_transactions();
        for i in 0..self.starting_balances.len() {
            let account_idx = self.starting_balances[i].0;
            Client::account_info("setup_balance", &self.client_senders[0], self.accounts[account_idx].account_keys.account_id.clone());
        }
        // We assume the client responds in order...
        for i in 0..self.starting_balances.len() {
            let account_idx = self.starting_balances[i].0;
            let current_balance = match self.balance_receiver.recv() {
                Ok(balance) => balance / 10u32.pow(7),
                Err(_) => panic!("dddd"),
            };
            let difference: i64 = (self.starting_balances[i].1 - current_balance + 20) as i64;
            if difference >= 0 {
                let transaction = Client::create_payment_transaction(
                    difference as u32,
                    &self.accounts[account_idx].account_keys.account_id,
                    &self.accounts[0].account_keys.account_id,
                    None,
                    self.transactions.len(),
                    usize::MAX,
                );
                Client::sign_and_submit(&self.client_senders[0], "setup_balance_result", &transaction, &self.accounts[0].account_keys.master_seed);
            } else {
                let sequence = self.accounts[account_idx].sequence();
                let transaction = Client::create_payment_transaction(
                    -difference as u32,
                    &self.accounts[0].account_keys.account_id,
                    &self.accounts[account_idx].account_keys.account_id,
                    Some(sequence),
                    0,
                    usize::MAX,
                );
                Client::sign_and_submit(&self.client_senders[0], "setup_balance_result", &transaction, &self.accounts[account_idx].account_keys.master_seed);
            }
        }
        debug!("Waiting for balances to be set up");
        while node_states.get_number_min_validated_transactions() < self.starting_balances.len() {
            node_states.transactions_cvar.wait(&mut node_states.node_states.lock());
        }
        debug!("Done setting up balances")
    }

    /// Schedule transactions and wait for them to be validated correctly
    /// If the transactions are not validated correctly after MAX_EVENTS_TEST events have executed,
    /// Return false, so the fitness function knows this
    /// Else return true
    pub fn schedule_transactions(&mut self, node_states: Arc<MutexNodeStates>) -> bool {
        node_states.clear_transactions();
        node_states.clear_executions();
        node_states.clear_consensus_property_data();
        let number_of_transactions = self.transactions.len();
        let mut cloned_transactions: Vec<Transaction> = self.transactions.iter().map(|tx| tx.transaction.clone()).collect();
        let mut accounts_to_increment_seq = HashSet::new();
        for transaction in &self.transactions {
            let client_index = transaction.client_index.clone();
            let sequence = match self.accounts[transaction.from].transaction_sequence {
                0 => {
                    warn!("Transaction seq not set for accounts");
                    None
                },
                _ => if transaction.subsequent_seq {
                    Some(self.accounts[transaction.from].sequence())
                } else {
                    accounts_to_increment_seq.insert(transaction.from);
                    Some(self.accounts[transaction.from].transaction_sequence)
                }
            };
            Self::schedule_transaction(cloned_transactions.remove(0), sequence, self.accounts[transaction.from].account_keys.master_seed.clone(), transaction.delay, self.client_senders[client_index].clone());
        }
        for i in accounts_to_increment_seq {
            self.accounts[i].transaction_sequence += 1;
        }
        for _ in 0..number_of_transactions {
            match self.client_receiver.recv() {
                Ok((transaction, status)) => self.handle_transaction_submission(transaction, &status),
                Err(err) => error!("Client sender hung up: {}", err)
            }
        }
        // Wait for all transactions to have been validated or max events to have been executed
        let mut min_validated_transactions = node_states.get_min_validated_transactions_idx();
        let unfunded_payment_idxs = self.unfunded_transactions.iter()
            .filter_map(|tx| tx.source_tag).map(|tag| tag as usize).collect::<Vec<usize>>();
        let mut test_result = TransactionResult::check_transaction_results(&self.transaction_results, &min_validated_transactions, &unfunded_payment_idxs);
        while test_result == InProgress &&
            node_states.get_consensus_event_count() < MAX_EVENTS_TEST
        {
            node_states.transactions_cvar.wait_for(&mut node_states.node_states.lock(), Duration::from_millis(1000));
            min_validated_transactions = node_states.get_min_validated_transactions_idx();
            test_result = TransactionResult::check_transaction_results(&self.transaction_results, &min_validated_transactions, &unfunded_payment_idxs);
        }
        debug!("events during test: {}", node_states.get_consensus_event_count());
        let mut consensus_properties_violated = ConsensusProperties::check_validity_properties(&node_states);
        consensus_properties_violated.append(&mut ConsensusProperties::check_agreement_properties(&node_states));
        if test_result == Failed {
            consensus_properties_violated.push(ConsensusPropertyTypes::DoubleSpend);
        }
        if !consensus_properties_violated.is_empty() {
            match self.failure_sender.send(consensus_properties_violated) {
                Ok(_) => {}
                Err(err) => error!("Failure channel failed: {}", err)
            };
        }
        self.succeeded_transactions.clear();
        self.unfunded_transactions.clear();
        println!("Test harness over");
        if node_states.get_consensus_event_count() >= MAX_EVENTS_TEST {
            warn!("Event cap exceeded in test");
            false
        } else {
            true
        }
    }

    // Schedule a transaction according to its delay
    pub fn schedule_transaction(mut transaction: Transaction, sequence: Option<u32>, secret: String, delay: Duration, client_sender: Sender<Message<'static>>) {
        transaction.sequence = sequence;
        thread::spawn(move ||{
            let sleeper = SpinSleeper::default();
            sleeper.sleep(delay);
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
            "tecUNFUNDED_PAYMENT" => {
                debug!("Unfunded payment returned: {}, transaction: {:?}", status, transaction);
                self.unfunded_transactions.insert(transaction);
            },
            "tefINVARIANT_FAILED" => {
                error!("Invariant failed, excluded from ledger");
            },
            "tecINVARIANT_FAILED" => {
                error!("Invariant failed, included in ledger");
            }
            "error" => {
                error!("Transaction error, resubmitting...");
                let timed_tx = self.transactions.iter().find(|tx| tx.transaction.source_tag == transaction.source_tag).unwrap();
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
                    0,
                    usize::MAX
                ),
                Some(self.accounts[0].sequence()),
                self.accounts[0].account_keys.master_seed.clone(),
                Duration::ZERO,
                self.client_senders[0].clone(),
            )
        }
    }
}

/// A transaction coupled with its delay and client
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct TransactionTimed {
    pub transaction: Transaction,
    delay: Duration,
    client_index: usize,
    from: usize,
    subsequent_seq: bool,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub account_keys: AccountKeys,
    pub transaction_sequence: u32,
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

/// Transaction indices grouped by double spend effort. 'Exactly one' should be successfully validated
/// subsequent_seq determines whether subsequent seq numbers should be given to the transactions.
/// true: all validated tx, all tx_cost, 1 tes_success, rest tec_unfunded
/// false: 1 validated tx, 1 tx_cost, 1 tes_success, rest tef_past_seq
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct TransactionResult {
    pub transaction_indices: Vec<usize>,
    pub subsequent_seq: bool,
}

impl TransactionResult {
    pub fn new(transaction_indices: Vec<usize>, subsequent_seq: bool) -> Self {
        Self { transaction_indices, subsequent_seq }
    }

    pub fn result_is_met(&self, validated_transactions: &Vec<(usize, TransactionResultCode)>, unfunded_payment_idxs: &Vec<usize>) -> TestResult {
        let matched_transactions = validated_transactions.iter()
            .filter(|tx| self.transaction_indices.contains(&tx.0)).collect_vec();
        match self.subsequent_seq {
            true => {
                let grouped_transactions = matched_transactions.iter().counts_by(|tx| &tx.1);
                if grouped_transactions.get(&TransactionResultCode::TesSuccess).unwrap_or(&0) > &1 {
                    error!("Test failed!, multiple transactions in {:?} have been validated as successful, DOUBLE SPEND", matched_transactions);
                    return Failed;
                } else if grouped_transactions.get(&TransactionResultCode::TesSuccess).unwrap_or(&0) == &1 {
                    if grouped_transactions.get(&TransactionResultCode::TecUnfundedPayment).unwrap_or(&0) == &(self.transaction_indices.len() - 1) {
                        trace!("Test passed! All transactions properly validated");
                        return Success
                    }
                }
                InProgress
            }
            false => {
                let grouped_transactions = matched_transactions.iter().counts_by(|tx| &tx.1);
                if grouped_transactions.get(&TransactionResultCode::TesSuccess).unwrap_or(&0) > &1 {
                    error!("Test failed!, multiple transactions in {:?} have been validated, DOUBLE SPEND", matched_transactions);
                    Failed
                }
                else if grouped_transactions.get(&TransactionResultCode::TecUnfundedPayment).unwrap_or(&0) != &unfunded_payment_idxs.len() {
                    InProgress
                }
                else if grouped_transactions.get(&TransactionResultCode::TesSuccess).unwrap_or(&0) == &1 {
                    trace!("Transaction: {:?} has succeeded, {:?} have failed", matched_transactions[0], self.transaction_indices.iter().filter(|tx| *tx != &matched_transactions[0].0).collect_vec());
                    Success
                } else {
                    InProgress
                }
            }
        }
    }

    pub fn check_transaction_results(
        expected_results: &Vec<TransactionResult>,
        validated_transactions: &Vec<(usize, TransactionResultCode)>,
        unfunded_payment_idxs: &Vec<usize>
    ) -> TestResult {
        if validated_transactions.iter().any(|(_tx, code)| code == &TransactionResultCode::TecInvariantFailed) {
            return Failed;
        }
        let actual_results = expected_results.iter().map(|result| result.result_is_met(&validated_transactions, &unfunded_payment_idxs)).collect_vec();
        let mut result = InProgress;
        if actual_results.iter().all(|x| *x == Success) {
            result = Success;
        } else if actual_results.iter().any(|x| *x == Failed) {
            result = Failed;
        }
        if result == Success {
            debug!("Test passed, transactions properly validated");
        }
        result
    }
}

#[derive(PartialEq, Debug, Copy, Clone, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TransactionResultCode {
    TesSuccess,
    TecUnfundedPayment,
    TefPastSeq,
    TecInvariantFailed,
    Other,
}

impl TransactionResultCode {
    pub fn parse(code: &str) -> Self {
        match code {
            "tesSUCCESS" => Self::TesSuccess,
            "tecUNFUNDED_PAYMENT" => Self::TecUnfundedPayment,
            "tefPAST_SEQ" => Self::TefPastSeq,
            "tecINVARIANT_FAILED" => Self::TecInvariantFailed,
            _ => {
                error!("Got other result code!");
                Self::Other
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestResult {
    Success,
    InProgress,
    Failed,
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
    use crate::client::{Client, Transaction};
    use crate::collector::RippleMessage;
    use crate::container_manager::AccountKeys;
    use crate::ga::encoding::delay_encoding::{DelayMapPhenotype};
    use crate::ga::encoding::{ExtendedPhenotype, num_genes};
    use crate::LOG_FOLDER;
    use crate::node_state::{MutexNodeStates, NodeState, NodeStates};
    use crate::test_harness::{Account, TestHarness, TransactionResult, TransactionResultCode, TransactionTimed};
    use crate::test_harness::TestResult::{Failed, InProgress, Success};

    fn parse_harness() -> (TestHarness<'static>, TestHarness<'static>, Vec<Receiver<Message<'static>>>) {
        crate::container_manager::start_key_generator();
        let (tx_1, rx_1) = channel();
        let (tx_2, rx_2) = channel();
        let (_client_tx, client_rx) = channel();
        let (_balance_tx, balance_rx) = channel();
        let (_expected_client_tx, expected_client_rx) = channel();
        let (_expected_balance_tx, expected_balance_rx) = channel();
        let (failure_tx, _failure_rx) = channel();
        let mut actual_harness = TestHarness::parse_test_harness(vec![tx_1.clone(), tx_2.clone()], client_rx, balance_rx, failure_tx.clone(), Some("harness_test.txt"));
        for i in 1..actual_harness.accounts.len() {
            actual_harness.accounts[i].transaction_sequence = 1;
        }
        let accounts = actual_harness.accounts.clone();
        let transaction1 = TransactionTimed {
            transaction: Client::create_payment_transaction(80, &accounts[1].account_keys.account_id, &accounts[0].account_keys.account_id, None, 3, 0),
            delay: Duration::from_millis(0),
            client_index: 0,
            from: 0,
            subsequent_seq: true
        };
        let transaction2 = TransactionTimed {
            transaction: Client::create_payment_transaction(80, &accounts[2].account_keys.account_id, &accounts[1].account_keys.account_id, None, 0, 1),
            delay: Duration::from_millis(1000),
            client_index: 0,
            from: 1,
            subsequent_seq: true,
        };
        let transaction3 = TransactionTimed {
            transaction: Client::create_payment_transaction(80, &accounts[3].account_keys.account_id, &accounts[1].account_keys.account_id, None, 0, 2),
            delay: Duration::from_millis(1000),
            client_index: 1,
            from: 1,
            subsequent_seq: true
        };
        let transactions = vec![transaction1, transaction2, transaction3];
        let expected_transaction_results = vec![TransactionResult::new(vec![0], true), TransactionResult::new(vec![1,2], true)];
        let expected_harness = TestHarness { transactions, accounts, starting_balances: vec![], client_senders: vec![tx_1.clone(), tx_2.clone()], client_receiver: expected_client_rx, balance_receiver: expected_balance_rx, failure_sender: failure_tx, succeeded_transactions: HashSet::new(), unfunded_transactions: HashSet::new(), transaction_results: expected_transaction_results  };
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
        assert_eq!(transaction_2, sign_and_submit_message(&mut expected_harness, 1, true));
        assert_eq!(transaction_3, sign_and_submit_message(&mut expected_harness, 2, true));
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
        let (number_of_transaction, number_of_accounts, _number_of_balance_setups, transaction_results) = TestHarness::parse_first_line("3;3;1;[0y,1|2y]");
        assert_eq!(number_of_transaction, 3);
        assert_eq!(number_of_accounts, 3);
        let expected_transaction_results = vec![TransactionResult::new(vec![0], true), TransactionResult::new(vec![1,2], true)];
        assert_eq!(transaction_results, expected_transaction_results);
    }

    #[test]
    fn test_check_transaction_results() {
        let (_number_of_transaction, _number_of_accounts, _number_of_starting_balances, transaction_results) = TestHarness::parse_first_line("3;3;[0y,1|2y]");
        let zero_success = (0, TransactionResultCode::TesSuccess);
        let zero_cost = (0, TransactionResultCode::TecUnfundedPayment);
        let one_success = (1, TransactionResultCode::TesSuccess);
        let one_cost = (1, TransactionResultCode::TecUnfundedPayment);
        let two_success = (2, TransactionResultCode::TesSuccess);
        let two_cost = (2, TransactionResultCode::TecUnfundedPayment);
        let expected_failed_multiple = TransactionResult::check_transaction_results(&transaction_results, &vec![zero_success, one_success, two_success], &vec![]);
        let expected_in_progress_none = TransactionResult::check_transaction_results(&transaction_results, &vec![zero_success], &vec![]);
        let expected_in_progress_one_none = TransactionResult::check_transaction_results(&transaction_results, &vec![], &vec![]);
        let expected_in_progress_zero_cost = TransactionResult::check_transaction_results(&transaction_results, &vec![zero_cost, one_success, two_cost], &vec![]);
        let expected_success_1 = TransactionResult::check_transaction_results(&transaction_results, &vec![zero_success, one_success, two_cost], &vec![]);
        let expected_success_2 = TransactionResult::check_transaction_results(&transaction_results, &vec![zero_success, one_cost, two_success], &vec![]);
        assert_eq!(expected_failed_multiple, Failed);
        assert_eq!(expected_in_progress_none, InProgress);
        assert_eq!(expected_in_progress_one_none, InProgress);
        assert_eq!(expected_in_progress_zero_cost, InProgress);
        assert_eq!(expected_success_1, Success);
        assert_eq!(expected_success_2, Success);
    }

    #[test]
    fn test_check_transaction_results_2() {
        let (_number_of_transaction, _number_of_accounts, _number_of_starting_balances, transaction_results) = TestHarness::parse_first_line("5;3;[0y,1|2|3|4y]");
        let zero_success = (0, TransactionResultCode::TesSuccess);
        let one_success = (1, TransactionResultCode::TesSuccess);
        let two_cost = (2, TransactionResultCode::TecUnfundedPayment);
        let three_cost = (3, TransactionResultCode::TecUnfundedPayment);
        let four_cost = (4, TransactionResultCode::TecUnfundedPayment);
        let expected_success_1 = TransactionResult::check_transaction_results(&transaction_results, &vec![zero_success, one_success, two_cost, three_cost, four_cost], &vec![]);
        let expected_in_progress_1 = TransactionResult::check_transaction_results(&transaction_results, &vec![zero_success, one_success, two_cost], &vec![]);
        assert_eq!(expected_success_1, Success);
        assert_eq!(expected_in_progress_1, InProgress);
    }

    #[test]
    #[ignore]
    fn test_failure_writer() {
        println!("{}", *LOG_FOLDER);
        let (_actual_harness, _expected_harness, _receivers) = parse_harness();
        let node_states = Arc::new(MutexNodeStates::new(NodeStates::new( vec![NodeState::new(0); 5])));
        let mut ripple_message = RippleMessage::default();
        ripple_message.from_node = "Ripple1".to_string();
        ripple_message.to_node = "Ripple2".to_string();
        node_states.add_execution(ripple_message);
        node_states.add_validated_transaction(3, Transaction::default(), TransactionResultCode::TesSuccess);
        let mut validated_ledger = crate::client::ValidatedLedger::default();
        validated_ledger.ledger_hash = "LedgerHash".to_string();
        validated_ledger.txn_count = 1;
        node_states.set_validated_ledger(2, validated_ledger);
        let current_individual: DelayMapPhenotype = DelayMapPhenotype::from_genes(&vec![100u32; num_genes()]);
        node_states.set_current_individual(current_individual.display_genotype_by_message());
    }
}
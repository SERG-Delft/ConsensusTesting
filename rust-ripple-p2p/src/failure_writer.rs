use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvError};
use std::thread;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use log::error;
use petgraph::Graph;
use crate::client::{Transaction, ValidatedLedger};
use crate::collector::RippleMessage;
use crate::node_state::{DependencyEvent, MutexNodeStates};
use crate::{LOG_FOLDER, NUM_NODES};
use crate::test_harness::TransactionResultCode;

/// Struct responsible for writing state to failure file in case of consensus property violation
/// Components send the consensus properties that are violated to this struct
pub struct FailureWriter {
    failure_receiver: Receiver<Vec<ConsensusPropertyTypes>>,
    failure_writer: BufWriter<File>,
    node_states: Arc<MutexNodeStates>,
}

impl FailureWriter {
    pub fn start_failure_writer(failure_receiver: Receiver<Vec<ConsensusPropertyTypes>>,
                                node_states: Arc<MutexNodeStates>)
    {
        let mut failure_writer = Self {
            failure_receiver,
            failure_writer: BufWriter::new(
                File::create(
                    Path::new(format!("{}\\failure_file.txt", *LOG_FOLDER).as_str()))
                    .expect("Opening failure file failed")
            ),
            node_states,
        };
        thread::spawn(move ||{
            loop {
                match failure_writer.failure_receiver.recv() {
                    Ok(consensus_properties_violated) => {
                        let failure = node_states.create_failure_data(consensus_properties_violated);
                        serde_json::to_writer(&mut failure_writer.failure_writer, &failure);
                    }
                    Err(err) => {
                        error!("Failure channel failed: {}", err);
                    }
                }
            }
        });
    }

    /// Might use in the future
    // fn handle_test_failure(failure_writer: &mut BufWriter<File>, failure: Failure) {
    //     error!("Storing failed test info...");
    //     self.failure_writer.write_all(format!("Test failure at time: {}\n", Utc::now()).as_bytes()).unwrap();
    //     self.failure_writer.write_all("Transaction state:\n".as_bytes()).unwrap();
    //     for i in 0..*NUM_NODES {
    //         self.failure_writer.write_all(format!("Peer {} validated transactions: {:?} \n", i, node_states.get_validated_transaction(i)).as_bytes()).unwrap();
    //     }
    //     self.failure_writer.write_all("Validated ledger state:\n".as_bytes()).unwrap();
    //     for i in 0..*NUM_NODES {
    //         let validated_ledger = node_states.get_validated_ledger(i);
    //         self.failure_writer.write_all(format!("Peer {} latest validated ledger index: {:?}\n", i, validated_ledger).as_bytes()).unwrap();
    //     }
    //     self.failure_writer.write_all("Current individual:\n".as_bytes()).unwrap();
    //     self.failure_writer.write_all(node_states.get_current_individual().as_bytes()).unwrap();
    //     self.failure_writer.write_all("Execution trace:\n".as_bytes()).unwrap();
    //     for event in node_states.get_executions() {
    //         self.failure_writer.write_all(event.to_string().as_bytes()).unwrap();
    //     }
    //     self.failure_writer.write_all("Dependency graph:\n".as_bytes()).unwrap();
    //     self.failure_writer.write_all(format!("{:?}", petgraph::dot::Dot::with_config(&node_states.get_dependency_graph(), &[petgraph::dot::Config::EdgeNoLabel])).as_bytes()).unwrap();
    // }
}

#[derive(Serialize, Deserialize)]
pub struct Failure {
    pub time: DateTime<Utc>,
    pub validated_transactions: Vec<Vec<(Transaction, TransactionResultCode)>>,
    pub validated_ledgers: Vec<ValidatedLedger>,
    pub current_individual: String,
    pub execution: Vec<RippleMessage>,
    pub trace_graph: Graph<DependencyEvent, ()>,
    pub consensus_properties_violated: Vec<ConsensusPropertyTypes>,
}

#[derive(Serialize, Deserialize)]
pub enum ConsensusPropertyTypes {
    Termination,
    Validity1,
    Validity2,
    Integrity1,
    Integrity2,
    Agreement1,
    Agreement2,
    DoubleSpend,
}
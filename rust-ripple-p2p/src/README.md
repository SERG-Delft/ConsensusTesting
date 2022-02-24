### Scheduler
The control flow for executing test harnesses and delaying messages can be found in [scheduler.rs](scheduler.rs).
The `start` function spawns several threads. The `listen_to_peers` function listens to messages sent by peers and
relays the message (with delay if applicable) to their destination. The `listen_to_ga` function continually listens
to the genetic algorithm for a new set of delays. The `update_current_round` and `update_latest_validated_ledger` functions
keep track of when all nodes have started on a new round or validated a new ledger. The `harness_controller` is responsible for
checking and updating the stability of the network, starting the test harness and sending the found fitness to the ga component.

### Genetic Algorithm
The genetic algorithm uses [genevo](../../genevo) as a base implementation.
In [genetic_algorithm.rs](ga/genetic_algorithm.rs) the `run` function
starts the ga and configures the parameters used in the run. This function also starts the [scheduler handler](ga/fitness.rs).
The scheduler handler is responsible for communicating new chromosome/scheduler/delays to the scheduler. Fitness functions request evaluations from this handler.

#### Gaussian Mutation
The guassian mutation operator is implemented in [mutation.rs](ga/mutation.rs). The logic can be found [here](https://github.com/SERG-Delft/ConsensusTesting/blob/37aa4476e6d59b886c2529fe1052e0a26aad3962/rust-ripple-p2p/src/ga/mutation.rs#L69).

### Fitness Functions
The different fitness functions are found in the [fitness](ga/fitness) folder. All fitness functions implement the trait
[ExtendedFitness](https://github.com/SERG-Delft/ConsensusTesting/blob/37aa4476e6d59b886c2529fe1052e0a26aad3962/rust-ripple-p2p/src/ga/fitness.rs#L22)
which contains the `run_harness` function. In this function, the test harness is run and the fitness function can check the state before
and after the test harness to determine the fitness of the chromosome.

Genevo uses a struct implementing the trait FitnessFunction. Every time the fitness of an individual needs to evaluated,
it calls the `fitness_of` function. In the code this trait is implemented by [FitnessCalculation\<T\>](https://github.com/SERG-Delft/ConsensusTesting/blob/37aa4476e6d59b886c2529fe1052e0a26aad3962/rust-ripple-p2p/src/ga/fitness.rs#L34)
Where T is any fitness function as defined above. This struct then tells the scheduler handler to run a particular individual and returns the fitness.

To use a different fitness function, change the CurrentFitness type in [genetic_algorithm.rs](ga/genetic_algorithm.rs) to the desired fitness functions.

### Node States
In [node_state.rs](node_state.rs) the state of the different nodes is tracked. This struct is a mutex which allows it to be shared and edited by many different threads.
Almost all components of the system use the node state in some way. It is used by the fitness functions to poll state before and after a test harness.
The scheduler informs the node state of message [sends](https://github.com/SERG-Delft/ConsensusTesting/blob/37aa4476e6d59b886c2529fe1052e0a26aad3962/rust-ripple-p2p/src/scheduler.rs#L123) and [receives](https://github.com/SERG-Delft/ConsensusTesting/blob/37aa4476e6d59b886c2529fe1052e0a26aad3962/rust-ripple-p2p/src/scheduler.rs#L96) to build the trace graphs.

### Client
The [client.rs](client.rs) file contains code for sending client commands to the nodes and receiving their responses.
For every node, a client is started in [app.rs](app.rs). Several client commands are used.
1. [Subscriptions](https://xrpl.org/subscribe.html): The nodes send updates with regard to consensus phase, ledgers, validations, etc. This information is sent to [collector.rs](collector.rs) and stored in the [node_state.rs](node_state.rs).
2. [Transactions](https://xrpl.org/submit.html#sign-and-submit-mode): The [test harness](test_harness.rs) submits transactions at scheduled times during the test harness.
3. [State Accounting](https://xrpl.org/server_state.html): The state accounting fitness function polls the state accounting info through the server_state command before and after the test harness to determine fitness.

### Collector
The [collector](collector.rs) is responsible for collecting data from the nodes, writing this data to files and updating the node states.

### Test Harness
[Test_harness.rs](test_harness.rs) is used for running a test case. It reads what transactions to apply and at what time from `harness.txt`. 
The fitness functions use this test harness in `run_harness` to schedule the transactions. In the `schedule_transactions` function the transactions are scheduled
and the function waits for all transactions to appear in a validated ledger, after which it will return to the `run_harness` function in the fitness function.

### Experiments
The [trace_comparisons.rs](trace_comparisons.rs) file contains code for collecting the data used in the trace graph and fitness function comparison experiment.
Instead of running the genetic algorithm [scheduler handler](ga/fitness.rs) and starting the GA, a custom SchedulerHandler is used which supplies predetermined delays and writes their fitness to disk.
For the fitness comparison experiment, use the [compared_fitness_functions.rs](ga/fitness/compared_fitness_functions.rs) as the CurrentFitness type. This combines different fitness functions. See the `run_harness` function.
The experiments are conducted in the [trace_comparisons](trace_comparisons) folder.
# Papers

Overview of [DS testing approaches](https://github.com/asatarin/testing-distributed-systems)

[Exploratory testing](Exploratory_testing_concurrent_systems.pdf)

Fuzzing and test case reducing tool. https://github.com/NetSys/demi, see https://colin-scott.github.io/blog/2015/10/07/fuzzing-raft-for-fun-and-profit/ demi [paper](Demi_event_minimization.pdf)

Studies of bugs in distributed systems: https://ucare.cs.uchicago.edu/pdf/socc14-cbs.pdf, https://ucare.cs.uchicago.edu/pdf/asplos16-TaxDC.pdf, https://blog.acolyer.org/2019/06/21/what-bugs-cause-cloud-production-incidents/

Testing DSes by reducing state-space: [paper](FlyMC_state_space_reduction.pdf)

Shadow: Running Tor in a box (could be useful for running ripple). https://www.robgjansen.com/publications/shadow-ndss2012.pdf

Amazon's DS testing approach: http://lamport.azurewebsites.net/tla/formal-methods-amazon.pdf

[Blockchain testing challenges](Blockchain_testing_challenges.pdf)

Ripple paper about network parameters, contains interesting links to network simulator: [paper](Testing_ripple_fault_tolerance.pdf)
  * Contains several metrics for the overall health of the network. Potential for fitness function? NHI and convergence time
  
Ripple simulators: https://github.com/ripple/simulator/, https://github.com/ripple/rippled/tree/develop/src/test/csf Perhaps modify this to run the algorithm?
  * It has an event/collector framework for monitoring invariants and statistics. It's able to collect after every round. Seems ideal for calculating fitness functions.
	This also has a way to progress the algorithm synchronously (sim.run(1) runs the simulation for one ledger or phase). Might need more fine-grained synchrony, ie. each round.
	Used to unit test ripple. Easy to define network configurations.
	Extensions needed in terms of finegrained control over message delivery. Maybe create a custom node in the center that controls delivery to all other nodes.
	Other solution is to position on a lower level as middleware.
	A lot of freedom in terms of manipulating the protocol. Has possibilities for executing arbitrary code at arbitrary times.
	
Fitness function for test data generation, mutation adequacy: [paper](Search_based_test_data_generation.pdf) (Mutate the system under test, the more tests fail the better)

General search-based software testing: [paper](Search_based_testing_survey.pdf)
Two ingredients: Encoding and fitness function.
	
Random testing for partition tolerance: [paper](Random_testing_partition_tolerance.pdf)

Communication closure: [paper](Communication_closure.pdf) \
	parameters: n processes, p phases, r rounds, k recovery period in rounds, d number of processes isolated in run.
	
Ripple code analysis: [paper](Ripple_code_analysis.pdf)

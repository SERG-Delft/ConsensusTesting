# Papers

Overview of DS testing approaches: https://github.com/asatarin/testing-distributed-systems

Exploratory testing: https://www.pdl.cmu.edu/PDL-FTP/associated/CMU-PDL-11-113.pdf

Fuzzing and test case reducing tool. https://github.com/NetSys/demi, see https://colin-scott.github.io/blog/2015/10/07/fuzzing-raft-for-fun-and-profit/ https://colin-scott.github.io/personal_website/research/nsdi16.pdf

Study of bugs in distributed systems: https://ucare.cs.uchicago.edu/pdf/socc14-cbs.pdf, https://ucare.cs.uchicago.edu/pdf/asplos16-TaxDC.pdf, https://blog.acolyer.org/2019/06/21/what-bugs-cause-cloud-production-incidents/

Testing DSes by reducing state-space: https://ucare.cs.uchicago.edu/pdf/eurosys19-flyMC.pdf

Shadow: Running Tor in a box (could be useful for running ripple). https://www.robgjansen.com/publications/shadow-ndss2012.pdf

Amazon's DS testing approach: http://lamport.azurewebsites.net/tla/formal-methods-amazon.pdf

Blockchain testing challenges: https://sci-hub.se/https://ieeexplore.ieee.org/abstract/document/8529728

Ripple paper about network parameters, contains interesting links to network simulator: https://www.mdpi.com/1999-5903/12/3/53/htm
  * Contains several metrics for the overall health of the network. Potential for fitness function? NHI and convergence time
  
Ripple simulators: https://github.com/ripple/simulator/, https://github.com/ripple/rippled/tree/develop/src/test/csf Perhaps modify this to run the algorithm?
  * It has an event/collector framework for monitoring invariants and statistics. It's able to collect after every round. Seems ideal for calculating fitness functions.
	This also has a way to progress the algorithm synchronously (sim.run(1) runs the simulation for one ledger or phase). Might need more fine-grained synchrony, ie. each round.
	The simulation runs the ripple consensus protocol under the hood. Used to unit test ripple. Easy to define network configurations.
	Extensions needed in terms of finegrained control over message delivery. Maybe create a custom node in the center that controls delivery to all other nodes.
	Other solution is to position on a lower level as middleware.
	A lot of freedom in terms of manipulating the protocol. Has possibilities for executing arbitrary code at arbitrary times.
	
Fitness function for test data generation, mutation adequacy: https://core.ac.uk/reader/46564727 (Mutate the system under test, the more tests fail the better)

General search-based software testing: https://ieeexplore.ieee.org/abstract/document/5954405?casa_token=9lZA58d-2lQAAAAA:WCzII1Pbk2nIrJflKKQNyeROMu24uAak3p7h9ebHgAD_JHQaMn4kDvk0y_198KJkt_ko-yXzww \
Two ingredients: Encoding and fitness function.
	
Random testing for partition tolerance: https://dl.acm.org/doi/pdf/10.1145/3158134

Communication closure: https://dl.acm.org/doi/10.1145/3428278 \
	parameters: n processes, p phases, r rounds, k recovery period in rounds, d number of processes isolated in run.
	
Ripple code analysis: https://arxiv.org/pdf/2011.14816.pdf
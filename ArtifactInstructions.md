# Artifact Instructions

This document provides the instructions to reproduce the experimental results in the paper "Randomized Testing of Byzantine Fault Tolerant Consensus Algorithms" for testing an implementation of the Ripple (v1.7.2) consensus algorithm.

## Running Quick Tests

1. In order to build the required Docker images, the following two commands can be used:

   ```
   docker build -t ripple -f rippled.Dockerfile .
   DOCKER_BUILDKIT=1 docker build -t byzzfuzz .
   ```

   This step requires at least 12 GiB of RAM and is expected to take several hours.

2. Run one Byzzfuzz test for each configuration of Table 5 which have different values for the parameter $c$, rounds with network partitions, and $d$, rounds with process faults, of the testing algorithm:

   ```
   ./run.sh 1
   ```

   It is expected that the command executes within 25 minutes, after which it will timeout. Error messages printed to the console can be ignored if the command does not time out. In case of success,  the output of the executions will be saved to the `traces` folder.

3. After running the executions, use the `analyze.py` script to aggregate the results into a table as follows:

   ```
   python3 analyze.py
   ```

   The script is expected to print out a table with the structure of the example below:

   ```
              |TOTAL|CORRECT|INSUFFICIENT|INCOMPATIBLE|TIMEOUT|INCOMPLETE|UNCATEGORIZED
    d=0 c=0 bs|    1|      1|           0|           0|      0|         0|            0 []
    d=1 c=0 as|    1|      1|           0|           0|      0|         0|            0 []
    d=2 c=0 as|    1|      1|           0|           0|      0|         0|            0 []
    d=0 c=1 as|    1|      1|           0|           0|      0|         0|            0 []
    d=0 c=1 ss|    1|      1|           0|           0|      0|         0|            0 []
    d=1 c=1 as|    1|      1|           0|           0|      0|         0|            0 []
    d=1 c=1 ss|    1|      1|           0|           0|      0|         0|            0 []
    d=2 c=1 as|    1|      0|           1|           0|      0|         0|            0 []
    d=2 c=1 ss|    1|      1|           0|           0|      0|         0|            0 []
    d=0 c=2 as|    1|      1|           0|           0|      0|         0|            0 []
    d=0 c=2 ss|    1|      1|           0|           0|      0|         0|            0 []
    0 uncategorized
   ```

   In rare cases, the fuzzer can get stuck. The `run.sh` script will timeout after 25 minutes. Please, start the script again in case it times out and the `analyze.py` script does not output a table.

## Step-by-Step Instructions

1. In order to build the required Docker images, the following two commands can be used:

   ```
   docker build -t ripple -f rippled.Dockerfile .
   DOCKER_BUILDKIT=1 docker build -t byzzfuzz .
   ```

   This step requires at least 12 GiB of memory, 8 GiB of swap space, and 15 GiB of free storage and is expected to take several hours.

2. To rerun the experiments used for Table 5, use the `run.sh` script to execute each configuration 300 times, which is expected to take at approximately 4 days:

   ```
   ./run.sh 300
   ```

   The output of the executions will be saved to the `traces` folder. For each configuration, a folder is created (e.g. `buggy-7-1-1-6-any-scope-0.2.4`, following the scheme `buggy-7-{c}-{d}-{any-scope|small-scope|baseline}-{version-of-code}`) in which a subfolder for each run is created with the UNIX timestamp of its creation as name. In that folder, for each run the full message lineage is saved to `execution.txt` and ByzzFuzz's output is saved to `results.txt`. The `subscription_*.json` and `validator_*.txt` files contain the events published by each process and its logs, respectively.

3. After running the executions, use the `analyze.py` script to aggregate the results into a table as follows:

   ```
   python3 analyze.py
   ```

   The script is expected to print out a table with the structure of the example below:

   ```
              |TOTAL|CORRECT|INSUFFICIENT|INCOMPATIBLE|TIMEOUT|INCOMPLETE|UNCATEGORIZED
    c=0 d=0 bs|  300|    259|           0|           0|     41|         0|            0 []
    d=1 c=0 as|  300|    280|          19|           0|      0|         0|            1 ['1666645928']
    d=2 c=0 as|  300|    280|          16|           0|      2|         0|            2 ['1666506880', '1666900309']
    d=0 c=1 as|  300|    270|          29|           0|      1|         0|            0 []
    d=0 c=1 ss|  300|    281|          19|           0|      0|         0|            0 []
    d=1 c=1 as|  300|    267|          28|           0|      5|         0|            0 []
    d=1 c=1 ss|  300|    270|          28|           2|      0|         0|            0 []
    d=2 c=1 as|  300|    272|          28|           0|      0|         0|            0 []
    d=2 c=1 ss|  300|    267|          26|           2|      1|         0|            4 ['1667247396', '1666941103', '1667144360', '1666498776']
    d=0 c=2 as|  300|    258|          40|           1|      2|         0|            0 []
    d=0 c=2 ss|  300|    251|          47|           5|      1|         0|            0 []
    7 uncategorized
   ```

## (Optional) Configuring and Extending for Reuse

The implementation of ByzzFuzz for the Ripple blockchain can be modified to support different mutations or even different distributed systems. In the following, we give hints to researchers that wish to extend our work. A certain level of familiarity with Rust is expected.

#### Adapting the Mutations

In the `rust-ripple-p2p` crate, the `apply_mutation` function of the `ByzzFuzz` struct is responsible for applying the mutations to the messages. The message is passed in binary form to the function, together with a seed, and the function can modify the message before it returns it to the interception layer. By manipulating the bytes, ByzzFuzz is able to mutate the messages. A parser for Ripple's canonical binary format can be found in the `serialize` crate.

When writing mutations, Ripple's documentation on the [XRPL Developer Portal](https://xrpl.org/protocol-reference.html#main-page-header) can be of assistance with respect to message encodings and conventions regarding serialization and deserialization.

#### Adapting to a Different Distributed System

Even though the implementation is catered to the Ripple blockchain, the message interception layer developed in Rust can be reused for other distributed systems. The core implementation of ByzzFuzz with its logic to simulate network partitions is flexible enough to suit many other systems.

It is advised to also package the individual processes in Docker containers that can be spawned by the implementation. `peer_connection.rs` must be updated to implement the required network handshakes that the processes use to establish connections. Further, the contents of the `serialize` crate need to be replaced with code that suits the specific binary format used to exchange messages between the processes. The `specs` module of the `rust-ripple-p2p` crate provides an interface where custom assertions that are executed during the execution of the program can be defined.
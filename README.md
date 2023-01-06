# Byzzfuzz randomized testing for checking Byzantine fault-tolerance of the XRP Ledger Consensus Protocol of Ripple

This repository contains the implementation of Byzzfuzz testing algorithm for testing the implementation of the Ripple consensus protocol.

You can access the repository for the implementation of `rippled` server [here](https://github.com/XRPLF/rippled) and a proof of concept used as a basis for our work [here](https://github.com/fanatid/rust-ripple-p2p).

## Requirements 
- [Docker](https://docs.docker.com/get-started/)
- [Rust](https://www.rust-lang.org/learn/get-started)

## Context

We tested Ripple in a network of 7 processes (`p0`, `p1`, `p2`, `p3`, `p4`, `p5`, `p6`). Processes {`p0`, `p1`, `p2`} trust `UNL1` = {`p0`, `p1`, `p2`, `p3`, `p4`}
and {`p4`, `p5`, `p6`} trust `UNL2` = {`p2`, `p3`, `p4`, `p5`, `p6`}.

Given the number of rounds with network faults (`d`), the number of rounds with process faults (`c`), and the number of rounds (`r`) to distribute the faults as test parameters, Byzzfuzz randomly generates a test execution that injects `d` random network and `c` random process faults into the execution. Alternatively, it can enforce the execution of a given fault configuration of which faults to inject in which rounds.

## Building and running the system

Build the container from the Dockerfile:

```
docker build -t ripple -f rippled.Dockerfile .
DOCKER_BUILDKIT=1 docker build -t byzzfuzz .
```

Run the container for a certain time of iterations using the following:
```
./run.sh <number of iterations>
```

The output of the executions will be saved to the `traces` folder. For each configuration, a folder is created (e.g. `buggy-7-1-1-6-any-scope-0.2.4`, following the scheme `buggy-7-{c}-{d}-{any-scope|small-scope|baseline}-{version-of-code}`) in which a subfolder for each run is created with the UNIX timestamp of its creation as name. In that folder, for each run the full message lineage is saved to `execution.txt` and ByzzFuzz's output is saved to `results.txt`. The `subscription_*.json` and `validator_*.txt` files contain the events published by each process and its logs, respectively.

## Analyzing Results
After running hundreds of iterations of the different executions, you can use the `analyze.py` script to aggregate the results into a table as follows:
   
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

## Repository Overview
This repository is comprised of three crates:
- `rust-ripple-p2p` contains the main implementation
- `serialize` is a library that is capable of parsing Ripple's canonical binary format
- `analyzer` contains functions that can be used to quickly inspect and aggregate a single run

`This README.md file is an addition to the extensive guide this artifact contains. For detailed explanations, consult the artifact's README.md.`

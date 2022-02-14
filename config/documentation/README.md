# Documentation for Ripple Docker Scripts

### 1. Adding new ripple nodes to the network
##### Create a new validator key pair and token
https://github.com/ripple/validator-keys-tool/blob/master/doc/validator-keys-tool-guide.md
1. Create the "config/validator-keys_{i}.json", following the format of the other files.
2. Add the public_key from that file to "config/validators.txt" at the ith position.
3. add ripple{i}:51235 to the [ips_fixed] in "config/rippled.cfg"
4. Create a cluster seed and key-pair: "rippled validation_create "BAWL MAN JADE MOON DOVE GEM SON NOW HAD ADEN GLOW TIRE""
5. Add this to "config/cluster_seeds.json"
6. Go through "rippled-docker/ConfigCreator.ps1" and make sure rippled.cfg line indices are correct now that more lines are added.
7. "rippled-docker/Run.ps1 {n} p" should now start more ripple docker containers that validate ledgers.
8. Use ripple client commands to verify peer lists and UNL.
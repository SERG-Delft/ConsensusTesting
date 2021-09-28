# ripple-p2p

[Ripple](https://ripple.com/) have own unique handshake in p2p network. Shared data for [DH exchange](https://en.wikipedia.org/wiki/Diffie%E2%80%93Hellman_key_exchange) come from messages between peers, so it's can be not possible do connection from some languages (see original [issue](https://github.com/ripple/rippled/issues/2413) in rippled repository).

I was interesting how connect to network from [Rust](https://www.rust-lang.org/) and made this dirty PoC. There a lot of space for improvements and I recognize this, but I was need only connect to peer and parse incoming messages, not create some app for usage.

# project-takoyaki-botnet

A proof-of-concept peer-to-peer botnet implemented in Rust using the [libp2p](https://libp2p.io) networking stack. This
project was made for research purposes and showcases how to implement a secure, distributed
peer-to-peer infrastructure with built-in cryptographic mechanisms and scripting for flexibility.

## About

This project was initiated to explore the Kademlia Distributed Hash Table (DHT) and the mathematical principles that
support it. Additionally, it aims to provide insights into how peer-to-peer (P2P) interactions are managed within
blockchain networks, such as Ethereum.

## Protocol

### Peer Discovery

1. **Bootstrapping**: When a new peer joins the network, it connects to a known peer by setting the `BOOTSTRAP_ADDRESS`
   environment variable to the peer-to-peer multiaddress of any available peer. This serves as the initial entry into
   the network.
2. **Peer Identification**: Upon establishing a connection, peers utilize the libp2p Identify protocol to exchange peer
   information. This verifies version compatibility and initializes end-to-end encrypted peer-to-peer tunnels.
3. **External Address Verification**: To confirm a peer's external reachability, each peer uses the libp2p AutoNAT v2
   protocol to test NAT traversal capabilities. This mutual verification allows peers to confirm external addresses.
   If an external address is discovered, the peer configures itself to be a server node, enhancing the Kademlia network
   quality.

### Achieving Persistence

After peer-to-peer identification, peers update their internal persistent storage to allow for routing across reboots.
Each peer also saves its keypair and assigned address to this storage. The storage is saved to disk if possible and
is encrypted using an obfuscated AES-256 key.

### Communication

Peers in the network use the libp2p GossipSub protocol to broadcast and listen for commands
and [Dilithium](https://pq-crystals.org/dilithium/) for payload signing. A peer can only send commands if it verifies
the authenticity of a Dilithium private key from a stored file. Only commands signed with the private key will be
acknowledged by other peers. Having the `project-takoyaki.key` file present allows the peer to send commands by reading
from the standard input.

## Structure

The configuration and keys for the network can be generated using `project-takoyaki-keygen`, which will create the
AES-256 key used for encrypting the storage file, the network secret key, and the Dilithium keypair. It will generate
`config.rs` and `project-takoyaki.key`, with the latter containing the Dilithium private key.

The main peer software is in `project-takoyaki-peer`, containing everything related to peer-to-peer communication and
the network.

## Features

- **Decentralized Peer-to-Peer Discovery**
- **Secure Persistent Storage**
- **Adaptive Peer Management**
- **Post-Quantum Cryptography**
- **Lua 5.4 Scripting Integration**

## Disclaimer

**This project is for educational and research purposes only**. It should not be used for any activities that are
illegal or violate laws and regulations. The authors and contributors disclaim any responsibility for misuse of the
project.

## License

This project is licensed under the MIT license. See [LICENSE](LICENSE) for details.

# project-takoyaki-botnet

A proof-of-concept peer-to-peer botnet implemented in Rust using the [libp2p](https://libp2p.io) networking stack.

## About

This project uses the [Kademlia distributed hash table](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf) for scalable peer-to-peer networking. It is inspired by Kademlia's underlying mathematics, which enables distance-based node discovery and routing. Kademlia is a key component in blockchain systems like Ethereum, where it helps manage decentralized data efficiently. This project applies Kademlia in the context of cybersecurity, aiming to improve the resilience and efficiency of decentralized networks.

> **Disclaimer:** This project is for educational and research purposes only. It should not be used for any activities that violate laws or regulations. The authors and contributors disclaim any responsibility for misuse of the project.

## Protocol

### Peer Discovery

#### 1. Bootstrapping
When a new peer joins the network, it connects to a known peer specified by the `BOOTSTRAP_ADDRESS` environment variable, which contains the peer-to-peer multiaddress of an available peer. This serves as the initial entry point into the network.

#### 2. Peer Identification
Upon establishing a connection, peers use the libp2p Identify protocol to exchange peer information, verify version compatibility, and initialize end-to-end encrypted peer-to-peer tunnels.

#### 3. External Address Verification
To confirm external reachability, each peer uses the libp2p AutoNAT v2 protocol to test NAT traversal capabilities. This process allows peers to verify each other's external addresses. If an external address is discovered, the peer configures itself as a server node to improve network quality.

### Persistence and Data Management

After peer-to-peer identification, each peer updates its internal persistent storage, enabling routing continuity across reboots. Each peer saves its keypair and assigned address to a file encrypted using AES-256-GCM.

### Peer Communication

Peers in the network use the libp2p GossipSub protocol to broadcast and listen for commands. For payload signing, they use [Dilithium](https://pq-crystals.org/dilithium/), a post-quantum cryptographic algorithm. A peer can only send commands after verifying the authenticity of a Dilithium private key stored in the `project-takoyaki.key` file. Only commands signed with this key will be accepted by other peers. An authenticated peer sends commands by reading from standard input.

## Project Components

### `project-takoyaki-keygen`

A command-line tool for generating and managing cryptographic keys and configuration files used by `project-takoyaki-peer`.

#### Generated Assets

- `dilithium-private-key`
- `dilithium-public-key`
- `network-name`
- `storage-encryption-key`
- `swarm-encryption-key`

### `project-takoyaki-peer`

The core module for coordinating command-and-control operations over a decentralized peer-to-peer network.

#### Features

- Distance-based peer discovery
- Secure persistent storage
- Adaptive peer management
- Post-quantum cryptography
- Lua 5.4 scripting integration

## Building

Clone the `project-takoyaki-botnet` repository:

```bash
git clone https://github.com/project-takoyaki/project-takoyaki-botnet.git
cd project-takoyaki-botnet
```
> Ensure you have [Rust](https://www.rust-lang.org/) installed.

Generate the required keys:

```bash
cargo run --bin project-takoyaki-keygen --release
```

> This project comes with pre-generated keys, but it is strongly reccomended to generate your own.

Start a node:

```bash
cargo run --bin project-takoyaki-peer --release
```

> To connect to an existing network, set the `BOOTSTRAP_ADDRESS` environment variable to a sever node's multiaddress before starting `project-takoyaki-peer`.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

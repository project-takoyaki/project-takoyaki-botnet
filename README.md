# project-takoyaki-botnet

This repository contains the source code for a proof-of-concept peer-to-peer botnet written entirely in Rust, built
using [libp2p](https://libp2p.io).

## About

This project uses Kademlia for peer-to-peer discovery. Kademlia is a distributed hash table (DHT) protocol that enables
decentralized storage and retrieval of key-value pairs across the network. This project demonstrates the potential of
decentralized networks in building resilient and scalable systems.

## Features

- **Peer-to-Peer Discovery**: Utilizes Kademlia for decentralized file storage and discovery.
- **Persistent Storage**: Securely stores peer information and critical data using AES-256 encryption.
- **Dynamic Peer Management**: Adds and removes peers dynamically, allowing for flexible network management.
- **Logging**: Provides configurable logging for debugging and monitoring purposes.

## Disclaimer

This project is intended for educational and research purposes only. It is not to be used for any malicious activities
or actions that violate any laws or regulations. The authors and contributors of this project are not responsible for
any misuse of the code or any damages caused by the use of this project.

## License

This project is licensed under the terms of the BSD-3 license. See [LICENSE](LICENSE) for more information.

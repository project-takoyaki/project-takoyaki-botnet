/* the pre-shared key used to authenticate communication within the swarm */
pub const SWARM_PRESHARED_KEY: [u8; 32] = [
  0x75, 0x95, 0x70, 0x75, 0xa3, 0x31, 0x4a, 0xb6, 0x9e, 0x47, 0x89, 0x35, 0x34, 0x7d, 0x2b, 0x37,
  0xf2, 0x5f, 0xaf, 0x2d, 0x6d, 0x76, 0xd2, 0x54, 0x91, 0x3f, 0x96, 0x63, 0x73, 0xbb, 0xf1, 0x60,
];

/* the filename used to override the random keypair for the node */
pub const KEYPAIR_OVERRIDE_FILENAME: &str = "project-takoyaki-key.override";

/* the unique protocol identifier for node identity */
pub const IDENTIFY_PROTOCOL_VERSION: &str = "/project-takoyaki/id/1.0.0";

/* the unique protocol identifier for Kademlia */
pub const KADEMLIA_PROTOCOL_VERSION: &str = "/project-takoyaki/kad/1.0.0";

/* the gossipsub topic to subscribe to */
pub const GOSSIPSUB_TOPIC: &str = "project-takoyaki";

/* list of multiadresses used to bootstrap the network */
pub const SEED_NODES: [&str; 2] = [
  "/ip4/192.168.1.231/tcp/64406/p2p/12D3KooWQzVCYAtTghHPo6VWGQ27aV1tEPPzSpeESzmXTcmnWMnL",
  "/ip4/192.168.1.231/tcp/64406/p2p/12D3KooWQzVCYAtTghHPo6VWGQ27aV1tEPPzSpeESzmXTcmnWMnL",
];

/* default port for peers to bind to when establishing connections */
pub const DEFAULT_PORT: u16 = 0;

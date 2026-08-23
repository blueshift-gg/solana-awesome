//! A single crate that collects the Solana ecosystem crates and exposes each
//! one behind a feature flag.
//!
//! Enable only what you need:
//!
//! ```toml
//! [dependencies]
//! solana-awesome = { version = "0.1", features = ["pubkey", "rpc-client"] }
//! ```
//!
//! Or grab a group: `core` (all SDK primitives), `clients` (all client
//! crates), or `full` (everything).
//!
//! Upstream crate features (`serde`, `borsh`, `verify`, `rand`, ...) are
//! forwarded as weak pass-through features of the same name: they apply to
//! whichever enabled crates support them and never pull a crate in by
//! themselves.
//!
//! ```toml
//! [dependencies]
//! solana-awesome = { version = "0.1", features = ["core", "serde"] }
//! ```
//!
//! ```ignore
//! use solana_awesome::pubkey::Pubkey;
//! use solana_awesome::rpc_client::rpc_client::RpcClient;
//! ```

// --- Core SDK ---

#[cfg(feature = "pubkey")]
pub use solana_pubkey as pubkey;

#[cfg(feature = "keypair")]
pub use solana_keypair as keypair;

#[cfg(feature = "signer")]
pub use solana_signer as signer;

#[cfg(feature = "signature")]
pub use solana_signature as signature;

#[cfg(feature = "instruction")]
pub use solana_instruction as instruction;

#[cfg(feature = "message")]
pub use solana_message as message;

#[cfg(feature = "transaction")]
pub use solana_transaction as transaction;

#[cfg(feature = "hash")]
pub use solana_hash as hash;

#[cfg(feature = "account")]
pub use solana_account as account;

#[cfg(feature = "commitment-config")]
pub use solana_commitment_config as commitment_config;

#[cfg(feature = "native-token")]
pub use solana_native_token as native_token;

#[cfg(feature = "system-interface")]
pub use solana_system_interface as system_interface;

#[cfg(feature = "compute-budget-interface")]
pub use solana_compute_budget_interface as compute_budget_interface;

// --- Umbrellas ---

#[cfg(feature = "program")]
pub use solana_program as program;

// --- Clients ---

#[cfg(feature = "client")]
pub use solana_client as client;

#[cfg(feature = "rpc-client")]
pub use solana_rpc_client as rpc_client;

#[cfg(feature = "rpc-client-api")]
pub use solana_rpc_client_api as rpc_client_api;

#[cfg(feature = "tpu-client")]
pub use solana_tpu_client as tpu_client;

#[cfg(feature = "quic-client")]
pub use solana_quic_client as quic_client;

#[cfg(feature = "udp-client")]
pub use solana_udp_client as udp_client;

#[cfg(feature = "connection-cache")]
pub use solana_connection_cache as connection_cache;

#[cfg(feature = "pubsub-client")]
pub use solana_pubsub_client as pubsub_client;

#[cfg(feature = "transaction-status")]
pub use solana_transaction_status as transaction_status;

#[cfg(feature = "account-decoder")]
pub use solana_account_decoder as account_decoder;

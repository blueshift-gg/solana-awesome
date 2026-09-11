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
//! crates), `onchain` (all program-side crates), `interfaces` (all
//! `*-interface` crates), or `full` (everything).
//!
//! The crate is `no_std`, so an on-chain program can depend on it and pull
//! in only the crates it enabled.
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

// Pure re-exports, so the crate itself needs no `std`: an on-chain `no_std`
// program can depend on it and pull in only the crates it enabled.
#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

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

// --- On-chain program crates ---

// core types
#[cfg(feature = "account-info")]
pub use solana_account_info as account_info;

#[cfg(feature = "instruction-error")]
pub use solana_instruction_error as instruction_error;

#[cfg(feature = "sdk-ids")]
pub use solana_sdk_ids as sdk_ids;

// hashing
#[cfg(feature = "blake3-hasher")]
pub use solana_blake3_hasher as blake3_hasher;

#[cfg(feature = "keccak-hasher")]
pub use solana_keccak_hasher as keccak_hasher;

#[cfg(feature = "sha256-hasher")]
pub use solana_sha256_hasher as sha256_hasher;

// clock / epoch / slot data
#[cfg(feature = "clock")]
pub use solana_clock as clock;

#[cfg(feature = "epoch-rewards")]
pub use solana_epoch_rewards as epoch_rewards;

#[cfg(feature = "epoch-schedule")]
pub use solana_epoch_schedule as epoch_schedule;

#[cfg(feature = "epoch-stake")]
pub use solana_epoch_stake as epoch_stake;

#[cfg(feature = "fee-calculator")]
pub use solana_fee_calculator as fee_calculator;

#[cfg(feature = "last-restart-slot")]
pub use solana_last_restart_slot as last_restart_slot;

#[cfg(feature = "slot-hashes")]
pub use solana_slot_hashes as slot_hashes;

#[cfg(feature = "slot-history")]
pub use solana_slot_history as slot_history;

// other sysvars
#[cfg(feature = "instructions-sysvar")]
pub use solana_instructions_sysvar as instructions_sysvar;

#[cfg(feature = "rent")]
pub use solana_rent as rent;

#[cfg(feature = "sysvar")]
pub use solana_sysvar as sysvar;

#[cfg(feature = "sysvar-id")]
pub use solana_sysvar_id as sysvar_id;

// program scaffolding
#[cfg(feature = "cpi")]
pub use solana_cpi as cpi;

#[cfg(feature = "msg")]
pub use solana_msg as msg;

#[cfg(feature = "program-entrypoint")]
pub use solana_program_entrypoint as program_entrypoint;

#[cfg(feature = "program-error")]
pub use solana_program_error as program_error;

#[cfg(feature = "program-memory")]
pub use solana_program_memory as program_memory;

#[cfg(feature = "program-option")]
pub use solana_program_option as program_option;

#[cfg(feature = "program-pack")]
pub use solana_program_pack as program_pack;

// serialization helpers
//
// `borsh_utils`, not `borsh`: the `borsh` feature name is already the
// pass-through that turns on borsh support across the other crates.
#[cfg(feature = "borsh-utils")]
pub use solana_borsh as borsh_utils;

#[cfg(feature = "serde-varint")]
pub use solana_serde_varint as serde_varint;

#[cfg(feature = "serialize-utils")]
pub use solana_serialize_utils as serialize_utils;

#[cfg(feature = "short-vec")]
pub use solana_short_vec as short_vec;

#[cfg(feature = "stable-layout")]
pub use solana_stable_layout as stable_layout;

// crypto / math
#[cfg(feature = "big-mod-exp")]
pub use solana_big_mod_exp as big_mod_exp;

#[cfg(feature = "secp256k1-recover")]
pub use solana_secp256k1_recover as secp256k1_recover;

// syscalls
#[cfg(feature = "define-syscall")]
pub use solana_define_syscall as define_syscall;

// --- Program interfaces ---

#[cfg(feature = "address-lookup-table-interface")]
pub use solana_address_lookup_table_interface as address_lookup_table_interface;

#[cfg(feature = "feature-gate-interface")]
pub use solana_feature_gate_interface as feature_gate_interface;

#[cfg(feature = "loader-v3-interface")]
pub use solana_loader_v3_interface as loader_v3_interface;

#[cfg(feature = "stake-interface")]
pub use solana_stake_interface as stake_interface;

#[cfg(feature = "vote-interface")]
pub use solana_vote_interface as vote_interface;

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

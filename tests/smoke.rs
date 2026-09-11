//! Smoke tests exercising the feature-gated re-exports.
//! Run with: cargo test --all-features
//! (--all-features covers `full` plus every pass-through feature; each test
//! only fires when the features it needs are enabled, so subsets work too.)

#[cfg(feature = "pubkey")]
#[test]
fn pubkey_reexport() {
    use solana_awesome::pubkey::Pubkey;
    use std::str::FromStr;

    let key = Pubkey::from_str("11111111111111111111111111111111").unwrap();
    assert_eq!(key, Pubkey::default());
}

#[cfg(all(feature = "keypair", feature = "signer", feature = "signature"))]
#[test]
fn keypair_signs() {
    use solana_awesome::keypair::Keypair;
    use solana_awesome::signer::Signer;

    let keypair = Keypair::new();
    let signature = keypair.sign_message(b"hello");
    assert!(signature.verify(keypair.pubkey().as_ref(), b"hello"));
}

#[cfg(all(
    feature = "system-interface",
    feature = "pubkey",
    feature = "instruction"
))]
#[test]
fn system_transfer_instruction() {
    use solana_awesome::pubkey::Pubkey;
    use solana_awesome::system_interface::instruction::transfer;

    let from = Pubkey::new_unique();
    let to = Pubkey::new_unique();
    let ix = transfer(&from, &to, 1);
    assert_eq!(ix.program_id, solana_awesome::system_interface::program::ID);
}

#[cfg(all(feature = "rpc-client", feature = "commitment-config"))]
#[test]
fn rpc_client_constructs() {
    use solana_awesome::commitment_config::CommitmentConfig;
    use solana_awesome::rpc_client::rpc_client::RpcClient;

    let client = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".to_string(),
        CommitmentConfig::confirmed(),
    );
    assert_eq!(client.commitment(), CommitmentConfig::confirmed());
}

#[cfg(feature = "native-token")]
#[test]
fn native_token_constants() {
    use solana_awesome::native_token::LAMPORTS_PER_SOL;
    assert_eq!(LAMPORTS_PER_SOL, 1_000_000_000);
}

// --- Pass-through features ---
// These only fire when the pass-through feature is enabled on top of the
// crate feature, e.g. `cargo test --features full,serde,verify`.

#[cfg(all(feature = "pubkey", feature = "serde"))]
#[test]
fn pubkey_serde_passthrough() {
    use solana_awesome::pubkey::Pubkey;

    let key = Pubkey::new_unique();
    let json = serde_json::to_string(&key).unwrap();
    let back: Pubkey = serde_json::from_str(&json).unwrap();
    assert_eq!(key, back);
}

#[cfg(all(
    feature = "keypair",
    feature = "signer",
    feature = "signature",
    feature = "verify"
))]
#[test]
fn signature_verify_passthrough() {
    use solana_awesome::keypair::Keypair;
    use solana_awesome::signer::Signer;

    let keypair = Keypair::new();
    let signature = keypair.sign_message(b"pass-through");
    assert!(signature.verify(keypair.pubkey().as_ref(), b"pass-through"));
}

#[cfg(all(feature = "hash", feature = "serde"))]
#[test]
fn hash_serde_passthrough() {
    use solana_awesome::hash::Hash;

    let hash = Hash::new_from_array([7; 32]);
    let json = serde_json::to_string(&hash).unwrap();
    let back: Hash = serde_json::from_str(&json).unwrap();
    assert_eq!(hash, back);
}

#[cfg(all(feature = "signature", feature = "serde"))]
#[test]
fn signature_serde_passthrough() {
    use solana_awesome::signature::Signature;

    let signature = Signature::from([42; 64]);
    let json = serde_json::to_string(&signature).unwrap();
    let back: Signature = serde_json::from_str(&json).unwrap();
    assert_eq!(signature, back);
}

#[cfg(all(feature = "commitment-config", feature = "serde"))]
#[test]
fn commitment_config_serde_passthrough() {
    use solana_awesome::commitment_config::CommitmentConfig;

    let config = CommitmentConfig::finalized();
    let json = serde_json::to_string(&config).unwrap();
    let back: CommitmentConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, back);
}

#[cfg(all(feature = "account", feature = "pubkey", feature = "serde"))]
#[test]
fn account_serde_passthrough() {
    use solana_awesome::account::Account;

    let account = Account {
        lamports: 42,
        data: vec![1, 2, 3],
        owner: Default::default(),
        executable: false,
        rent_epoch: 0,
    };
    let json = serde_json::to_string(&account).unwrap();
    let back: Account = serde_json::from_str(&json).unwrap();
    assert_eq!(account, back);
}

#[cfg(all(
    feature = "instruction",
    feature = "system-interface",
    feature = "pubkey",
    feature = "serde"
))]
#[test]
fn instruction_serde_passthrough() {
    use solana_awesome::instruction::Instruction;
    use solana_awesome::pubkey::Pubkey;
    use solana_awesome::system_interface::instruction::transfer;

    let ix = transfer(&Pubkey::new_unique(), &Pubkey::new_unique(), 1);
    let json = serde_json::to_string(&ix).unwrap();
    let back: Instruction = serde_json::from_str(&json).unwrap();
    assert_eq!(ix, back);
}

#[cfg(all(feature = "transaction", feature = "message", feature = "serde"))]
#[test]
fn transaction_serde_passthrough() {
    use solana_awesome::transaction::Transaction;

    let tx = Transaction::default();
    let json = serde_json::to_string(&tx).unwrap();
    let back: Transaction = serde_json::from_str(&json).unwrap();
    assert_eq!(tx, back);
}

#[cfg(all(feature = "instruction", feature = "pubkey", feature = "borsh"))]
#[test]
fn instruction_borsh_passthrough() {
    use solana_awesome::instruction::Instruction;
    use solana_awesome::pubkey::Pubkey;

    // `Instruction::new_with_borsh` only exists with the borsh feature.
    let ix = Instruction::new_with_borsh(Pubkey::new_unique(), &42u64, vec![]);
    assert_eq!(ix.data, borsh::to_vec(&42u64).unwrap());
}

#[cfg(all(feature = "compute-budget-interface", feature = "borsh"))]
#[test]
fn compute_budget_borsh_passthrough() {
    use solana_awesome::compute_budget_interface::ComputeBudgetInstruction;

    let ix = ComputeBudgetInstruction::SetComputeUnitLimit(200_000);
    let bytes = borsh::to_vec(&ix).unwrap();
    let back: ComputeBudgetInstruction = borsh::from_slice(&bytes).unwrap();
    assert_eq!(ix, back);
}

#[cfg(all(feature = "message", feature = "hash", feature = "blake3"))]
#[test]
fn message_blake3_passthrough() {
    use solana_awesome::message::Message;

    // `Message::hash_raw_message` only exists with the blake3 feature.
    let first = Message::hash_raw_message(b"pass-through");
    let second = Message::hash_raw_message(b"pass-through");
    let other = Message::hash_raw_message(b"different");
    assert_eq!(first, second);
    assert_ne!(first, other);
}

#[cfg(all(feature = "pubkey", feature = "keypair", feature = "signer", feature = "curve25519"))]
#[test]
fn pubkey_curve25519_passthrough() {
    use solana_awesome::keypair::Keypair;
    use solana_awesome::signer::Signer;

    // `Pubkey::is_on_curve` only exists with the curve25519 feature; a real
    // ed25519 public key is always on the curve.
    assert!(Keypair::new().pubkey().is_on_curve());
}

#[cfg(all(feature = "pubkey", feature = "sha2", feature = "curve25519"))]
#[test]
fn pubkey_sha2_passthrough() {
    use solana_awesome::pubkey::Pubkey;

    // PDA derivation needs sha2 (and curve25519 for the off-curve check).
    let program_id = Pubkey::new_unique();
    let (pda, bump) = Pubkey::find_program_address(&[b"pass-through"], &program_id);
    assert!(!pda.is_on_curve());
    assert_eq!(
        pda,
        Pubkey::create_program_address(&[b"pass-through", &[bump]], &program_id).unwrap()
    );
}

#[cfg(all(feature = "hash", feature = "decode"))]
#[test]
fn hash_decode_passthrough() {
    use solana_awesome::hash::Hash;
    use std::str::FromStr;

    // Parsing from base58 only exists with the decode feature.
    let hash = Hash::from_str("11111111111111111111111111111111").unwrap();
    assert_eq!(hash, Hash::default());
}

#[cfg(all(feature = "hash", feature = "copy"))]
#[test]
fn hash_copy_passthrough() {
    use solana_awesome::hash::Hash;

    // Only compiles if the copy feature adds `impl Copy for Hash`:
    // both bindings use the same value without a clone.
    let hash = Hash::new_from_array([9; 32]);
    let a = hash;
    let b = hash;
    assert_eq!(a, b);
}

#[cfg(all(feature = "hash", feature = "bytemuck"))]
#[test]
fn hash_bytemuck_passthrough() {
    use solana_awesome::hash::Hash;

    // `bytemuck::bytes_of` requires the Pod impl from the bytemuck feature.
    let hash = Hash::new_from_array([3; 32]);
    assert_eq!(bytemuck::bytes_of(&hash), &[3; 32]);
}

#[cfg(all(feature = "pubkey", feature = "bytemuck"))]
#[test]
fn pubkey_bytemuck_passthrough() {
    use solana_awesome::pubkey::Pubkey;

    let key = Pubkey::new_from_array([5; 32]);
    assert_eq!(bytemuck::bytes_of(&key), &[5; 32]);
}

#[cfg(all(feature = "transaction", feature = "message", feature = "wincode"))]
#[test]
fn transaction_wincode_passthrough() {
    use solana_awesome::transaction::Transaction;

    // The wincode feature derives SchemaWrite/SchemaRead for Transaction.
    let tx = Transaction::default();
    let bytes = wincode::serialize(&tx).unwrap();
    let back: Transaction = wincode::deserialize(&bytes).unwrap();
    assert_eq!(tx, back);
}

#[cfg(all(
    feature = "keypair",
    feature = "signer",
    feature = "seed-derivable"
))]
#[test]
fn keypair_seed_derivable_passthrough() {
    use solana_awesome::keypair::Keypair;
    use solana_awesome::signer::Signer;

    // `keypair_from_seed` only exists with the seed-derivable feature and
    // must be deterministic.
    let a = solana_awesome::keypair::keypair_from_seed(&[7; 32]).unwrap();
    let b = solana_awesome::keypair::keypair_from_seed(&[7; 32]).unwrap();
    let c = solana_awesome::keypair::keypair_from_seed(&[8; 32]).unwrap();
    let _: &Keypair = &a;
    assert_eq!(a.pubkey(), b.pubkey());
    assert_ne!(a.pubkey(), c.pubkey());
}

// --- On-chain program crates ---
// Cheap, runtime-free assertions only: anything needing a validator, a real
// account, or an SBF syscall has no meaningful off-chain test.

#[cfg(feature = "clock")]
#[test]
fn clock_constants() {
    use solana_awesome::clock::{Clock, DEFAULT_MS_PER_SLOT, DEFAULT_TICKS_PER_SLOT};

    assert_eq!(DEFAULT_MS_PER_SLOT, 400);
    assert_eq!(DEFAULT_TICKS_PER_SLOT, 64);
    assert_eq!(Clock::default().slot, 0);
}

#[cfg(feature = "rent")]
#[test]
fn rent_minimum_balance() {
    use solana_awesome::rent::Rent;

    let rent = Rent::default();
    // Larger accounts are never cheaper to keep rent-exempt.
    assert!(rent.minimum_balance(0) > 0);
    assert!(rent.minimum_balance(165) > rent.minimum_balance(0));
}

#[cfg(feature = "epoch-schedule")]
#[test]
fn epoch_schedule_epochs() {
    use solana_awesome::epoch_schedule::{EpochSchedule, MINIMUM_SLOTS_PER_EPOCH};

    let schedule = EpochSchedule::custom(MINIMUM_SLOTS_PER_EPOCH, 0, false);
    assert_eq!(schedule.get_epoch(0), 0);
    assert_eq!(schedule.get_epoch(MINIMUM_SLOTS_PER_EPOCH), 1);
}

#[cfg(feature = "epoch-rewards")]
#[test]
fn epoch_rewards_default() {
    use solana_awesome::epoch_rewards::EpochRewards;

    let rewards = EpochRewards::default();
    assert_eq!(rewards.num_partitions, 0);
    assert_eq!(rewards.total_rewards, rewards.distributed_rewards);
}

#[cfg(feature = "last-restart-slot")]
#[test]
fn last_restart_slot_default() {
    use solana_awesome::last_restart_slot::LastRestartSlot;

    assert_eq!(LastRestartSlot::default().last_restart_slot, 0);
}

#[cfg(feature = "fee-calculator")]
#[test]
fn fee_calculator_new() {
    use solana_awesome::fee_calculator::FeeCalculator;

    assert_eq!(FeeCalculator::new(42).lamports_per_signature, 42);
}

#[cfg(all(feature = "slot-hashes", feature = "hash"))]
#[test]
fn slot_hashes_lookup() {
    use solana_awesome::{hash::Hash, slot_hashes::SlotHashes};

    let mut hashes = SlotHashes::default();
    let hash = Hash::new_from_array([1; 32]);
    hashes.add(7, hash);
    assert_eq!(hashes.get(&7), Some(&hash));
    assert_eq!(hashes.get(&8), None);
}

#[cfg(feature = "slot-history")]
#[test]
fn slot_history_check() {
    use solana_awesome::slot_history::{Check, SlotHistory};

    let mut history = SlotHistory::default();
    history.add(1);
    assert_eq!(history.check(1), Check::Found);
}

#[cfg(feature = "program-option")]
#[test]
fn program_option_coption() {
    use solana_awesome::program_option::COption;

    let some: COption<u64> = COption::Some(5);
    assert!(some.is_some());
    assert_eq!(some.unwrap_or(0), 5);
    assert_eq!(COption::<u64>::None.unwrap_or(9), 9);
}

#[cfg(feature = "program-error")]
#[test]
fn program_error_custom() {
    use solana_awesome::program_error::ProgramError;

    let err = ProgramError::Custom(7);
    assert_eq!(ProgramError::from(u64::from(err)), ProgramError::Custom(7));
}

#[cfg(feature = "instruction-error")]
#[test]
fn instruction_error_custom() {
    use solana_awesome::instruction_error::InstructionError;

    assert_eq!(InstructionError::Custom(3), InstructionError::Custom(3));
    assert_ne!(InstructionError::Custom(3), InstructionError::InvalidArgument);
}

#[cfg(feature = "short-vec")]
#[test]
fn short_vec_shortu16_len() {
    use solana_awesome::short_vec::decode_shortu16_len;

    // 0x7f fits in one byte; 0x80 0x01 is the two-byte encoding of 128.
    assert_eq!(decode_shortu16_len(&[0x7f]), Ok((0x7f, 1)));
    assert_eq!(decode_shortu16_len(&[0x80, 0x01]), Ok((128, 2)));
}

#[cfg(all(feature = "serialize-utils", feature = "pubkey"))]
#[test]
fn serialize_utils_roundtrip() {
    use solana_awesome::{
        pubkey::Pubkey,
        serialize_utils::{append_slice, append_u16, read_pubkey, read_u16},
    };

    let key = Pubkey::new_unique();
    let mut buf = vec![];
    append_u16(&mut buf, 0xbeef);
    append_slice(&mut buf, key.as_ref());

    let mut cursor = 0;
    assert_eq!(read_u16(&mut cursor, &buf), Ok(0xbeef));
    assert_eq!(read_pubkey(&mut cursor, &buf), Ok(key));
}

#[cfg(feature = "borsh-utils")]
#[test]
fn borsh_utils_try_from_slice_unchecked() {
    use solana_awesome::borsh_utils::v1::try_from_slice_unchecked;

    // Unlike `borsh::from_slice`, trailing bytes are tolerated.
    let mut bytes = borsh::to_vec(&42u64).unwrap();
    bytes.push(0xff);
    assert_eq!(try_from_slice_unchecked::<u64>(&bytes).unwrap(), 42);
}

#[cfg(feature = "big-mod-exp")]
#[test]
fn big_mod_exp_computes() {
    use solana_awesome::big_mod_exp::big_mod_exp;

    // 2^10 mod 1000 == 24, big-endian in a modulus-width buffer.
    assert_eq!(big_mod_exp(&[2], &[10], &[0x03, 0xe8]), vec![0x00, 0x18]);
}

#[cfg(all(feature = "sdk-ids", feature = "system-interface"))]
#[test]
fn sdk_ids_agree_with_interfaces() {
    assert_eq!(
        solana_awesome::sdk_ids::system_program::ID,
        solana_awesome::system_interface::program::ID
    );
}

// --- Program interfaces ---

#[cfg(all(feature = "stake-interface", feature = "sdk-ids"))]
#[test]
fn stake_interface_program_id() {
    assert_eq!(
        solana_awesome::stake_interface::program::ID,
        solana_awesome::sdk_ids::stake::ID
    );
}

#[cfg(all(feature = "vote-interface", feature = "sdk-ids"))]
#[test]
fn vote_interface_program_id() {
    assert_eq!(
        solana_awesome::vote_interface::program::ID,
        solana_awesome::sdk_ids::vote::ID
    );
}

#[cfg(all(feature = "address-lookup-table-interface", feature = "sdk-ids"))]
#[test]
fn address_lookup_table_interface_program_id() {
    assert_eq!(
        solana_awesome::address_lookup_table_interface::program::ID,
        solana_awesome::sdk_ids::address_lookup_table::ID
    );
}

#[cfg(all(feature = "loader-v3-interface", feature = "pubkey"))]
#[test]
fn loader_v3_program_data_address() {
    use solana_awesome::{loader_v3_interface::get_program_data_address, pubkey::Pubkey};

    let program = Pubkey::new_unique();
    // Deterministic PDA, and never the program address itself.
    assert_eq!(
        get_program_data_address(&program),
        get_program_data_address(&program)
    );
    assert_ne!(get_program_data_address(&program), program);
}

#[cfg(all(feature = "feature-gate-interface", feature = "sdk-ids"))]
#[test]
fn feature_gate_interface_program_id() {
    // The instruction builders here are behind the un-forwarded `bincode`
    // feature, so this checks the id and state size instead.
    use solana_awesome::feature_gate_interface::Feature;

    assert_eq!(
        solana_awesome::feature_gate_interface::ID,
        solana_awesome::sdk_ids::feature::ID
    );
    assert_eq!(Feature::size_of(), 9);
}

// --- Pass-through features on the on-chain crates ---

#[cfg(all(feature = "sha256-hasher", feature = "sha2", feature = "hash"))]
#[test]
fn sha256_hasher_sha2_passthrough() {
    use solana_awesome::sha256_hasher::{hash, hashv};

    // The off-chain implementation only exists with the sha2 feature.
    assert_eq!(hash(b"pass-through"), hashv(&[b"pass-", b"through"]));
    assert_ne!(hash(b"pass-through"), hash(b"different"));
}

#[cfg(all(feature = "keccak-hasher", feature = "sha3"))]
#[test]
fn keccak_hasher_sha3_passthrough() {
    use solana_awesome::keccak_hasher::hashv;

    assert_eq!(hashv(&[b"pass-through"]), hashv(&[b"pass-", b"through"]));
    assert_ne!(hashv(&[b"pass-through"]), hashv(&[b"different"]));
}

#[cfg(all(feature = "blake3-hasher", feature = "blake3"))]
#[test]
fn blake3_hasher_passthrough() {
    use solana_awesome::blake3_hasher::{hash, hashv};

    assert_eq!(hash(b"pass-through"), hashv(&[b"pass-", b"through"]));
    assert_ne!(hash(b"pass-through"), hash(b"different"));
}

#[cfg(all(feature = "clock", feature = "serde"))]
#[test]
fn clock_serde_passthrough() {
    use solana_awesome::clock::Clock;

    let clock = Clock::default();
    let json = serde_json::to_string(&clock).unwrap();
    let back: Clock = serde_json::from_str(&json).unwrap();
    assert_eq!(clock, back);
}

#[cfg(all(feature = "rent", feature = "serde"))]
#[test]
fn rent_serde_passthrough() {
    use solana_awesome::rent::Rent;

    let rent = Rent::default();
    let json = serde_json::to_string(&rent).unwrap();
    let back: Rent = serde_json::from_str(&json).unwrap();
    assert_eq!(rent, back);
}

#[cfg(all(feature = "sysvar", feature = "bytemuck", feature = "clock"))]
#[test]
fn sysvar_bytemuck_passthrough() {
    use solana_awesome::{clock::Clock, sysvar::SysvarSerialize};

    // `SysvarSerialize::size_of` is the serialized sysvar account length.
    assert!(<Clock as SysvarSerialize>::size_of() > 0);
}

#[cfg(all(feature = "stake-interface", feature = "borsh"))]
#[test]
fn stake_interface_borsh_passthrough() {
    use solana_awesome::stake_interface::state::Lockup;

    let lockup = Lockup::default();
    let bytes = borsh::to_vec(&lockup).unwrap();
    let back: Lockup = borsh::from_slice(&bytes).unwrap();
    assert_eq!(lockup, back);
}

#[cfg(all(feature = "program-option", feature = "nullable"))]
#[test]
fn program_option_nullable_passthrough() {
    use solana_awesome::program_option::COption;

    // The nullable feature adds the bytemuck-friendly `Nullable` impls.
    let some: COption<u64> = COption::Some(1);
    assert!(some.is_some());
}

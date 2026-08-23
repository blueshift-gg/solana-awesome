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

# solana-awesome

A single crate that collects the Solana ecosystem crates and exposes each one
behind a feature flag. Instead of managing a dozen `solana-*` dependencies
(and their version compatibility) in every project, depend on this one crate
and turn on only what you need.

## Usage

```toml
[dependencies]
solana-awesome = { version = "0.1", features = ["pubkey", "keypair", "signer", "rpc-client"] }
```

```rust
use solana_awesome::keypair::Keypair;
use solana_awesome::pubkey::Pubkey;
use solana_awesome::rpc_client::rpc_client::RpcClient;
use solana_awesome::signer::Signer;

let client = RpcClient::new("https://api.devnet.solana.com".to_string());
let payer = Keypair::new();
let balance = client.get_balance(&payer.pubkey())?;
```

Every crate is re-exported as a module named after it, minus the `solana-`
prefix: `solana_awesome::pubkey` is [`solana-pubkey`], `solana_awesome::rpc_client`
is [`solana-rpc-client`], and so on.

## Feature flags

No features are enabled by default.

### Core SDK

| Feature | Crate |
|---|---|
| `pubkey` | `solana-pubkey` |
| `keypair` | `solana-keypair` |
| `signer` | `solana-signer` |
| `signature` | `solana-signature` |
| `instruction` | `solana-instruction` |
| `message` | `solana-message` |
| `transaction` | `solana-transaction` |
| `hash` | `solana-hash` |
| `account` | `solana-account` |
| `commitment-config` | `solana-commitment-config` |
| `native-token` | `solana-native-token` |
| `system-interface` | `solana-system-interface` (with `wincode`, so instruction builders work) |
| `compute-budget-interface` | `solana-compute-budget-interface` |

### Umbrella crates

| Feature | Crate |
|---|---|
| `program` | `solana-program` |

### Clients

| Feature | Crate |
|---|---|
| `client` | `solana-client` |
| `rpc-client` | `solana-rpc-client` |
| `rpc-client-api` | `solana-rpc-client-api` |
| `tpu-client` | `solana-tpu-client` |
| `quic-client` | `solana-quic-client` |
| `udp-client` | `solana-udp-client` |
| `connection-cache` | `solana-connection-cache` |
| `pubsub-client` | `solana-pubsub-client` |
| `transaction-status` | `solana-transaction-status` |
| `account-decoder` | `solana-account-decoder` |

### Groups

| Feature | Enables |
|---|---|
| `core` | all Core SDK features above |
| `clients` | all Client features above |
| `full` | `core` + `clients` + `program` |

### Pass-through features

These forward a feature of the same name to the underlying crates. They are
*weak*: each one applies only to the crates you have already enabled and never
pulls a crate in by itself. So `features = ["core", "serde"]` enables serde on
every core crate, while `serde` alone enables nothing.

```toml
[dependencies]
solana-awesome = { version = "0.1", features = ["pubkey", "transaction", "serde"] }
```

| Feature | Forwards to |
|---|---|
| `serde` | `account`, `commitment-config`, `compute-budget-interface`, `hash`, `instruction`, `message`, `pubkey`, `signature`, `system-interface`, `transaction` |
| `borsh` | `compute-budget-interface`, `hash`, `instruction`, `program`, `pubkey` |
| `wincode` | `account`, `hash`, `instruction`, `message`, `pubkey`, `signature`, `system-interface`, `transaction` |
| `bytemuck` | `hash`, `pubkey`, `signature` |
| `rand` | `pubkey`, `signature` |
| `blake3` | `message`, `transaction` |
| `verify` | `signature`, `transaction` |
| `curve25519` | `pubkey` |
| `sha2` | `pubkey` |
| `seed-derivable` | `keypair` |
| `atomic` | `hash` |
| `decode` | `hash` |
| `sanitize` | `hash` |
| `copy` | `hash` |
| `syscalls` | `instruction` |
| `agave-unstable-api` | all client crates |

Not forwarded: `bincode` (legacy — upstream is migrating serialization to
`wincode`, which is forwarded instead), `frozen-abi` and
`dev-context-only-utils` (internal Agave tooling), `std`/`alloc` (on by
default upstream), and `spinner` (on by default in `solana-rpc-client` /
`solana-tpu-client`). Features that only exist in later minors than our
minimum requirements (`solana-signature`'s `batch-verify` / `parallel`, added
in 3.4; `solana-hash`'s `rand`, added in 4.5) are also not forwarded — they
could break a build that unifies down to an earlier minor. If you need any of
these, depend on the underlying crate directly with the feature enabled;
cargo will unify it with the copy this crate uses.

## Version pinning notes

- Core crates use loose requirements (`"3"` / `"4"`) so cargo can unify them
  with the exact minor versions the Agave client crates pin. Where a minor is
  given (`"4.1"` / `"3.2"`), it is the first release of that major carrying
  the `wincode` feature — upstream is migrating serialization from bincode to
  wincode, and the `wincode` pass-through must exist in every version the
  requirement can resolve to.
- Client crates are pinned to `"4.2"` so the whole group resolves together:
  when core and client versions drift apart, the tree splits across two
  incompatible `wincode` 0.x majors, which fails to compile. Keep the client
  group on the same minor when bumping, and check that
  `cargo tree -i wincode` shows a single version under the solana crates
  (the `wincode` dev-dependency must match it too).

## Adding a new crate

1. `cargo add --optional solana-<name>` (use a major-only version, e.g. `"3"`).
2. Replace the implicit `solana-<name>` feature in `[features]` with a short
   one: `<name> = ["dep:solana-<name>"]`.
3. Add the re-export to `src/lib.rs`:
   `#[cfg(feature = "<name>")] pub use solana_<name> as <name>;`
4. Add it to the `core`/`clients` group if it belongs there, and to the table
   above.
5. Forward its useful features in the pass-through section of `[features]`
   using the weak syntax (`"solana-<name>?/<feature>"`), and update the
   pass-through table above. Only forward a feature if every version the
   requirement can resolve to has it — with major-only requirements that
   means it must exist in the earliest minor of that major.
6. `cargo check --features full` and `cargo test --all-features` (the latter
   also exercises every pass-through feature).

## Keeping it current

`scripts/check_crates.py` reports (read-only) what crates.io has that this
crate doesn't: version bumps for existing dependencies, and new `solana-*`
crates published by the trusted owners of `solana-pubkey` (so name-squatters
never appear). Crates rejected for inclusion are recorded in
`scripts/crates-denylist.txt` so they aren't re-surfaced.

The `/update-crates` skill (`.claude/skills/update-crates/`) runs the script,
curates the candidates, wires accepted crates through `Cargo.toml`,
`src/lib.rs`, this README, and the smoke tests, then validates with
`cargo test --features full`.

## Release

Releases are manual:

1. Make sure the version in `Cargo.toml` was bumped (patch for dependency
   requirement updates, minor for new features).
2. `cargo test --all-features`
3. `cargo publish --dry-run`
4. `cargo publish`
5. `git tag v<version> && git push --tags`

[`solana-pubkey`]: https://crates.io/crates/solana-pubkey
[`solana-rpc-client`]: https://crates.io/crates/solana-rpc-client

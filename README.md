# solana-awesome

A single crate that collects the Solana ecosystem crates and exposes each one
behind a feature flag. Instead of managing a dozen `solana-*` dependencies
(and their version compatibility) in every project, depend on this one crate
and turn on only what you need.

## Usage

```toml
[dependencies]
solana-awesome = { version = "0.2", features = ["pubkey", "keypair", "signer", "rpc-client"] }
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

### On-chain program crates

The program-side primitives, migrated from `solana-onchain`. All of them are
`no_std`-friendly.

| Feature | Crate |
|---|---|
| `account-info` | `solana-account-info` |
| `big-mod-exp` | `solana-big-mod-exp` |
| `blake3-hasher` | `solana-blake3-hasher` |
| `borsh-utils` | `solana-borsh` (named `borsh-utils` so it doesn't clash with the `borsh` pass-through; the module is `borsh_utils`) |
| `clock` | `solana-clock` (with `sysvar`) |
| `cpi` | `solana-cpi` |
| `define-syscall` | `solana-define-syscall` |
| `epoch-rewards` | `solana-epoch-rewards` (with `sysvar`) |
| `epoch-schedule` | `solana-epoch-schedule` (with `sysvar`) |
| `epoch-stake` | `solana-epoch-stake` |
| `fee-calculator` | `solana-fee-calculator` |
| `instruction-error` | `solana-instruction-error` |
| `instructions-sysvar` | `solana-instructions-sysvar` |
| `keccak-hasher` | `solana-keccak-hasher` |
| `last-restart-slot` | `solana-last-restart-slot` (with `sysvar`) |
| `msg` | `solana-msg` |
| `program-entrypoint` | `solana-program-entrypoint` |
| `program-error` | `solana-program-error` |
| `program-memory` | `solana-program-memory` |
| `program-option` | `solana-program-option` |
| `program-pack` | `solana-program-pack` |
| `rent` | `solana-rent` (with `sysvar`) |
| `sdk-ids` | `solana-sdk-ids` |
| `secp256k1-recover` | `solana-secp256k1-recover` |
| `serde-varint` | `solana-serde-varint` |
| `serialize-utils` | `solana-serialize-utils` |
| `sha256-hasher` | `solana-sha256-hasher` |
| `short-vec` | `solana-short-vec` |
| `slot-hashes` | `solana-slot-hashes` (with `sysvar`) |
| `slot-history` | `solana-slot-history` (with `sysvar`) |
| `stable-layout` | `solana-stable-layout` |
| `sysvar` | `solana-sysvar` |
| `sysvar-id` | `solana-sysvar-id` |

The sysvar-data crates above enable their upstream `sysvar` feature (the
`SysvarId` / `Sysvar` impls) rather than exposing it as a pass-through: it is
what makes them usable from a program, and the `sysvar` feature name here
already belongs to `solana-sysvar`.

### Program interfaces

| Feature | Crate |
|---|---|
| `address-lookup-table-interface` | `solana-address-lookup-table-interface` |
| `compute-budget-interface` | `solana-compute-budget-interface` |
| `feature-gate-interface` | `solana-feature-gate-interface` |
| `loader-v3-interface` | `solana-loader-v3-interface` |
| `stake-interface` | `solana-stake-interface` |
| `system-interface` | `solana-system-interface` |
| `vote-interface` | `solana-vote-interface` |

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
| `onchain` | all On-chain program crate features above |
| `interfaces` | all Program interface features above |
| `full` | `core` + `clients` + `onchain` + `interfaces` + `program` |

### Pass-through features

These forward a feature of the same name to the underlying crates. They are
*weak*: each one applies only to the crates you have already enabled and never
pulls a crate in by itself. So `features = ["core", "serde"]` enables serde on
every core crate, while `serde` alone enables nothing.

```toml
[dependencies]
solana-awesome = { version = "0.2", features = ["pubkey", "transaction", "serde"] }
```

| Feature | Forwards to |
|---|---|
| `serde` | `account`, `address-lookup-table-interface`, `clock`, `commitment-config`, `compute-budget-interface`, `epoch-rewards`, `epoch-schedule`, `fee-calculator`, `feature-gate-interface`, `hash`, `instruction`, `instruction-error`, `last-restart-slot`, `loader-v3-interface`, `message`, `program-error`, `pubkey`, `rent`, `short-vec`, `signature`, `slot-hashes`, `slot-history`, `stake-interface`, `system-interface`, `sysvar`, `transaction`, `vote-interface` |
| `borsh` | `compute-budget-interface`, `hash`, `instruction`, `program`, `program-error`, `pubkey`, `secp256k1-recover`, `stake-interface` |
| `wincode` | `account`, `address-lookup-table-interface`, `clock`, `epoch-rewards`, `epoch-schedule`, `fee-calculator`, `hash`, `instruction`, `last-restart-slot`, `message`, `pubkey`, `rent`, `signature`, `slot-hashes`, `slot-history`, `stake-interface`, `system-interface`, `sysvar`, `transaction` |
| `bytemuck` | `address-lookup-table-interface`, `hash`, `pubkey`, `signature`, `sysvar` |
| `rand` | `pubkey`, `signature` |
| `blake3` | `blake3-hasher`, `message`, `transaction` |
| `verify` | `signature`, `transaction` |
| `curve25519` | `pubkey` |
| `sha2` | `pubkey`, `sha256-hasher` |
| `sha3` | `keccak-hasher` |
| `num-traits` | `instruction-error` |
| `nullable` | `program-option` |
| `unstable-static-syscalls` | `define-syscall` |
| `seed-derivable` | `keypair` |
| `atomic` | `hash` |
| `decode` | `hash` |
| `sanitize` | `hash` |
| `copy` | `hash` |
| `syscalls` | `instruction` |
| `agave-unstable-api` | all client crates |

The hashers are the ones to watch: `sha256-hasher`, `keccak-hasher` and
`blake3-hasher` compile without `sha2` / `sha3` / `blake3`, but off-chain
their `hash`/`hashv` functions are only implemented when the matching
pass-through is on (on-chain they use a syscall either way). Enable the pair
if you hash outside a program.

Not forwarded: `bincode` (legacy — upstream is migrating serialization to
`wincode`, which is forwarded instead), `frozen-abi` and
`dev-context-only-utils` (internal Agave tooling), `std`/`alloc` (on by
default upstream), and `spinner` (on by default in `solana-rpc-client` /
`solana-tpu-client`). Features that only exist in later minors than our
minimum requirements (`solana-signature`'s `batch-verify` / `parallel`, added
in 3.4; `solana-hash`'s `rand`, added in 4.5; `wincode` on `short-vec`,
`feature-gate-interface`, `loader-v3-interface` and `vote-interface`) are
also not forwarded — they could break a build that unifies down to an
earlier minor. `codama` (IDL codegen for `solana-stake-interface`) is
tooling, not a library concern, and `solana-instruction-error`'s `wincode`
is held out on purpose: that crate moved to wincode 0.6 ahead of the rest of
the tree, so forwarding it would resolve two incompatible wincode 0.x majors
at once. If you need any of these, depend on the underlying crate directly
with the feature enabled; cargo will unify it with the copy this crate uses.

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
- The on-chain crates are *not* on their newest published minors. The Agave
  client crates at `4.2` pin exact minors of the same primitives
  (`solana-clock`, `solana-rent`, `solana-sysvar`, the `*-interface` crates,
  ...), so each requirement here is the highest published floor that still
  unifies with them — enabling `clients` and `onchain` together has to yield
  one copy of each crate, or `solana_awesome::clock` and
  `solana_awesome::sysvar` would disagree on the same types. These catch up
  on their own once Agave does; `scripts/bump_requirements.py` raises them
  in the daily job and reports the ones still held back.

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
6. `scripts/test-features.sh ci` — or `scripts/test-features.sh all` to also
   check the new crate builds on its own, with no other feature enabled.

## Testing

`scripts/test-features.sh` builds and tests the crate across its feature
matrix. Since this crate is only feature-gated re-exports, what breaks is a
feature *combination* rather than a function, and each suite covers one of
those failure modes:

| Suite | What it runs |
|---|---|
| `ci` (default) | the gate `daily-deps.yml` runs: `--all-features`, then `full` alone, then no features at all |
| `groups` | every group feature (`core`, `clients`, `onchain`, ...) on its own |
| `leaves` | every single-crate feature on its own — catches a `#[cfg]` block reaching for a module some other feature happens to enable |
| `passthrough` | `full` plus each pass-through (`serde`, `borsh`, `wincode`, ...) one at a time, since a weak forward means nothing without crates enabled |
| `wincode` | one `wincode` major across the whole tree (see the version pinning notes above) |
| `all` | all of the above |

```sh
scripts/test-features.sh              # the CI gate, seconds when warm
scripts/test-features.sh all          # the whole matrix, a few minutes cold
scripts/test-features.sh leaves -f    # stop at the first feature that fails alone
scripts/test-features.sh groups -t    # cargo test, not just cargo check
```

The feature lists come from `cargo metadata`, so a crate added to `Cargo.toml`
is covered the moment it lands — there is no list in the script to keep in
sync. `--list` prints the classification, and failures are reported together
at the end with a log path each.

## Keeping it current

`scripts/check_crates.py` reports (read-only) new `solana-*` crates this
crate doesn't re-export yet, published by the trusted owners of
`solana-pubkey` (so name-squatters never appear). Crates rejected for
inclusion are recorded in `scripts/crates-denylist.txt` so they aren't
re-surfaced. Version bumps for existing dependencies are handled by
`scripts/bump_requirements.py` instead (see below).

The `/update-crates` skill (`.claude/skills/update-crates/`) runs the script,
curates the candidates, wires accepted crates through `Cargo.toml`,
`src/lib.rs`, this README, and the smoke tests, then validates with
`cargo test --features full`.

A daily GitHub Actions job (`.github/workflows/daily-deps.yml`, also runnable
manually from the Actions tab) keeps the crate caught up with upstream:
`scripts/bump_requirements.py` raises each `solana-*` requirement to the
highest published `major.minor` that still resolves against the rest of the
tree (bumps blocked by other crates' internal pins are held back and listed),
bumps the package version (patch, or minor when a dependency changed major),
validates the full feature matrix (`--all-features`, `full`-only, and
no-features builds, a single-wincode tree check, and `cargo publish
--dry-run`), and opens a PR with the diff. Merging it and running the release
steps below publishes a solana-awesome that tracks the latest solana crates.
If the bump breaks the build or tests, the job files/updates an issue
instead. New crates still go through `/update-crates`; the job prints the
`check_crates.py` report in its run summary as a reminder.

## Release

Releases are manual:

1. Make sure the version in `Cargo.toml` was bumped (patch for dependency
   requirement updates, minor for new features or a dependency major bump —
   the daily catch-up PRs already include the right bump).
2. `scripts/test-features.sh all`
3. `cargo publish --dry-run`
4. `cargo publish`
5. `git tag v<version> && git push --tags`

[`solana-pubkey`]: https://crates.io/crates/solana-pubkey
[`solana-rpc-client`]: https://crates.io/crates/solana-rpc-client

---
name: update-crates
description: Check crates.io for new solana-* crates, curate and wire them into solana-awesome, validate, and prep (but not perform) a release. Version bumps of existing deps are automated by the daily-deps workflow.
---

# Update solana-awesome from crates.io

You are maintaining an umbrella crate that re-exports Solana ecosystem crates
behind feature flags. Version requirement bumps of existing dependencies are
automated (`scripts/bump_requirements.py`, run daily by
`.github/workflows/daily-deps.yml`, which opens catch-up PRs) — this skill's
job is the part that needs judgment: curating and wiring in **new** crates.

## 1. Gather the report

Run `python3 scripts/check_crates.py` from the repo root (read-only; takes a
few minutes because it rate-limits crates.io requests). It reports new
trusted-owner candidates, already filtered against
`scripts/crates-denylist.txt` and staleness/placeholder heuristics.

## 2. Requirement philosophy (for the crates you add)

- Every requirement is **major.minor**, tracking the highest published floor
  that still resolves against the rest of the tree — the daily job keeps
  these current, so for a new crate just start at the latest `major.minor`
  that resolves.
- Client crates (the `solana-client`/`solana-rpc-client` family) must move
  together on the same minor, and core and client crates must agree on one
  `wincode` 0.x (see README version pinning notes).

## 3. Curate new candidates

Include a candidate only if downstream app or program developers would use it
as a library: SDK primitives and types, `*-interface` crates, clients,
crypto/hashing utilities, dev-test frameworks like `solana-program-test`.

Reject validator/node internals, on-chain program runtime implementations
(prefer their `*-interface` crate), CLIs/binaries, internal-use-only crates,
wasm/JS bindings, and deprecated or 0.0.x placeholder crates. When in doubt,
check the crate's docs.rs page and reverse dependencies.

**Every rejection must be recorded**: append the crate name to
`scripts/crates-denylist.txt` under the fitting section with a `# reason`
comment when the name alone isn't self-explanatory. This is what keeps the
next run's report short.

## 4. Wire in each accepted crate

For a crate `solana-foo-bar`:
1. `Cargo.toml` dependencies: `solana-foo-bar = { version = "N.M", optional = true }`
   in the matching section. Keep the section sorted alphabetically.
2. `Cargo.toml` features: `foo-bar = ["dep:solana-foo-bar"]`, and add
   `foo-bar` to the right group feature (`core`, `clients`, `onchain`,
   `interfaces`, or a new group if a genuinely new category emerges —
   remember `full` must cover all groups). If the crate name collides with a
   pass-through feature, name the feature something else and say so in the
   README (see `borsh-utils` for `solana-borsh`).
3. Forward its useful features in the pass-through section of `[features]`
   using the weak syntax (`"solana-foo-bar?/<feature>"`), following the
   README's pass-through rules (only features present in every version the
   requirement can resolve to; no `bincode` — the ecosystem is migrating to
   `wincode`).
4. `src/lib.rs`: `#[cfg(feature = "foo-bar")] pub use solana_foo_bar as foo_bar;`
   under the matching section comment.
5. `README.md`: add a row to the matching feature table (and the pass-through
   table if features were forwarded).
6. `tests/smoke.rs`: add a feature-gated test when there's an obvious cheap
   assertion (construct a type, check a constant or program ID). Skip if a
   meaningful test would need a network or validator.

## 5. Validate

```
cargo test --all-features
cargo test --features full
cargo check --no-default-features
cargo doc --features full --no-deps
```

Also spot-check that a single lone feature compiles, e.g.
`cargo check --no-default-features --features foo-bar` for one new crate, and
that `cargo tree --all-features | grep wincode` shows a single wincode
version.

A forward can split the tree even when the crate itself resolves fine: a
crate that has moved ahead to a newer `wincode` 0.x only pulls it in once
the `wincode` feature is on. After adding forwards, re-check the wincode
count and compare `cargo metadata --all-features` duplicate counts against
the previous manifest — they should not grow.

## 6. Version and summarize

Bump `package.version` in `Cargo.toml`: **patch** if only dependency
requirements changed, **minor** if features were added. Summarize what was
bumped, added, and denylisted.

**Do not publish or push.** Releases are tag-triggered — point the user at the
Release section in `README.md`. Pushing an annotated tag that matches
`package.version` (`git tag -a v<version> -m "Release v<version>" &&
git push origin v<version>`) runs the validation and publish workflow.

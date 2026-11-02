# Contributing

Thanks for helping make Soroban storage safer. Issues labelled
`good first issue` or `help wanted` are scoped so they can be picked up
independently. Comment on an issue to claim it before starting.

## Setup

1. Install Rust (stable, 1.91 or newer) and the `wasm32v1-none` target:
   `rustup target add wasm32v1-none`.
2. For end-to-end tests, also install Docker and
   [stellar-cli](https://github.com/stellar/stellar-cli) v25.2 or newer.

## Before opening a pull request

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/e2e-local.sh        # if you touched TTL or storage behaviour
```

## Guidelines

- **Every behaviour change needs a test.** TTL behaviour should be asserted
  with exact values (`key.ttl(&env)`), not just "it didn't panic".
- **If you change TTL rules**, update the model in `src/tests/model.rs` to
  match. The property tests compare the library against it.
- **If you change anything on the write or read path**, rerun
  `cargo test -p soroban-storage costs -- --nocapture` and update the fee
  table in the README if the numbers moved.
- **New compile-time checks** get a case in `tests/ui/`. Generate the
  expected output with
  `TRYBUILD=overwrite cargo test -p soroban-storage --test compile_fail`
  and review it.
- **Keep the on-chain key layout stable.** Keys must keep encoding like
  `#[contracttype]` enum variants (see `keys_match_contracttype_enum_encoding`).
  Changing it would make deployed contracts lose access to their data.
- The library is `#![no_std]` and must keep building for `wasm32v1-none`.
- Public items need doc comments (`#![deny(missing_docs)]`).
- Add a line to `CHANGELOG.md` under "Unreleased".

## Error codes

`StorageError` reserves contract error codes `1100..1200`. Pick the next
free number for new variants and never renumber existing ones.

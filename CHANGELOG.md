# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0]

### Added

- `TypedKey` trait: storage keys that carry their value type, storage kind
  (`Persistent`, `Temporary`, `Instance`) and `TtlPolicy`.
- `storage_keys!` macro generating typed keys whose ledger encoding matches
  `#[contracttype]` enum variants.
- Typed operations, as free functions and `KeyExt` methods: `get`,
  `try_get`, `require`, `require_or`, `get_or`, `get_or_else`, `has`, `set`,
  `update`, `try_update`, `remove`, `take`, `extend`, plus
  `extend_instance`.
- Automatic TTL extension: new entries get the policy's full `extend_to`;
  reads and overwrites extend only at or below `threshold`.
- Compile-time validation of TTL policies; runtime clamping to the
  network's maximum TTL.
- `StorageError` contract errors (`NotFound = 1100`, `WrongType = 1101`).
- `minutes`, `hours`, `days` ledger-time helpers.
- `testutils` feature with `ttl(&env, &key)` for asserting TTLs in tests.

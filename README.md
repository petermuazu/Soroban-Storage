# Soroban-Storage

Typed storage wrappers and lifecycle macros for Soroban smart contracts.

`soroban-storage` makes every storage key declare **where** it lives
(persistent, temporary or instance), **what** it holds, and **how long** it
should live. It then extends TTLs automatically on access, so entries don't
get archived or deleted by accident.

It is the storage half of the planned `soroban-guard` suite (storage
lifecycle + linter + fuzzer + rent-keeper daemon).

## Why

With the raw SDK, nothing stops you from:

- writing a balance to `temporary()` storage, where it is **permanently
  deleted** when its TTL runs out;
- reading a key with the wrong value type;
- forgetting to call `extend_ttl`, so data gets archived;
- calling `.unwrap()` on a missing entry and getting an opaque failure.

With `soroban-storage`, the kind, value type and TTL policy are part of the
key's type. Reads and writes keep the TTL topped up.

## Quick start

```toml
[dependencies]
soroban-sdk = "29"
soroban-storage = "0.1"

[dev-dependencies]
soroban-storage = { version = "0.1", features = ["testutils"] }
```

```rust
use soroban_sdk::{Address, Env};
use soroban_storage::{days, storage_keys, KeyExt, TtlPolicy};

const INSTANCE_TTL: TtlPolicy = TtlPolicy::new(days(7), days(30));
const BALANCE_TTL: TtlPolicy = TtlPolicy::new(days(30), days(120));
const NONCE_TTL: TtlPolicy = TtlPolicy::new(days(1), days(2));

storage_keys! {
    /// Contract administrator.
    pub instance(INSTANCE_TTL) Admin => Address;
    /// Token balance per holder.
    pub persistent(BALANCE_TTL) Balance(owner: Address) => i128;
    /// Replay protection; fine to lose once expired.
    pub temporary(NONCE_TTL) Nonce(owner: Address, nonce: u64) => ();
}

fn deposit(env: &Env, owner: Address, amount: i128) -> i128 {
    // Writes the new balance and extends its TTL to 120 days.
    Balance(owner).update(env, |old| old.unwrap_or(0) + amount)
}

fn admin(env: &Env) -> Address {
    // Fails with StorageError::NotFound (contract error 1100) if unset.
    Admin.require(env)
}
```

See [`examples/vault`](examples/vault/src/lib.rs) for a full contract.

## How TTLs are managed

A `TtlPolicy { threshold, extend_to }` is attached to every key:

| Operation | TTL effect |
|---|---|
| `set` / `update` / `try_update` creating a new entry | Extended to `extend_to` |
| `set` / `update` / `try_update` on an existing entry | Extended to `extend_to` only if the TTL is at or below `threshold` |
| `get`, `has`, `require`, `try_get`, `get_or`, `extend` | Extended to `extend_to` only if the TTL is at or below `threshold` |
| `remove`, `take` | Entry deleted |

**Guarantee:** as long as an entry is accessed at least once every
`extend_to - threshold` ledgers, its TTL never drops below `threshold`.

- Policies are checked **at compile time**: `threshold > extend_to` or
  `extend_to == 0` fails the build.
- `extend_to` is clamped to the network's current maximum TTL, so a generous
  policy never makes a transaction fail.
- Use `TtlPolicy::new(..).write_only()` to skip read-time extensions on hot
  paths.
- For **instance** keys, extending means extending the whole contract
  instance and code. `extend_instance(&env, &POLICY)` does that explicitly
  for entry points that don't touch any instance key.
- Helpers `minutes(n)`, `hours(n)` and `days(n)` convert time to ledgers
  (5-second ledgers).

### What it costs

Measured with the SDK's mainnet fee model for a persistent `i128` balance
with policy `TtlPolicy::new(days(30), days(60))`. Fees are in stroops, as
total / rent part (`cargo test -p soroban-storage costs -- --nocapture`):

| | create | overwrite | read |
|---|---:|---:|---:|
| raw SDK, no TTL management | 12,267 / 8,072 | 4,196 / 0 | 11 / 0 |
| raw SDK + `extend_ttl` on every write | 1,406,742 / 1,402,542 | 6,745 / 2,544 | 11 / 0 |
| **soroban-storage** | 1,406,743 / 1,402,542 | **4,201 / 0** | 13 / 0 |

Creating an entry costs the same as the hand-written pattern: 60 days of
rent, paid up front. Overwrites and reads of a healthy entry pay no rent and
cost within a few stroops of raw SDK calls.

## Missing, expired and archived entries

| Situation | What you get |
|---|---|
| Key never written | `get` → `None`, `try_get` → `Err(StorageError::NotFound)`, `require` → contract error `1100` |
| Temporary entry whose TTL ran out | Same as never written: the entry is deleted |
| Persistent / instance entry whose TTL ran out | Archived. Since protocol 23 it is restored automatically when it's in the transaction footprint, so the contract sees the value. Otherwise the transaction fails before the contract runs |
| Stored value has a different type | Contract error `1101` (`StorageError::WrongType`) |

To fail with your own error instead: `Admin.require_or(&env, MyError::NotInitialized)`.

## API overview

All functions exist as free functions (`soroban_storage::get(&env, &key)`)
and as methods via `KeyExt` (`key.get(&env)`):

`get`, `try_get`, `require`, `require_or`, `get_or`, `get_or_else`, `has`,
`set`, `update`, `try_update`, `remove`, `take`, `extend`, plus
`extend_instance`. With the `testutils` feature, `ttl(&env, &key)` returns an
entry's remaining TTL for use in tests.

You can also implement `TypedKey` by hand instead of using the macro.

## On-chain layout and migration

`storage_keys!` encodes keys exactly like `#[contracttype]` enum variants:
`Balance(owner)` is stored under the same ledger key as
`DataKey::Balance(owner)`. If your contract already uses a `DataKey` enum,
name the key structs after the old variants and existing data stays readable.
No migration needed.

## Testing

| Suite | What it checks |
|---|---|
| Unit tests (`src/tests/mod.rs`) | Exact TTLs after each operation, temporary expiry, instance TTL, clamping, error codes, key layout |
| Property tests (`src/tests/model.rs`) | Random operation sequences and ledger jumps against a model of the TTL rules; value and exact TTL compared after every step |
| Fee comparison (`src/tests/costs.rs`) | The table above, with assertions that overwrites and reads pay no rent |
| Compile-fail tests (`tests/ui`) | Invalid policies, wrong value types and unknown storage kinds don't compile |
| Example contract (`examples/vault`) | All three storage kinds used together in a contract |
| End-to-end (`scripts/e2e-local.sh`) | Deploys the example to a local Stellar network and checks the TTLs the network records |

```bash
cargo test --workspace
scripts/e2e-local.sh   # needs docker and stellar-cli
```

## Development

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
stellar contract build --package vault-example
```

Layout:

```
crates/soroban-storage/   # the library
examples/vault/           # example contract using all three storage kinds
scripts/e2e-local.sh      # end-to-end test on a local network
```

Minimum supported Rust version: 1.91 (same as `soroban-sdk` 29).

See [CONTRIBUTING.md](CONTRIBUTING.md) to get involved.

## License

Apache-2.0. See [LICENSE](LICENSE).

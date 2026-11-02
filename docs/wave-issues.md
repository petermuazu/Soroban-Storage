# Drafted issues for contributors

Well-scoped, independent tasks for Drips Wave contributors. Each `##`
section is one issue: title, labels, then the body. None should be
pre-assigned.

---

## Add `extend_many` for extending a batch of keys of the same type

Labels: `enhancement`, `good first issue`

Contracts often need to keep a list of entries alive (for example every
holder's balance during a migration). Add
`pub fn extend_many<K: TypedKey>(env: &Env, keys: impl IntoIterator<Item = K>) -> u32`
that extends each existing key using its policy and returns how many were
extended, plus a matching `KeyExt`-style entry point if it fits.

**Done when:** unit tests assert exact TTLs for present and missing keys; README API list updated.

---

## Make an unknown storage kind produce a single compiler error

Labels: `enhancement`, `good first issue`, `macros`

`storage_keys! { forever(P) X => u32; }` currently reports the intended
"unknown storage kind" error **and** a confusing second error
(`(): StorageKind is not satisfied`). See `tests/ui/unknown_kind.stderr`.
Change the macro so only the first error is shown.

**Done when:** `tests/ui/unknown_kind.stderr` contains only the intended error.

---

## Test that `storage_keys!` works with `#[contracttype]` structs, `Vec` and `Map` values

Labels: `testing`, `good first issue`

Values are only tested with primitives and `Address`. Add unit tests for
keys whose value is a `#[contracttype]` struct, an enum, a `Vec<_>` and a
`Map<_, _>`, including `update` and the `WrongType` error path.

**Done when:** tests cover each value kind with set/get/update.

---

## Add an `assert_ttl!` test helper

Labels: `enhancement`, `testing`

Behind the `testutils` feature, add a macro such as
`assert_ttl!(env, contract_id, Balance(addr), BALANCE_TTL.extend_to)` that
wraps `env.as_contract` and prints the key, expected and actual TTL on
failure.

**Done when:** the helper is documented, tested, and used in `examples/vault` tests.

---

## Measure fees with a Wasm contract instead of a native one

Labels: `performance`, `testing`

`src/tests/costs.rs` registers the contract natively, so VM costs are not
included. Build a small Wasm fixture, register it with `env.register(wasm, ())`,
and report realistic totals. Keep the existing assertions.

**Done when:** the README fee table shows Wasm-based numbers and says so.

---

## End-to-end: verify read-path extension below the threshold on a real network

Labels: `testing`, `e2e`

`scripts/e2e-local.sh` checks create/overwrite/temporary behaviour but not
that a *read* extends an entry once its TTL drops below the threshold.
Add a tiny e2e-only contract with a short policy (for example
`TtlPolicy::new(20, 40)`), wait for ledgers to pass, read, and check the
new `live_until_ledger`.

**Done when:** the script has a passing check for read-path extension.

---

## Add a SEP-41 token example built on soroban-storage

Labels: `example`, `documentation`

Add `examples/token` implementing the SEP-41 token interface with balances
and allowances stored via `storage_keys!` (allowances in temporary storage
with an expiration ledger). Include tests with TTL assertions.

**Done when:** example builds with `stellar contract build` and its tests pass in CI.

---

## Write a migration guide from a hand-written `DataKey` enum

Labels: `documentation`, `good first issue`

Write `docs/migration.md` showing how to move an existing contract from
`#[contracttype] enum DataKey` + raw `env.storage()` calls to
`storage_keys!` without moving data (same variant names, same field order),
including how to upgrade the deployed contract.

**Done when:** guide exists, is linked from the README, and its code compiles (doc test or example).

---

## Add `TtlPolicy::max()` for "keep alive as long as the network allows"

Labels: `enhancement`

Add a const policy whose `extend_to` is `u32::MAX` (clamped at runtime to
the network maximum) with a sensible threshold, and document the rent cost
trade-off.

**Done when:** a unit test shows the TTL equals `env.storage().max_ttl()` after a write.

---

## CI job against the soroban-sdk 30 release candidate

Labels: `ci`

Add a non-blocking CI job that patches the workspace to `soroban-sdk = "30.0.0-rc.1"`
and runs the tests, so breakage is spotted before the release.

**Done when:** the job runs on pull requests and is marked `continue-on-error`.

---

## Compile-fail test for keys with more than 12 fields

Labels: `testing`, `macros`, `good first issue`

Keys are encoded as tuples, which convert to `Val` for up to 12 elements
(plus the name). Add a `tests/ui` case showing what happens with 13 fields,
and improve the error with a clear message if it is confusing.

**Done when:** a new `tests/ui` case and expected output exist.

---

## Document cost and behaviour of instance keys

Labels: `documentation`

Instance keys share one TTL (the contract instance) and are loaded on every
call. Add a README section with guidance on what belongs in instance
storage, how many bytes are reasonable, and measured fees for an instance
read/write using the existing cost test setup.

**Done when:** README section added with measured numbers.

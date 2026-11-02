//! Fee comparisons between raw SDK storage calls and this crate.
//!
//! Run with `cargo test -p soroban-storage costs -- --nocapture` to print the
//! table. Fees use the SDK's hardcoded mainnet fee snapshot, and the contract
//! is registered natively (no Wasm), so absolute numbers leave out VM costs.
//! The comparisons between rows are what matter.

extern crate std;

use crate::*;
use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    Address, Env,
};

const POLICY: TtlPolicy = TtlPolicy::new(days(30), days(60));

storage_keys! {
    persistent(POLICY) Bal(owner: Address) => i128;
}

#[contracttype]
enum RawKey {
    Bal(Address),
}

#[contract]
pub struct CostContract;

#[contractimpl]
impl CostContract {
    /// Plain SDK write, no TTL management at all.
    pub fn raw_set(env: Env, owner: Address, v: i128) {
        env.storage().persistent().set(&RawKey::Bal(owner), &v);
    }

    /// The common hand-written pattern: write, then extend unconditionally.
    pub fn raw_set_extend(env: Env, owner: Address, v: i128) {
        let key = RawKey::Bal(owner);
        env.storage().persistent().set(&key, &v);
        env.storage()
            .persistent()
            .extend_ttl(&key, POLICY.extend_to, POLICY.extend_to);
    }

    pub fn typed_set(env: Env, owner: Address, v: i128) {
        Bal(owner).set(&env, &v);
    }

    pub fn raw_get(env: Env, owner: Address) -> Option<i128> {
        env.storage().persistent().get(&RawKey::Bal(owner))
    }

    pub fn typed_get(env: Env, owner: Address) -> Option<i128> {
        Bal(owner).get(&env)
    }
}

struct Fees {
    total: i64,
    rent: i64,
}

fn last_fees(env: &Env) -> Fees {
    let fee = env.cost_estimate().fee();
    Fees {
        total: fee.total,
        rent: fee.persistent_entry_rent,
    }
}

/// Fees for: create, overwrite 1 ledger later, read 1 ledger later.
fn measure(
    call_set: impl Fn(&CostContractClient, &Address, i128),
    call_get: impl Fn(&CostContractClient, &Address),
) -> [Fees; 3] {
    let env = Env::default();
    let id = env.register(CostContract, ());
    let client = CostContractClient::new(&env, &id);
    let owner = Address::generate(&env);

    call_set(&client, &owner, 1);
    let create = last_fees(&env);

    env.ledger().with_mut(|l| l.sequence_number += 1);
    call_set(&client, &owner, 2);
    let overwrite = last_fees(&env);

    env.ledger().with_mut(|l| l.sequence_number += 1);
    call_get(&client, &owner);
    let read = last_fees(&env);

    [create, overwrite, read]
}

#[test]
fn costs_compared_to_raw_sdk() {
    let raw = measure(
        |c, o, v| c.raw_set(o, &v),
        |c, o| {
            c.raw_get(o);
        },
    );
    let naive = measure(
        |c, o, v| c.raw_set_extend(o, &v),
        |c, o| {
            c.raw_get(o);
        },
    );
    let typed = measure(
        |c, o, v| c.typed_set(o, &v),
        |c, o| {
            c.typed_get(o);
        },
    );

    std::println!("fees in stroops (total / persistent rent)");
    std::println!(
        "{:<28}{:>18}{:>18}{:>18}",
        "",
        "create",
        "overwrite",
        "read"
    );
    for (name, row) in [
        ("raw SDK (no TTL mgmt)", &raw),
        ("raw SDK + extend every write", &naive),
        ("soroban-storage", &typed),
    ] {
        std::println!(
            "{:<28}{:>18}{:>18}{:>18}",
            name,
            std::format!("{} / {}", row[0].total, row[0].rent),
            std::format!("{} / {}", row[1].total, row[1].rent),
            std::format!("{} / {}", row[2].total, row[2].rent),
        );
    }

    // Creating an entry buys the policy's full lifetime, like the naive
    // pattern does.
    assert_eq!(typed[0].rent, naive[0].rent);
    assert!(typed[0].rent > raw[0].rent);
    // Overwriting a healthy entry pays no rent, unlike the naive pattern.
    assert_eq!(typed[1].rent, 0);
    assert!(naive[1].rent > 0);
    // Reading a healthy entry pays no rent either.
    assert_eq!(typed[2].rent, 0);
}

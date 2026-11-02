//! Model-based property tests.
//!
//! Random sequences of storage operations and ledger jumps run against both
//! the real host and a small model of the TTL rules. After every step the
//! stored value and the exact remaining TTL must match the model.

extern crate std;

use crate::*;
use proptest::prelude::*;
use soroban_sdk::{
    contract,
    testutils::{Address as _, Ledger},
    Address, Env,
};
use std::vec::Vec;

const THRESHOLD: u32 = 50;
const EXTEND_TO: u32 = 200;
const POLICY: TtlPolicy = TtlPolicy::new(THRESHOLD, EXTEND_TO);
const LAZY: TtlPolicy = POLICY.write_only();

storage_keys! {
    persistent(POLICY) P(id: Address) => i128;
    temporary(POLICY) T(id: Address) => i128;
    persistent(LAZY) L(id: Address) => i128;
}

#[contract]
pub struct ModelContract;

#[derive(Clone, Debug)]
enum Op {
    Set(i128),
    Update(i128),
    TryUpdateFail,
    Get,
    Has,
    Remove,
    Take,
    Extend,
    Advance(u32),
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        any::<i64>().prop_map(|v| Op::Set(v as i128)),
        (-1000i128..1000).prop_map(Op::Update),
        Just(Op::TryUpdateFail),
        Just(Op::Get),
        Just(Op::Has),
        Just(Op::Remove),
        Just(Op::Take),
        Just(Op::Extend),
        (0u32..=EXTEND_TO + 20).prop_map(Op::Advance),
    ]
}

/// What the model expects to be stored: the value and its remaining TTL.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Entry {
    val: i128,
    ttl: u32,
}

struct Model {
    entry: Option<Entry>,
    temporary: bool,
    bump_on_read: bool,
}

impl Model {
    fn touch_read(&mut self) {
        if self.bump_on_read {
            self.touch();
        }
    }
    fn touch(&mut self) {
        if let Some(e) = &mut self.entry {
            if e.ttl <= THRESHOLD {
                e.ttl = EXTEND_TO;
            }
        }
    }
    fn write(&mut self, val: i128) {
        match &mut self.entry {
            Some(e) => {
                e.val = val;
                if e.ttl <= THRESHOLD {
                    e.ttl = EXTEND_TO;
                }
            }
            None => {
                self.entry = Some(Entry {
                    val,
                    ttl: EXTEND_TO,
                })
            }
        }
    }
    fn advance(&mut self, n: u32) {
        if let Some(e) = &mut self.entry {
            if n > e.ttl {
                // Temporary entries are deleted. (Persistent runs never jump
                // past the TTL; see `run`.)
                assert!(self.temporary);
                self.entry = None;
            } else {
                e.ttl -= n;
            }
        }
    }
}

fn run<K: TypedKey<Value = i128> + Clone>(
    make: impl Fn(Address) -> K,
    temporary: bool,
    bump_on_read: bool,
    ops: Vec<Op>,
) {
    let env = Env::default();
    // Small minimum TTLs, so new entries start below the policy and the
    // "top up on create" rule is observable.
    env.ledger().with_mut(|l| {
        l.min_persistent_entry_ttl = 10;
        l.min_temp_entry_ttl = 10;
    });
    let id = env.register(ModelContract, ());
    let key = make(Address::generate(&env));
    let mut model = Model {
        entry: None,
        temporary,
        bump_on_read,
    };

    for op in ops {
        env.as_contract(&id, || match op {
            Op::Set(v) => {
                key.set(&env, &v);
                model.write(v);
            }
            Op::Update(d) => {
                let expected = model.entry.map_or(0, |e| e.val) + d;
                assert_eq!(key.update(&env, |v| v.unwrap_or(0) + d), expected);
                model.write(expected);
            }
            Op::TryUpdateFail => {
                assert_eq!(key.try_update(&env, |_| Err(())), Err(()));
            }
            Op::Get => {
                assert_eq!(key.get(&env), model.entry.map(|e| e.val));
                model.touch_read();
            }
            Op::Has => {
                assert_eq!(key.has(&env), model.entry.is_some());
                model.touch_read();
            }
            Op::Remove => {
                key.remove(&env);
                model.entry = None;
            }
            Op::Take => {
                assert_eq!(key.take(&env), model.entry.map(|e| e.val));
                model.entry = None;
            }
            Op::Extend => {
                assert_eq!(key.extend(&env), model.entry.is_some());
                model.touch();
            }
            Op::Advance(_) => {}
        });
        if let Op::Advance(n) = op {
            // Never let a persistent entry expire: in tests it would be
            // auto-restored, which is outside what this model covers.
            let n = match (temporary, model.entry) {
                (false, Some(e)) => n.min(e.ttl),
                _ => n,
            };
            env.ledger().with_mut(|l| l.sequence_number += n);
            model.advance(n);
        }

        env.as_contract(&id, || {
            // Inspect without bumping: read raw TTL and value.
            let raw = key.to_key_val(&env);
            let present = K::Kind::has(&env, &raw);
            assert_eq!(present, model.entry.is_some(), "presence after {op:?}");
            if let Some(e) = model.entry {
                assert_eq!(K::Kind::get_ttl(&env, &raw), e.ttl, "ttl after {op:?}");
            }
        });
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn persistent_matches_model(ops in prop::collection::vec(op(), 1..40)) {
        run(P, false, true, ops);
    }

    #[test]
    fn temporary_matches_model(ops in prop::collection::vec(op(), 1..40)) {
        run(T, true, true, ops);
    }

    #[test]
    fn write_only_matches_model(ops in prop::collection::vec(op(), 1..40)) {
        run(L, false, false, ops);
    }
}

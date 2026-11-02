use soroban_sdk::Address;
use soroban_storage::{storage_keys, TtlPolicy};

storage_keys! {
    persistent(TtlPolicy::new(500, 100)) Balance(owner: Address) => i128;
}

fn main() {}

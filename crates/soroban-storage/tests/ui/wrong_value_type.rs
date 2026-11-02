use soroban_sdk::{Address, Env};
use soroban_storage::{days, storage_keys, KeyExt, TtlPolicy};

storage_keys! {
    persistent(TtlPolicy::new(days(1), days(2))) Balance(owner: Address) => i128;
}

fn write(env: &Env, owner: Address) {
    let amount: u32 = 5;
    Balance(owner).set(env, &amount);
}

fn main() {}

use soroban_storage::{days, storage_keys, TtlPolicy};

storage_keys! {
    forever(TtlPolicy::new(days(1), days(2))) Counter => u64;
}

fn main() {}

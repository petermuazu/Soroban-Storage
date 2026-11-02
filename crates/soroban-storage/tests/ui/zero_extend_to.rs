use soroban_storage::{storage_keys, TtlPolicy};

const BROKEN: TtlPolicy = TtlPolicy {
    threshold: 0,
    extend_to: 0,
    bump_on_read: true,
};

storage_keys! {
    instance(BROKEN) Counter => u64;
}

fn main() {}

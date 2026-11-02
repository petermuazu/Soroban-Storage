//! Checks that misuse is rejected at compile time.
//!
//! Compiler messages change between Rust releases. If these fail only
//! because the wording changed, regenerate the expected output with
//! `TRYBUILD=overwrite cargo test -p soroban-storage --test compile_fail`
//! and review the diff.

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

//! Typed Soroban storage with explicit lifecycles and automatic TTL
//! management.
#![no_std]
#![deny(missing_docs)]

mod ttl;

pub use ttl::{
    days, hours, minutes, TtlPolicy, LEDGERS_PER_DAY, LEDGERS_PER_HOUR, LEDGERS_PER_MINUTE,
    LEDGER_SECONDS,
};

//! Typed Soroban storage with explicit lifecycles and automatic TTL
//! management.
#![no_std]
#![deny(missing_docs)]

mod kind;
mod ttl;

pub use kind::{Instance, Lifecycle, Persistent, StorageKind, Temporary};
pub use ttl::{
    days, hours, minutes, TtlPolicy, LEDGERS_PER_DAY, LEDGERS_PER_HOUR, LEDGERS_PER_MINUTE,
    LEDGER_SECONDS,
};

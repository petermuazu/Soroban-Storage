// `#[contracterror]` generates undocumented public items.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

/// Errors raised by this crate.
///
/// These are contract errors (Soroban only lets a contract fail with
/// contract error codes), using the reserved range `1100..1200` so they do
/// not clash with the small codes contracts usually define. Use
/// [`require_or`](crate::require_or) or [`try_get`](crate::try_get) to fail with your own error instead.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StorageError {
    /// No value is stored under the key. For temporary storage this includes
    /// entries whose TTL ran out.
    NotFound = 1100,
    /// The stored value could not be converted to the key's value type.
    WrongType = 1101,
}

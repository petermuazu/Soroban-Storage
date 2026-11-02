//! Storage kinds: the three Soroban storage types as marker types.

use soroban_sdk::{Env, Val};

/// Runtime description of a storage kind.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Lifecycle {
    /// Survives indefinitely as long as rent is paid. Archived (not deleted)
    /// when its TTL runs out and can be restored.
    Persistent,
    /// Cheap, short-lived data. **Permanently deleted** when its TTL runs out.
    Temporary,
    /// Stored inside the contract instance entry and shares its TTL. Loaded on
    /// every invocation, so keep it small.
    Instance,
}

mod sealed {
    pub trait Sealed {}
}

/// A Soroban storage type. Implemented only by [`Persistent`],
/// [`Temporary`] and [`Instance`].
///
/// The methods work on raw [`Val`] keys and values; contract code should use
/// the typed functions at the crate root instead.
pub trait StorageKind: sealed::Sealed {
    /// Which storage type this is.
    const LIFECYCLE: Lifecycle;

    #[doc(hidden)]
    fn has(env: &Env, key: &Val) -> bool;
    #[doc(hidden)]
    fn get(env: &Env, key: &Val) -> Option<Val>;
    #[doc(hidden)]
    fn set(env: &Env, key: &Val, val: &Val);
    #[doc(hidden)]
    fn remove(env: &Env, key: &Val);
    /// Extends the TTL of the entry under `key`. For [`Instance`] this
    /// extends the whole contract instance (and code); `key` is ignored.
    #[doc(hidden)]
    fn extend(env: &Env, key: &Val, threshold: u32, extend_to: u32);
    #[cfg(any(test, feature = "testutils"))]
    #[doc(hidden)]
    fn get_ttl(env: &Env, key: &Val) -> u32;
}

/// Marker for `env.storage().persistent()`.
pub struct Persistent;
/// Marker for `env.storage().temporary()`.
pub struct Temporary;
/// Marker for `env.storage().instance()`.
pub struct Instance;

impl sealed::Sealed for Persistent {}
impl sealed::Sealed for Temporary {}
impl sealed::Sealed for Instance {}

macro_rules! keyed_kind {
    ($ty:ident, $accessor:ident) => {
        impl StorageKind for $ty {
            const LIFECYCLE: Lifecycle = Lifecycle::$ty;

            fn has(env: &Env, key: &Val) -> bool {
                env.storage().$accessor().has(key)
            }
            fn get(env: &Env, key: &Val) -> Option<Val> {
                env.storage().$accessor().get(key)
            }
            fn set(env: &Env, key: &Val, val: &Val) {
                env.storage().$accessor().set(key, val)
            }
            fn remove(env: &Env, key: &Val) {
                env.storage().$accessor().remove(key)
            }
            fn extend(env: &Env, key: &Val, threshold: u32, extend_to: u32) {
                env.storage()
                    .$accessor()
                    .extend_ttl(key, threshold, extend_to)
            }
            #[cfg(any(test, feature = "testutils"))]
            fn get_ttl(env: &Env, key: &Val) -> u32 {
                use soroban_sdk::testutils::storage::$ty as _;
                env.storage().$accessor().get_ttl(key)
            }
        }
    };
}

keyed_kind!(Persistent, persistent);
keyed_kind!(Temporary, temporary);

impl StorageKind for Instance {
    const LIFECYCLE: Lifecycle = Lifecycle::Instance;

    fn has(env: &Env, key: &Val) -> bool {
        env.storage().instance().has(key)
    }
    fn get(env: &Env, key: &Val) -> Option<Val> {
        env.storage().instance().get(key)
    }
    fn set(env: &Env, key: &Val, val: &Val) {
        env.storage().instance().set(key, val)
    }
    fn remove(env: &Env, key: &Val) {
        env.storage().instance().remove(key)
    }
    fn extend(env: &Env, _key: &Val, threshold: u32, extend_to: u32) {
        env.storage().instance().extend_ttl(threshold, extend_to)
    }
    #[cfg(any(test, feature = "testutils"))]
    fn get_ttl(env: &Env, _key: &Val) -> u32 {
        use soroban_sdk::testutils::storage::Instance as _;
        env.storage().instance().get_ttl()
    }
}

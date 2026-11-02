//! Ledger-time helpers and TTL policies.
//!
//! Soroban measures entry lifetimes ("time to live") in ledgers, not seconds.
//! The helpers here convert human time spans into ledger counts using the
//! network's target close time of [`LEDGER_SECONDS`].

use soroban_sdk::Env;

/// Target ledger close time on Stellar networks, in seconds.
pub const LEDGER_SECONDS: u32 = 5;
/// Approximate number of ledgers closed per minute.
pub const LEDGERS_PER_MINUTE: u32 = 60 / LEDGER_SECONDS;
/// Approximate number of ledgers closed per hour.
pub const LEDGERS_PER_HOUR: u32 = 60 * LEDGERS_PER_MINUTE;
/// Approximate number of ledgers closed per day.
pub const LEDGERS_PER_DAY: u32 = 24 * LEDGERS_PER_HOUR;

const fn mul(n: u32, factor: u32) -> u32 {
    match n.checked_mul(factor) {
        Some(v) => v,
        None => panic!("ledger count overflows u32"),
    }
}

/// Number of ledgers in `n` minutes.
pub const fn minutes(n: u32) -> u32 {
    mul(n, LEDGERS_PER_MINUTE)
}

/// Number of ledgers in `n` hours.
pub const fn hours(n: u32) -> u32 {
    mul(n, LEDGERS_PER_HOUR)
}

/// Number of ledgers in `n` days.
pub const fn days(n: u32) -> u32 {
    mul(n, LEDGERS_PER_DAY)
}

/// How and when a key's TTL is extended.
///
/// * **Creating** an entry (`set`, `update`, ... on a missing key) extends
///   it so it lives for `extend_to` more ledgers.
/// * **Reading or overwriting** an existing entry extends it to `extend_to`
///   only when its remaining TTL is at or below `threshold`, so frequent access
///   does not pay rent for a small extension every time. Disable read
///   extensions with [`TtlPolicy::write_only`].
///
/// As long as an entry is accessed at least once every
/// `extend_to - threshold` ledgers, it never drops below `threshold`.
///
/// Build policies with [`TtlPolicy::new`] in a `const` context so invalid
/// policies are rejected at compile time:
///
/// ```
/// use soroban_storage::{days, TtlPolicy};
///
/// const BALANCE_TTL: TtlPolicy = TtlPolicy::new(days(30), days(90));
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TtlPolicy {
    /// Extend only when the remaining TTL is at or below this many ledgers.
    pub threshold: u32,
    /// Remaining TTL (in ledgers) after an extension.
    pub extend_to: u32,
    /// Whether reads (`get`, `has`, ...) extend the TTL. Writes always do.
    pub bump_on_read: bool,
}

impl TtlPolicy {
    /// Creates a policy that extends the TTL on both reads and writes.
    ///
    /// # Panics
    ///
    /// Panics (at compile time when used in a `const`) if `extend_to` is zero
    /// or `threshold` is greater than `extend_to`.
    pub const fn new(threshold: u32, extend_to: u32) -> Self {
        Self {
            threshold,
            extend_to,
            bump_on_read: true,
        }
        .validated()
    }

    /// Returns a copy of this policy that only extends the TTL on writes.
    ///
    /// Useful for hot read paths where the extra host call is not wanted and
    /// the entry is known to be written regularly.
    pub const fn write_only(self) -> Self {
        Self {
            bump_on_read: false,
            ..self
        }
    }

    /// Checks the policy invariants, panicking if they do not hold.
    pub const fn validated(self) -> Self {
        if self.extend_to == 0 {
            panic!("TtlPolicy: extend_to must be greater than zero");
        }
        if self.threshold > self.extend_to {
            panic!("TtlPolicy: threshold must not exceed extend_to");
        }
        self
    }

    /// The `(threshold, extend_to)` pair actually sent to the host, clamped
    /// to the network's current maximum TTL.
    ///
    /// Clamping keeps extensions from failing on networks whose maximum entry
    /// TTL is lower than the policy asks for.
    pub fn effective(&self, env: &Env) -> (u32, u32) {
        let max = env.storage().max_ttl();
        let extend_to = self.extend_to.min(max);
        let threshold = self.threshold.min(extend_to);
        (threshold, extend_to)
    }
}

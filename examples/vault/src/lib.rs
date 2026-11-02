//! A small vault contract showing `soroban-storage` in practice.
//!
//! * the admin lives in **instance** storage (small, loaded on every call);
//! * balances live in **persistent** storage (must never be lost);
//! * withdrawal cooldowns live in **temporary** storage (fine to lose: an
//!   expired cooldown simply means the holder may withdraw again).
#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, Address, Env};
use soroban_storage::{days, extend_instance, hours, storage_keys, KeyExt, TtlPolicy};

pub const INSTANCE_TTL: TtlPolicy = TtlPolicy::new(days(7), days(30));
pub const BALANCE_TTL: TtlPolicy = TtlPolicy::new(days(30), days(120));
pub const COOLDOWN_LEDGERS: u32 = hours(1);
pub const COOLDOWN_TTL: TtlPolicy = TtlPolicy::new(COOLDOWN_LEDGERS, COOLDOWN_LEDGERS);

storage_keys! {
    /// Vault administrator.
    pub instance(INSTANCE_TTL) Admin => Address;
    /// Deposited balance per holder.
    pub persistent(BALANCE_TTL) Balance(owner: Address) => i128;
    /// Present while a holder is in their post-withdrawal cooldown.
    pub temporary(COOLDOWN_TTL) Cooldown(owner: Address) => ();
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VaultError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InsufficientBalance = 4,
    CoolingDown = 5,
}

#[contract]
pub struct Vault;

#[contractimpl]
impl Vault {
    pub fn __constructor(env: Env, admin: Address) {
        Admin.set(&env, &admin);
    }

    pub fn admin(env: Env) -> Result<Address, VaultError> {
        Admin.try_get(&env).map_err(|_| VaultError::NotInitialized)
    }

    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<i128, VaultError> {
        from.require_auth();
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        extend_instance(&env, &INSTANCE_TTL);
        Ok(Balance(from).update(&env, |b| b.unwrap_or(0) + amount))
    }

    pub fn withdraw(env: Env, to: Address, amount: i128) -> Result<i128, VaultError> {
        to.require_auth();
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        if Cooldown(to.clone()).has(&env) {
            return Err(VaultError::CoolingDown);
        }
        extend_instance(&env, &INSTANCE_TTL);
        let left = Balance(to.clone()).try_update(&env, |b| {
            let b = b.unwrap_or(0);
            if b < amount {
                Err(VaultError::InsufficientBalance)
            } else {
                Ok(b - amount)
            }
        })?;
        Cooldown(to).set(&env, &());
        Ok(left)
    }

    pub fn balance(env: Env, owner: Address) -> i128 {
        Balance(owner).get_or(&env, 0)
    }
}

#[cfg(test)]
mod test;

//! EVM persistent storage with cold/warm access tracking (EIP-2929).
//!
//! In Yellow Paper notation, storage is σ[a]_s — a mapping from 256-bit keys
//! to 256-bit values, scoped per account address. Unlike memory, storage
//! persists between transactions.
//!
//! EIP-2929 (Berlin upgrade) introduced the "accessed storage keys" set.
//! The first access to a slot in a transaction is "cold" (2100 gas for SLOAD),
//! and subsequent accesses are "warm" (100 gas).

use std::collections::{HashMap, HashSet};

use crate::types::U256;

/// Gas cost for a cold SLOAD (first access in this transaction).
pub const COLD_SLOAD_COST: u64 = 2100;

/// Gas cost for a warm SLOAD (already accessed in this transaction).
pub const WARM_STORAGE_READ_COST: u64 = 100;

/// Gas cost for SSTORE when the slot is being set from zero to non-zero.
pub const SSTORE_SET_COST: u64 = 20000;

/// Gas cost for SSTORE when modifying an existing non-zero value.
pub const SSTORE_RESET_COST: u64 = 2900;

/// Persistent key-value storage for a single account.
pub struct Storage {
    /// The actual storage slots.
    data: HashMap<U256, U256>,
    /// Set of storage keys accessed in this transaction (for EIP-2929).
    warm_slots: HashSet<U256>,
}

impl Storage {
    /// Create a new, empty storage.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            warm_slots: HashSet::new(),
        }
    }

    /// Load a value from storage (SLOAD semantics).
    ///
    /// Returns the value and whether this was a cold access.
    pub fn load(&mut self, key: U256) -> (U256, bool) {
        let cold = !self.warm_slots.contains(&key);
        self.warm_slots.insert(key);
        let value = self.data.get(&key).copied().unwrap_or(U256::ZERO);
        (value, cold)
    }

    /// Store a value to storage (SSTORE semantics).
    ///
    /// Returns (cold_access, was_zero, new_is_zero) for gas calculation.
    pub fn store(&mut self, key: U256, value: U256) -> (bool, bool, bool) {
        let cold = !self.warm_slots.contains(&key);
        self.warm_slots.insert(key);
        let old_value = self.data.get(&key).copied().unwrap_or(U256::ZERO);
        let was_zero = old_value.is_zero();
        let new_is_zero = value.is_zero();

        if value.is_zero() {
            self.data.remove(&key);
        } else {
            self.data.insert(key, value);
        }

        (cold, was_zero, new_is_zero)
    }

    /// Get the gas cost for an SLOAD operation.
    pub fn sload_gas(&self, key: &U256) -> u64 {
        if self.warm_slots.contains(key) {
            WARM_STORAGE_READ_COST
        } else {
            COLD_SLOAD_COST
        }
    }

    /// Get the gas cost for an SSTORE operation (simplified).
    pub fn sstore_gas(&self, key: &U256) -> u64 {
        let cold_cost = if self.warm_slots.contains(key) {
            0
        } else {
            COLD_SLOAD_COST
        };
        let current = self.data.get(key).copied().unwrap_or(U256::ZERO);
        let base = if current.is_zero() {
            SSTORE_SET_COST
        } else {
            SSTORE_RESET_COST
        };
        cold_cost + base
    }

    /// Check if a slot has been accessed (is warm).
    pub fn is_warm(&self, key: &U256) -> bool {
        self.warm_slots.contains(key)
    }

    /// Get the current value of a slot without marking it warm.
    pub fn peek(&self, key: &U256) -> U256 {
        self.data.get(key).copied().unwrap_or(U256::ZERO)
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Exercise 3.3: SSTORE/SLOAD round-trip
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_3_store_and_load() {
        let mut storage = Storage::new();
        let key = U256::from(1u64);
        let value = U256::from(42u64);

        storage.store(key, value);
        let (loaded, _cold) = storage.load(key);
        assert_eq!(loaded, value);
    }

    #[test]
    fn ex_3_3_unset_slot_returns_zero() {
        let mut storage = Storage::new();
        let (value, cold) = storage.load(U256::from(99u64));
        assert_eq!(value, U256::ZERO);
        assert!(cold); // first access is cold
    }

    #[test]
    fn ex_3_3_overwrite_slot() {
        let mut storage = Storage::new();
        let key = U256::from(1u64);
        storage.store(key, U256::from(10u64));
        storage.store(key, U256::from(20u64));
        let (value, _) = storage.load(key);
        assert_eq!(value, U256::from(20u64));
    }

    // ---------------------------------------------------------------
    // Exercise 3.4: Cold vs. warm SLOAD cost
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_4_cold_sload_cost() {
        let storage = Storage::new();
        let key = U256::from(1u64);
        assert_eq!(storage.sload_gas(&key), COLD_SLOAD_COST); // 2100
    }

    #[test]
    fn ex_3_4_warm_sload_cost() {
        let mut storage = Storage::new();
        let key = U256::from(1u64);
        storage.load(key); // warms the slot
        assert_eq!(storage.sload_gas(&key), WARM_STORAGE_READ_COST); // 100
    }

    #[test]
    fn ex_3_4_sstore_warms_slot() {
        let mut storage = Storage::new();
        let key = U256::from(1u64);
        storage.store(key, U256::from(5u64)); // warms the slot
        assert!(storage.is_warm(&key));
        assert_eq!(storage.sload_gas(&key), WARM_STORAGE_READ_COST);
    }
}

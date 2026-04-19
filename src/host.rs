//! The Host trait: an abstraction for the external world.
//!
//! The EVM interpreter needs to interact with the "outside" — reading other
//! contracts' code, checking balances, creating new contracts. Rather than
//! hardcoding a specific state implementation, we define a trait that
//! decouples the interpreter from state management.
//!
//! This is the same pattern used by `revm` (the `Host` trait) and allows
//! testing with a `MockHost` backed by HashMaps.

use crate::context::Address;
use crate::types::U256;

/// A log entry emitted by LOG0..LOG4 opcodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// The address of the contract that emitted the log.
    pub address: Address,
    /// The indexed topics (0 to 4).
    pub topics: Vec<U256>,
    /// The unindexed data bytes.
    pub data: Vec<u8>,
}

// ANCHOR: host_trait
/// The Host trait abstracts external state access for the interpreter.
pub trait Host {
    /// Get the code deployed at an address. Returns empty for EOAs.
    fn code(&self, address: &Address) -> Vec<u8>;

    /// Get the code size at an address.
    fn code_size(&self, address: &Address) -> usize {
        self.code(address).len()
    }

    /// Get the balance of an address (in wei).
    fn balance(&self, address: &Address) -> U256;

    /// Emit a log entry.
    fn emit_log(&mut self, log: LogEntry);

    /// Get all emitted logs (for testing).
    fn logs(&self) -> &[LogEntry];
}
// ANCHOR_END: host_trait

/// A mock host backed by HashMaps, suitable for testing.
pub struct MockHost {
    /// Deployed contract code, keyed by address.
    codes: std::collections::HashMap<Address, Vec<u8>>,
    /// Account balances.
    balances: std::collections::HashMap<Address, U256>,
    /// Emitted log entries.
    log_entries: Vec<LogEntry>,
}

impl MockHost {
    pub fn new() -> Self {
        Self {
            codes: std::collections::HashMap::new(),
            balances: std::collections::HashMap::new(),
            log_entries: Vec::new(),
        }
    }

    /// Deploy code at an address.
    pub fn deploy_code(&mut self, address: Address, code: Vec<u8>) {
        self.codes.insert(address, code);
    }

    /// Set the balance of an address.
    pub fn set_balance(&mut self, address: Address, balance: U256) {
        self.balances.insert(address, balance);
    }
}

impl Default for MockHost {
    fn default() -> Self {
        Self::new()
    }
}

impl Host for MockHost {
    fn code(&self, address: &Address) -> Vec<u8> {
        self.codes.get(address).cloned().unwrap_or_default()
    }

    fn balance(&self, address: &Address) -> U256 {
        self.balances.get(address).copied().unwrap_or(U256::ZERO)
    }

    fn emit_log(&mut self, log: LogEntry) {
        self.log_entries.push(log);
    }

    fn logs(&self) -> &[LogEntry] {
        &self.log_entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Exercise 4.3: Host trait compilation
    // ---------------------------------------------------------------
    #[test]
    fn ex_4_3_mock_host_compiles() {
        let mut host = MockHost::new();
        let addr = Address::from(0x42u64);

        // Deploy code
        host.deploy_code(addr, vec![0x60, 0x00, 0x00]);
        assert_eq!(host.code_size(&addr), 3);
        assert_eq!(host.code(&addr), vec![0x60, 0x00, 0x00]);

        // EOA has no code
        let eoa = Address::from(0x01u64);
        assert_eq!(host.code_size(&eoa), 0);

        // Balance
        host.set_balance(addr, U256::from(1000u64));
        assert_eq!(host.balance(&addr), U256::from(1000u64));
        assert_eq!(host.balance(&eoa), U256::ZERO);
    }

    // ---------------------------------------------------------------
    // Exercise 4.4: EXTCODESIZE via Host
    // ---------------------------------------------------------------
    #[test]
    fn ex_4_4_extcodesize() {
        let mut host = MockHost::new();
        let addr = Address::from(0x42u64);
        host.deploy_code(addr, vec![0x00; 100]); // 100 bytes of code
        assert_eq!(host.code_size(&addr), 100);
        assert_eq!(host.code_size(&Address::ZERO), 0);
    }

    // ---------------------------------------------------------------
    // Exercise 4.5/4.6: LOG gas and topics
    // ---------------------------------------------------------------
    #[test]
    fn ex_4_5_log0_structure() {
        let mut host = MockHost::new();
        let log = LogEntry {
            address: Address::from(0x01u64),
            topics: vec![],
            data: vec![0xAB; 32],
        };
        host.emit_log(log.clone());
        assert_eq!(host.logs().len(), 1);
        assert_eq!(host.logs()[0], log);
    }

    #[test]
    fn ex_4_6_log2_topics() {
        let mut host = MockHost::new();
        let log = LogEntry {
            address: Address::from(0x01u64),
            topics: vec![U256::from(0xAAu64), U256::from(0xBBu64)],
            data: vec![0xCC; 10],
        };
        host.emit_log(log);
        assert_eq!(host.logs()[0].topics.len(), 2);
        assert_eq!(host.logs()[0].topics[0], U256::from(0xAAu64));
        assert_eq!(host.logs()[0].topics[1], U256::from(0xBBu64));
    }

    #[test]
    fn ex_4_5_log0_gas_cost() {
        // LOG0 gas = 375 + 8 * data_length
        // For 32 bytes: 375 + 8 * 32 = 631
        let base = 375u64;
        let per_byte = 8u64;
        let data_len = 32u64;
        assert_eq!(base + per_byte * data_len, 631);
    }
}

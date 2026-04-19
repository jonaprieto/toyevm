//! Call mechanics: CALL, DELEGATECALL, STATICCALL, and the call stack.
//!
//! In Yellow Paper notation, the message call function is Θ.
//! Each call creates a new execution frame with its own stack, memory,
//! and PC, but shares storage (for DELEGATECALL) or accesses the callee's
//! storage (for CALL/STATICCALL).

use crate::context::Address;
use crate::types::U256;

/// Maximum call depth (protocol constant).
pub const MAX_CALL_DEPTH: usize = 1024;

/// The type of call being made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallType {
    /// Regular CALL: new context with callee's storage.
    Call,
    /// DELEGATECALL: callee's code runs in caller's context.
    DelegateCall,
    /// STATICCALL: like CALL but state modifications are forbidden.
    StaticCall,
    /// CALLCODE: deprecated predecessor to DELEGATECALL.
    CallCode,
}

/// A single frame on the call stack.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// The type of call.
    pub call_type: CallType,
    /// Address of the contract whose code is being executed.
    pub code_address: Address,
    /// Address of the "current" contract (for storage access).
    /// For DELEGATECALL, this is the caller's address.
    pub context_address: Address,
    /// The caller's address (msg.sender).
    pub caller: Address,
    /// Value sent with the call (msg.value).
    pub value: U256,
    /// Gas allocated to this call frame.
    pub gas: u64,
    /// Whether state modifications are forbidden (STATICCALL propagates this).
    pub is_static: bool,
    /// Input data for the call.
    pub input: Vec<u8>,
    /// Return data from the sub-call (populated after execution).
    pub return_data: Vec<u8>,
    /// Whether the sub-call succeeded.
    pub success: bool,
}

impl CallFrame {
    /// Create a new CALL frame.
    pub fn new_call(
        code_address: Address,
        caller: Address,
        value: U256,
        gas: u64,
        input: Vec<u8>,
    ) -> Self {
        Self {
            call_type: CallType::Call,
            code_address,
            context_address: code_address,
            caller,
            value,
            gas,
            is_static: false,
            input,
            return_data: Vec::new(),
            success: false,
        }
    }

    /// Create a new DELEGATECALL frame.
    pub fn new_delegatecall(
        code_address: Address,
        context_address: Address,
        caller: Address,
        value: U256,
        gas: u64,
        input: Vec<u8>,
    ) -> Self {
        Self {
            call_type: CallType::DelegateCall,
            code_address,
            context_address,
            caller,
            value,
            gas,
            is_static: false,
            input,
            return_data: Vec::new(),
            success: false,
        }
    }

    /// Create a new STATICCALL frame.
    pub fn new_staticcall(
        code_address: Address,
        caller: Address,
        gas: u64,
        input: Vec<u8>,
    ) -> Self {
        Self {
            call_type: CallType::StaticCall,
            code_address,
            context_address: code_address,
            caller,
            value: U256::ZERO,
            gas,
            is_static: true,
            input,
            return_data: Vec::new(),
            success: false,
        }
    }
}

/// Calculate the gas to forward to a sub-call using the 63/64 rule (EIP-150).
///
/// The caller can specify a gas amount, but at most 63/64 of remaining gas
/// is forwarded. This prevents gas exhaustion attacks through deep call trees.
pub fn gas_to_forward(gas_remaining: u64, gas_requested: u64) -> u64 {
    let max_forward = gas_remaining - gas_remaining / 64; // 63/64 of remaining
    gas_requested.min(max_forward)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Exercise 5.1: CallFrame construction
    // ---------------------------------------------------------------
    #[test]
    fn ex_5_1_call_frame() {
        let frame = CallFrame::new_call(
            Address::from(0x42u64),
            Address::from(0x01u64),
            U256::from(100u64),
            10000,
            vec![0xAB],
        );
        assert_eq!(frame.call_type, CallType::Call);
        assert_eq!(frame.code_address, Address::from(0x42u64));
        assert_eq!(frame.context_address, Address::from(0x42u64));
        assert_eq!(frame.caller, Address::from(0x01u64));
        assert_eq!(frame.value, U256::from(100u64));
        assert!(!frame.is_static);
    }

    #[test]
    fn ex_5_1_delegatecall_frame() {
        let frame = CallFrame::new_delegatecall(
            Address::from(0x42u64), // code from here
            Address::from(0x01u64), // but context (storage) is caller's
            Address::from(0x99u64), // original caller preserved
            U256::from(50u64),
            10000,
            vec![],
        );
        assert_eq!(frame.call_type, CallType::DelegateCall);
        assert_eq!(frame.code_address, Address::from(0x42u64));
        assert_eq!(frame.context_address, Address::from(0x01u64));
    }

    #[test]
    fn ex_5_1_staticcall_frame() {
        let frame = CallFrame::new_staticcall(
            Address::from(0x42u64),
            Address::from(0x01u64),
            10000,
            vec![],
        );
        assert_eq!(frame.call_type, CallType::StaticCall);
        assert!(frame.is_static);
        assert_eq!(frame.value, U256::ZERO);
    }

    // ---------------------------------------------------------------
    // Exercise 5.2: Depth limit
    // ---------------------------------------------------------------
    #[test]
    fn ex_5_2_max_depth_constant() {
        assert_eq!(MAX_CALL_DEPTH, 1024);
    }

    // ---------------------------------------------------------------
    // Exercise 5.4: Gas forwarding (63/64 rule)
    // ---------------------------------------------------------------
    #[test]
    fn ex_5_4_gas_forwarding_rule() {
        // With 6400 gas remaining, max forward = 6400 - 100 = 6300
        assert_eq!(gas_to_forward(6400, 10000), 6300);
        // If requested less than 63/64, use requested
        assert_eq!(gas_to_forward(6400, 1000), 1000);
    }

    #[test]
    fn ex_5_4_gas_forwarding_small_remaining() {
        // With 64 gas, max forward = 64 - 1 = 63
        assert_eq!(gas_to_forward(64, 1000), 63);
        // With 1 gas, max forward = 1 - 0 = 1
        assert_eq!(gas_to_forward(1, 1000), 1);
    }
}

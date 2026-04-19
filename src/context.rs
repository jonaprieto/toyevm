//! Execution context: the environment in which bytecode runs.
//!
//! In Yellow Paper notation, this is I — the execution environment containing
//! the sender, value, calldata, and block context.

use crate::types::U256;

/// A 20-byte Ethereum address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub const ZERO: Self = Self([0u8; 20]);
}

impl From<u64> for Address {
    fn from(val: u64) -> Self {
        let mut bytes = [0u8; 20];
        bytes[12..20].copy_from_slice(&val.to_be_bytes());
        Self(bytes)
    }
}

/// Transaction and call context for the interpreter.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Address of the executing contract (I_a).
    pub address: Address,
    /// Address of the caller (I_s).
    pub caller: Address,
    /// Address of the original transaction sender (I_o).
    pub origin: Address,
    /// Value (in wei) sent with this call (I_v).
    pub call_value: U256,
    /// Input data (calldata) for this call (I_d).
    pub calldata: Vec<u8>,
    /// The bytecode being executed (I_b).
    pub code: Vec<u8>,
    /// Gas price of the transaction.
    pub gas_price: U256,
    /// Block number.
    pub block_number: U256,
    /// Block timestamp.
    pub block_timestamp: U256,
    /// Block coinbase (miner/validator address).
    pub block_coinbase: Address,
    /// Block difficulty / prevrandao.
    pub block_difficulty: U256,
    /// Block gas limit.
    pub block_gaslimit: U256,
    /// Chain ID.
    pub chain_id: U256,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            address: Address::ZERO,
            caller: Address::ZERO,
            origin: Address::ZERO,
            call_value: U256::ZERO,
            calldata: Vec::new(),
            code: Vec::new(),
            gas_price: U256::ZERO,
            block_number: U256::ZERO,
            block_timestamp: U256::ZERO,
            block_coinbase: Address::ZERO,
            block_difficulty: U256::ZERO,
            block_gaslimit: U256::ZERO,
            chain_id: U256::from(1u64),
        }
    }
}

/// Builder for ExecutionContext.
pub struct ContextBuilder {
    ctx: ExecutionContext,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            ctx: ExecutionContext::default(),
        }
    }

    pub fn address(mut self, addr: Address) -> Self {
        self.ctx.address = addr;
        self
    }

    pub fn caller(mut self, addr: Address) -> Self {
        self.ctx.caller = addr;
        self
    }

    pub fn origin(mut self, addr: Address) -> Self {
        self.ctx.origin = addr;
        self
    }

    pub fn call_value(mut self, value: U256) -> Self {
        self.ctx.call_value = value;
        self
    }

    pub fn calldata(mut self, data: Vec<u8>) -> Self {
        self.ctx.calldata = data;
        self
    }

    pub fn code(mut self, code: Vec<u8>) -> Self {
        self.ctx.code = code;
        self
    }

    pub fn block_number(mut self, n: U256) -> Self {
        self.ctx.block_number = n;
        self
    }

    pub fn block_timestamp(mut self, ts: U256) -> Self {
        self.ctx.block_timestamp = ts;
        self
    }

    pub fn chain_id(mut self, id: U256) -> Self {
        self.ctx.chain_id = id;
        self
    }

    pub fn build(self) -> ExecutionContext {
        self.ctx
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_produces_correct_context() {
        let ctx = ContextBuilder::new()
            .caller(Address::from(1u64))
            .call_value(U256::from(100u64))
            .calldata(vec![0xAB, 0xCD])
            .build();

        assert_eq!(ctx.caller, Address::from(1u64));
        assert_eq!(ctx.call_value, U256::from(100u64));
        assert_eq!(ctx.calldata, vec![0xAB, 0xCD]);
    }

    #[test]
    fn default_context_is_zeroed() {
        let ctx = ExecutionContext::default();
        assert_eq!(ctx.address, Address::ZERO);
        assert_eq!(ctx.call_value, U256::ZERO);
        assert!(ctx.calldata.is_empty());
    }
}

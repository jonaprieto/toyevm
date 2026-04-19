use crate::context::{Address, ExecutionContext};
use crate::gas;
use crate::host::LogEntry;
use crate::memory::Memory;
use crate::opcode::Opcode;
use crate::stack::{Stack, StackError};
use crate::storage::Storage;
use crate::types::U256;

/// The result of a successful execution.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Data returned by RETURN or empty for STOP.
    pub output: Vec<u8>,
    /// Total gas consumed during execution.
    pub gas_used: u64,
    /// Whether execution was reverted (REVERT opcode).
    pub reverted: bool,
}

/// Errors that halt execution abnormally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmError {
    /// Ran out of gas.
    OutOfGas,
    /// Stack overflow or underflow.
    StackError(StackError),
    /// Encountered an unknown or invalid opcode.
    InvalidOpcode(u8),
    /// JUMP/JUMPI to a non-JUMPDEST location.
    InvalidJump,
    /// Program counter went out of bounds.
    PcOutOfBounds,
}

impl std::fmt::Display for EvmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvmError::OutOfGas => write!(f, "out of gas"),
            EvmError::StackError(e) => write!(f, "stack error: {e}"),
            EvmError::InvalidOpcode(byte) => write!(f, "invalid opcode: 0x{byte:02x}"),
            EvmError::InvalidJump => write!(f, "invalid jump destination"),
            EvmError::PcOutOfBounds => write!(f, "program counter out of bounds"),
        }
    }
}

impl std::error::Error for EvmError {}

impl From<StackError> for EvmError {
    fn from(e: StackError) -> Self {
        EvmError::StackError(e)
    }
}

/// The interpreter executes EVM bytecode.
pub struct Interpreter {
    /// The bytecode being executed.
    bytecode: Vec<u8>,
    /// Program counter — index into bytecode.
    pc: usize,
    /// The operand stack.
    stack: Stack,
    /// Linear memory.
    memory: Memory,
    /// Persistent storage.
    storage: Storage,
    /// Execution context (caller, calldata, block info, etc.).
    context: ExecutionContext,
    /// Gas remaining.
    gas_remaining: u64,
    /// Gas consumed so far.
    gas_used: u64,
    /// Emitted log entries.
    logs: Vec<LogEntry>,
    /// Set of valid JUMPDEST offsets (precomputed).
    valid_jumpdests: Vec<bool>,
}

impl Interpreter {
    /// Create a new interpreter for the given bytecode and gas limit.
    pub fn new(bytecode: Vec<u8>, gas_limit: u64) -> Self {
        let valid_jumpdests = Self::compute_jumpdests(&bytecode);
        Self {
            bytecode,
            pc: 0,
            stack: Stack::new(),
            memory: Memory::new(),
            storage: Storage::new(),
            context: ExecutionContext::default(),
            gas_remaining: gas_limit,
            gas_used: 0,
            logs: Vec::new(),
            valid_jumpdests,
        }
    }

    /// Create a new interpreter with a specific execution context and storage.
    pub fn with_context(
        bytecode: Vec<u8>,
        gas_limit: u64,
        context: ExecutionContext,
        storage: Storage,
    ) -> Self {
        let valid_jumpdests = Self::compute_jumpdests(&bytecode);
        Self {
            bytecode,
            pc: 0,
            stack: Stack::new(),
            memory: Memory::new(),
            storage,
            context,
            gas_remaining: gas_limit,
            gas_used: 0,
            logs: Vec::new(),
            valid_jumpdests,
        }
    }

    /// Get a reference to the storage (for inspection after execution).
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Get the emitted log entries.
    pub fn logs(&self) -> &[LogEntry] {
        &self.logs
    }

    /// Precompute the set of valid JUMPDEST positions.
    ///
    /// A byte is a valid JUMPDEST only if:
    /// 1. It is the JUMPDEST opcode (0x5B)
    /// 2. It is not part of a PUSH immediate sequence
    fn compute_jumpdests(bytecode: &[u8]) -> Vec<bool> {
        let mut valid = vec![false; bytecode.len()];
        let mut i = 0;
        while i < bytecode.len() {
            if bytecode[i] == Opcode::JUMPDEST as u8 {
                valid[i] = true;
            }
            // Skip PUSH immediate bytes
            if let Some(op) = Opcode::from_byte(bytecode[i]) {
                i += 1 + op.immediate_size();
            } else {
                i += 1;
            }
        }
        valid
    }

    /// Execute the bytecode until completion.
    pub fn run(&mut self) -> Result<ExecutionResult, EvmError> {
        loop {
            // Check if we've reached the end of bytecode — implicit STOP.
            if self.pc >= self.bytecode.len() {
                return Ok(ExecutionResult {
                    output: Vec::new(),
                    gas_used: self.gas_used,
                    reverted: false,
                });
            }

            let byte = self.bytecode[self.pc];
            let opcode = Opcode::from_byte(byte).ok_or(EvmError::InvalidOpcode(byte))?;

            // Deduct gas for this opcode.
            let gas_cost = self.static_gas_cost(opcode);
            self.use_gas(gas_cost)?;

            match opcode {
                Opcode::STOP => {
                    return Ok(ExecutionResult {
                        output: Vec::new(),
                        gas_used: self.gas_used,
                        reverted: false,
                    });
                }

                Opcode::ADD => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a + b)?;
                    self.pc += 1;
                }

                Opcode::MUL => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a * b)?;
                    self.pc += 1;
                }

                Opcode::SUB => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a - b)?;
                    self.pc += 1;
                }

                Opcode::DIV => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a.checked_div(b))?;
                    self.pc += 1;
                }

                Opcode::SDIV => {
                    // Signed division — simplified for now.
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    // TODO: proper signed division
                    self.stack.push(a.checked_div(b))?;
                    self.pc += 1;
                }

                Opcode::MOD => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a.checked_mod(b))?;
                    self.pc += 1;
                }

                Opcode::SMOD => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    // TODO: proper signed modulo
                    self.stack.push(a.checked_mod(b))?;
                    self.pc += 1;
                }

                Opcode::ADDMOD => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    let n = self.stack.pop()?;
                    if n.is_zero() {
                        self.stack.push(U256::ZERO)?;
                    } else {
                        self.stack.push((a + b).checked_mod(n))?;
                    }
                    self.pc += 1;
                }

                Opcode::MULMOD => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    let n = self.stack.pop()?;
                    if n.is_zero() {
                        self.stack.push(U256::ZERO)?;
                    } else {
                        self.stack.push((a * b).checked_mod(n))?;
                    }
                    self.pc += 1;
                }

                Opcode::EXP => {
                    let base = self.stack.pop()?;
                    let exp = self.stack.pop()?;
                    self.stack.push(Self::u256_exp(base, exp))?;
                    self.pc += 1;
                }

                Opcode::SIGNEXTEND => {
                    let _b = self.stack.pop()?;
                    let _x = self.stack.pop()?;
                    // TODO: proper sign extension
                    self.stack.push(_x)?;
                    self.pc += 1;
                }

                Opcode::LT => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack
                        .push(if a < b { U256::ONE } else { U256::ZERO })?;
                    self.pc += 1;
                }

                Opcode::GT => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack
                        .push(if a > b { U256::ONE } else { U256::ZERO })?;
                    self.pc += 1;
                }

                Opcode::SLT | Opcode::SGT => {
                    let _a = self.stack.pop()?;
                    let _b = self.stack.pop()?;
                    // TODO: proper signed comparison
                    self.stack.push(U256::ZERO)?;
                    self.pc += 1;
                }

                Opcode::EQ => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack
                        .push(if a == b { U256::ONE } else { U256::ZERO })?;
                    self.pc += 1;
                }

                Opcode::ISZERO => {
                    let a = self.stack.pop()?;
                    self.stack
                        .push(if a.is_zero() { U256::ONE } else { U256::ZERO })?;
                    self.pc += 1;
                }

                Opcode::AND => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a & b)?;
                    self.pc += 1;
                }

                Opcode::OR => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a | b)?;
                    self.pc += 1;
                }

                Opcode::XOR => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a ^ b)?;
                    self.pc += 1;
                }

                Opcode::NOT => {
                    let a = self.stack.pop()?;
                    self.stack.push(!a)?;
                    self.pc += 1;
                }

                Opcode::BYTE => {
                    let i = self.stack.pop()?;
                    let x = self.stack.pop()?;
                    let result = if let Some(idx) = i.to_usize() {
                        if idx < 32 {
                            U256::from(x.to_bytes()[idx] as u64)
                        } else {
                            U256::ZERO
                        }
                    } else {
                        U256::ZERO
                    };
                    self.stack.push(result)?;
                    self.pc += 1;
                }

                Opcode::SHL => {
                    let shift = self.stack.pop()?;
                    let value = self.stack.pop()?;
                    self.stack.push(Self::u256_shl(value, shift))?;
                    self.pc += 1;
                }

                Opcode::SHR => {
                    let shift = self.stack.pop()?;
                    let value = self.stack.pop()?;
                    self.stack.push(Self::u256_shr(value, shift))?;
                    self.pc += 1;
                }

                Opcode::SAR => {
                    let _shift = self.stack.pop()?;
                    let _value = self.stack.pop()?;
                    // TODO: arithmetic shift right
                    self.stack.push(U256::ZERO)?;
                    self.pc += 1;
                }

                Opcode::POP => {
                    self.stack.pop()?;
                    self.pc += 1;
                }

                Opcode::MLOAD => {
                    let offset = self.stack.pop()?;
                    let offset = offset.to_usize().ok_or(EvmError::OutOfGas)?;
                    self.charge_memory_expansion(offset, 32)?;
                    let value = self.memory.load_u256(offset);
                    self.stack.push(value)?;
                    self.pc += 1;
                }

                Opcode::MSTORE => {
                    let offset = self.stack.pop()?;
                    let value = self.stack.pop()?;
                    let offset = offset.to_usize().ok_or(EvmError::OutOfGas)?;
                    self.charge_memory_expansion(offset, 32)?;
                    self.memory.store_u256(offset, value);
                    self.pc += 1;
                }

                Opcode::MSTORE8 => {
                    let offset = self.stack.pop()?;
                    let value = self.stack.pop()?;
                    let offset = offset.to_usize().ok_or(EvmError::OutOfGas)?;
                    self.charge_memory_expansion(offset, 1)?;
                    self.memory.store_byte(offset, value.to_bytes()[31]);
                    self.pc += 1;
                }

                Opcode::SLOAD => {
                    let key = self.stack.pop()?;
                    let gas_cost = self.storage.sload_gas(&key);
                    self.use_gas(gas_cost)?;
                    let (value, _cold) = self.storage.load(key);
                    self.stack.push(value)?;
                    self.pc += 1;
                }

                Opcode::SSTORE => {
                    let key = self.stack.pop()?;
                    let value = self.stack.pop()?;
                    let gas_cost = self.storage.sstore_gas(&key);
                    self.use_gas(gas_cost)?;
                    self.storage.store(key, value);
                    self.pc += 1;
                }

                Opcode::KECCAK256 => {
                    let offset = self.stack.pop()?;
                    let size = self.stack.pop()?;
                    let offset = offset.to_usize().ok_or(EvmError::OutOfGas)?;
                    let size = size.to_usize().ok_or(EvmError::OutOfGas)?;
                    if size > 0 {
                        self.charge_memory_expansion(offset, size)?;
                    }
                    // Simplified: return zero hash (proper keccak not implemented yet)
                    self.stack.push(U256::ZERO)?;
                    self.pc += 1;
                }

                Opcode::ADDRESS => {
                    let addr = self.context.address;
                    self.stack.push(Self::address_to_u256(addr))?;
                    self.pc += 1;
                }

                Opcode::BALANCE | Opcode::SELFBALANCE => {
                    if matches!(opcode, Opcode::BALANCE) {
                        self.stack.pop()?; // consume address argument
                    }
                    // Simplified: return 0 balance
                    self.stack.push(U256::ZERO)?;
                    self.pc += 1;
                }

                Opcode::ORIGIN => {
                    let addr = self.context.origin;
                    self.stack.push(Self::address_to_u256(addr))?;
                    self.pc += 1;
                }

                Opcode::CALLER => {
                    let addr = self.context.caller;
                    self.stack.push(Self::address_to_u256(addr))?;
                    self.pc += 1;
                }

                Opcode::CALLVALUE => {
                    self.stack.push(self.context.call_value)?;
                    self.pc += 1;
                }

                Opcode::CALLDATALOAD => {
                    let offset = self.stack.pop()?;
                    let offset = offset.to_usize().unwrap_or(usize::MAX);
                    let mut bytes = [0u8; 32];
                    for (i, byte) in bytes.iter_mut().enumerate() {
                        let idx = offset.saturating_add(i);
                        if idx < self.context.calldata.len() {
                            *byte = self.context.calldata[idx];
                        }
                        // else: zero-padded (already 0)
                    }
                    self.stack.push(U256::from_bytes(bytes))?;
                    self.pc += 1;
                }

                Opcode::CALLDATASIZE => {
                    self.stack
                        .push(U256::from(self.context.calldata.len() as u64))?;
                    self.pc += 1;
                }

                Opcode::CALLDATACOPY => {
                    let dest_offset = self.stack.pop()?;
                    let data_offset = self.stack.pop()?;
                    let size = self.stack.pop()?;
                    let dest = dest_offset.to_usize().ok_or(EvmError::OutOfGas)?;
                    let src = data_offset.to_usize().unwrap_or(usize::MAX);
                    let len = size.to_usize().ok_or(EvmError::OutOfGas)?;
                    if len > 0 {
                        self.charge_memory_expansion(dest, len)?;
                        let mut data = vec![0u8; len];
                        for (i, byte) in data.iter_mut().enumerate() {
                            let idx = src.saturating_add(i);
                            if idx < self.context.calldata.len() {
                                *byte = self.context.calldata[idx];
                            }
                        }
                        self.memory.write_slice(dest, &data);
                    }
                    self.pc += 1;
                }

                Opcode::CODESIZE => {
                    self.stack.push(U256::from(self.bytecode.len() as u64))?;
                    self.pc += 1;
                }

                Opcode::CODECOPY => {
                    let dest_offset = self.stack.pop()?;
                    let code_offset = self.stack.pop()?;
                    let size = self.stack.pop()?;
                    let dest = dest_offset.to_usize().ok_or(EvmError::OutOfGas)?;
                    let src = code_offset.to_usize().unwrap_or(usize::MAX);
                    let len = size.to_usize().ok_or(EvmError::OutOfGas)?;
                    if len > 0 {
                        self.charge_memory_expansion(dest, len)?;
                        let mut data = vec![0u8; len];
                        for (i, byte) in data.iter_mut().enumerate() {
                            let idx = src.saturating_add(i);
                            if idx < self.bytecode.len() {
                                *byte = self.bytecode[idx];
                            }
                        }
                        self.memory.write_slice(dest, &data);
                    }
                    self.pc += 1;
                }

                Opcode::GASPRICE => {
                    self.stack.push(self.context.gas_price)?;
                    self.pc += 1;
                }

                Opcode::BLOCKHASH => {
                    // Simplified: consume block number, return zero
                    self.stack.pop()?;
                    self.stack.push(U256::ZERO)?;
                    self.pc += 1;
                }

                Opcode::COINBASE => {
                    let addr = self.context.block_coinbase;
                    self.stack.push(Self::address_to_u256(addr))?;
                    self.pc += 1;
                }

                Opcode::TIMESTAMP => {
                    self.stack.push(self.context.block_timestamp)?;
                    self.pc += 1;
                }

                Opcode::NUMBER => {
                    self.stack.push(self.context.block_number)?;
                    self.pc += 1;
                }

                Opcode::DIFFICULTY => {
                    self.stack.push(self.context.block_difficulty)?;
                    self.pc += 1;
                }

                Opcode::GASLIMIT => {
                    self.stack.push(self.context.block_gaslimit)?;
                    self.pc += 1;
                }

                Opcode::CHAINID => {
                    self.stack.push(self.context.chain_id)?;
                    self.pc += 1;
                }

                Opcode::JUMP => {
                    let dest = self.stack.pop()?;
                    let dest = dest.to_usize().ok_or(EvmError::InvalidJump)?;
                    if dest >= self.bytecode.len()
                        || !self.valid_jumpdests.get(dest).copied().unwrap_or(false)
                    {
                        return Err(EvmError::InvalidJump);
                    }
                    self.pc = dest;
                }

                Opcode::JUMPI => {
                    let dest = self.stack.pop()?;
                    let cond = self.stack.pop()?;
                    if !cond.is_zero() {
                        let dest = dest.to_usize().ok_or(EvmError::InvalidJump)?;
                        if dest >= self.bytecode.len()
                            || !self.valid_jumpdests.get(dest).copied().unwrap_or(false)
                        {
                            return Err(EvmError::InvalidJump);
                        }
                        self.pc = dest;
                    } else {
                        self.pc += 1;
                    }
                }

                Opcode::PC => {
                    self.stack.push(U256::from(self.pc as u64))?;
                    self.pc += 1;
                }

                Opcode::MSIZE => {
                    self.stack.push(U256::from(self.memory.len() as u64))?;
                    self.pc += 1;
                }

                Opcode::GAS => {
                    self.stack.push(U256::from(self.gas_remaining))?;
                    self.pc += 1;
                }

                Opcode::JUMPDEST => {
                    // No-op marker for valid jump targets.
                    self.pc += 1;
                }

                Opcode::PUSH0 => {
                    self.stack.push(U256::ZERO)?;
                    self.pc += 1;
                }

                Opcode::PUSH1
                | Opcode::PUSH2
                | Opcode::PUSH3
                | Opcode::PUSH4
                | Opcode::PUSH5
                | Opcode::PUSH6
                | Opcode::PUSH7
                | Opcode::PUSH8
                | Opcode::PUSH9
                | Opcode::PUSH10
                | Opcode::PUSH11
                | Opcode::PUSH12
                | Opcode::PUSH13
                | Opcode::PUSH14
                | Opcode::PUSH15
                | Opcode::PUSH16
                | Opcode::PUSH17
                | Opcode::PUSH18
                | Opcode::PUSH19
                | Opcode::PUSH20
                | Opcode::PUSH21
                | Opcode::PUSH22
                | Opcode::PUSH23
                | Opcode::PUSH24
                | Opcode::PUSH25
                | Opcode::PUSH26
                | Opcode::PUSH27
                | Opcode::PUSH28
                | Opcode::PUSH29
                | Opcode::PUSH30
                | Opcode::PUSH31
                | Opcode::PUSH32 => {
                    let n = opcode.immediate_size();
                    let start = self.pc + 1;
                    let end = start + n;

                    // Read immediate bytes, right-aligned into a 32-byte array.
                    let mut bytes = [0u8; 32];
                    let available = end.min(self.bytecode.len());
                    let actual_len = available.saturating_sub(start);
                    bytes[32 - n..32 - n + actual_len]
                        .copy_from_slice(&self.bytecode[start..start + actual_len]);

                    self.stack.push(U256::from_bytes(bytes))?;
                    self.pc = end;
                }

                Opcode::DUP1
                | Opcode::DUP2
                | Opcode::DUP3
                | Opcode::DUP4
                | Opcode::DUP5
                | Opcode::DUP6
                | Opcode::DUP7
                | Opcode::DUP8
                | Opcode::DUP9
                | Opcode::DUP10
                | Opcode::DUP11
                | Opcode::DUP12
                | Opcode::DUP13
                | Opcode::DUP14
                | Opcode::DUP15
                | Opcode::DUP16 => {
                    let depth = (opcode as u8 - Opcode::DUP1 as u8 + 1) as usize;
                    self.stack.dup(depth)?;
                    self.pc += 1;
                }

                Opcode::SWAP1
                | Opcode::SWAP2
                | Opcode::SWAP3
                | Opcode::SWAP4
                | Opcode::SWAP5
                | Opcode::SWAP6
                | Opcode::SWAP7
                | Opcode::SWAP8
                | Opcode::SWAP9
                | Opcode::SWAP10
                | Opcode::SWAP11
                | Opcode::SWAP12
                | Opcode::SWAP13
                | Opcode::SWAP14
                | Opcode::SWAP15
                | Opcode::SWAP16 => {
                    let depth = (opcode as u8 - Opcode::SWAP1 as u8 + 1) as usize;
                    self.stack.swap(depth)?;
                    self.pc += 1;
                }

                Opcode::RETURN => {
                    let offset = self.stack.pop()?;
                    let size = self.stack.pop()?;
                    let offset = offset.to_usize().ok_or(EvmError::OutOfGas)?;
                    let size = size.to_usize().ok_or(EvmError::OutOfGas)?;
                    if size > 0 {
                        self.charge_memory_expansion(offset, size)?;
                    }
                    let output = self.memory.read_slice(offset, size);
                    return Ok(ExecutionResult {
                        output,
                        gas_used: self.gas_used,
                        reverted: false,
                    });
                }

                Opcode::REVERT => {
                    let offset = self.stack.pop()?;
                    let size = self.stack.pop()?;
                    let offset = offset.to_usize().ok_or(EvmError::OutOfGas)?;
                    let size = size.to_usize().ok_or(EvmError::OutOfGas)?;
                    if size > 0 {
                        self.charge_memory_expansion(offset, size)?;
                    }
                    let output = self.memory.read_slice(offset, size);
                    return Ok(ExecutionResult {
                        output,
                        gas_used: self.gas_used,
                        reverted: true,
                    });
                }

                Opcode::LOG0 | Opcode::LOG1 | Opcode::LOG2 | Opcode::LOG3 | Opcode::LOG4 => {
                    let offset = self.stack.pop()?;
                    let size = self.stack.pop()?;
                    let num_topics = (opcode as u8 - Opcode::LOG0 as u8) as usize;

                    let mut topics = Vec::with_capacity(num_topics);
                    for _ in 0..num_topics {
                        topics.push(self.stack.pop()?);
                    }

                    let offset = offset.to_usize().ok_or(EvmError::OutOfGas)?;
                    let size = size.to_usize().ok_or(EvmError::OutOfGas)?;

                    // LOG gas: 375 + 8 * data_size + 375 * num_topics
                    let log_gas = 375 + 8 * size as u64 + 375 * num_topics as u64;
                    self.use_gas(log_gas)?;

                    if size > 0 {
                        self.charge_memory_expansion(offset, size)?;
                    }

                    let data = self.memory.read_slice(offset, size);
                    self.logs.push(LogEntry {
                        address: self.context.address,
                        topics,
                        data,
                    });
                    self.pc += 1;
                }

                Opcode::INVALID => {
                    return Err(EvmError::InvalidOpcode(0xFE));
                }
            }
        }
    }

    /// Convert a 20-byte address to a U256 (right-aligned).
    fn address_to_u256(addr: Address) -> U256 {
        let mut bytes = [0u8; 32];
        bytes[12..32].copy_from_slice(&addr.0);
        U256::from_bytes(bytes)
    }

    /// Deduct gas. Returns OutOfGas if insufficient.
    fn use_gas(&mut self, cost: u64) -> Result<(), EvmError> {
        if cost > self.gas_remaining {
            return Err(EvmError::OutOfGas);
        }
        self.gas_remaining -= cost;
        self.gas_used += cost;
        Ok(())
    }

    /// Charge gas for memory expansion if needed.
    fn charge_memory_expansion(&mut self, offset: usize, size: usize) -> Result<(), EvmError> {
        let current_words = self.memory.word_count();
        let new_words = self.memory.expansion_size(offset, size);
        let cost = gas::memory_expansion_cost(current_words, new_words);
        if cost > 0 {
            self.use_gas(cost)?;
        }
        Ok(())
    }

    /// Look up the static gas cost of an opcode.
    fn static_gas_cost(&self, opcode: Opcode) -> u64 {
        match opcode {
            Opcode::STOP | Opcode::JUMPDEST => gas::GAS_ZERO,

            Opcode::PUSH0 => gas::GAS_BASE,
            Opcode::PC | Opcode::MSIZE | Opcode::GAS => gas::GAS_BASE,
            Opcode::POP => gas::GAS_BASE,

            Opcode::ADD
            | Opcode::SUB
            | Opcode::LT
            | Opcode::GT
            | Opcode::SLT
            | Opcode::SGT
            | Opcode::EQ
            | Opcode::ISZERO
            | Opcode::AND
            | Opcode::OR
            | Opcode::XOR
            | Opcode::NOT
            | Opcode::BYTE
            | Opcode::SHL
            | Opcode::SHR
            | Opcode::SAR => gas::GAS_VERY_LOW,

            Opcode::PUSH1
            | Opcode::PUSH2
            | Opcode::PUSH3
            | Opcode::PUSH4
            | Opcode::PUSH5
            | Opcode::PUSH6
            | Opcode::PUSH7
            | Opcode::PUSH8
            | Opcode::PUSH9
            | Opcode::PUSH10
            | Opcode::PUSH11
            | Opcode::PUSH12
            | Opcode::PUSH13
            | Opcode::PUSH14
            | Opcode::PUSH15
            | Opcode::PUSH16
            | Opcode::PUSH17
            | Opcode::PUSH18
            | Opcode::PUSH19
            | Opcode::PUSH20
            | Opcode::PUSH21
            | Opcode::PUSH22
            | Opcode::PUSH23
            | Opcode::PUSH24
            | Opcode::PUSH25
            | Opcode::PUSH26
            | Opcode::PUSH27
            | Opcode::PUSH28
            | Opcode::PUSH29
            | Opcode::PUSH30
            | Opcode::PUSH31
            | Opcode::PUSH32 => gas::GAS_VERY_LOW,

            Opcode::DUP1
            | Opcode::DUP2
            | Opcode::DUP3
            | Opcode::DUP4
            | Opcode::DUP5
            | Opcode::DUP6
            | Opcode::DUP7
            | Opcode::DUP8
            | Opcode::DUP9
            | Opcode::DUP10
            | Opcode::DUP11
            | Opcode::DUP12
            | Opcode::DUP13
            | Opcode::DUP14
            | Opcode::DUP15
            | Opcode::DUP16 => gas::GAS_VERY_LOW,

            Opcode::SWAP1
            | Opcode::SWAP2
            | Opcode::SWAP3
            | Opcode::SWAP4
            | Opcode::SWAP5
            | Opcode::SWAP6
            | Opcode::SWAP7
            | Opcode::SWAP8
            | Opcode::SWAP9
            | Opcode::SWAP10
            | Opcode::SWAP11
            | Opcode::SWAP12
            | Opcode::SWAP13
            | Opcode::SWAP14
            | Opcode::SWAP15
            | Opcode::SWAP16 => gas::GAS_VERY_LOW,

            Opcode::MUL
            | Opcode::DIV
            | Opcode::SDIV
            | Opcode::MOD
            | Opcode::SMOD
            | Opcode::SIGNEXTEND => gas::GAS_LOW,

            Opcode::ADDMOD | Opcode::MULMOD | Opcode::JUMP => gas::GAS_MID,

            Opcode::JUMPI => gas::GAS_HIGH,

            Opcode::EXP => 10, // base cost, dynamic part not included here

            Opcode::MLOAD | Opcode::MSTORE | Opcode::MSTORE8 => gas::GAS_VERY_LOW,

            // SLOAD/SSTORE gas is dynamic (cold/warm), charged separately in run()
            Opcode::SLOAD | Opcode::SSTORE => gas::GAS_ZERO,

            // Environmental info
            Opcode::ADDRESS
            | Opcode::ORIGIN
            | Opcode::CALLER
            | Opcode::CALLVALUE
            | Opcode::CALLDATASIZE
            | Opcode::CODESIZE
            | Opcode::GASPRICE
            | Opcode::COINBASE
            | Opcode::TIMESTAMP
            | Opcode::NUMBER
            | Opcode::DIFFICULTY
            | Opcode::GASLIMIT
            | Opcode::CHAINID => gas::GAS_BASE,

            Opcode::SELFBALANCE => gas::GAS_LOW, // EIP-1884: costs 5

            Opcode::CALLDATALOAD => gas::GAS_VERY_LOW,
            Opcode::CALLDATACOPY | Opcode::CODECOPY => gas::GAS_VERY_LOW,
            Opcode::BALANCE => 100, // warm access cost (simplified)
            Opcode::BLOCKHASH => 20,
            Opcode::KECCAK256 => 30, // base cost, dynamic part not included here

            // LOG gas is dynamic, charged separately in run()
            Opcode::LOG0 | Opcode::LOG1 | Opcode::LOG2 | Opcode::LOG3 | Opcode::LOG4 => {
                gas::GAS_ZERO
            }

            Opcode::RETURN | Opcode::REVERT => gas::GAS_ZERO,

            Opcode::INVALID => gas::GAS_ZERO,
        }
    }

    /// Modular exponentiation: base^exp mod 2^256.
    fn u256_exp(mut base: U256, mut exp: U256) -> U256 {
        let mut result = U256::ONE;
        while !exp.is_zero() {
            // If lowest bit is set, multiply result by base.
            if exp.to_bytes()[31] & 1 == 1 {
                result = result * base;
            }
            exp = Self::u256_shr(exp, U256::ONE);
            base = base * base;
        }
        result
    }

    /// Logical shift left.
    fn u256_shl(value: U256, shift: U256) -> U256 {
        let shift = match shift.to_usize() {
            Some(s) if s < 256 => s,
            _ => return U256::ZERO,
        };
        let bytes = value.to_bytes();
        let byte_shift = shift / 8;
        let bit_shift = shift % 8;
        let mut result = [0u8; 32];
        for (i, r) in result.iter_mut().enumerate() {
            let src = i + byte_shift;
            if src < 32 {
                *r |= bytes[src] << bit_shift;
            }
            if bit_shift > 0 && src + 1 < 32 {
                *r |= bytes[src + 1] >> (8 - bit_shift);
            }
        }
        U256::from_bytes(result)
    }

    /// Logical shift right.
    fn u256_shr(value: U256, shift: U256) -> U256 {
        let shift = match shift.to_usize() {
            Some(s) if s < 256 => s,
            _ => return U256::ZERO,
        };
        let bytes = value.to_bytes();
        let byte_shift = shift / 8;
        let bit_shift = shift % 8;
        let mut result = [0u8; 32];
        for i in (0..32).rev() {
            if i >= byte_shift {
                let src = i - byte_shift;
                result[i] |= bytes[src] >> bit_shift;
                if bit_shift > 0 && src > 0 {
                    result[i] |= bytes[src - 1] << (8 - bit_shift);
                }
            }
        }
        U256::from_bytes(result)
    }
}

/// Helper: run bytecode with the given gas limit and return the result.
pub fn run_bytecode(bytecode: &[u8], gas_limit: u64) -> Result<ExecutionResult, EvmError> {
    let mut interp = Interpreter::new(bytecode.to_vec(), gas_limit);
    interp.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENOUGH_GAS: u64 = 1_000_000;

    // ---------------------------------------------------------------
    // Exercise 2.2: PUSH immediate decoding
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_2_push3_immediate() {
        // PUSH3 0x01 0x02 0x03 → pushes 0x010203
        let bytecode = vec![0x62, 0x01, 0x02, 0x03, 0x00]; // PUSH3 + STOP
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output, Vec::<u8>::new()); // STOP returns empty
    }

    // ---------------------------------------------------------------
    // Exercise 2.3: ADD opcode execution
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_3_add_two_values() {
        // PUSH1 3, PUSH1 5, ADD, PUSH1 0, MSTORE, PUSH1 32, PUSH1 0, RETURN
        // This pushes 3, pushes 5, adds (=8), stores at memory[0], returns 32 bytes
        let bytecode = vec![
            0x60, 0x03, // PUSH1 3
            0x60, 0x05, // PUSH1 5
            0x01, // ADD
            0x60, 0x00, // PUSH1 0
            0x52, // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xF3, // RETURN
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        // The return data should be 32 bytes with value 8 at the end.
        assert_eq!(result.output.len(), 32);
        assert_eq!(result.output[31], 8);
    }

    // ---------------------------------------------------------------
    // Exercise 2.4: Gas deduction for ADD
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_4_gas_accounting() {
        // PUSH1 3, PUSH1 5, ADD, STOP
        let bytecode = vec![
            0x60, 0x03, // PUSH1 3 (cost: 3)
            0x60, 0x05, // PUSH1 5 (cost: 3)
            0x01, // ADD       (cost: 3)
            0x00, // STOP      (cost: 0)
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.gas_used, 9); // 3 + 3 + 3 + 0
    }

    // ---------------------------------------------------------------
    // Exercise 2.5: MUL, SUB, DIV
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_5_mul() {
        // PUSH1 6, PUSH1 7, MUL → 42
        let bytecode = vec![
            0x60, 0x06, // PUSH1 6
            0x60, 0x07, // PUSH1 7
            0x02, // MUL
            0x60, 0x00, 0x52, // MSTORE at 0
            0x60, 0x20, 0x60, 0x00, 0xF3, // RETURN 32 bytes
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output[31], 42);
    }

    #[test]
    fn ex_2_5_sub() {
        // PUSH1 3, PUSH1 10, SUB → 10 - 3 = 7
        // Note: SUB pops a then b and computes a - b
        let bytecode = vec![
            0x60, 0x03, // PUSH1 3
            0x60, 0x0A, // PUSH1 10
            0x03, // SUB (10 - 3 = 7)
            0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3,
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output[31], 7);
    }

    #[test]
    fn ex_2_5_div() {
        // PUSH1 7, PUSH1 42, DIV → 42 / 7 = 6
        let bytecode = vec![
            0x60, 0x07, // PUSH1 7
            0x60, 0x2A, // PUSH1 42
            0x04, // DIV
            0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3,
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output[31], 6);
    }

    // ---------------------------------------------------------------
    // Exercise 2.6: Division by zero
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_6_div_by_zero() {
        // PUSH1 0, PUSH1 5, DIV → 5 / 0 = 0 (EVM convention)
        let bytecode = vec![
            0x60, 0x00, // PUSH1 0
            0x60, 0x05, // PUSH1 5
            0x04, // DIV
            0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3,
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output[31], 0);
    }

    // ---------------------------------------------------------------
    // Exercise 2.7: LT, GT, EQ
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_7_lt() {
        // PUSH1 10, PUSH1 5, LT → 5 < 10 = 1
        let bytecode = vec![
            0x60, 0x0A, // PUSH1 10
            0x60, 0x05, // PUSH1 5
            0x10, // LT (5 < 10 = true = 1)
            0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3,
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output[31], 1);
    }

    #[test]
    fn ex_2_7_gt() {
        // PUSH1 5, PUSH1 10, GT → 10 > 5 = 1
        let bytecode = vec![
            0x60, 0x05, // PUSH1 5
            0x60, 0x0A, // PUSH1 10
            0x11, // GT (10 > 5 = true = 1)
            0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3,
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output[31], 1);
    }

    #[test]
    fn ex_2_7_eq() {
        // PUSH1 42, PUSH1 42, EQ → 1
        let bytecode = vec![
            0x60, 0x2A, // PUSH1 42
            0x60, 0x2A, // PUSH1 42
            0x14, // EQ
            0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3,
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output[31], 1);
    }

    // ---------------------------------------------------------------
    // Exercise 2.8: Out-of-gas
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_8_out_of_gas() {
        // PUSH1 1 costs 3 gas. Run with gas_limit=1 → OutOfGas
        let bytecode = vec![0x60, 0x01]; // PUSH1 1
        let err = run_bytecode(&bytecode, 1).unwrap_err();
        assert_eq!(err, EvmError::OutOfGas);
    }

    #[test]
    fn ex_2_8_exact_gas() {
        // PUSH1 costs 3, STOP costs 0. With gas_limit=3, just enough.
        let bytecode = vec![0x60, 0x01, 0x00]; // PUSH1 1, STOP
        let result = run_bytecode(&bytecode, 3).unwrap();
        assert_eq!(result.gas_used, 3);
    }

    // ---------------------------------------------------------------
    // Exercise 2.9: STOP opcode
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_9_stop() {
        let bytecode = vec![0x00]; // STOP
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert!(result.output.is_empty());
        assert_eq!(result.gas_used, 0); // STOP costs 0
        assert!(!result.reverted);
    }

    // ---------------------------------------------------------------
    // Exercise 2.10: RETURN opcode
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_10_return_memory_range() {
        // Store 0xFF at memory[31], RETURN 32 bytes from offset 0.
        let bytecode = vec![
            0x60, 0xFF, // PUSH1 0xFF
            0x60, 0x00, // PUSH1 0
            0x52, // MSTORE (stores 0xFF at byte 31)
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xF3, // RETURN
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output.len(), 32);
        assert_eq!(result.output[31], 0xFF);
        assert!(!result.reverted);
    }

    #[test]
    fn ex_2_10_return_empty() {
        // RETURN with size 0
        let bytecode = vec![
            0x60, 0x00, // PUSH1 0 (size)
            0x60, 0x00, // PUSH1 0 (offset)
            0xF3, // RETURN
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert!(result.output.is_empty());
    }

    // ---------------------------------------------------------------
    // Integration: full arithmetic pipeline
    // ---------------------------------------------------------------
    #[test]
    fn integration_compute_and_return() {
        // Compute (3 + 5) * 2 = 16, return it
        let bytecode = vec![
            0x60, 0x03, // PUSH1 3
            0x60, 0x05, // PUSH1 5
            0x01, // ADD → 8
            0x60, 0x02, // PUSH1 2
            0x02, // MUL → 16
            0x60, 0x00, // PUSH1 0
            0x52, // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xF3, // RETURN
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output[31], 16);
    }

    // ---------------------------------------------------------------
    // Exercise 3.1: MLOAD/MSTORE round-trip via interpreter
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_1_mload_mstore_roundtrip() {
        // PUSH1 0xAB, PUSH1 0, MSTORE, PUSH1 0, MLOAD, PUSH1 0, MSTORE, RETURN 32
        let bytecode = vec![
            0x60, 0xAB, // PUSH1 0xAB
            0x60, 0x00, // PUSH1 0
            0x52, // MSTORE (stores 0xAB at mem[0..32])
            0x60, 0x00, // PUSH1 0
            0x51, // MLOAD (loads mem[0..32])
            0x60, 0x00, // PUSH1 0
            0x52, // MSTORE (store it back)
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xF3, // RETURN
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output[31], 0xAB);
    }

    // ---------------------------------------------------------------
    // Exercise 3.2: Memory expansion gas
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_2_memory_expansion_gas() {
        // Store at offset 0 (expands to 1 word = 3 gas expansion)
        let small = vec![
            0x60, 0x01, // PUSH1 1
            0x60, 0x00, // PUSH1 0
            0x52, // MSTORE
            0x00, // STOP
        ];
        let r1 = run_bytecode(&small, ENOUGH_GAS).unwrap();

        // Store at offset 992 (expands to 32 words)
        let large = vec![
            0x60, 0x01, // PUSH1 1
            0x61, 0x03, 0xE0, // PUSH2 992
            0x52, // MSTORE
            0x00, // STOP
        ];
        let r2 = run_bytecode(&large, ENOUGH_GAS).unwrap();

        // Large offset should cost more gas due to memory expansion.
        assert!(r2.gas_used > r1.gas_used);
    }

    // ---------------------------------------------------------------
    // Exercise 3.5: JUMPDEST validation
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_5_invalid_jump_target() {
        // PUSH1 0x02, JUMP — byte at offset 2 is the PUSH1 immediate, not JUMPDEST
        let bytecode = vec![
            0x60, 0x04, // PUSH1 4 (jump to offset 4)
            0x56, // JUMP
            0x00, // STOP (offset 3)
            0x60, 0x01, // PUSH1 1 (offset 4 — not a JUMPDEST!)
        ];
        let err = run_bytecode(&bytecode, ENOUGH_GAS).unwrap_err();
        assert_eq!(err, EvmError::InvalidJump);
    }

    // ---------------------------------------------------------------
    // Exercise 3.6: JUMP opcode
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_6_jump() {
        // PUSH1 4, JUMP, INVALID, JUMPDEST, STOP
        let bytecode = vec![
            0x60, 0x04, // PUSH1 4
            0x56, // JUMP
            0xFE, // INVALID (should be skipped)
            0x5B, // JUMPDEST (offset 4)
            0x00, // STOP
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert!(result.output.is_empty()); // reached STOP normally
    }

    // ---------------------------------------------------------------
    // Exercise 3.7: JUMPI true branch
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_7_jumpi_true() {
        // PUSH1 1 (condition=true), PUSH1 7, JUMPI, INVALID, ..., JUMPDEST, STOP
        let bytecode = vec![
            0x60, 0x01, // PUSH1 1 (nonzero = true)
            0x60, 0x07, // PUSH1 7 (dest)
            0x57, // JUMPI
            0xFE, // INVALID (skipped)
            0xFE, // INVALID (skipped)
            0x5B, // JUMPDEST (offset 7)
            0x00, // STOP
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert!(result.output.is_empty());
    }

    // ---------------------------------------------------------------
    // Exercise 3.8: JUMPI false branch
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_8_jumpi_false() {
        // PUSH1 0 (condition=false), PUSH1 7, JUMPI, PUSH1 42, STOP, ...
        let bytecode = vec![
            0x60, 0x00, // PUSH1 0 (zero = false)
            0x60, 0x07, // PUSH1 7 (dest — would jump here if true)
            0x57, // JUMPI (falls through)
            0x60, 0x2A, // PUSH1 42
            0x00, // STOP (offset 7 doesn't matter since we don't jump)
        ];
        // Just check it doesn't error — it should stop normally.
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert!(result.output.is_empty());
    }

    // ---------------------------------------------------------------
    // Exercise 3.9: CALLDATALOAD
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_9_calldataload() {
        use crate::context::ContextBuilder;

        // Calldata: 32 bytes with 0xAB at position 0
        let mut calldata = vec![0u8; 32];
        calldata[0] = 0xAB;

        let ctx = ContextBuilder::new().calldata(calldata).build();

        // CALLDATALOAD at offset 0, MSTORE, RETURN 32 bytes
        let bytecode = vec![
            0x60, 0x00, // PUSH1 0
            0x35, // CALLDATALOAD
            0x60, 0x00, // PUSH1 0
            0x52, // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xF3, // RETURN
        ];

        let mut interp = Interpreter::with_context(bytecode, ENOUGH_GAS, ctx, Storage::new());
        let result = interp.run().unwrap();
        assert_eq!(result.output[0], 0xAB);
    }

    // ---------------------------------------------------------------
    // Exercise 3.10: Loop in bytecode
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_10_loop_in_bytecode() {
        // Implement: i = 0; while (i < 5) { i++ }; return i
        // This is a real bytecode loop using JUMP/JUMPI.
        //
        // Pseudocode:
        // Bytecode layout (i = 0; while i < 5: i++; return i):
        //
        //  0: PUSH1 0       — i = 0
        //  2: JUMPDEST       — loop_start
        //  3: PUSH1 5        — push limit
        //  5: DUP2           — copy i to top: stack = [..., i, 5, i]
        //  6: LT             — pops a=i, b=5, pushes (i < 5)
        //  7: PUSH1 18       — loop_body offset
        //  9: JUMPI          — jump if i < 5
        // 10: PUSH1 0        — offset for MSTORE
        // 12: MSTORE         — store i at mem[0]
        // 13: PUSH1 32       — return size
        // 15: PUSH1 0        — return offset
        // 17: RETURN
        // 18: JUMPDEST       — loop_body
        // 19: PUSH1 1
        // 21: ADD             — i = i + 1
        // 22: PUSH1 2        — loop_start offset
        // 24: JUMP
        let bytecode = vec![
            0x60, 0x00, // 0: PUSH1 0
            0x5B, // 2: JUMPDEST (loop_start)
            0x60, 0x05, // 3: PUSH1 5
            0x81, // 5: DUP2 (copy i)
            0x10, // 6: LT (i < 5)
            0x60, 0x12, // 7: PUSH1 18 (loop_body)
            0x57, // 9: JUMPI
            0x60, 0x00, // 10: PUSH1 0
            0x52, // 12: MSTORE
            0x60, 0x20, // 13: PUSH1 32
            0x60, 0x00, // 15: PUSH1 0
            0xF3, // 17: RETURN
            0x5B, // 18: JUMPDEST (loop_body)
            0x60, 0x01, // 19: PUSH1 1
            0x01, // 21: ADD
            0x60, 0x02, // 22: PUSH1 2 (loop_start)
            0x56, // 24: JUMP
        ];

        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        // After the loop, i = 5
        assert_eq!(result.output[31], 5);
    }

    // ---------------------------------------------------------------
    // Exercise 3.3/3.4: SSTORE/SLOAD via interpreter
    // ---------------------------------------------------------------
    #[test]
    fn ex_3_3_sstore_sload_interpreter() {
        // PUSH1 42, PUSH1 0, SSTORE — stores 42 at slot 0
        // PUSH1 0, SLOAD — loads slot 0
        // PUSH1 0, MSTORE, RETURN 32 bytes
        let bytecode = vec![
            0x60, 0x2A, // PUSH1 42
            0x60, 0x00, // PUSH1 0 (slot)
            0x55, // SSTORE
            0x60, 0x00, // PUSH1 0 (slot)
            0x54, // SLOAD
            0x60, 0x00, // PUSH1 0
            0x52, // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xF3, // RETURN
        ];
        let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();
        assert_eq!(result.output[31], 42);
    }

    #[test]
    fn ex_3_4_cold_warm_sload_gas() {
        // First SLOAD is cold (2100 gas), second SLOAD is warm (100 gas).
        // We measure gas difference.
        let bytecode_cold = vec![
            0x60, 0x00, // PUSH1 0
            0x54, // SLOAD (cold)
            0x50, // POP
            0x00, // STOP
        ];
        let r1 = run_bytecode(&bytecode_cold, ENOUGH_GAS).unwrap();

        // Two SLOADs to the same slot
        let bytecode_warm = vec![
            0x60, 0x00, // PUSH1 0
            0x54, // SLOAD (cold)
            0x50, // POP
            0x60, 0x00, // PUSH1 0
            0x54, // SLOAD (warm)
            0x50, // POP
            0x00, // STOP
        ];
        let r2 = run_bytecode(&bytecode_warm, ENOUGH_GAS).unwrap();

        // The gas difference between r2 and r1 should be approximately:
        // PUSH1(3) + SLOAD_warm(100) + POP(2) = 105
        // (not SLOAD_cold(2100))
        let diff = r2.gas_used - r1.gas_used;
        assert!(
            diff < 200,
            "Second SLOAD should be warm (cheap), but gas diff was {diff}"
        );
    }
}

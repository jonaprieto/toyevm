//! Intermediate representation for EVM bytecode analysis.
//!
//! We lift flat bytecode into a structured IR of basic blocks and instructions,
//! which enables control flow analysis and optimization.

use crate::opcode::Opcode;

/// A unique identifier for a basic block within a CFG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub usize);

/// A single instruction in the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// The offset in the original bytecode.
    pub offset: usize,
    /// The opcode.
    pub opcode: Opcode,
    /// Immediate bytes (for PUSH instructions).
    pub immediate: Vec<u8>,
}

impl Instruction {
    /// How many stack elements this instruction consumes (pops).
    pub fn stack_input(&self) -> usize {
        match self.opcode {
            Opcode::STOP
            | Opcode::JUMPDEST
            | Opcode::PC
            | Opcode::MSIZE
            | Opcode::GAS
            | Opcode::PUSH0 => 0,
            op if op.is_push() => 0,
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
            | Opcode::CHAINID
            | Opcode::SELFBALANCE => 0,
            Opcode::POP
            | Opcode::JUMP
            | Opcode::ISZERO
            | Opcode::NOT
            | Opcode::BALANCE
            | Opcode::CALLDATALOAD
            | Opcode::BLOCKHASH
            | Opcode::SLOAD
            | Opcode::MLOAD => 1,
            Opcode::ADD
            | Opcode::MUL
            | Opcode::SUB
            | Opcode::DIV
            | Opcode::SDIV
            | Opcode::MOD
            | Opcode::SMOD
            | Opcode::EXP
            | Opcode::SIGNEXTEND
            | Opcode::LT
            | Opcode::GT
            | Opcode::SLT
            | Opcode::SGT
            | Opcode::EQ
            | Opcode::AND
            | Opcode::OR
            | Opcode::XOR
            | Opcode::BYTE
            | Opcode::SHL
            | Opcode::SHR
            | Opcode::SAR
            | Opcode::JUMPI
            | Opcode::MSTORE
            | Opcode::MSTORE8
            | Opcode::SSTORE
            | Opcode::RETURN
            | Opcode::REVERT => 2,
            Opcode::ADDMOD
            | Opcode::MULMOD
            | Opcode::CALLDATACOPY
            | Opcode::CODECOPY
            | Opcode::KECCAK256 => 3,
            Opcode::LOG0 => 2,
            Opcode::LOG1 => 3,
            Opcode::LOG2 => 4,
            Opcode::LOG3 => 5,
            Opcode::LOG4 => 6,
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
            | Opcode::DUP16 => (self.opcode as u8 - Opcode::DUP1 as u8 + 1) as usize,
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
            | Opcode::SWAP16 => (self.opcode as u8 - Opcode::SWAP1 as u8 + 2) as usize,
            Opcode::INVALID => 0,
            // All remaining opcodes (PUSH1..PUSH32) consume 0
            _ => 0,
        }
    }

    /// How many stack elements this instruction produces (pushes).
    pub fn stack_output(&self) -> usize {
        match self.opcode {
            Opcode::STOP | Opcode::RETURN | Opcode::REVERT | Opcode::INVALID | Opcode::JUMP => 0,
            Opcode::POP
            | Opcode::MSTORE
            | Opcode::MSTORE8
            | Opcode::SSTORE
            | Opcode::JUMPDEST
            | Opcode::CALLDATACOPY
            | Opcode::CODECOPY
            | Opcode::LOG0
            | Opcode::LOG1
            | Opcode::LOG2
            | Opcode::LOG3
            | Opcode::LOG4 => 0,
            Opcode::JUMPI => 0,
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
            | Opcode::DUP16 => (self.opcode as u8 - Opcode::DUP1 as u8 + 2) as usize,
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
            | Opcode::SWAP16 => (self.opcode as u8 - Opcode::SWAP1 as u8 + 2) as usize,
            // Everything else pushes 1 value
            _ => 1,
        }
    }
}

/// A basic block: a maximal sequence of instructions with no internal jumps.
/// Control flow enters at the first instruction and exits at the last.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier.
    pub id: BlockId,
    /// The starting byte offset in the original bytecode.
    pub start_offset: usize,
    /// The ending byte offset (exclusive).
    pub end_offset: usize,
    /// Instructions in this block.
    pub instructions: Vec<Instruction>,
    /// IDs of successor blocks.
    pub successors: Vec<BlockId>,
    /// IDs of predecessor blocks.
    pub predecessors: Vec<BlockId>,
}

impl BasicBlock {
    /// Returns true if this block ends with a terminator (STOP, RETURN, REVERT, JUMP, JUMPI, INVALID).
    pub fn is_terminated(&self) -> bool {
        self.instructions.last().is_some_and(|inst| {
            matches!(
                inst.opcode,
                Opcode::STOP
                    | Opcode::RETURN
                    | Opcode::REVERT
                    | Opcode::JUMP
                    | Opcode::JUMPI
                    | Opcode::INVALID
            )
        })
    }

    /// Returns the last opcode in this block, if any.
    pub fn terminator(&self) -> Option<Opcode> {
        self.instructions.last().map(|i| i.opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_stack_effects() {
        let add = Instruction {
            offset: 0,
            opcode: Opcode::ADD,
            immediate: vec![],
        };
        assert_eq!(add.stack_input(), 2);
        assert_eq!(add.stack_output(), 1);

        let push1 = Instruction {
            offset: 0,
            opcode: Opcode::PUSH1,
            immediate: vec![0x42],
        };
        assert_eq!(push1.stack_input(), 0);
        assert_eq!(push1.stack_output(), 1);

        let stop = Instruction {
            offset: 0,
            opcode: Opcode::STOP,
            immediate: vec![],
        };
        assert_eq!(stop.stack_input(), 0);
        assert_eq!(stop.stack_output(), 0);
    }
}

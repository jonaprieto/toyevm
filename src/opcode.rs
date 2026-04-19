/// EVM opcodes represented as a `repr(u8)` enum.
///
/// Each variant maps directly to its single-byte encoding in EVM bytecode.
/// The compiler generates a jump table for `match` on this enum, making
/// dispatch O(1) — much faster than a HashMap lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // 0x00: Stop and Arithmetic
    STOP = 0x00,
    ADD = 0x01,
    MUL = 0x02,
    SUB = 0x03,
    DIV = 0x04,
    SDIV = 0x05,
    MOD = 0x06,
    SMOD = 0x07,
    ADDMOD = 0x08,
    MULMOD = 0x09,
    EXP = 0x0A,
    SIGNEXTEND = 0x0B,

    // 0x10: Comparison and Bitwise Logic
    LT = 0x10,
    GT = 0x11,
    SLT = 0x12,
    SGT = 0x13,
    EQ = 0x14,
    ISZERO = 0x15,
    AND = 0x16,
    OR = 0x17,
    XOR = 0x18,
    NOT = 0x19,
    BYTE = 0x1A,
    SHL = 0x1B,
    SHR = 0x1C,
    SAR = 0x1D,

    // 0x20: Keccak256
    KECCAK256 = 0x20,

    // 0x30: Environmental information
    ADDRESS = 0x30,
    BALANCE = 0x31,
    ORIGIN = 0x32,
    CALLER = 0x33,
    CALLVALUE = 0x34,
    CALLDATALOAD = 0x35,
    CALLDATASIZE = 0x36,
    CALLDATACOPY = 0x37,
    CODESIZE = 0x38,
    CODECOPY = 0x39,
    GASPRICE = 0x3A,

    // 0x40: Block information
    BLOCKHASH = 0x40,
    COINBASE = 0x41,
    TIMESTAMP = 0x42,
    NUMBER = 0x43,
    DIFFICULTY = 0x44,
    GASLIMIT = 0x45,
    CHAINID = 0x46,
    SELFBALANCE = 0x47,

    // 0x50: Stack, Memory, Storage, Flow
    POP = 0x50,
    MLOAD = 0x51,
    MSTORE = 0x52,
    MSTORE8 = 0x53,
    SLOAD = 0x54,
    SSTORE = 0x55,
    JUMP = 0x56,
    JUMPI = 0x57,
    PC = 0x58,
    MSIZE = 0x59,
    GAS = 0x5A,
    JUMPDEST = 0x5B,

    // 0x5F-0x7F: Push operations
    PUSH0 = 0x5F,
    PUSH1 = 0x60,
    PUSH2 = 0x61,
    PUSH3 = 0x62,
    PUSH4 = 0x63,
    PUSH5 = 0x64,
    PUSH6 = 0x65,
    PUSH7 = 0x66,
    PUSH8 = 0x67,
    PUSH9 = 0x68,
    PUSH10 = 0x69,
    PUSH11 = 0x6A,
    PUSH12 = 0x6B,
    PUSH13 = 0x6C,
    PUSH14 = 0x6D,
    PUSH15 = 0x6E,
    PUSH16 = 0x6F,
    PUSH17 = 0x70,
    PUSH18 = 0x71,
    PUSH19 = 0x72,
    PUSH20 = 0x73,
    PUSH21 = 0x74,
    PUSH22 = 0x75,
    PUSH23 = 0x76,
    PUSH24 = 0x77,
    PUSH25 = 0x78,
    PUSH26 = 0x79,
    PUSH27 = 0x7A,
    PUSH28 = 0x7B,
    PUSH29 = 0x7C,
    PUSH30 = 0x7D,
    PUSH31 = 0x7E,
    PUSH32 = 0x7F,

    // 0x80-0x8F: Duplication operations
    DUP1 = 0x80,
    DUP2 = 0x81,
    DUP3 = 0x82,
    DUP4 = 0x83,
    DUP5 = 0x84,
    DUP6 = 0x85,
    DUP7 = 0x86,
    DUP8 = 0x87,
    DUP9 = 0x88,
    DUP10 = 0x89,
    DUP11 = 0x8A,
    DUP12 = 0x8B,
    DUP13 = 0x8C,
    DUP14 = 0x8D,
    DUP15 = 0x8E,
    DUP16 = 0x8F,

    // 0x90-0x9F: Exchange operations
    SWAP1 = 0x90,
    SWAP2 = 0x91,
    SWAP3 = 0x92,
    SWAP4 = 0x93,
    SWAP5 = 0x94,
    SWAP6 = 0x95,
    SWAP7 = 0x96,
    SWAP8 = 0x97,
    SWAP9 = 0x98,
    SWAP10 = 0x99,
    SWAP11 = 0x9A,
    SWAP12 = 0x9B,
    SWAP13 = 0x9C,
    SWAP14 = 0x9D,
    SWAP15 = 0x9E,
    SWAP16 = 0x9F,

    // 0xF3, 0xFD: Return and Revert
    RETURN = 0xF3,
    REVERT = 0xFD,

    // 0xFE: Invalid
    INVALID = 0xFE,
}

impl Opcode {
    /// Decode a single byte into an opcode.
    /// Returns `None` for bytes that don't correspond to a known opcode.
    pub fn from_byte(byte: u8) -> Option<Self> {
        // We use a match rather than transmute for safety — only known opcodes
        // are valid. Unknown bytes return None, which the interpreter treats
        // as an invalid opcode error.
        match byte {
            0x00 => Some(Self::STOP),
            0x01 => Some(Self::ADD),
            0x02 => Some(Self::MUL),
            0x03 => Some(Self::SUB),
            0x04 => Some(Self::DIV),
            0x05 => Some(Self::SDIV),
            0x06 => Some(Self::MOD),
            0x07 => Some(Self::SMOD),
            0x08 => Some(Self::ADDMOD),
            0x09 => Some(Self::MULMOD),
            0x0A => Some(Self::EXP),
            0x0B => Some(Self::SIGNEXTEND),
            0x10 => Some(Self::LT),
            0x11 => Some(Self::GT),
            0x12 => Some(Self::SLT),
            0x13 => Some(Self::SGT),
            0x14 => Some(Self::EQ),
            0x15 => Some(Self::ISZERO),
            0x16 => Some(Self::AND),
            0x17 => Some(Self::OR),
            0x18 => Some(Self::XOR),
            0x19 => Some(Self::NOT),
            0x1A => Some(Self::BYTE),
            0x1B => Some(Self::SHL),
            0x1C => Some(Self::SHR),
            0x1D => Some(Self::SAR),
            0x20 => Some(Self::KECCAK256),
            0x30 => Some(Self::ADDRESS),
            0x31 => Some(Self::BALANCE),
            0x32 => Some(Self::ORIGIN),
            0x33 => Some(Self::CALLER),
            0x34 => Some(Self::CALLVALUE),
            0x35 => Some(Self::CALLDATALOAD),
            0x36 => Some(Self::CALLDATASIZE),
            0x37 => Some(Self::CALLDATACOPY),
            0x38 => Some(Self::CODESIZE),
            0x39 => Some(Self::CODECOPY),
            0x3A => Some(Self::GASPRICE),
            0x40 => Some(Self::BLOCKHASH),
            0x41 => Some(Self::COINBASE),
            0x42 => Some(Self::TIMESTAMP),
            0x43 => Some(Self::NUMBER),
            0x44 => Some(Self::DIFFICULTY),
            0x45 => Some(Self::GASLIMIT),
            0x46 => Some(Self::CHAINID),
            0x47 => Some(Self::SELFBALANCE),
            0x50 => Some(Self::POP),
            0x51 => Some(Self::MLOAD),
            0x52 => Some(Self::MSTORE),
            0x53 => Some(Self::MSTORE8),
            0x54 => Some(Self::SLOAD),
            0x55 => Some(Self::SSTORE),
            0x56 => Some(Self::JUMP),
            0x57 => Some(Self::JUMPI),
            0x58 => Some(Self::PC),
            0x59 => Some(Self::MSIZE),
            0x5A => Some(Self::GAS),
            0x5B => Some(Self::JUMPDEST),
            0x5F => Some(Self::PUSH0),
            0x60..=0x7F => {
                // PUSH1 through PUSH32
                // Safety: we manually map each value
                let push_opcodes = [
                    Self::PUSH1,
                    Self::PUSH2,
                    Self::PUSH3,
                    Self::PUSH4,
                    Self::PUSH5,
                    Self::PUSH6,
                    Self::PUSH7,
                    Self::PUSH8,
                    Self::PUSH9,
                    Self::PUSH10,
                    Self::PUSH11,
                    Self::PUSH12,
                    Self::PUSH13,
                    Self::PUSH14,
                    Self::PUSH15,
                    Self::PUSH16,
                    Self::PUSH17,
                    Self::PUSH18,
                    Self::PUSH19,
                    Self::PUSH20,
                    Self::PUSH21,
                    Self::PUSH22,
                    Self::PUSH23,
                    Self::PUSH24,
                    Self::PUSH25,
                    Self::PUSH26,
                    Self::PUSH27,
                    Self::PUSH28,
                    Self::PUSH29,
                    Self::PUSH30,
                    Self::PUSH31,
                    Self::PUSH32,
                ];
                Some(push_opcodes[(byte - 0x60) as usize])
            }
            0x80..=0x8F => {
                let dup_opcodes = [
                    Self::DUP1,
                    Self::DUP2,
                    Self::DUP3,
                    Self::DUP4,
                    Self::DUP5,
                    Self::DUP6,
                    Self::DUP7,
                    Self::DUP8,
                    Self::DUP9,
                    Self::DUP10,
                    Self::DUP11,
                    Self::DUP12,
                    Self::DUP13,
                    Self::DUP14,
                    Self::DUP15,
                    Self::DUP16,
                ];
                Some(dup_opcodes[(byte - 0x80) as usize])
            }
            0x90..=0x9F => {
                let swap_opcodes = [
                    Self::SWAP1,
                    Self::SWAP2,
                    Self::SWAP3,
                    Self::SWAP4,
                    Self::SWAP5,
                    Self::SWAP6,
                    Self::SWAP7,
                    Self::SWAP8,
                    Self::SWAP9,
                    Self::SWAP10,
                    Self::SWAP11,
                    Self::SWAP12,
                    Self::SWAP13,
                    Self::SWAP14,
                    Self::SWAP15,
                    Self::SWAP16,
                ];
                Some(swap_opcodes[(byte - 0x90) as usize])
            }
            0xF3 => Some(Self::RETURN),
            0xFD => Some(Self::REVERT),
            0xFE => Some(Self::INVALID),
            _ => None,
        }
    }

    /// Returns the number of immediate bytes following this opcode.
    /// PUSH1 has 1, PUSH2 has 2, ..., PUSH32 has 32. All others have 0.
    pub fn immediate_size(&self) -> usize {
        let byte = *self as u8;
        if (0x60..=0x7F).contains(&byte) {
            (byte - 0x5F) as usize // PUSH1=1, PUSH2=2, ..., PUSH32=32
        } else {
            0
        }
    }

    /// Returns true if this is a PUSH opcode (PUSH0 through PUSH32).
    pub fn is_push(&self) -> bool {
        let byte = *self as u8;
        (0x5F..=0x7F).contains(&byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Exercise 2.1: Opcode decoding
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_1_known_opcodes_decode() {
        assert_eq!(Opcode::from_byte(0x00), Some(Opcode::STOP));
        assert_eq!(Opcode::from_byte(0x01), Some(Opcode::ADD));
        assert_eq!(Opcode::from_byte(0x60), Some(Opcode::PUSH1));
        assert_eq!(Opcode::from_byte(0x7F), Some(Opcode::PUSH32));
        assert_eq!(Opcode::from_byte(0xF3), Some(Opcode::RETURN));
    }

    #[test]
    fn ex_2_1_unknown_byte_returns_none() {
        // 0xFF is not a valid opcode
        assert_eq!(Opcode::from_byte(0xFF), None);
        // 0x21 is in a gap
        assert_eq!(Opcode::from_byte(0x21), None);
    }

    #[test]
    fn ex_2_1_dup_and_swap_decode() {
        assert_eq!(Opcode::from_byte(0x80), Some(Opcode::DUP1));
        assert_eq!(Opcode::from_byte(0x8F), Some(Opcode::DUP16));
        assert_eq!(Opcode::from_byte(0x90), Some(Opcode::SWAP1));
        assert_eq!(Opcode::from_byte(0x9F), Some(Opcode::SWAP16));
    }

    // ---------------------------------------------------------------
    // Exercise 2.2: PUSH immediate sizes
    // ---------------------------------------------------------------
    #[test]
    fn ex_2_2_push_immediate_sizes() {
        assert_eq!(Opcode::PUSH0.immediate_size(), 0);
        assert_eq!(Opcode::PUSH1.immediate_size(), 1);
        assert_eq!(Opcode::PUSH2.immediate_size(), 2);
        assert_eq!(Opcode::PUSH16.immediate_size(), 16);
        assert_eq!(Opcode::PUSH32.immediate_size(), 32);
    }

    #[test]
    fn ex_2_2_non_push_has_zero_immediates() {
        assert_eq!(Opcode::ADD.immediate_size(), 0);
        assert_eq!(Opcode::STOP.immediate_size(), 0);
        assert_eq!(Opcode::RETURN.immediate_size(), 0);
    }
}

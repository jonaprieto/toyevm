//! Static gas costs for EVM opcodes.
//!
//! These are the "base" costs that don't depend on operand values or memory
//! expansion. Dynamic costs (e.g., memory expansion, EXP exponent size)
//! are computed at execution time.
//!
//! Reference: Yellow Paper Appendix H, EIP-2929 (Berlin), EIP-3529 (London).

/// Gas cost of zero: FREE opcodes (STOP, JUMPDEST).
pub const GAS_ZERO: u64 = 0;

/// Gas cost of base: opcodes like ADDRESS, ORIGIN, CALLER, etc.
pub const GAS_BASE: u64 = 2;

/// Gas cost of very low: ADD, SUB, LT, GT, EQ, ISZERO, AND, OR, XOR, NOT, etc.
pub const GAS_VERY_LOW: u64 = 3;

/// Gas cost of low: MUL, DIV, SDIV, MOD, SMOD, SIGNEXTEND.
pub const GAS_LOW: u64 = 5;

/// Gas cost of mid: ADDMOD, MULMOD, JUMP.
pub const GAS_MID: u64 = 8;

/// Gas cost of high: JUMPI.
pub const GAS_HIGH: u64 = 10;

/// Gas cost to compute the memory expansion cost.
/// Memory cost = 3 * words + words^2 / 512
pub fn memory_gas_cost(word_count: usize) -> u64 {
    let words = word_count as u64;
    3 * words + words * words / 512
}

/// Compute the gas cost of expanding memory from `current_words` to `new_words`.
/// Returns 0 if no expansion is needed.
pub fn memory_expansion_cost(current_words: usize, new_words: usize) -> u64 {
    if new_words <= current_words {
        return 0;
    }
    memory_gas_cost(new_words) - memory_gas_cost(current_words)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_cost_zero_words() {
        assert_eq!(memory_gas_cost(0), 0);
    }

    #[test]
    fn memory_cost_one_word() {
        // 3 * 1 + 1 / 512 = 3
        assert_eq!(memory_gas_cost(1), 3);
    }

    #[test]
    fn memory_cost_quadratic_growth() {
        // At 32 words: 3*32 + 32*32/512 = 96 + 2 = 98
        assert_eq!(memory_gas_cost(32), 98);
        // The quadratic term grows faster — this is by design.
        assert!(memory_gas_cost(100) > 3 * 100);
    }

    #[test]
    fn expansion_cost_no_expansion() {
        assert_eq!(memory_expansion_cost(5, 5), 0);
        assert_eq!(memory_expansion_cost(5, 3), 0);
    }

    #[test]
    fn expansion_cost_from_zero() {
        assert_eq!(memory_expansion_cost(0, 1), memory_gas_cost(1));
    }
}

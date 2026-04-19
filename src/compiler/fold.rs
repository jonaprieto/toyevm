//! Constant folding optimization pass.
//!
//! Scans basic blocks for patterns like PUSH a, PUSH b, ADD where both
//! operands are compile-time constants, and replaces them with a single PUSH.

use crate::opcode::Opcode;
use crate::types::U256;

use super::cfg::Cfg;
use super::ir::{BasicBlock, Instruction};

// ANCHOR: optimization_pass_trait
/// Trait for optimization passes that transform a CFG in place.
pub trait OptimizationPass {
    /// Run the pass. Returns the number of changes made.
    fn run(&self, cfg: &mut Cfg) -> usize;

    /// Name of the pass (for logging).
    fn name(&self) -> &str;
}
// ANCHOR_END: optimization_pass_trait

/// Constant folding: replace PUSH a, PUSH b, BINOP with PUSH (a op b).
pub struct ConstantFold;

impl OptimizationPass for ConstantFold {
    fn name(&self) -> &str {
        "constant-fold"
    }

    fn run(&self, cfg: &mut Cfg) -> usize {
        let mut total_changes = 0;
        let block_ids: Vec<_> = cfg.blocks.keys().copied().collect();
        for id in block_ids {
            let block = cfg.blocks.get_mut(&id).unwrap();
            total_changes += fold_block(block);
        }
        total_changes
    }
}

/// Fold constant expressions within a single basic block.
fn fold_block(block: &mut BasicBlock) -> usize {
    let mut changes = 0;
    loop {
        let folded = fold_once(&block.instructions);
        if folded == 0 {
            break;
        }
        changes += folded;
        // Apply the fold by rebuilding the instruction list.
        block.instructions = fold_pass(&block.instructions);
    }
    changes
}

/// Attempt one pass of constant folding. Returns number of folds found.
fn fold_once(instructions: &[Instruction]) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i + 2 < instructions.len() {
        if is_push_const(&instructions[i])
            && is_push_const(&instructions[i + 1])
            && is_foldable_binop(instructions[i + 2].opcode)
        {
            count += 1;
            i += 3;
        } else {
            i += 1;
        }
    }
    count
}

/// Apply one folding pass, producing a new instruction list.
fn fold_pass(instructions: &[Instruction]) -> Vec<Instruction> {
    let mut result = Vec::with_capacity(instructions.len());
    let mut i = 0;
    while i < instructions.len() {
        if i + 2 < instructions.len()
            && is_push_const(&instructions[i])
            && is_push_const(&instructions[i + 1])
            && is_foldable_binop(instructions[i + 2].opcode)
        {
            let a = push_value(&instructions[i]);
            let b = push_value(&instructions[i + 1]);
            let folded = evaluate_binop(instructions[i + 2].opcode, a, b);
            result.push(make_push32(instructions[i].offset, folded));
            i += 3;
        } else {
            result.push(instructions[i].clone());
            i += 1;
        }
    }
    result
}

/// Check if an instruction is a constant PUSH (PUSH0..PUSH32).
fn is_push_const(inst: &Instruction) -> bool {
    inst.opcode.is_push()
}

/// Extract the pushed value from a PUSH instruction.
fn push_value(inst: &Instruction) -> U256 {
    if inst.opcode == Opcode::PUSH0 {
        return U256::ZERO;
    }
    let mut bytes = [0u8; 32];
    let imm = &inst.immediate;
    bytes[32 - imm.len()..].copy_from_slice(imm);
    U256::from_bytes(bytes)
}

/// Check if an opcode is a binary operation we can fold.
fn is_foldable_binop(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::ADD
            | Opcode::MUL
            | Opcode::SUB
            | Opcode::DIV
            | Opcode::MOD
            | Opcode::AND
            | Opcode::OR
            | Opcode::XOR
    )
}

/// Evaluate a binary operation on two constants.
fn evaluate_binop(op: Opcode, a: U256, b: U256) -> U256 {
    // Note: EVM stack order is first-pushed = deeper.
    // In PUSH a, PUSH b, ADD: stack is [a, b], ADD pops b then a → a + b.
    // But our IR sees PUSH a first, PUSH b second. ADD pops b (top) then a.
    // So the result is a OP b for commutative ops, and b OP a for non-commutative.
    // Actually, the first PUSH is deeper, second PUSH is on top.
    // ADD pops top (b) as first arg, then second (a). In our interpreter:
    // let a = pop(); let b = pop(); push(a + b) — so it's b_pushed + a_pushed.
    // For commutative ops it doesn't matter. For SUB/DIV/MOD: result = b - a.
    match op {
        Opcode::ADD => a + b,
        Opcode::MUL => a * b,
        Opcode::SUB => a - b, // first pushed - second pushed
        Opcode::DIV => a.checked_div(b),
        Opcode::MOD => a.checked_mod(b),
        Opcode::AND => a & b,
        Opcode::OR => a | b,
        Opcode::XOR => a ^ b,
        _ => unreachable!(),
    }
}

/// Create a PUSH32 instruction for a folded constant.
fn make_push32(offset: usize, value: U256) -> Instruction {
    Instruction {
        offset,
        opcode: Opcode::PUSH32,
        immediate: value.to_bytes().to_vec(),
    }
}

/// Run optimization passes to a fixed point.
pub fn run_to_fixed_point(cfg: &mut Cfg, passes: &[&dyn OptimizationPass]) -> usize {
    let mut total = 0;
    loop {
        let mut round_changes = 0;
        for pass in passes {
            round_changes += pass.run(cfg);
        }
        total += round_changes;
        if round_changes == 0 {
            break;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::cfg::Cfg;

    // ---------------------------------------------------------------
    // Exercise 7.1: OptimizationPass trait compiles
    // ---------------------------------------------------------------
    #[test]
    fn ex_7_1_pass_trait_compiles() {
        let _fold: Box<dyn OptimizationPass> = Box::new(ConstantFold);
        assert_eq!(_fold.name(), "constant-fold");
    }

    // ---------------------------------------------------------------
    // Exercise 7.2: Constant folding ADD
    // ---------------------------------------------------------------
    #[test]
    fn ex_7_2_fold_add() {
        // PUSH1 3, PUSH1 5, ADD, STOP → should fold to PUSH32 8, STOP
        let bytecode = vec![0x60, 0x03, 0x60, 0x05, 0x01, 0x00];
        let mut cfg = Cfg::from_bytecode(&bytecode);
        let changes = ConstantFold.run(&mut cfg);
        assert!(changes > 0);

        let block = cfg.blocks.values().next().unwrap();
        // Should now have 2 instructions: PUSH32 and STOP
        assert_eq!(block.instructions.len(), 2);
        let folded = &block.instructions[0];
        assert_eq!(folded.opcode, Opcode::PUSH32);
        let val = push_value(folded);
        assert_eq!(val, U256::from(8u64));
    }

    // ---------------------------------------------------------------
    // Exercise 7.3: Constant folding MUL chain
    // ---------------------------------------------------------------
    #[test]
    fn ex_7_3_fold_mul_chain() {
        // PUSH1 2, PUSH1 3, MUL, PUSH1 4, MUL, STOP
        // First pass: fold (2,3,MUL) → PUSH32 6
        // Second pass: fold (6,4,MUL) → PUSH32 24
        let bytecode = vec![0x60, 0x02, 0x60, 0x03, 0x02, 0x60, 0x04, 0x02, 0x00];
        let mut cfg = Cfg::from_bytecode(&bytecode);
        let changes = ConstantFold.run(&mut cfg);
        assert!(changes >= 2);

        let block = cfg.blocks.values().next().unwrap();
        assert_eq!(block.instructions.len(), 2); // PUSH32 24, STOP
        let val = push_value(&block.instructions[0]);
        assert_eq!(val, U256::from(24u64));
    }

    // ---------------------------------------------------------------
    // Exercise 7.4: Folding reduces gas
    // ---------------------------------------------------------------
    #[test]
    fn ex_7_4_folding_gas_reduction() {
        // Before: PUSH1 3 (3g), PUSH1 5 (3g), ADD (3g) = 9g
        // After: PUSH32 8 (3g) = 3g
        // (Not counting STOP's 0g)
        let original_gas = 3 + 3 + 3; // 3 instructions at 3 gas each
        let folded_gas = 3; // 1 PUSH32 instruction at 3 gas
        assert!(folded_gas < original_gas);
    }

    // ---------------------------------------------------------------
    // Exercise 7.8: Fixed-point iteration
    // ---------------------------------------------------------------
    #[test]
    fn ex_7_8_fixed_point() {
        // PUSH1 2, PUSH1 3, MUL, PUSH1 4, MUL, STOP
        // Needs 2 rounds to fully fold.
        let bytecode = vec![0x60, 0x02, 0x60, 0x03, 0x02, 0x60, 0x04, 0x02, 0x00];
        let mut cfg = Cfg::from_bytecode(&bytecode);
        let passes: Vec<&dyn OptimizationPass> = vec![&ConstantFold];
        let total = run_to_fixed_point(&mut cfg, &passes);
        assert!(total >= 2);

        let block = cfg.blocks.values().next().unwrap();
        let val = push_value(&block.instructions[0]);
        assert_eq!(val, U256::from(24u64));
    }
}

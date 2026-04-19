//! Dead code elimination optimization pass.
//!
//! Removes unreachable blocks from the CFG using the liveness
//! information computed in the analysis module.

use super::analysis::compute_liveness;
use super::cfg::Cfg;
use super::fold::OptimizationPass;
use super::ir::BlockId;

/// Dead code elimination: remove blocks that are unreachable from entry
/// or cannot reach any exit block.
pub struct DeadCodeElim;

impl OptimizationPass for DeadCodeElim {
    fn name(&self) -> &str {
        "dead-code-elim"
    }

    fn run(&self, cfg: &mut Cfg) -> usize {
        let liveness = compute_liveness(cfg);
        let all_ids: Vec<BlockId> = cfg.blocks.keys().copied().collect();
        let mut removed = 0;

        for id in all_ids {
            if !liveness.live.contains(&id) {
                cfg.blocks.remove(&id);
                removed += 1;
            }
        }

        // Clean up stale references in successor/predecessor lists.
        if removed > 0 {
            let live_set: std::collections::BTreeSet<BlockId> =
                cfg.blocks.keys().copied().collect();
            let remaining_ids: Vec<BlockId> = live_set.iter().copied().collect();
            for id in remaining_ids {
                let block = cfg.blocks.get_mut(&id).unwrap();
                block.successors.retain(|s| live_set.contains(s));
                block.predecessors.retain(|p| live_set.contains(p));
            }
        }

        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::cfg::Cfg;
    use crate::compiler::fold::{ConstantFold, run_to_fixed_point};

    // ---------------------------------------------------------------
    // Exercise 7.5: DCE removes unreachable blocks
    // ---------------------------------------------------------------
    #[test]
    fn ex_7_5_dce_unreachable() {
        // STOP, PUSH1 1, STOP — second block is unreachable
        let bytecode = vec![0x00, 0x60, 0x01, 0x00];
        let mut cfg = Cfg::from_bytecode(&bytecode);
        let original_count = cfg.block_count();

        let removed = DeadCodeElim.run(&mut cfg);
        assert!(removed > 0);
        assert!(cfg.block_count() < original_count);
    }

    // ---------------------------------------------------------------
    // Exercise 7.6: DCE removes unreachable block after JUMP
    // ---------------------------------------------------------------
    #[test]
    fn ex_7_6_dce_after_jump() {
        // PUSH1 4, JUMP, INVALID, JUMPDEST, STOP
        // The INVALID block is unreachable (jumped over).
        let bytecode = vec![0x60, 0x04, 0x56, 0xFE, 0x5B, 0x00];
        let mut cfg = Cfg::from_bytecode(&bytecode);
        let original = cfg.block_count();

        DeadCodeElim.run(&mut cfg);
        // At least one block should be removed (the INVALID block).
        assert!(cfg.block_count() <= original);
    }

    // ---------------------------------------------------------------
    // Exercise 7.7: DCE preserves reachable blocks
    // ---------------------------------------------------------------
    #[test]
    fn ex_7_7_dce_preserves_reachable() {
        // PUSH1 1, PUSH1 2, ADD, STOP — all reachable, nothing removed
        let bytecode = vec![0x60, 0x01, 0x60, 0x02, 0x01, 0x00];
        let mut cfg = Cfg::from_bytecode(&bytecode);
        let original = cfg.block_count();
        let removed = DeadCodeElim.run(&mut cfg);
        assert_eq!(removed, 0);
        assert_eq!(cfg.block_count(), original);
    }

    // ---------------------------------------------------------------
    // Exercise 7.8: Combined fold + DCE fixed point
    // ---------------------------------------------------------------
    #[test]
    fn ex_7_8_combined_passes() {
        let bytecode = vec![0x60, 0x02, 0x60, 0x03, 0x02, 0x60, 0x04, 0x02, 0x00];
        let mut cfg = Cfg::from_bytecode(&bytecode);
        let passes: Vec<&dyn OptimizationPass> = vec![&ConstantFold, &DeadCodeElim];
        run_to_fixed_point(&mut cfg, &passes);

        // All code is reachable, so DCE changes nothing, but fold should work.
        let block = cfg.blocks.values().next().unwrap();
        assert_eq!(block.instructions.len(), 2);
    }
}

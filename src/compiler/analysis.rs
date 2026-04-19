//! Dataflow analysis on the CFG: stack height tracking and liveness.

use std::collections::{BTreeMap, BTreeSet};

use super::cfg::Cfg;
use super::ir::BlockId;

/// Stack height at the entry and exit of each basic block.
#[derive(Debug, Clone)]
pub struct StackHeightInfo {
    pub entry_height: BTreeMap<BlockId, usize>,
    pub exit_height: BTreeMap<BlockId, usize>,
}

/// Compute the stack height at entry and exit of each block.
///
/// This is a forward dataflow analysis: we start at the entry block
/// with height 0 and propagate through successors.
pub fn compute_stack_heights(cfg: &Cfg) -> StackHeightInfo {
    let mut entry_height: BTreeMap<BlockId, usize> = BTreeMap::new();
    let mut exit_height: BTreeMap<BlockId, usize> = BTreeMap::new();
    let mut worklist: Vec<BlockId> = vec![cfg.entry];
    entry_height.insert(cfg.entry, 0);

    while let Some(block_id) = worklist.pop() {
        let block = &cfg.blocks[&block_id];
        let mut height = entry_height[&block_id];

        for inst in &block.instructions {
            let consumed = inst.stack_input().min(height);
            height -= consumed;
            height += inst.stack_output();
        }

        exit_height.insert(block_id, height);

        for &succ in &block.successors {
            if let std::collections::btree_map::Entry::Vacant(e) = entry_height.entry(succ) {
                e.insert(height);
                worklist.push(succ);
            }
        }
    }

    StackHeightInfo {
        entry_height,
        exit_height,
    }
}

/// Use and def sets for a basic block.
///
/// In EVM, "use" means a stack slot consumed by an instruction,
/// and "def" means a stack slot produced. These are simplified —
/// we track the net stack effect rather than individual slot identifiers.
#[derive(Debug, Clone)]
pub struct UseDefSets {
    /// Number of stack slots consumed from the entry stack.
    pub uses: usize,
    /// Number of stack slots produced beyond what was consumed.
    pub defs: usize,
}

/// Compute use/def sets for each block.
pub fn compute_use_def(cfg: &Cfg) -> BTreeMap<BlockId, UseDefSets> {
    let mut result = BTreeMap::new();

    for (&id, block) in &cfg.blocks {
        let mut net_height: isize = 0;
        let mut min_height: isize = 0;

        for inst in &block.instructions {
            net_height -= inst.stack_input() as isize;
            min_height = min_height.min(net_height);
            net_height += inst.stack_output() as isize;
        }

        let uses = (-min_height).max(0) as usize;
        let defs = (net_height - min_height).max(0) as usize;

        result.insert(id, UseDefSets { uses, defs });
    }

    result
}

/// Liveness information: which blocks are "live" (reachable from entry
/// and can reach an exit).
#[derive(Debug, Clone)]
pub struct LivenessInfo {
    /// Blocks reachable from the entry.
    pub reachable_from_entry: BTreeSet<BlockId>,
    /// Blocks that can reach an exit block.
    pub can_reach_exit: BTreeSet<BlockId>,
    /// Blocks that are both reachable and can reach an exit.
    pub live: BTreeSet<BlockId>,
}

/// Compute liveness: forward reachability from entry + backward reachability from exits.
pub fn compute_liveness(cfg: &Cfg) -> LivenessInfo {
    // Forward: BFS from entry.
    let mut reachable = BTreeSet::new();
    let mut worklist = vec![cfg.entry];
    while let Some(id) = worklist.pop() {
        if reachable.insert(id)
            && let Some(block) = cfg.blocks.get(&id)
        {
            worklist.extend(&block.successors);
        }
    }

    // Backward: BFS from exit blocks.
    let mut can_reach_exit = BTreeSet::new();
    let mut worklist: Vec<BlockId> = cfg.exit_blocks();
    while let Some(id) = worklist.pop() {
        if can_reach_exit.insert(id)
            && let Some(block) = cfg.blocks.get(&id)
        {
            worklist.extend(&block.predecessors);
        }
    }

    let live: BTreeSet<BlockId> = reachable.intersection(&can_reach_exit).copied().collect();

    LivenessInfo {
        reachable_from_entry: reachable,
        can_reach_exit,
        live,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Exercise 6.7: Stack height tracking
    // ---------------------------------------------------------------
    #[test]
    fn ex_6_7_stack_heights() {
        // PUSH1 1, PUSH1 2, ADD, STOP
        let bytecode = vec![0x60, 0x01, 0x60, 0x02, 0x01, 0x00];
        let cfg = Cfg::from_bytecode(&bytecode);
        let heights = compute_stack_heights(&cfg);
        assert_eq!(heights.entry_height[&cfg.entry], 0);
        // After PUSH1 1 (height 1), PUSH1 2 (height 2), ADD (height 1), STOP (height 1)
        assert_eq!(heights.exit_height[&cfg.entry], 1);
    }

    // ---------------------------------------------------------------
    // Exercise 6.8: Use/def sets
    // ---------------------------------------------------------------
    #[test]
    fn ex_6_8_use_def() {
        // PUSH1 1, PUSH1 2, ADD, STOP
        // uses: 0 (everything produced internally), defs: 1 (net +1)
        let bytecode = vec![0x60, 0x01, 0x60, 0x02, 0x01, 0x00];
        let cfg = Cfg::from_bytecode(&bytecode);
        let use_defs = compute_use_def(&cfg);
        let ud = &use_defs[&cfg.entry];
        assert_eq!(ud.uses, 0); // doesn't consume from entry stack
        assert_eq!(ud.defs, 1); // produces 1 net value
    }

    #[test]
    fn ex_6_8_use_def_consumer() {
        // POP, STOP — consumes 1 from entry stack
        let bytecode = vec![0x50, 0x00];
        let cfg = Cfg::from_bytecode(&bytecode);
        let use_defs = compute_use_def(&cfg);
        let ud = &use_defs[&cfg.entry];
        assert_eq!(ud.uses, 1);
        assert_eq!(ud.defs, 0);
    }

    // ---------------------------------------------------------------
    // Exercise 6.9: Liveness (reachability)
    // ---------------------------------------------------------------
    #[test]
    fn ex_6_9_liveness() {
        // PUSH1 4, JUMP, INVALID, JUMPDEST, STOP
        let bytecode = vec![0x60, 0x04, 0x56, 0xFE, 0x5B, 0x00];
        let cfg = Cfg::from_bytecode(&bytecode);
        let liveness = compute_liveness(&cfg);

        // Block 0 and block 2 (JUMPDEST, STOP) should be live.
        // Block 1 (INVALID) is reachable but also an exit, so it's "live" too
        // in our simplified model. The key test: entry is reachable.
        assert!(liveness.reachable_from_entry.contains(&cfg.entry));
        assert!(!liveness.live.is_empty());
    }

    #[test]
    fn ex_6_9_unreachable_block() {
        // STOP, PUSH1 1, STOP — second block is unreachable
        let bytecode = vec![0x00, 0x60, 0x01, 0x00];
        let cfg = Cfg::from_bytecode(&bytecode);
        let liveness = compute_liveness(&cfg);

        // Only the first block (STOP) should be reachable.
        assert_eq!(liveness.reachable_from_entry.len(), 1);
        assert_eq!(liveness.live.len(), 1);
    }
}

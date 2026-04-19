//! Control Flow Graph construction from EVM bytecode.
//!
//! We segment bytecode into basic blocks and connect them with edges
//! representing control flow (fall-through, JUMP, JUMPI).

use std::collections::{BTreeMap, BTreeSet};

use crate::opcode::Opcode;

use super::ir::{BasicBlock, BlockId, Instruction};

/// A control flow graph over EVM bytecode.
#[derive(Debug)]
pub struct Cfg {
    /// Basic blocks, keyed by their ID.
    pub blocks: BTreeMap<BlockId, BasicBlock>,
    /// The entry block ID (always BlockId(0) for offset 0).
    pub entry: BlockId,
}

impl Cfg {
    /// Build a CFG from raw bytecode.
    pub fn from_bytecode(bytecode: &[u8]) -> Self {
        let leaders = find_leaders(bytecode);
        let blocks = extract_blocks(bytecode, &leaders);
        let mut cfg = Cfg {
            blocks,
            entry: BlockId(0),
        };
        cfg.compute_edges(bytecode);
        cfg
    }

    /// Compute successor and predecessor edges.
    fn compute_edges(&mut self, _bytecode: &[u8]) {
        // Collect the start offsets → block IDs for jump target resolution.
        let offset_to_block: BTreeMap<usize, BlockId> = self
            .blocks
            .values()
            .map(|b| (b.start_offset, b.id))
            .collect();

        // Collect edge info first to avoid borrow issues.
        let mut edges: Vec<(BlockId, BlockId)> = Vec::new();

        let block_ids: Vec<BlockId> = self.blocks.keys().copied().collect();
        for &block_id in &block_ids {
            let block = &self.blocks[&block_id];
            if block.instructions.is_empty() {
                continue;
            }
            let last = block.instructions.last().unwrap();
            match last.opcode {
                Opcode::JUMP => {
                    if let Some(target_offset) = self.resolve_static_jump(block_id)
                        && let Some(&target_block) = offset_to_block.get(&target_offset)
                    {
                        edges.push((block_id, target_block));
                    }
                }
                Opcode::JUMPI => {
                    if let Some(target_offset) = self.resolve_static_jump(block_id)
                        && let Some(&target_block) = offset_to_block.get(&target_offset)
                    {
                        edges.push((block_id, target_block));
                    }
                    // Fall-through edge.
                    if let Some(&fall_block) = offset_to_block.get(&block.end_offset) {
                        edges.push((block_id, fall_block));
                    }
                }
                Opcode::STOP | Opcode::RETURN | Opcode::REVERT | Opcode::INVALID => {
                    // Terminal — no successors.
                }
                _ => {
                    // Fall-through to next block.
                    if let Some(&fall_block) = offset_to_block.get(&block.end_offset) {
                        edges.push((block_id, fall_block));
                    }
                }
            }
        }

        // Apply edges.
        for (from, to) in edges {
            self.blocks.get_mut(&from).unwrap().successors.push(to);
            self.blocks.get_mut(&to).unwrap().predecessors.push(from);
        }
    }

    /// Try to resolve a static jump target from the block.
    /// Looks for a PUSH immediately before JUMP/JUMPI.
    fn resolve_static_jump(&self, block_id: BlockId) -> Option<usize> {
        let block = &self.blocks[&block_id];
        let instructions = &block.instructions;
        if instructions.len() < 2 {
            return None;
        }
        let push_inst = &instructions[instructions.len() - 2];
        if !push_inst.opcode.is_push() {
            return None;
        }
        // Convert immediate bytes to a usize offset.
        let mut val: usize = 0;
        for &b in &push_inst.immediate {
            val = val.checked_shl(8)?.checked_add(b as usize)?;
        }
        Some(val)
    }

    /// Get all block IDs that are exit blocks (STOP, RETURN, REVERT).
    pub fn exit_blocks(&self) -> Vec<BlockId> {
        self.blocks
            .values()
            .filter(|b| {
                b.instructions.last().is_some_and(|i| {
                    matches!(
                        i.opcode,
                        Opcode::STOP | Opcode::RETURN | Opcode::REVERT | Opcode::INVALID
                    )
                })
            })
            .map(|b| b.id)
            .collect()
    }

    /// Number of basic blocks.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}

impl std::fmt::Display for Cfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, block) in &self.blocks {
            writeln!(
                f,
                "Block {} (offset {:#x}..{:#x}):",
                id.0, block.start_offset, block.end_offset
            )?;
            for inst in &block.instructions {
                if inst.immediate.is_empty() {
                    writeln!(f, "  {:#06x}: {:?}", inst.offset, inst.opcode)?;
                } else {
                    writeln!(
                        f,
                        "  {:#06x}: {:?} {:02x?}",
                        inst.offset, inst.opcode, inst.immediate
                    )?;
                }
            }
            let succs: Vec<usize> = block.successors.iter().map(|s| s.0).collect();
            writeln!(f, "  successors: {succs:?}")?;
        }
        Ok(())
    }
}

/// Identify leader positions in the bytecode.
///
/// A leader is the first instruction of a basic block:
/// 1. The first byte (offset 0)
/// 2. Any JUMPDEST target
/// 3. The instruction after a JUMP/JUMPI/STOP/RETURN/REVERT
pub fn find_leaders(bytecode: &[u8]) -> BTreeSet<usize> {
    let mut leaders = BTreeSet::new();
    if bytecode.is_empty() {
        return leaders;
    }
    leaders.insert(0); // Rule 1: first instruction

    let mut i = 0;
    while i < bytecode.len() {
        let byte = bytecode[i];
        if let Some(op) = Opcode::from_byte(byte) {
            let imm_size = op.immediate_size();
            let next = i + 1 + imm_size;

            match op {
                Opcode::JUMPDEST => {
                    leaders.insert(i); // Rule 2
                }
                Opcode::JUMP
                | Opcode::JUMPI
                | Opcode::STOP
                | Opcode::RETURN
                | Opcode::REVERT
                | Opcode::INVALID => {
                    if next < bytecode.len() {
                        leaders.insert(next); // Rule 3
                    }
                }
                _ => {}
            }
            i = next;
        } else {
            i += 1;
        }
    }
    leaders
}

/// Extract basic blocks from bytecode given the set of leader positions.
fn extract_blocks(bytecode: &[u8], leaders: &BTreeSet<usize>) -> BTreeMap<BlockId, BasicBlock> {
    let leader_vec: Vec<usize> = leaders.iter().copied().collect();
    let mut blocks = BTreeMap::new();

    for (idx, &start) in leader_vec.iter().enumerate() {
        let end = leader_vec.get(idx + 1).copied().unwrap_or(bytecode.len());
        let mut instructions = Vec::new();
        let mut i = start;

        while i < end {
            let byte = bytecode[i];
            if let Some(op) = Opcode::from_byte(byte) {
                let imm_size = op.immediate_size();
                let imm_end = (i + 1 + imm_size).min(bytecode.len());
                let immediate = bytecode[i + 1..imm_end].to_vec();
                instructions.push(Instruction {
                    offset: i,
                    opcode: op,
                    immediate,
                });
                i = imm_end;
            } else {
                // Unknown opcode — treat as single-byte instruction.
                instructions.push(Instruction {
                    offset: i,
                    opcode: Opcode::INVALID,
                    immediate: vec![],
                });
                i += 1;
            }
        }

        let block_id = BlockId(idx);
        blocks.insert(
            block_id,
            BasicBlock {
                id: block_id,
                start_offset: start,
                end_offset: end,
                instructions,
                successors: Vec::new(),
                predecessors: Vec::new(),
            },
        );
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Exercise 6.1: Leader identification
    // ---------------------------------------------------------------
    #[test]
    fn ex_6_1_leaders_simple() {
        // PUSH1 4, JUMP, INVALID, JUMPDEST, STOP
        let bytecode = vec![0x60, 0x04, 0x56, 0xFE, 0x5B, 0x00];
        let leaders = find_leaders(&bytecode);
        assert!(leaders.contains(&0)); // first instruction
        assert!(leaders.contains(&3)); // after JUMP
        assert!(leaders.contains(&4)); // JUMPDEST
    }

    #[test]
    fn ex_6_1_leaders_with_jumpi() {
        // PUSH1 0, PUSH1 5, JUMPI, STOP, JUMPDEST, STOP
        let bytecode = vec![0x60, 0x00, 0x60, 0x05, 0x57, 0x5B, 0x00];
        let leaders = find_leaders(&bytecode);
        assert!(leaders.contains(&0));
        assert!(leaders.contains(&5)); // after JUMPI AND JUMPDEST
    }

    // ---------------------------------------------------------------
    // Exercise 6.2: Basic block extraction
    // ---------------------------------------------------------------
    #[test]
    fn ex_6_2_block_count() {
        // PUSH1 4, JUMP | INVALID | JUMPDEST, STOP
        let bytecode = vec![0x60, 0x04, 0x56, 0xFE, 0x5B, 0x00];
        let cfg = Cfg::from_bytecode(&bytecode);
        assert_eq!(cfg.block_count(), 3);
    }

    // ---------------------------------------------------------------
    // Exercise 6.3: Fall-through edges
    // ---------------------------------------------------------------
    #[test]
    fn ex_6_3_fallthrough_edges() {
        // PUSH1 1, PUSH1 2, ADD, STOP
        // This is a single block (no jumps), no edges needed.
        let bytecode = vec![0x60, 0x01, 0x60, 0x02, 0x01, 0x00];
        let cfg = Cfg::from_bytecode(&bytecode);
        assert_eq!(cfg.block_count(), 1);
        let block = cfg.blocks.values().next().unwrap();
        assert!(block.successors.is_empty()); // STOP = terminal
    }

    // ---------------------------------------------------------------
    // Exercise 6.4: JUMP edges
    // ---------------------------------------------------------------
    #[test]
    fn ex_6_4_jump_edge() {
        // PUSH1 4, JUMP, INVALID, JUMPDEST, STOP
        let bytecode = vec![0x60, 0x04, 0x56, 0xFE, 0x5B, 0x00];
        let cfg = Cfg::from_bytecode(&bytecode);
        // Block 0 (PUSH1 4, JUMP) should have an edge to the JUMPDEST block.
        let entry = &cfg.blocks[&cfg.entry];
        assert_eq!(entry.successors.len(), 1);
    }

    // ---------------------------------------------------------------
    // Exercise 6.5: JUMPI edges (two outgoing)
    // ---------------------------------------------------------------
    #[test]
    fn ex_6_5_jumpi_two_edges() {
        // PUSH1 1, PUSH1 7, JUMPI, STOP, ..., JUMPDEST, STOP
        let bytecode = vec![
            0x60, 0x01, // PUSH1 1
            0x60, 0x07, // PUSH1 7
            0x57, // JUMPI
            0x00, // STOP (fall-through)
            0xFE, // INVALID (padding)
            0x5B, // JUMPDEST
            0x00, // STOP
        ];
        let cfg = Cfg::from_bytecode(&bytecode);
        let entry = &cfg.blocks[&cfg.entry];
        // JUMPI should have 2 successors: jump target and fall-through
        assert_eq!(entry.successors.len(), 2);
    }

    // ---------------------------------------------------------------
    // Exercise 6.6: Entry and exit blocks
    // ---------------------------------------------------------------
    #[test]
    fn ex_6_6_entry_and_exit() {
        let bytecode = vec![0x60, 0x04, 0x56, 0xFE, 0x5B, 0x00];
        let cfg = Cfg::from_bytecode(&bytecode);
        assert_eq!(cfg.entry, BlockId(0));
        let exits = cfg.exit_blocks();
        assert!(!exits.is_empty());
    }

    // ---------------------------------------------------------------
    // Display
    // ---------------------------------------------------------------
    #[test]
    fn cfg_display_does_not_panic() {
        let bytecode = vec![0x60, 0x01, 0x60, 0x02, 0x01, 0x00];
        let cfg = Cfg::from_bytecode(&bytecode);
        let output = format!("{cfg}");
        assert!(output.contains("Block 0"));
    }
}

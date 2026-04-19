# From Bytecode to IR: Control Flow Graphs

Today we shift from executing bytecode to **analyzing** it. We lift the flat byte stream into a structured intermediate representation (IR) of basic blocks connected by control flow edges.

## Why analyze bytecode?

Direct interpretation has limits. To optimize, we need structure:
- Which code is reachable? (Dead code elimination)
- Which values are used? (Constant folding, redundant load removal)
- What does the control flow look like? (Loop detection, inlining)

The **control flow graph** (CFG) is the compiler's primary IR for answering these questions.

## Basic blocks

A **basic block** is a maximal sequence of instructions with:
- **One entry point** (the first instruction)
- **One exit point** (the last instruction)
- **No internal jumps** — if the first instruction executes, all instructions in the block execute

A **leader** (the first instruction of a block) is identified by three rules:
1. The first byte of the bytecode (offset 0)
2. Any `JUMPDEST` instruction
3. The instruction immediately after a `JUMP`, `JUMPI`, `STOP`, `RETURN`, or `REVERT`

## CFG edges

Edges represent possible control flow between blocks:

| Last instruction | Outgoing edges |
|-----------------|----------------|
| Fall-through (no terminator) | 1 edge to next block |
| `JUMP` | 1 edge to target (if statically resolvable) |
| `JUMPI` | 2 edges: true branch (target) + false branch (fall-through) |
| `STOP` / `RETURN` / `REVERT` | 0 edges (terminal) |

## Static jump resolution

EVM jumps are technically **dynamic** — the destination is a runtime stack value. But most jumps are preceded by a `PUSH` with a constant, making them **statically resolvable**:

```
PUSH1 0x0A    ; push target offset
JUMP          ; jump to offset 0x0A
```

Our CFG builder looks for this PUSH+JUMP pattern and resolves the edge. Truly dynamic jumps (destination computed at runtime) cannot be resolved statically — this is what makes EVM analysis harder than x86 analysis.

## Stack height analysis

We track the stack height at the entry and exit of each block. This is a forward dataflow analysis:
- Entry block starts at height 0
- Each instruction adjusts height: `height = height - inputs + outputs`
- Successor blocks inherit the exit height

## Use/def sets and liveness

For each block, we compute:
- **Uses**: stack slots consumed from the entry stack
- **Defs**: stack slots produced beyond what was consumed

Liveness combines forward reachability (from entry) and backward reachability (from exits). A block is **live** if it's both reachable from the entry and can reach an exit. Dead blocks can be eliminated.

## Exercises

- **6.1** — Leader identification on sample bytecode
- **6.2** — Block extraction: correct number of blocks
- **6.3** — Fall-through edges for sequential code
- **6.4** — JUMP edge resolution
- **6.5** — JUMPI produces two outgoing edges
- **6.6** — Entry and exit block identification
- **6.7** — Stack height tracking
- **6.8** — Use/def set computation
- **6.9** — Liveness worklist convergence

Run: `cargo test compiler`

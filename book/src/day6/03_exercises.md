# Exercises — Day 6

```admonish example title="Exercise Setup" collapsible=true
    git checkout -b my-day-6 my-day-5

Create the compiler module:

    mkdir -p src/compiler

- `src/compiler/mod.rs` — `pub mod ir; pub mod cfg; pub mod analysis;`
- `src/compiler/ir.rs` — `BlockId`, `Instruction`, `BasicBlock`
- `src/compiler/cfg.rs` — `Cfg`, `find_leaders()`, edge computation
- `src/compiler/analysis.rs` — `compute_stack_heights()`, `compute_use_def()`, `compute_liveness()`

Add `pub mod compiler;` to `src/lib.rs`.

Peek: `git show day-6:src/compiler/cfg.rs | head -100`

Compare: `git diff day-6 -- src/compiler/`
```

These exercises build the control flow graph (CFG) step by step: identify leaders, extract blocks, add edges, and run dataflow analyses.

---

## Exercise 6.1 — Leader identification

**What to do.** Given the bytecode `[PUSH1 0x06, JUMP, INVALID, PUSH1 0x00, JUMPDEST, STOP]`,
call your leader-identification function and assert that offsets `0`, `3` (instruction after
JUMP), and `4` (JUMPDEST) are all marked as leaders. Offset `2` (INVALID) is **not** a leader.

**How to verify.**

```
cargo test ex_6_1
```

**Hint.** Three rules produce leaders: offset 0 is always a leader; any JUMPDEST is a
leader; the instruction immediately following JUMP, JUMPI, STOP, RETURN, or REVERT is a
leader (even if that offset is INVALID).

> **Rust pattern — `BTreeSet` for ordered collections.** Use `BTreeSet<usize>` to store
> leader offsets. It deduplicates automatically and iterates in ascending order, which
> matches how you walk the bytecode left-to-right to form blocks.

---

## Exercise 6.2 — Basic block extraction

**What to do.** Using the same bytecode as 6.1, run your block-extraction pass and assert
that exactly three basic blocks are produced. Assert that each block's `start` offset
matches one of the three leader offsets identified in 6.1.

**How to verify.**

```
cargo test ex_6_2
```

**Hint.** A block spans from its leader to (but not including) the next leader. The last
block spans to the end of the bytecode. Build a sorted `Vec<usize>` of leaders to compute
block boundaries with a simple pairwise iteration.

> **Rust pattern — `BTreeMap<BlockId, BasicBlock>`.** Keying blocks by a `BlockId`
> (a `usize` or newtype wrapper) gives O(log n) lookup and deterministic iteration order,
> which simplifies debugging and makes test assertions order-independent.

---

## Exercise 6.3 — Fall-through edges for sequential code

**What to do.** Build a CFG from bytecode that has no explicit jump (`PUSH1 1, PUSH1 2,
ADD, STOP`). Assert that block 0 has zero outgoing edges (it ends with STOP) and that no
fall-through edge is added past a terminal instruction.

Then use bytecode with two sequential blocks where block 0 does not end with a terminal
(`PUSH1 0, POP, JUMPDEST, STOP`). Assert that block 0 has exactly one outgoing edge
pointing to block 1.

**How to verify.**

```
cargo test ex_6_3
```

**Hint.** A fall-through edge is added from block B to its successor when B's last
instruction is **not** a terminal (JUMP, JUMPI, STOP, RETURN, REVERT, INVALID).

> **Rust pattern — graph representation.** Store edges as
> `HashMap<BlockId, Vec<BlockId>>` on the `Cfg` struct. Each key is a source block;
> the value is the list of successor block IDs.

---

## Exercise 6.4 — JUMP edges are statically resolved

**What to do.** Build a CFG from `PUSH1 0x04, JUMP, INVALID, JUMPDEST, STOP`. Assert
that block 0 (containing the PUSH + JUMP) has exactly one outgoing edge pointing to the
block that starts at offset `0x04` (the JUMPDEST).

**How to verify.**

```
cargo test ex_6_4
```

**Hint.** Look for the pattern `PUSH* immediate, JUMP` in the instruction list. The
immediate value is the jump target. Verify the target is a valid JUMPDEST before adding
the edge; otherwise the jump is invalid and should produce no edge (or an error edge).

> **Rust pattern — pattern matching on instruction sequences.** Use a sliding window
> (`instructions.windows(2)`) or a manual two-pointer walk to find the `[Push(n), Jump]`
> pattern. Destructuring in the `match` arm keeps the logic readable.

---

## Exercise 6.5 — JUMPI produces two outgoing edges

**What to do.** Build a CFG from `PUSH1 0x06, PUSH1 0x01, JUMPI, PUSH1 0x00, STOP,
JUMPDEST, STOP`. Assert that the block ending with JUMPI has **exactly two** outgoing
edges: one to the JUMPDEST target and one fall-through to the instruction immediately
after JUMPI.

**How to verify.**

```
cargo test ex_6_5
```

**Hint.** The true branch goes to the resolved target (if the condition is non-zero);
the false branch falls through. Both edges must always be present in the static CFG,
even though at runtime only one is taken.

> **Rust pattern — conditional edges in CFGs.** Model both edges as plain entries in
> the successor list. Downstream analyses (liveness, DCE) correctly propagate along
> both edges without needing an "edge condition" annotation.

---

## Exercise 6.6 — Entry and exit block identification

**What to do.** Build a CFG with at least three blocks. Assert that exactly one block
has `block_id == 0` (entry). Assert that the blocks with no outgoing edges (ending with
STOP, RETURN, REVERT) are the exit blocks. Also assert that the entry block is not an
exit block (it has at least one outgoing edge in the test bytecode).

**How to verify.**

```
cargo test ex_6_6
```

**Hint.** Entry is always the block at offset 0. Exit blocks are those whose successor
list is empty after edge construction.

> **Rust pattern — filtering with iterators.** Use
> `cfg.blocks.values().filter(|b| b.successors.is_empty())` to collect exit blocks
> without a manual loop. This composes naturally with `.count()`, `.map()`, or
> `.collect()`.

---

## Exercise 6.7 — Stack height tracking (forward dataflow)

**What to do.** Build a CFG for `PUSH1 1, PUSH1 2, ADD, PUSH1 3, STOP`. Run the stack
height analysis and assert:

- Entry height of block 0 is `0`.
- After `PUSH1 1`: height = 1. After `PUSH1 2`: height = 2. After `ADD`: height = 1. After `PUSH1 3`: height = 2.
- Exit height of block 0 is `2`.

**How to verify.**

```
cargo test ex_6_7
```

**Hint.** Each instruction adjusts height by `outputs - inputs`. For PUSH1: `+1`.
For ADD: `2 inputs, 1 output → -1`. Propagate the exit height of a block to the entry
height of each successor.

> **Rust pattern — worklist algorithm.** Initialize a `VecDeque` with the entry block.
> Pop a block, compute its exit height, and push any successor whose entry height
> has changed. Repeat until the queue is empty.

---

## Exercise 6.8 — Use/def sets (net stack effects)

**What to do.** For a block containing `DUP1, ADD, POP` (net effect: pops 1, pushes 0),
assert that `uses = 2` (DUP1 reads depth 1 from entry stack; ADD reads two) and
`defs = 0` (POP removes the result).

Also test `PUSH1 x, PUSH1 y, ADD` — this block produces one value from constants only,
so `uses = 0` and `defs = 1`.

**How to verify.**

```
cargo test ex_6_8
```

**Hint.** Walk the instruction list tracking a virtual stack height starting at 0.
When an input comes from below height 0, it is a "use" of the entry stack. Each output
pushed above the high-water mark is a "def".

> **Rust pattern — `isize` for signed arithmetic.** Use `isize` for the running virtual
> stack height so underflows (reading from the entry stack) produce negative values that
> you can count as uses, without special-casing.

---

## Exercise 6.9 — Liveness (forward + backward reachability)

**What to do.** Build a CFG with one unreachable block: block A → block B → block C,
where there is also block D with no predecessors. Run the liveness pass and assert:

- Blocks A, B, C are live.
- Block D is **not** live (unreachable from entry).

Also verify that a block only reachable from entry but having no path to any exit is not
live (backward unreachable).

**How to verify.**

```
cargo test ex_6_9
```

**Hint.** Forward reachability: BFS/DFS from entry, marking blocks reachable.
Backward reachability: BFS/DFS from all exits in the reversed graph.
A block is live iff it appears in **both** sets.

> **Rust pattern — set intersection (`BTreeSet`).** Compute forward and backward
> reachable sets separately, then use `forward.intersection(&backward)` to obtain the
> live set in one expression.

---

Run all Day 6 tests: `cargo test compiler`

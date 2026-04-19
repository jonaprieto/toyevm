# Exercises — Day 7

These exercises implement constant folding and dead code elimination (DCE) as composable optimization passes over the CFG from Day 6.

## OptimizationPass trait (reference)

```rust
{{#include ../../../src/compiler/fold.rs:optimization_pass_trait}}
```

---

## Exercise 7.1 — OptimizationPass trait compiles for both passes

**What to do.** Instantiate both `ConstantFold` and `Dce` (dead code elimination) as
`Box<dyn OptimizationPass>` values. Call `pass.name()` on each and assert the returned
strings are non-empty. The test should compile and pass without running any bytecode.

**How to verify.**

```
cargo test ex_7_1
```

**Hint.** If the trait bound is satisfied, `Box<dyn OptimizationPass>` lets you store
both passes in a `Vec<Box<dyn OptimizationPass>>` and iterate over them uniformly — this
is the exact shape of a production optimizer pipeline.

> **Rust pattern — trait objects (`Box<dyn OptimizationPass>`).**
> Dynamic dispatch here is intentional: the pipeline doesn't know (or care) which concrete
> pass it holds at compile time. The trade-off is one vtable lookup per `run` call, which
> is negligible compared to the CFG traversal cost.

---

## Exercise 7.2 — Constant fold ADD

**What to do.** Build a single-block CFG containing exactly `PUSH1 3, PUSH1 5, ADD`.
Run `ConstantFold`. Assert that:

1. The pass returns `changes > 0`.
2. The block now contains exactly **one** instruction.
3. That instruction pushes the value `8`.

**How to verify.**

```
cargo test ex_7_2
```

**Hint.** After folding, the three instructions (`PUSH1 3`, `PUSH1 5`, `ADD`) are
replaced by a single `PUSH32 8` (the folder always uses PUSH32 to avoid recomputing the
minimal push width). Check `instruction.immediate == U256::from(8)`.

> **Rust pattern — peephole pattern matching.** The folder uses a sliding window of
> three instructions. Matching `[Instruction::Push(a), Instruction::Push(b), Instruction::BinOp(op)]`
> in a `match` arm is cleaner than three nested `if let` chains.

---

## Exercise 7.3 — Fold a MUL chain to fixed point

**What to do.** Build a single-block CFG with `PUSH1 2, PUSH1 3, MUL, PUSH1 4, MUL`.
Run `ConstantFold` **once**. Assert the block has two instructions (the first fold reduces
`2 * 3` to `6`, leaving `PUSH32 6, PUSH1 4, MUL`). Run `ConstantFold` again. Assert the
block has exactly one instruction with value `24`.

**How to verify.**

```
cargo test ex_7_3
```

**Hint.** One pass of the folder is not enough for chained arithmetic. The pass must be
repeated until it returns `0`. Use a `while pass.run(cfg) > 0 {}` loop (the fixed-point
loop from exercise 7.8) or call it manually twice in this test to observe the two-step
reduction.

> **Rust pattern — iterative refinement.** Each call to `run` is idempotent for already-folded
> instructions: it only changes things that can still be improved. Counting returned changes
> drives the fixed-point loop without needing a separate "did anything change" flag.

---

## Exercise 7.4 — Folding reduces gas cost

**What to do.** Compute the gas cost of `[PUSH1 3, PUSH1 5, ADD]` (three `PUSH`/`ADD`
instructions at 3 gas each = 9 gas). After folding, compute the gas cost of the resulting
single `PUSH32 8` (3 gas). Assert `cost_after < cost_before` and that the difference
is exactly `6`.

**How to verify.**

```
cargo test ex_7_4
```

**Hint.** You do not need to execute bytecode — just multiply `instruction_count * 3`
for base-cost instructions. In a real optimizer you would sum per-instruction gas costs
from the gas schedule table.

> **Rust pattern — reasoning about costs.** Writing a cost-delta assertion alongside
> the transformation test ties correctness ("the value is right") to efficiency
> ("the cost went down"). If a future refactor breaks the fold, both assertions fail
> and the reason is immediately clear.

---

## Exercise 7.5 — DCE removes unreachable blocks

**What to do.** Build a CFG with three blocks: block 0 jumps to block 2, block 1 has no
predecessors (unreachable), and block 2 is a terminal. Run the `Dce` pass. Assert that
the returned `changes > 0` and that block 1 no longer exists in `cfg.blocks`.

**How to verify.**

```
cargo test ex_7_5
```

**Hint.** The `Dce` pass relies on the liveness analysis from Day 6. Run liveness
first (or have `Dce::run` call it internally) to determine which blocks are dead,
then remove them from `cfg.blocks` and drop their edges.

> **Rust pattern — graph pruning.** Collect dead block IDs into a `Vec`, then
> `cfg.blocks.remove(&id)` in a second loop. Avoid mutating the map while iterating
> over it — collect first, remove second.

---

## Exercise 7.6 — DCE removes dead code after an unconditional JUMP

**What to do.** Build a CFG where block 0 ends with `PUSH1 target, JUMP` to block 2,
and block 1 (at offset between the JUMP and the JUMPDEST) contains unreachable
instructions. Run `Dce`. Assert block 1 is removed and blocks 0 and 2 remain.

**How to verify.**

```
cargo test ex_7_6
```

**Hint.** Block 1 is unreachable because the CFG builder adds no edges into it — the
JUMP skips over it and the block has no JUMPDEST. DCE's forward-reachability pass
will never visit it.

> **Rust pattern — liveness as a prerequisite.** DCE cannot run correctly without
> knowing which blocks are reachable. Express this dependency by calling the liveness
> pass inside `Dce::run`, or by requiring a `LivenessInfo` argument. Encoding
> prerequisites in the type signature prevents calling DCE on a stale CFG.

---

## Exercise 7.7 — DCE preserves all reachable blocks (no false positives)

**What to do.** Build a CFG where every block is reachable from the entry and every block
can reach an exit (a diamond-shaped CFG: block 0 → blocks 1 and 2 via JUMPI → block 3).
Run `Dce`. Assert that `changes == 0` and that all four blocks still exist.

**How to verify.**

```
cargo test ex_7_7
```

**Hint.** This is the "no false positive" invariant: DCE must not remove live blocks.
If your liveness analysis is incorrect (e.g., it misses a backward-reachability path),
this test catches it before the optimizer silently corrupts correct code.

> **Rust pattern — testing invariants.** Testing that an optimization does **nothing**
> on a valid input is as important as testing that it does something on an invalid input.
> Add both to every optimization pass.

---

## Exercise 7.8 — Fixed-point iteration with combined passes

**What to do.** Build a CFG with a MUL chain in block 0 and an unreachable block 1.
Combine both passes in a loop:

```rust
let passes: Vec<Box<dyn OptimizationPass>> = vec![
    Box::new(ConstantFold),
    Box::new(Dce),
];
let mut total = 1;
while total > 0 {
    total = passes.iter().map(|p| p.run(cfg)).sum();
}
```

Assert that after the loop exits, the MUL chain is fully folded **and** block 1 is gone.
Assert the loop terminates (runs at most a small fixed number of times in practice).

**How to verify.**

```
cargo test ex_7_8
```

**Hint.** Constant folding may expose new dead blocks (if folding changes a conditional
branch to a constant), and DCE may enable further folding (by removing instructions
that were only live due to a dead successor). The fixed-point loop handles both cases
automatically.

> **Rust pattern — loop until stable.** Summing changes across all passes per iteration
> and looping while `total > 0` is the standard fixed-point driver. It is guaranteed to
> terminate because each iteration either reduces the instruction count or the block count
> — both are finite and non-negative.

---

Run all Day 7 tests: `cargo test fold` and `cargo test dce`

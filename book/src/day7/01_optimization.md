# Optimization Passes

Today we close the loop: we take the CFG from Day 6 and transform it. Two classic optimization passes — constant folding and dead code elimination — demonstrate the fundamental techniques that production compilers like `evm_mlir` use at scale.

## The OptimizationPass trait

Every optimization conforms to a common interface:

```rust
pub trait OptimizationPass {
    fn run(&self, cfg: &mut Cfg) -> usize;  // returns number of changes
    fn name(&self) -> &str;
}
```

This enables fixed-point iteration: run all passes in a loop until none of them make changes.

## Constant Folding

The simplest optimization: if both operands of an arithmetic operation are compile-time constants, evaluate it now and replace the three instructions with a single PUSH.

**Before:**
```
PUSH1 3    ; 3 gas
PUSH1 5    ; 3 gas
ADD        ; 3 gas
           ; total: 9 gas, 3 instructions
```

Before: $3 + 3 + 3 = 9$ gas (3 instructions)

**After:**
```
PUSH32 8   ; 3 gas
           ; total: 3 gas, 1 instruction
```

After: $3$ gas (1 instruction)

This saves 6 gas per fold. Across a contract with many constant expressions, the savings accumulate.

### Chained folding

Consider: `PUSH1 2, PUSH1 3, MUL, PUSH1 4, MUL`. The first pass folds `(2, 3, MUL)` → `PUSH32 6`. The second pass sees `(PUSH32 6, PUSH1 4, MUL)` and folds to `PUSH32 24`. This is why we need **fixed-point iteration** — a single pass isn't enough.

## Dead Code Elimination

DCE removes blocks that are unreachable from the entry or cannot reach any exit. This uses the liveness analysis from Day 6.

A block is **dead** if:
- It's unreachable from the entry block (forward), OR
- It cannot reach any exit block (backward)

Example:
```
Block 0: PUSH1 4, JUMP → Block 2
Block 1: INVALID        ← unreachable! (DCE removes this)
Block 2: JUMPDEST, STOP
```

## Superinstructions (bonus concept)

A **superinstruction** fuses a common sequence into a single compound opcode. For example, `PUSH1 x, PUSH1 y, ADD` could become `PUSHADD x y` — one dispatch instead of three. This reduces interpreter loop overhead and can enable further constant folding.

Production interpreters like CPython and LuaJIT use superinstructions extensively. In the EVM context, `revm`'s analysis phase effectively does this by pre-computing gas costs and skipping redundant checks.

## AOT Compilation (discussion)

Our optimizer works on the IR but still outputs bytecode for the interpreter. The next step would be **ahead-of-time compilation**: transpile the CFG into native machine code.

This is exactly what [LambdaClass's evm_mlir](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) does:
1. Lift EVM bytecode → MLIR dialect
2. Apply standard MLIR optimization passes
3. Lower to LLVM IR
4. LLVM backend → native x86/ARM

Their benchmarks show 20x+ speedup over interpretation. The key insight: gas metering is compiled in as a counter decrement at each basic block entry, not checked per-opcode.

## JIT vs. AOT in blockchain context

A **JIT compiler** compiles code on-the-fly during execution; an **AOT compiler** compiles before execution begins.

| | JIT | AOT |
|---|-----|-----|
| Startup | Slow (compile on first call) | Slow (compile at deploy) |
| Steady-state | Fast (native code) | Fast (native code) |
| Determinism | Harder (optimization choices vary) | Easier (compile once, cache) |
| Cache | Must invalidate on upgrade | Stored with contract |

Blockchain requires **deterministic execution** — every node must produce the same result. JIT is risky because different optimization levels could produce different gas measurements. AOT with a fixed compilation pipeline is safer.

```admonish note title="Real-World Reference: evm_mlir"
LambdaClass's `evm_mlir` achieves 20x+ speedup by compiling EVM bytecode through MLIR → LLVM → native code. Gas metering is compiled as a counter decrement at each basic block entry, replacing per-opcode checks.
```

## Exercises

- **7.1** — OptimizationPass trait compiles for both passes
- **7.2** — Constant folding: PUSH 3, PUSH 5, ADD → PUSH 8
- **7.3** — Chained folding: MUL chain → single PUSH
- **7.4** — Folding reduces gas cost
- **7.5** — DCE removes unreachable blocks
- **7.6** — DCE removes block after unconditional JUMP
- **7.7** — DCE preserves all reachable blocks
- **7.8** — Fixed-point iteration with multiple passes

Run: `cargo test fold` and `cargo test dce`

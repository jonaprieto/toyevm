# Optimization Passes

Today we close the loop: we take the CFG from Day 6 and transform it. Two classic optimization passes — constant folding and dead code elimination — demonstrate the fundamental techniques that production compilers like `evm_mlir` use at scale. We also explore how to make the interpreter itself faster — the techniques that separate a naive EVM from a production one.

## The OptimizationPass trait

Every optimization conforms to a common interface:

```rust
pub trait OptimizationPass {
    fn run(&self, cfg: &mut Cfg) -> usize;  // returns number of changes
    fn name(&self) -> &str;
}
```

This enables **fixed-point iteration**: run all passes in a loop until none of them make changes. The trait is the standard pattern for composable compiler passes — LLVM, GCC, and Cranelift all use variations of this interface.

```admonish tip title="Rust Pattern: Trait Objects for Passes"
You can collect passes into a `Vec<Box<dyn OptimizationPass>>` and iterate over them generically. This decouples the pass manager from the individual passes — adding a new optimization requires no changes to the driver loop.
```

## Constant Folding

The simplest optimization: if both operands of an arithmetic operation are compile-time constants, evaluate it now and replace the three instructions with a single PUSH.

**Before:**
```
PUSH1 3    ; 3 gas
PUSH1 5    ; 3 gas
ADD        ; 3 gas
           ; total: 9 gas, 3 instructions
```

**After:**
```
PUSH1 8    ; 3 gas
           ; total: 3 gas, 1 instruction
```

This saves 6 gas per fold — a 67% reduction for this pattern. Across a contract with many constant expressions (e.g., ABI offset calculations, storage slot computations), the savings accumulate significantly.

### Chained folding

Consider: `PUSH1 2, PUSH1 3, MUL, PUSH1 4, MUL`. The first pass folds `(2, 3, MUL)` → `PUSH1 6`. The second pass sees `(PUSH1 6, PUSH1 4, MUL)` and folds to `PUSH1 24`. This is why we need **fixed-point iteration** — a single pass isn't enough.

The fixed-point loop terminates because each iteration strictly reduces the instruction count (three instructions become one). With $N$ instructions, at most $N/2$ folds can occur, so the loop terminates in $O(N)$ iterations.

```admonish warning title="Pitfall: Division by Zero in Folding"
When folding `PUSH 5, PUSH 0, DIV`, the result must match the EVM specification: division by zero returns zero (not a panic). Your `evaluate_binop` function must handle this case explicitly.
```

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

```admonish note title="Why We Don't Remove the Entry Block"
Even if static analysis cannot prove the entry block reaches an exit (e.g., all outgoing jumps are dynamically resolved), the entry block must be preserved — removing it would destroy the program.
```

## Making the interpreter faster

Our interpreter works correctly, but it is far from optimal. Production EVMs like [revm](https://github.com/bluealloy/revm) use several techniques to minimize per-opcode overhead. These are the techniques that matter most, ordered by impact.

### 1. JUMPDEST bitmap pre-computation

Every `JUMP`/`JUMPI` must validate that the target is a `JUMPDEST`. A naive approach scans the bytecode on each jump — $O(N)$ per jump. Instead, pre-compute a bitmap at contract load time:

```rust
fn compute_jumpdest_bitmap(bytecode: &[u8]) -> Vec<bool> {
    let mut bitmap = vec![false; bytecode.len()];
    let mut i = 0;
    while i < bytecode.len() {
        if bytecode[i] == 0x5B { // JUMPDEST
            bitmap[i] = true;
        }
        // Skip PUSH immediates
        if (0x60..=0x7F).contains(&bytecode[i]) {
            i += (bytecode[i] - 0x5F) as usize;
        }
        i += 1;
    }
    bitmap
}
```

Jump validation becomes a single array lookup: $O(1)$ instead of $O(N)$.

### 2. Padded bytecode (bounds-check elimination)

The interpreter's hot loop checks `pc < bytecode.len()` on every iteration. By appending STOP bytes to the bytecode (padding to the next cache-line boundary), the bounds check becomes redundant — running past the end hits a STOP and halts normally.

```rust
let mut padded = bytecode.to_vec();
padded.resize(padded.len() + 33, 0x00); // 33 for max PUSH32 + STOP
```

This eliminates a branch from the hottest loop in the program. At billions of iterations per block, even one fewer branch per iteration matters.

### 3. Gas pre-computation

Instead of looking up gas costs in a match table on every opcode dispatch, pre-compute the static gas cost for each instruction at analysis time and store it alongside the opcode:

```rust
struct AnalyzedOp {
    opcode: Opcode,
    static_gas: u64,
    // immediate bytes already decoded
}
```

The interpreter can then deduct gas with a single subtraction, no dispatch needed. Dynamic gas (memory expansion, SLOAD cold/warm) still requires runtime logic, but static gas covers the majority of opcodes.

### 4. `repr(u8)` jump table dispatch

Our opcode enum already uses `#[repr(u8)]`, which Rust compiles into a jump table — a single indexed memory load rather than a chain of if-else comparisons. This is $O(1)$ dispatch with good branch-predictor behavior.

The key detail: the `match` must be **exhaustive** and the enum values must be **dense** (no large gaps). Both conditions hold for EVM opcodes. This is the same technique Python's CPython ceval loop and the HotSpot JVM interpreter use.

### 5. Stack representation

Our stack uses `Vec<U256>`, which is heap-allocated. A production interpreter might use a fixed-size array on the stack frame:

```rust
struct FastStack {
    data: [U256; 1024],  // fixed-size, no heap allocation
    top: usize,
}
```

This avoids heap allocation entirely and improves cache locality — the entire stack fits in L1/L2 cache.

```admonish note title="Benchmark before optimizing"
These techniques are well-known, but their impact depends on the workload. Always profile with realistic contracts before optimizing. The [evm.codes playground](https://www.evm.codes/playground) and [Foundry's `forge test --gas-report`](https://getfoundry.sh/forge/gas-tracking) are useful for generating realistic benchmarks.
```

## Superinstructions (bonus concept)

A **superinstruction** fuses a common sequence into a single compound opcode. For example, `PUSH1 x, PUSH1 y, ADD` could become `PUSHADD x y` — one dispatch instead of three. This reduces interpreter loop overhead and can enable further constant folding.

Common EVM superinstruction candidates (from profiling real contract bytecode):

| Pattern | Frequency | Savings |
|---------|-----------|---------|
| `PUSH1, PUSH1, ADD/MUL` | Very high (offset math) | 2 dispatches |
| `DUP1, PUSH1, LT/GT` | High (loop bounds) | 1 dispatch |
| `PUSH1, MLOAD/MSTORE` | High (memory access) | 1 dispatch |
| `ISZERO, PUSH, JUMPI` | High (conditional branch) | 2 dispatches |

Production interpreters like CPython and LuaJIT use superinstructions extensively. In the EVM context, `revm`'s analysis phase effectively does this by pre-computing gas costs and skipping redundant checks.

## AOT Compilation (discussion)

Our optimizer works on the IR but still outputs bytecode for the interpreter. The next step would be **ahead-of-time compilation**: transpile the CFG into native machine code.

This is exactly what [LambdaClass's evm_mlir](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) does:

1. **Lift** EVM bytecode → MLIR dialect (each opcode becomes an MLIR operation)
2. **Optimize** with standard MLIR passes (constant folding, DCE, LICM)
3. **Lower** to LLVM IR
4. **Codegen** via LLVM backend → native x86/ARM

Their benchmarks show **3–7x speedup** over interpretation (varying by contract and hardware). The key insight: gas metering is compiled as a counter decrement at each basic block entry, not checked per-opcode.

```mermaid
flowchart LR
    A["EVM Bytecode"] --> B["CFG"]
    B --> C["MLIR Dialect"]
    C --> D["LLVM IR"]
    D --> E["Native Code"]
    style E stroke:#2e7d32,stroke-width:2px,stroke-dasharray: 5 5
```

## JIT vs. AOT in blockchain context

A **JIT compiler** compiles code on-the-fly during execution; an **AOT compiler** compiles before execution begins.

| | JIT | AOT |
|---|-----|-----|
| **Startup** | Slow (compile on first call) | Slow (compile at deploy) |
| **Steady-state** | Fast (native code) | Fast (native code) |
| **Determinism** | Harder (optimization choices vary) | Easier (compile once, cache) |
| **Cache** | Must invalidate on upgrade | Stored with contract |

Blockchain requires **deterministic execution** — every node must produce the same result. JIT is risky because different optimization levels could produce different gas measurements. AOT with a fixed compilation pipeline is safer.

[evm_mlir](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) takes the AOT approach for this reason.

## References

- [revm source code](https://github.com/bluealloy/revm) — production Rust EVM with the optimization techniques described above
- [evm_mlir blog post](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) — AOT compilation pipeline and benchmarks
- [EOF EIPs](https://eips.ethereum.org/EIPS/eip-3540) — EIP-3540 (container format), EIP-4200 (static jumps), EIP-4750 (functions), EIP-663 (DUPN/SWAPN)

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

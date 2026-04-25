# Deep Dive — Module 7

Review questions for Module 7. Try answering before expanding.

---

**Q: Walk me through implementing constant folding on an IR.**

```admonish note title="Answer" collapsible=true
Scan each basic block for patterns where two PUSH instructions are immediately followed by a binary operation (ADD, MUL, SUB, etc.). Extract the constant values from both PUSHes, evaluate the operation at compile time, and replace all three instructions with a single PUSH of the result. Repeat until no more folds are possible (fixed-point iteration), because one fold may create new folding opportunities (e.g., `PUSH 2, PUSH 3, MUL` folds to `PUSH 6`, which combined with a subsequent `PUSH 4, MUL` folds to `PUSH 24`).

Edge cases to handle: division by zero must return zero (not panic), shifts greater than 255 return zero, and modulo by zero returns zero — all per the [Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf).
```

---

**Q: Why do we need liveness analysis before dead code elimination?**

```admonish note title="Answer" collapsible=true
Liveness tells us which values and blocks are actually needed. Without it, we might delete an instruction whose result is consumed much later (through a complex control flow path), or we might keep a block that's reachable but produces only dead values. Liveness combines forward reachability (can we reach this block from the entry?) with backward reachability (can this block reach an exit?). Only blocks satisfying both conditions are "live."
```

---

**Q: What makes EVM optimization different from x86 optimization?**

```admonish note title="Answer" collapsible=true
The objective function is **gas cost**, not wall-clock time. Storage accesses (SLOAD: 2100 gas cold) dominate — a single SLOAD costs as much as 700 ADD instructions. This means optimizations that reduce storage access (caching loads, combining writes) have outsized impact. Additionally, the EVM has no registers — only a stack — so register allocation passes don't apply. Instead, stack scheduling (minimizing DUP/SWAP) is the analog. Finally, determinism is mandatory: every optimization must preserve exact gas semantics, not just functional correctness.
```

---

**Q: How does evm_mlir compile EVM bytecode to native code?**

```admonish note title="Answer" collapsible=true
It lifts EVM bytecode into an [MLIR](https://mlir.llvm.org/) dialect where each opcode becomes an MLIR operation. Standard MLIR passes (constant folding, dead code elimination, loop invariant code motion) optimize the IR. Gas metering is inserted as a counter decrement at each basic block entry — this replaces per-opcode gas checks with a single comparison per block. The optimized MLIR is lowered to LLVM IR, and the LLVM backend produces native machine code. The result is a function that takes `(state, input)` and returns `(state', output, gas_used)` — the same pure function model as interpretation, but [significantly faster](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) (LambdaClass reports 3–7x speedups).
```

---

**Q: What is a superinstruction and when would I design one?**

```admonish note title="Answer" collapsible=true
A superinstruction is a compound opcode that implements a common multi-opcode sequence as a single dispatch. Example: `PUSH1 x, PUSH1 y, ADD` → `PUSHADD x y`. Benefits: reduces dispatch loop iterations (each dispatch has overhead from the fetch-decode-match cycle), enables the compiler to optimize the combined operation (e.g., avoid pushing x just to pop it immediately), and reduces code size. I'd design one when profiling shows a sequence appears frequently in real contracts and the dispatch overhead is measurable. CPython, LuaJIT, and Java's HotSpot all use superinstructions.
```

---

**Q: What are the most impactful interpreter-level optimizations for an EVM?**

```admonish note title="Answer" collapsible=true
In order of typical impact:

1. **JUMPDEST bitmap** — pre-scan the bytecode once at load time, building a `Vec<bool>` or bitset. Jump validation drops from $O(N)$ per jump to $O(1)$. This is the single highest-impact optimization because jumps are extremely frequent (every loop iteration, every function call dispatch).

2. **Padded bytecode** — append STOP bytes so the interpreter doesn't need bounds checks on every PC advance. Eliminates a branch from the hottest loop.

3. **Gas pre-computation** — cache the static gas cost per instruction at analysis time. The interpreter deducts gas with a single subtraction instead of dispatching to a cost lookup.

4. **Fixed-size stack** — use `[U256; 1024]` instead of `Vec<U256>`. No heap allocation, better cache locality, zero-cost push/pop (just increment a pointer).

5. **`repr(u8)` jump table** — ensure the opcode match compiles to a jump table (O(1) dispatch) rather than a chain of comparisons. Rust does this automatically with `#[repr(u8)]` on a contiguous enum.

These are the techniques `revm` uses. Combined, they bring the interpreter within 2–5x of native-compiled execution speed.
```

---

**Q: Why is JIT compilation not the right approach for EVM execution?**

```admonish note title="Answer" collapsible=true
The EVM workload is a poor fit for JIT compilation for several reinforcing reasons:

- **Compilation latency** is never amortized — most contracts are called rarely (median: fewer than 10 transactions over their lifetime). LLVM at `-O0` already takes milliseconds per contract, too slow for a node processing thousands of transactions per block.
- **Instruction cache pressure** — JIT-compiled native code is larger than bytecode. Thousands of compiled contracts fill the I-cache, and cache misses on x86 cost 100+ cycles, wiping out the speedup.
- **Branch prediction** favors the interpreter — a single hot dispatch loop is predictable; scattered generated code is not.
- **Determinism** — ensuring bit-identical gas accounting across x86/ARM, OS versions, and compiler versions is extremely difficult. Any divergence is a consensus failure.

The alternative that emerged is the **tail-call interpreter**: each opcode handler ends with a direct jump to the next handler, eliminating the dispatch loop. This gives near-native speed without compilation overhead: no dispatch branches, better branch prediction, no call stack growth, and full determinism. Rust does not yet have `#[musttail]` (there is an [accepted RFC](https://github.com/rust-lang/rfcs/pull/3407)), but a `match` with `#[repr(u8)]` compiles to a jump table that approaches the same performance. C++ EVMs like `evmone` use `__attribute__((musttail))` directly.

AOT compilation still has a niche for hot contracts (e.g., via `evm_mlir`), but the baseline for production EVMs is a fast interpreter — not a compiler.
```

---

**Q: How would speculative pre-execution work in practice, and what are its limits?**

```admonish note title="Answer" collapsible=true
The core mechanism is state forking: at a JUMPI, snapshot the stack and memory, then execute both successor blocks in parallel (or sequentially) on their own copies. When the branch condition resolves, discard the wrong snapshot.

The practical limits are:

1. **State forking cost.** Cloning the stack (1024 × 32 bytes = 32 KB max) and memory (which can be megabytes) on every JUMPI is expensive. A production implementation would use copy-on-write semantics — only clone the pages that are actually modified.

2. **Cascading speculation.** If a speculated block itself ends with JUMPI, you'd need to speculate recursively. This creates an exponential blowup: $k$ nested JUMPIs produce $2^k$ speculative paths. In practice, you'd limit speculation depth to 1 or 2 levels.

3. **Side-effect boundaries.** Any block that touches storage, emits logs, or makes external calls cannot be speculated — the effects can't be undone. In real Solidity contracts, most interesting branches (the ones that diverge based on user input) lead to side-effecting code almost immediately.

4. **Gas accounting.** Speculated paths consume real computation but the gas must only be charged for the path actually taken. This requires careful bookkeeping to avoid over- or under-charging.

The technique is most useful in static analysis tools (exploring all paths through a contract) and AOT compilers (pre-computing both branches at compile time for simple cases). For a runtime interpreter, the overhead of forking state typically exceeds the savings from avoiding the branch stall.
```

---

**Q: Why is determinism a constraint on EVM optimization?**

```admonish note title="Answer" collapsible=true
Every Ethereum node must independently execute every transaction and arrive at the exact same state. This means:

- Gas consumption must be **identical** across all implementations. An optimization that changes gas semantics (e.g., caching an SLOAD result so the second access is "free") would cause consensus failures.
- Arithmetic must be **bit-identical**. Reordering operations that affect wrapping behavior (even if mathematically equivalent for unbounded integers) can produce different 256-bit results.
- Memory expansion costs are based on the **high-water mark**, not actual usage. An optimization that delays memory writes doesn't reduce the gas charged.

This is why EVM optimizations focus on reducing dispatch overhead and pre-computing static analysis (which doesn't affect gas semantics) rather than rewriting the computation itself.
```

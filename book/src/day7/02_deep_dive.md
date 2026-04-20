# Deep Dive — Day 7

Try to answer each question before expanding the answer.

---

**Q: Walk me through implementing constant folding on an IR.**

```admonish note title="Answer" collapsible=true
Scan each basic block for patterns where two PUSH instructions are immediately followed by a binary operation (ADD, MUL, SUB, etc.). Extract the constant values from both PUSHes, evaluate the operation at compile time, and replace all three instructions with a single PUSH of the result. Repeat until no more folds are possible (fixed-point iteration), because one fold may create new folding opportunities (e.g., `PUSH 2, PUSH 3, MUL` folds to `PUSH 6`, which combined with a subsequent `PUSH 4, MUL` folds to `PUSH 24`).

Edge cases to handle: division by zero must return zero (not panic), shifts greater than 255 return zero, and modulo by zero returns zero — all per the Yellow Paper.
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

**Q: What is a superinstruction and when would you design one?**

```admonish note title="Answer" collapsible=true
A superinstruction is a compound opcode that implements a common multi-opcode sequence as a single dispatch. Example: `PUSH1 x, PUSH1 y, ADD` → `PUSHADD x y`. Benefits: reduces dispatch loop iterations (each dispatch has overhead from the fetch-decode-match cycle), enables the compiler to optimize the combined operation (e.g., avoid pushing x just to pop it immediately), and reduces code size. You design one when profiling shows a sequence appears frequently in real contracts and the dispatch overhead is measurable. CPython, LuaJIT, and Java's HotSpot all use superinstructions.
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

**Q: Why is determinism a constraint on EVM optimization?**

```admonish note title="Answer" collapsible=true
Every Ethereum node must independently execute every transaction and arrive at the exact same state. This means:

- Gas consumption must be **identical** across all implementations. An optimization that changes gas semantics (e.g., caching an SLOAD result so the second access is "free") would cause consensus failures.
- Arithmetic must be **bit-identical**. Reordering operations that affect wrapping behavior (even if mathematically equivalent for unbounded integers) can produce different 256-bit results.
- Memory expansion costs are based on the **high-water mark**, not actual usage. An optimization that delays memory writes doesn't reduce the gas charged.

This is why EVM optimizations focus on reducing dispatch overhead and pre-computing static analysis (which doesn't affect gas semantics) rather than rewriting the computation itself.
```

# Deep Dive — Day 7

---

**Q: Walk me through implementing constant folding on an IR.**

Scan each basic block for patterns where two PUSH instructions are immediately followed by a binary operation (ADD, MUL, SUB, etc.). Extract the constant values from both PUSHes, evaluate the operation at compile time, and replace all three instructions with a single PUSH of the result. Repeat until no more folds are possible (fixed-point iteration), because one fold may create new folding opportunities (e.g., `PUSH 2, PUSH 3, MUL` folds to `PUSH 6`, which combined with a subsequent `PUSH 4, MUL` folds to `PUSH 24`).

---

**Q: Why do we need liveness analysis before dead code elimination?**

Liveness tells us which values and blocks are actually needed. Without it, we might delete an instruction whose result is consumed much later (through a complex control flow path), or we might keep a block that's reachable but produces only dead values. Liveness combines forward reachability (can we reach this block from the entry?) with backward reachability (can this block reach an exit?). Only blocks satisfying both conditions are "live."

---

**Q: What makes EVM optimization different from x86 optimization?**

The objective function is **gas cost**, not wall-clock time. Storage accesses (SLOAD: 2100 gas cold) dominate — a single SLOAD costs as much as 700 ADD instructions. This means optimizations that reduce storage access (caching loads, combining writes) have outsized impact. Additionally, the EVM has no registers — only a stack — so register allocation passes don't apply. Instead, stack scheduling (minimizing DUP/SWAP) is the analog. Finally, determinism is mandatory: every optimization must preserve exact gas semantics, not just functional correctness.

---

**Q: How does evm_mlir compile EVM bytecode to native code?**

It lifts EVM bytecode into an MLIR dialect where each opcode becomes an MLIR operation. Standard MLIR passes (constant folding, dead code elimination, loop invariant code motion) optimize the IR. Gas metering is inserted as a counter decrement at each basic block entry — this replaces per-opcode gas checks with a single comparison per block. The optimized MLIR is lowered to LLVM IR, and the LLVM backend produces native machine code. The result is a function that takes (state, input) and returns (state', output, gas_used) — the same pure function model as interpretation, but 20x faster.

---

**Q: What is a superinstruction and when would you design one?**

A superinstruction is a compound opcode that implements a common multi-opcode sequence as a single dispatch. Example: `PUSH1 x, PUSH1 y, ADD` → `PUSHADD x y`. Benefits: reduces dispatch loop iterations (each dispatch has overhead from the fetch-decode-match cycle), enables the compiler to optimize the combined operation (e.g., avoid pushing x just to pop it immediately), and reduces code size. You design one when profiling shows a sequence appears frequently in real contracts and the dispatch overhead is measurable. CPython, LuaJIT, and Java's HotSpot all use superinstructions.

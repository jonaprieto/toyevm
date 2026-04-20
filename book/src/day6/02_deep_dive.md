# Deep Dive — Day 6

---

**Q: What is a basic block and why is it the unit of compiler analysis?**

A basic block is a maximal straight-line sequence of instructions — one entry point, one exit point, no internal branches. If the first instruction executes, all instructions execute. This property lets you summarize the block's effect as a single net operation (e.g., "pops 2, pushes 1, writes storage slot X") without reasoning about conditional paths. Every optimization pass — from constant folding to register allocation — operates on or between basic blocks.

---

**Q: Walk me through constructing a CFG from EVM bytecode.**

Three steps:

1. **Find leaders** — scan linearly, marking offset 0, every JUMPDEST, and every instruction after a terminator (JUMP/JUMPI/STOP/RETURN/REVERT). You must skip PUSH immediate bytes to avoid misidentifying a 0x5B byte inside a PUSH32 as a JUMPDEST.

2. **Extract blocks** — split the bytecode at leader boundaries. Each block gets an ID, a start/end offset, and a list of decoded instructions.

3. **Add edges** — for each block's last instruction: JUMP → resolve the target (look for PUSH before JUMP); JUMPI → two edges (target + fall-through); STOP/RETURN → no edges; anything else → fall-through to next block.

The critical subtlety is step 1: JUMPDEST validation. A byte `0x5B` that appears inside the immediate data of `PUSH32 0x...5B...` is **not** a valid jump target. This is a security property — without it, an attacker could craft a PUSH with embedded code and jump into the middle of it.

---

**Q: Why is EVM bytecode harder to analyze than x86 or LLVM IR?**

Three reasons:

1. **Dynamic jumps**: the destination is a runtime stack value. x86 has `jmp rax` but the register set is bounded and trackable; EVM's stack is unbounded and any computation can produce a jump target. This makes the CFG potentially incomplete.

2. **No explicit function boundaries**: Solidity's function dispatch is a chain of `CALLDATALOAD, SHR, EQ, JUMPI` — the compiler encodes function selectors as conditional jumps, not call/ret instructions. Recovering function boundaries requires pattern matching.

3. **Stack machine**: no named registers, no SSA form. The "liveness" of a value depends on its depth in the stack, which changes with every PUSH/POP/DUP/SWAP. Converting to SSA requires stack-to-register promotion — essentially, simulating the stack symbolically.

**EOF (EVM Object Format, EIP-3540)** solves all three: it introduces `RJUMP`/`RJUMPI` (static relative jumps), `CALLF`/`RETF` (explicit function calls), and `DUPN`/`SWAPN` (larger operand range). With EOF, CFG construction becomes trivial — no JUMPDEST scanning, no dynamic jump resolution.

---

**Q: What are dominators and why do they matter for loops?**

Block A **dominates** block B if every path from the entry to B must pass through A. The entry block dominates everything. The **immediate dominator** of B is the closest strict dominator.

Dominators matter because they define **natural loops**: a back-edge B→A where A dominates B identifies a loop with header A. The loop body is all blocks that can reach B without going through A, plus A itself.

Our toyevm doesn't compute dominators, but a production optimizer (like `revm`'s analysis or LLVM) needs them for:
- Loop-invariant code motion (hoist computations out of loops)
- SSA construction (placing φ-functions at dominance frontiers)
- Loop unrolling and strength reduction

The dominators can be computed in $O(N \cdot \alpha(N))$ time using the Lengauer-Tarjan algorithm, where $N$ is the number of blocks.

---

**Q: Explain the difference between block-level and value-level liveness.**

**Block-level** (what we implement): a block is live if it's reachable from entry AND can reach an exit. Used for dead block elimination — removing unreachable code.

**Value-level**: a specific value (stack slot, variable) is live at a program point if there exists a path from that point to a use of that value. Used for:
- **Dead store elimination**: if an SSTORE's value is overwritten before being read, remove it
- **Register allocation**: a register holding a dead value can be reused
- **Dead code elimination**: a PUSH whose value is never consumed can be removed

Value-level liveness requires tracking individual stack slots through DUP/SWAP operations, which is significantly more complex than block reachability. In the EVM stack machine, the "name" of a value is its depth, which shifts with every operation.

---

**Q: How does `revm` speed up execution without compiling to native code?**

`revm` applies CFG-level analysis **once** at contract load time, then interprets with reduced per-opcode overhead:

1. **JUMPDEST bitmap**: pre-scanned, so JUMP validation is a single bit check instead of a linear scan
2. **Gas pre-computation**: static gas costs are cached per instruction, avoiding the dispatch-to-cost lookup on every step
3. **Padded bytecode**: the bytecode array is extended with STOP bytes so the interpreter doesn't need bounds checks on every PC advance
4. **`repr(u8)` dispatch**: the opcode match compiles to a jump table — O(1) branch prediction-friendly dispatch

These are all techniques you'd discover by profiling: the interpreter loop is the hottest code in an Ethereum node, and every nanosecond per opcode matters when processing millions of transactions.

---

**Q: What would it take to go from your CFG to native code?**

Four additional passes:

1. **Stack-to-register promotion**: simulate the stack, assign each slot to a virtual register. This is the hardest step for EVM because DUP/SWAP create aliasing.

2. **SSA construction**: insert φ-functions at join points using the dominator tree. Each value gets a unique name, enabling powerful optimizations (GVN, SCCP, LICM).

3. **Optimization**: constant propagation, dead code elimination, loop invariant code motion, common subexpression elimination — all standard compiler passes, now applicable because we have SSA.

4. **Code generation**: lower SSA IR to machine code (x86/ARM) via LLVM or Cranelift. Gas metering is compiled as a counter decrement at each basic block entry.

This is exactly what [revmc](https://github.com/bluealloy/revmc) (revm's JIT/AOT compiler) and [evm_mlir](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) do. The result is 10-50x faster than interpretation.

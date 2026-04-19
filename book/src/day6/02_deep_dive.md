# Deep Dive — Day 6

---

**Q: What is a basic block and why is it the unit of compiler analysis?**

A basic block is a maximal straight-line sequence of instructions — it has one entry point, one exit point, and no internal branches. If the first instruction executes, all instructions execute. This makes it the natural unit for optimization: you can analyze the block as a single atomic operation, compute its net effect on the stack/memory/storage, and transform it without worrying about partial execution.

---

**Q: Why is EVM bytecode harder to analyze than x86 bytecode?**

EVM jumps are **dynamic** — the destination is a value popped from the stack at runtime, not an immediate operand in the instruction. While most real jumps are preceded by a PUSH (making them statically resolvable), the specification allows any stack value as a jump target. This means a complete CFG requires either abstract interpretation or a conservative over-approximation. x86 jumps typically use immediate offsets or register-indirect addressing where the register set is bounded and analyzable.

---

**Q: What is liveness analysis and what is it used for?**

Liveness analysis determines which values are "live" (will be used in the future) at each program point. It's a **backward** dataflow analysis: you start from value consumers and propagate backward. A value is dead if it's never used after the point where it's defined. Uses: dead code elimination (remove instructions producing dead values), register allocation (reuse registers holding dead values), and redundant store elimination.

---

**Q: How does `revm` speed up execution without compiling to native code?**

`revm` performs a bytecode analysis pass before execution: it pre-validates the JUMPDEST table (so JUMP doesn't need to rescan), identifies statically resolvable jump targets, caches gas costs per instruction, and uses a `repr(u8)` enum with match dispatch (which compiles to a jump table). These optimizations reduce per-opcode overhead from dynamic checks to a single indexed memory load, achieving significant speedup without leaving the interpreter paradigm.

---

**Q: What is the relationship between basic blocks and optimization passes?**

Each optimization pass operates on the CFG. **Local** optimizations work within a single basic block (constant folding, peephole). **Global** optimizations work across blocks using dataflow analysis (dead code elimination, common subexpression elimination). **Interprocedural** optimizations work across function boundaries (inlining). The basic block structure enables all of these by providing a clean abstraction over straight-line code sequences.

# Deep Dive — Day 2

---

**Q: Walk me through exactly what happens when the EVM executes an ADD instruction.**

The interpreter reads byte `0x01` at the current PC. It decodes this as the `ADD` opcode. It deducts the static gas cost (3 gas, "very low" tier) from the remaining gas — if insufficient gas remains, execution halts with an `OutOfGas` error. Then it pops two values from the stack. If the stack has fewer than 2 elements, execution halts with a `StackUnderflow` error. It computes `a + b` with wrapping arithmetic (mod 2^256), pushes the result onto the stack, and advances the PC by 1.

---

**Q: What is the gas cost of PUSH1 vs. PUSH32?**

Both cost 3 gas. The immediate bytes (1 byte for PUSH1, 32 bytes for PUSH32) are data, not separate instructions. They are consumed by advancing the PC past them, but they don't incur additional gas charges. This is a common technical discussion trick — candidates sometimes assume longer immediates cost more.

---

**Q: How does out-of-gas differ from a REVERT?**

Both discard all state changes made during the current call frame. The critical difference is gas handling: out-of-gas consumes **all remaining gas** (the caller loses everything), while REVERT **refunds unused gas** to the caller. REVERT can also include return data (typically an ABI-encoded error message), whereas out-of-gas returns nothing.

---

**Q: Why do we use a `match` on a `repr(u8)` enum rather than a HashMap for opcode dispatch?**

Performance and correctness. The Rust compiler compiles the `match` into a jump table — a single indexed memory load, O(1) with zero allocation. A HashMap would require hashing, bucket lookup, and heap-allocated function pointers. In production EVMs, the dispatch loop executes billions of times; even a few nanoseconds per dispatch matters. Additionally, `match` is exhaustive — the compiler verifies that every opcode variant is handled, preventing "missing case" bugs that a HashMap would silently ignore.

---

**Q: What happens when execution reaches the end of bytecode without encountering STOP?**

This is treated as an implicit STOP — execution halts normally with empty return data. The EVM specification defines this behavior explicitly. It's equivalent to the bytecode having a trailing `0x00` byte. The gas consumed up to that point is still charged.

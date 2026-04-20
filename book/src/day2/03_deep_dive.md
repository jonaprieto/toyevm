# Deep Dive — Day 2

Questions I worked through after implementing Day 2. I'd recommend trying to answer each one before expanding.

---

**Q: Walk me through exactly what happens when the EVM executes an ADD instruction.**

```admonish note title="Answer" collapsible=true
The interpreter reads byte `0x01` at the current PC and decodes it as the `ADD` opcode. It needs to deduct the static gas cost (3 gas, "very low" tier) from the remaining gas — if insufficient gas remains, execution halts with an `OutOfGas` error. Then it pops two values from the stack; if the stack has fewer than 2 elements, it halts with a `StackUnderflow` error. It computes `a + b` with wrapping arithmetic (mod 2^256), pushes the result onto the stack, and advances the PC by 1.
```

---

**Q: What is the gas cost of PUSH1 vs. PUSH32?**

```admonish note title="Answer" collapsible=true
Both cost 3 gas. The immediate bytes (1 byte for PUSH1, 32 bytes for PUSH32) are data, not separate instructions — they're consumed by advancing the PC past them, but don't incur additional gas charges. I found this trips people up in technical discussions; it's easy to assume longer immediates cost more.
```

---

**Q: How does out-of-gas differ from a REVERT?**

```admonish note title="Answer" collapsible=true
Both discard all state changes made during the current call frame. The critical difference I kept coming back to is gas handling: out-of-gas consumes **all remaining gas** (the caller loses everything), while REVERT **refunds unused gas** to the caller. REVERT can also include return data (typically an ABI-encoded error message), whereas out-of-gas returns nothing.
```

---

**Q: Why do we use a `match` on a `repr(u8)` enum rather than a HashMap for opcode dispatch?**

```admonish note title="Answer" collapsible=true
Performance and correctness. The Rust compiler compiles the `match` into a jump table — a single indexed memory load, O(1) with zero allocation. A HashMap would require hashing, bucket lookup, and heap-allocated function pointers. When I thought about production EVMs where the dispatch loop executes billions of times, a few nanoseconds per dispatch really does matter. The exhaustiveness of `match` also appealed to me — the compiler verifies every opcode variant is handled, preventing "missing case" bugs that a HashMap would silently ignore.
```

---

**Q: What happens when execution reaches the end of bytecode without encountering STOP?**

```admonish note title="Answer" collapsible=true
This is treated as an implicit STOP — execution halts normally with empty return data. I was glad to find the EVM specification defines this explicitly. It's equivalent to the bytecode having a trailing `0x00` byte, and the gas consumed up to that point is still charged.
```

# Interview Corner — Day 3

---

**Q: Why can't you jump to an arbitrary byte offset in EVM bytecode?**

JUMPDEST validation prevents landing inside a PUSH immediate sequence. Without it, an attacker could encode a `PUSH32` containing 0x5B (JUMPDEST) in its data bytes, then jump to that embedded byte — executing unintended code. The validator scans the bytecode linearly, skipping PUSH immediates, and marks only true JUMPDEST opcodes as valid targets.

---

**Q: What is the gas cost difference between a cold and warm SLOAD?**

Cold SLOAD costs 2100 gas, warm costs 100 gas. This was introduced in EIP-2929 (Berlin upgrade). The rationale is that the first access to a storage slot requires disk I/O, while subsequent accesses hit an in-memory cache. The 21x cost difference accurately reflects the performance gap between disk and memory.

---

**Q: What is the difference between CALLER and ORIGIN?**

`CALLER` (I_s) is the address that directly invoked the current execution context — it changes with each nested CALL. `ORIGIN` (I_o) is the externally-owned account (EOA) that initiated the transaction — it never changes regardless of call depth. Using `ORIGIN` for authorization is a security anti-pattern because a malicious contract could trick a user into signing a transaction that calls through it, making `ORIGIN` appear as the user's address.

---

**Q: How does memory expansion gas work?**

Memory cost = `3 * words + words² / 512`, where `words` is the number of 32-byte words. The cost is the delta between the new total and the old total when memory expands. The linear term (3 gas/word) makes small allocations cheap. The quadratic term (words²/512) makes large allocations progressively expensive — 320 KB costs ~226,000 gas, making it impractical to allocate unbounded memory. This prevents DoS attacks where contracts allocate gigabytes on every node.

---

**Q: Why does the EVM use JUMP/JUMPI instead of structured control flow?**

The EVM was designed as a minimal, low-level execution target — closer to assembly than to a high-level language. Structured control flow (if/else, for, while) is handled by the compiler (Solidity → bytecode). The EVM provides just the primitive jumps, which are simpler to implement correctly in consensus-critical code and easier to gas-meter. This design also makes the bytecode verifiable — you can statically analyze JUMPDEST positions without executing the code.

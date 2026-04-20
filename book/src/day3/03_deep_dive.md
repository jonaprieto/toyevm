# Deep Dive — Day 3

Questions I worked through after implementing Day 3. Try to answer each one before expanding.

---

**Q: Why can't you jump to an arbitrary byte offset in EVM bytecode?**

```admonish note title="Answer" collapsible=true
JUMPDEST validation prevents landing inside a PUSH immediate sequence. Without it, an attacker could encode a `PUSH32` containing 0x5B (JUMPDEST) in its data bytes, then jump to that embedded byte — executing unintended code. The validator scans the bytecode linearly, skipping PUSH immediates, and marks only true JUMPDEST opcodes as valid targets.
```

---

**Q: What is the gas cost difference between a cold and warm SLOAD?**

```admonish note title="Answer" collapsible=true
Cold SLOAD costs 2100 gas, warm costs 100 gas. This was introduced in [EIP-2929](https://eips.ethereum.org/EIPS/eip-2929) (Berlin upgrade). The rationale is that the first access to a storage slot requires disk I/O, while subsequent accesses hit an in-memory cache. The 21x cost difference reflects the performance gap between disk and memory.
```

---

**Q: What is the difference between CALLER and ORIGIN?**

```admonish note title="Answer" collapsible=true
`CALLER` (I_s) is the address that directly invoked the current execution context — it changes with each nested CALL. `ORIGIN` (I_o) is the externally-owned account (EOA) that initiated the transaction — it never changes regardless of call depth. Using `ORIGIN` for authorization is a security anti-pattern because a malicious contract could trick a user into signing a transaction that calls through it, making `ORIGIN` appear as the user's address.
```

---

**Q: How does memory expansion gas work?**

```admonish note title="Answer" collapsible=true
The total memory cost up to `words` 32-byte words is: $C_{mem} = 3 \cdot \text{words} + \lfloor \text{words}^2 / 512 \rfloor$. The gas actually **charged** on each expansion is the **delta**: $C_{mem}(\text{new}) - C_{mem}(\text{old})$. The linear term (3 gas/word) makes small allocations cheap. The quadratic term makes large allocations progressively expensive — 320 KB (10,000 words) costs ~225,312 gas total, making it impractical to allocate unbounded memory. This prevents DoS attacks where contracts allocate gigabytes on every node.
```

---

**Q: Why does the EVM use JUMP/JUMPI instead of structured control flow?**

```admonish note title="Answer" collapsible=true
The EVM was designed as a minimal, low-level execution target — closer to assembly than to a high-level language. Structured control flow (if/else, for, while) is handled by the compiler (Solidity → bytecode). The EVM provides just the primitive jumps, which are simpler to implement correctly in consensus-critical code and easier to gas-meter. This design also makes the bytecode verifiable — you can statically analyze JUMPDEST positions without executing the code.
```

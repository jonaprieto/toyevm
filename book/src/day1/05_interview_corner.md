# Interview Corner — Day 1

These are questions you might encounter in a compiler engineering or blockchain infrastructure interview. After completing today's exercises, you should be able to answer all of them.

---

**Q: Why does Ethereum use 256-bit words instead of 64-bit?**

Keccak-256 produces 32-byte hashes, and Ethereum uses Keccak pervasively — for storage keys, address derivation, and state root computation. A 256-bit word can hold a full hash without truncation. Additionally, 256-bit keys provide 128-bit security against brute-force preimage attacks, which is the standard cryptographic target.

---

**Q: What happens when the EVM stack overflows?**

If a contract pushes the 1025th element, the transaction immediately reverts. All state changes are discarded, and (depending on the call context) all gas may be consumed. This is a hard protocol limit, not a configurable parameter.

---

**Q: Is EVM memory persistent between transactions?**

No. Memory (μ_m) is ephemeral — it is allocated fresh for each execution context and destroyed when execution completes. **Storage** (σ[a]_s) is the persistent key-value map that survives between transactions. Confusing the two is a common mistake in interviews.

---

**Q: What does it mean for EVM execution to be deterministic?**

Given the same world state σ and the same transaction T, every EVM implementation must produce the exact same output state σ', return data, gas usage, and logs. There is no randomness, no access to wall-clock time (TIMESTAMP comes from the block header, which is agreed upon by consensus), and no system calls. This determinism is what enables thousands of independent nodes to reach consensus on the state of a single chain.

---

**Q: Why did you implement U256 as a newtype over `[u8; 32]` instead of using a library?**

Pedagogical choice. In production (e.g., `revm`), you'd use a crate like `ruint` or `alloy-primitives::U256` backed by `[u64; 4]` limbs for performance. But implementing from raw bytes teaches you exactly how big-endian layout works, how carries propagate in addition, and why wrapping is the correct semantics. You also learn the Rust newtype pattern, operator overloading via `impl Add`, and `From`/`Into` conversion idioms — all of which show up in real codebase work.

# Interview Corner — Day 4

---

**Q: How is a CREATE2 address computed, and why is it useful?**

`address = keccak256(0xff ++ sender ++ salt ++ keccak256(initcode))[12:]`. The `0xff` prefix prevents collisions with CREATE addresses. The salt is a user-chosen 32-byte value. This makes the address **deterministic** — you can compute it before deploying. Use cases: factory patterns (e.g., Uniswap pair creation), counterfactual instantiation (reference a contract before it's deployed), and Account Abstraction (pre-compute wallet addresses for gasless onboarding).

---

**Q: What is the Host trait pattern and why is it important?**

The Host trait separates EVM execution semantics from state management. The interpreter calls `host.code(addr)` to read other contracts' code, `host.balance(addr)` to check balances, and `host.emit_log()` to write events. In tests, a MockHost uses HashMaps. In production, a real Host reads from disk-backed Merkle tries. This inversion of control makes the interpreter testable, portable, and backend-agnostic — the same pattern `revm` uses.

---

**Q: What is the gas cost of emitting a LOG4 with 100 bytes of data?**

`375 + 8 * 100 + 375 * 4 = 375 + 800 + 1500 = 2675 gas`. The base cost is 375, each byte of data costs 8 gas, and each topic costs 375 gas. Interviewers use this to test whether you know the formula and can compute it under pressure.

---

**Q: What is the difference between CODECOPY and EXTCODECOPY?**

`CODECOPY` copies the current contract's own bytecode into memory. `EXTCODECOPY` copies another account's bytecode. Both are used in proxy patterns — a proxy reads the implementation contract's code to understand its ABI. `CODECOPY` is also used in initcode to copy the runtime bytecode into memory before returning it.

---

**Q: Why does contract creation use initcode instead of deploying runtime code directly?**

Initcode allows constructors — code that runs once at deployment to initialize storage, validate parameters, or compute runtime code dynamically. The separation also enables the proxy pattern: initcode can deploy minimal proxy bytecode that delegates to an implementation contract. Without initcode, every contract would need to be fully static at deployment time.

# Deep Dive — Module 4

Review questions for Module 4. Try answering before expanding.

---

**Q: How is a CREATE2 address computed, and why is it useful?**

```admonish note title="Answer" collapsible=true
`address = keccak256(0xff ++ sender ++ salt ++ keccak256(initcode))[12:]`. The `0xff` prefix prevents collisions with CREATE addresses. The salt is a user-chosen 32-byte value. This makes the address **deterministic** — you can compute it before deploying. Use cases: factory patterns (e.g., Uniswap pair creation), counterfactual instantiation (reference a contract before it's deployed), and Account Abstraction (pre-compute wallet addresses for gasless onboarding).
```

---

**Q: What is the Host trait pattern and why is it important?**

```admonish note title="Answer" collapsible=true
The Host trait separates EVM execution semantics from state management. The interpreter calls `host.code(addr)` to read other contracts' code, `host.balance(addr)` to check balances, and `host.emit_log()` to write events. In tests, a MockHost uses HashMaps. In production, a real Host reads from disk-backed Merkle tries. This inversion of control makes the interpreter testable, portable, and backend-agnostic — the same pattern [revm](https://github.com/bluealloy/revm) uses.
```

---

**Q: What is the gas cost of emitting a LOG4 with 100 bytes of data?**

```admonish note title="Answer" collapsible=true
$375 + 8 \times 100 + 375 \times 4 = 375 + 800 + 1500 = 2675$ gas. The base cost is 375 (`G_log`), each byte of data costs 8 gas (`G_logdata`), and each topic costs 375 gas (`G_logtopic`). See [evm.codes — LOG4](https://www.evm.codes/#a4) for the interactive breakdown.
```

---

**Q: What is the difference between CODECOPY and EXTCODECOPY?**

```admonish note title="Answer" collapsible=true
`CODECOPY` copies the current contract's own bytecode into memory. `EXTCODECOPY` copies another account's bytecode. Both are used in proxy patterns — a proxy reads the implementation contract's code to understand its ABI. `CODECOPY` is also used in initcode to copy the runtime bytecode into memory before returning it.
```

---

**Q: Why does contract creation use initcode instead of deploying runtime code directly?**

```admonish note title="Answer" collapsible=true
Initcode allows constructors — code that runs once at deployment to initialize storage, validate parameters, or compute runtime code dynamically. The separation also enables the proxy pattern: initcode can deploy minimal proxy bytecode that delegates to an implementation contract. Without initcode, every contract would need to be fully static at deployment time.
```

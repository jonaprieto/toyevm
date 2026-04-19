# Deep Dive — Day 5

---

**Q: Explain the 63/64 gas rule.**

EIP-150 mandates that at most 63/64 of the remaining gas can be forwarded to a sub-call. This ensures the caller always retains at least 1/64 of its gas after the call returns. Before this rule, an attacker could create deeply nested calls where each level consumed nearly all remaining gas, making the gas cost of the outer call unpredictable and enabling denial-of-service attacks. The formula is: `forwarded = min(requested, remaining - remaining/64)`.

---

**Q: What is the difference between CALL and DELEGATECALL?**

CALL creates a new execution context where `msg.sender` is the calling contract and storage access is scoped to the callee's address. DELEGATECALL runs the callee's code but in the **caller's context** — storage reads/writes go to the caller's slots, and `msg.sender` is preserved (it's the original external caller, not the proxy). This is the foundation of upgradeable proxy contracts: the proxy's storage holds the state, and DELEGATECALL to the implementation contract provides the logic.

---

**Q: Why can't reentrancy be prevented at the EVM protocol level?**

The EVM protocol is correct — it accurately executes what the bytecode instructs. Reentrancy is a logic bug in Solidity code, not a protocol flaw. When Contract A calls Contract B, and B calls back into A before A finishes updating its state, the protocol faithfully executes both calls. The fix is at the application level: the Checks-Effects-Interactions (CEI) pattern, or a reentrancy guard that prevents re-entering a function while it's still executing. Making this a protocol rule would break legitimate use cases like flash loans and callback patterns.

---

**Q: What does STATICCALL enforce and at what layer?**

STATICCALL sets a `is_static` flag on the call frame. The interpreter checks this flag before executing any state-modifying opcode — `SSTORE`, `LOG0`..`LOG4`, `CREATE`, `CREATE2`, `CALL` with non-zero value, and `SELFDESTRUCT`. If a modification is attempted, execution reverts immediately with a `StateModificationInStaticCall` error. This enforcement happens at the interpreter layer, not the compiler layer — the bytecode can contain SSTORE instructions, but they trap at runtime if called via STATICCALL.

---

**Q: How does the call depth limit interact with the 63/64 rule?**

They're complementary defenses. The depth limit (1024) provides a hard cap on call nesting. The 63/64 rule provides an economic cap — even with unlimited depth, gas would be exhausted after ~63 levels of full-gas forwarding (since 63/64^63 ≈ 0.37, you'd have about 37% of original gas left after 63 levels, and the gas budget is finite). Together, they make deep call tree attacks both practically impossible (gas) and theoretically bounded (depth).

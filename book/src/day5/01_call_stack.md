# The Call Stack

When one contract calls another, the EVM creates a new **call frame** — a fresh execution environment with its own stack, memory, and program counter. The original frame is suspended until the sub-call returns.

## Call types

| Opcode | msg.sender | Storage | State mods | Value transfer |
|--------|-----------|---------|-----------|----------------|
| `CALL` | Caller's address | Callee's | Allowed | Yes |
| `DELEGATECALL` | **Original** caller | **Caller's** | Allowed | No |
| `STATICCALL` | Caller's address | Callee's | **Forbidden** | No |
| `CALLCODE` | Caller's address | Caller's | Allowed | Yes (deprecated) |

### CALL
The standard call. Creates a new context where `msg.sender` is the calling contract and storage access is scoped to the callee. Value (ETH) can be transferred.

### DELEGATECALL
The foundation of the **proxy pattern**. The callee's code runs in the caller's storage context. `msg.sender` and `msg.value` are preserved from the original call. This is how upgradeable contracts work — the proxy's storage is modified by the implementation's logic.

```admonish warning title="Pitfall: DELEGATECALL Storage Collisions"
When Contract A DELEGATECALLs Contract B, B's code writes to A's storage slots. If A and B have different storage layouts, B will overwrite A's variables at the wrong slots. This is the most common proxy bug.
```

### STATICCALL
A read-only call. Any opcode that modifies state (`SSTORE`, `LOG`, `CREATE`, `CALL` with value) will revert the sub-call. Used for view functions and safe external reads.

## The 63/64 Rule (EIP-150)

Before EIP-150, a contract could forward all its remaining gas to a sub-call. This enabled gas-exhaustion attacks: a malicious contract could create deeply nested calls, each consuming almost all gas, leaving the caller with nothing.

EIP-150 limits forwarded gas to **63/64 of remaining gas**:

$$G_{forward} = \min\left(G_{requested},\; G_{remaining} - \lfloor G_{remaining} / 64 \rfloor\right)$$

```rust
let max_forward = gas_remaining - gas_remaining / 64;
let actual = gas_requested.min(max_forward);
```

This ensures the caller always retains at least 1/64 of its gas after a sub-call, making gas accounting predictable.

## Depth limit

The call depth is limited to **1024**. The 1025th nested call fails immediately. Combined with the 63/64 rule, this makes deep call tree attacks doubly impractical.

## Call frame lifecycle

1. **Push frame**: save current state, create new frame with fresh stack/memory
2. **Execute**: run the callee's bytecode in the new frame
3. **Pop frame**: restore caller's state, copy return data
4. **Result**: caller reads success/failure and return data

If the sub-call reverts, the caller's state is untouched — only the sub-call's changes are discarded.

## Precompiles

Addresses 0x1 through 0x9 are **precompiled contracts** — built-in functions with fixed gas costs:

| Address | Function | Gas |
|---------|----------|-----|
| 0x1 | ecrecover (signature recovery) | 3000 |
| 0x2 | SHA-256 | 60 + 12/word |
| 0x3 | RIPEMD-160 | 600 + 120/word |
| 0x4 | identity (memcpy) | 15 + 3/word |

Precompiles look like regular CALLs but are handled specially by the interpreter — no bytecode is executed.

## Exercises

- **5.1** — Construct CallFrame for each call type, verify fields
- **5.2** — Verify MAX_CALL_DEPTH constant
- **5.4** — Gas forwarding via the 63/64 rule

Run: `cargo test call`

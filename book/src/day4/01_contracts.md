# Contracts, Deployment, and the Host Trait

Today we cross the boundary from a single-contract interpreter to a system that can create contracts, emit logs, and interact with external state.

## CREATE and CREATE2

When a contract creates another contract, the EVM needs to determine the new contract's address:

**CREATE**: $\text{addr} = \text{keccak256}(\text{rlp}([\text{sender}, \text{nonce}]))[12{:}]$

The address depends on the sender and their nonce (a counter that increments with each CREATE). This means the address is predictable only if you know the current nonce.

**CREATE2**: $\text{addr} = \text{keccak256}(\texttt{0xff} \| \text{sender} \| \text{salt} \| \text{keccak256}(\text{initcode}))[12{:}]$

CREATE2 (EIP-1014) adds a `salt` parameter, making the address deterministic — you can compute it before deployment. This enables:
- **Factory patterns**: deploy contracts at known addresses
- **Counterfactual instantiation**: reference a contract before it exists
- **Account Abstraction**: pre-compute wallet addresses

## Initcode vs. Runtime Code

When you deploy a contract, you don't send the runtime bytecode directly. You send **initcode** — a program that executes once and whose RETURN value becomes the deployed code.

```
Transaction: { to: null, data: initcode }
  → EVM executes initcode
  → initcode calls RETURN(offset, size)
  → bytes at memory[offset..offset+size] become the deployed code
```

This is why Solidity constructors can run arbitrary logic — they're part of the initcode.

## The Host Trait

The interpreter needs to access external state: other contracts' code, account balances, etc. We use the **Host trait** to decouple the interpreter from state management:

```rust
pub trait Host {
    fn code(&self, address: &Address) -> Vec<u8>;
    fn code_size(&self, address: &Address) -> usize;
    fn balance(&self, address: &Address) -> U256;
    fn emit_log(&mut self, log: LogEntry);
    fn logs(&self) -> &[LogEntry];
}
```

For testing, `MockHost` implements this trait with HashMaps. In production, a real Host would read from a Merkle trie.

```admonish tip title="Rust Pattern: Trait Objects"
The `Host` trait enables *inversion of control*. The interpreter calls `host.code(addr)` without knowing whether it's backed by a HashMap (testing) or a Merkle trie (production). This is the same pattern `revm` uses.
```

## LOG Opcodes

### LOG0 through LOG4 — emit event logs

`LOGn` pops an offset and size from the stack, then pops `n` topic values. It reads `size` bytes from memory starting at `offset` and creates a log entry.

```
Before:  Stack [ ..., topic1, 32, 0x00 ]  ← offset on top, size below, then topics
         Memory[0x00..0x20] = <event data>
LOG1: pop offset (0x00), pop size (32), pop topic (topic1)
      emit log { topics: [topic1], data: memory[0x00..0x20] }
After:   Stack [ ... ]
```

Gas cost: $375 + 8 \times \text{data\_length} + 375 \times \text{num\_topics}$

| Opcode | Topics | Gas (32 bytes data) |
|--------|--------|-------------------|
| `LOG0` | 0 | $375 + 256 = 631$ |
| `LOG1` | 1 | $375 + 256 + 375 = 1006$ |
| `LOG2` | 2 | $375 + 256 + 750 = 1381$ |
| `LOG3` | 3 | $375 + 256 + 1125 = 1756$ |
| `LOG4` | 4 | $375 + 256 + 1500 = 2131$ |

Logs are not accessible from within the EVM — they are write-only. Off-chain clients read them via the JSON-RPC `eth_getLogs` API.

## Exercises

- **4.1** — CREATE address derivation is deterministic and varies with sender/nonce
- **4.2** — CREATE2 address derivation varies with salt and initcode
- **4.3** — MockHost compiles and implements all Host methods
- **4.4** — EXTCODESIZE returns correct values via Host
- **4.5** — LOG0 gas cost matches the formula
- **4.6** — LOG2 emits correct topics

Run: `cargo test contract` and `cargo test host`

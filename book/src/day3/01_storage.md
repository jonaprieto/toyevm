# Storage and EIP-2929

Storage is the EVM's persistent memory. Unlike memory ($\mu_m$), which is wiped after each call, storage ($\sigma[a]_s$) persists between transactions. I found it helpful to think of it as a contract's own key-value database — a mapping from 256-bit keys to 256-bit values, scoped per contract address.

## SLOAD and SSTORE

### SLOAD — read a value from storage

`SLOAD` pops a key from the stack and pushes the corresponding value from storage. I noticed that uninitialized slots return zero — no need to set a default.

```
Before:  Stack [ ..., 0x01 ]       ← key on top
         Storage { 0x01: 42 }
SLOAD: pop key (0x01), read storage[0x01]
After:   Stack [ ..., 42 ]         ← value from storage
```

### SSTORE — write a value to storage

`SSTORE` pops a key and a value, then writes the value to storage at that key. Writing zero clears the slot — which is how Solidity "deletes" a mapping entry.

```
Before:  Stack [ ..., 42, 0x01 ]   ← key on top, value below
         Storage { }
SSTORE: pop key (0x01), pop value (42)
        write storage[0x01] = 42
After:   Stack [ ... ]
         Storage { 0x01: 42 }
```

## Cold vs. Warm Access (EIP-2929)

Before the Berlin upgrade (EIP-2929), SLOAD cost 800 gas (since the Istanbul upgrade / EIP-1884; earlier it was 200, and originally 50). I was surprised to learn this was a security problem — an attacker could read many storage slots cheaply, while every node had to perform expensive disk I/O for each one.

EIP-2929 fixed this by introducing the **accessed storage keys** set:

| Access type | SLOAD cost | Description |
|------------|-----------|-------------|
| **Cold** | $G_{coldsload} = 2100$ | First access to a slot in this transaction |
| **Warm** | $G_{warmaccess} = 100$ | Slot already accessed earlier in this transaction |

The first time we touch a slot, the node must fetch it from disk (cold). After that, it's in memory (warm). The gas costs reflect this hardware reality.

```admonish danger title="EIP-2929: Breaking Change"
EIP-2929 raised cold SLOAD costs ~2.6x (800 → 2100). Combined with the earlier EIP-1884 increase (200 → 800), contracts that hardcoded gas assumptions broke on mainnet. Always use the warm/cold model.
```

```rust
pub fn sload_gas(&self, key: &U256) -> u64 {
    if self.warm_slots.contains(key) {
        100   // WARM_STORAGE_READ_COST
    } else {
        2100  // COLD_SLOAD_COST
    }
}
```

## SSTORE gas (simplified)

SSTORE has the most complex gas calculation I encountered in the EVM. Here's the simplified model we implement:

| Transition | Base cost |
|-----------|-----------|
| Zero → non-zero | 20,000 gas (new slot allocation) |
| Non-zero → non-zero | 2,900 gas (modification) |
| + cold access surcharge | +2,100 gas |

The full model (EIP-3529) also includes gas refunds for clearing storage, but I left that out here to keep things readable.

## Exercises

- **3.3** — Store a value, load it back, verify round-trip
- **3.4** — Verify cold SLOAD costs 2100 and warm SLOAD costs 100

Run: `cargo test storage` and `cargo test ex_3_3` and `cargo test ex_3_4`

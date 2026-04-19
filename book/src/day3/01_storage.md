# Storage and EIP-2929

Storage is the EVM's persistent memory. Unlike memory (μ_m), which is wiped after each call, storage (σ[a]_s) persists between transactions. It's a mapping from 256-bit keys to 256-bit values, scoped per contract address.

## SLOAD and SSTORE

| Opcode | Stack | Effect |
|--------|-------|--------|
| `SLOAD` | `[key]` → `[value]` | Read from storage |
| `SSTORE` | `[key, value]` → `[]` | Write to storage |

Uninitialized slots read as zero. Writing zero to a slot is equivalent to deleting it.

## Cold vs. Warm Access (EIP-2929)

Before the Berlin upgrade (EIP-2929), SLOAD cost 800 gas (since the Istanbul upgrade / EIP-1884; earlier it was 200, and originally 50). This was a security problem — an attacker could read many storage slots cheaply, while the node had to perform expensive disk I/O for each.

EIP-2929 introduced the **accessed storage keys** set:

| Access type | SLOAD cost | Description |
|------------|-----------|-------------|
| **Cold** | $G_{coldsload} = 2100$ | First access to a slot in this transaction |
| **Warm** | $G_{warmaccess} = 100$ | Slot already accessed earlier in this transaction |

The first time you touch a slot, the node must fetch it from disk (cold). After that, it's in memory (warm). The gas costs reflect this reality.

```admonish danger title="EIP-2929: Breaking Change"
EIP-2929 changed SLOAD costs by 26x (800 → 2100 for cold). Contracts that hardcoded gas assumptions (e.g., `.transfer()` with 2300 gas stipend) broke on mainnet. Always use the warm/cold model.
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

SSTORE is the most complex gas calculation in the EVM. The simplified model we implement:

| Transition | Base cost |
|-----------|-----------|
| Zero → non-zero | 20,000 gas (new slot allocation) |
| Non-zero → non-zero | 2,900 gas (modification) |
| + cold access surcharge | +2,100 gas |

The full model (EIP-3529) also includes gas refunds for clearing storage, but we omit that for clarity.

## Exercises

- **3.3** — Store a value, load it back, verify round-trip
- **3.4** — Verify cold SLOAD costs 2100 and warm SLOAD costs 100

Run: `cargo test storage` and `cargo test ex_3_3` and `cargo test ex_3_4`

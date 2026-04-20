# Exercises — Day 4

```admonish example title="Exercise Setup" collapsible=true
    git checkout -b my-day-4 my-day-3

Create stub files:

- `src/contract.rs` — `create_address()`, `create2_address()`, RLP helpers
- `src/host.rs` — `Host` trait, `LogEntry`, `MockHost`

Add `pub mod contract; pub mod host;` to `src/lib.rs`.

Peek at signatures: `git show day-4:src/host.rs | head -50`

Compare when done: `git diff day-4 -- src/contract.rs src/host.rs`
```

These exercises cover contract creation address derivation, the Host trait, and LOG opcode gas accounting.

## Host Trait (reference)

```rust
{{#include ../../../src/host.rs:host_trait}}
```

---

## Exercise 4.1 — CREATE address derivation

**What to do.** Write a test that calls your `create_address` helper with two different
sender/nonce pairs and asserts the outputs differ, then calls it twice with the **same**
pair and asserts the outputs are equal.

**How to verify.**

```
cargo test ex_4_1
```

**Hint.** The Yellow Paper formula is `address = keccak256(rlp([sender, nonce]))[12:]`.
Use the `Address` newtype so the compiler catches raw-byte mistakes at the type boundary.

> **Rust pattern — newtype for `Address`.** Wrap `[u8; 20]` in a tuple struct
> (`pub struct Address([u8; 20])`). This prevents accidentally passing a `[u8; 20]`
> hash where an address is expected.

---

## Exercise 4.2 — CREATE2 address derivation

**What to do.** Write a test that calls `create2_address` with two different salts and
asserts the results differ, then two different initcodes and asserts the results differ,
then a fixed (sender, salt, initcode) triple twice and asserts equality.

**How to verify.**

```
cargo test ex_4_2
```

**Hint.** The preimage is exactly `0xff ++ sender ++ salt ++ keccak256(initcode)`.
Build it as a `Vec<u8>` by concatenating byte slices; the length is always `1 + 20 + 32 + 32 = 85`.

> **Rust pattern — preimage construction.** Use `Vec::with_capacity(85)` and
> `extend_from_slice` to assemble multi-part byte strings without heap reallocations.

---

## Exercise 4.3 — MockHost compiles and implements Host

**What to do.** Confirm that `MockHost` satisfies all five methods of the `Host` trait.
Deploy code at an address, check `code_size`, read a balance, and verify an EOA has
zero code size and zero balance.

**How to verify.**

```
cargo test ex_4_3
```

**Hint.** The default method `code_size` is already provided by the trait — you get it
for free once `code` is implemented. Use this in your test to confirm default methods
are inherited.

> **Rust pattern — trait objects and inversion of control.** Store the host as
> `Box<dyn Host>` in integration tests. Swapping `MockHost` for a real host requires
> no changes to the interpreter code.

---

## Exercise 4.4 — EXTCODESIZE returns correct values

**What to do.** Deploy 100 bytes of code at address `0x42`. Assert `code_size` returns
`100`. Then call `code_size` on `Address::ZERO` (an EOA) and assert it returns `0`.

**How to verify.**

```
cargo test ex_4_4
```

**Hint.** The default `code_size` implementation delegates to `self.code(address).len()`.
You only need to implement `code` — the rest follows.

> **Rust pattern — default trait methods.** Providing `code_size` as a default method
> avoids duplicating logic in every implementor, while still allowing overrides for
> performance-critical cases (e.g., reading only the length from a trie node).

---

## Exercise 4.5 — LOG0 gas cost matches the formula

**What to do.** Write a pure arithmetic test that verifies
$G_{log} + G_{logdata} \times \text{data\_len} = \text{expected}$ for two inputs: 0 bytes (cost = 375) and 32 bytes
(cost = 631). Do not execute any bytecode — test the formula directly.

**How to verify.**

```
cargo test ex_4_5
```

**Hint.** Yellow Paper Section 9.2 defines $G_{log} = 375$, $G_{logdata} = 8$,
$G_{logtopic} = 375$. For LOG0 there are no topics, so the topic term is zero.

> **Rust pattern — formula verification in tests.** Small arithmetic unit tests pin
> protocol constants early and catch regressions if a constant is refactored.

---

## Exercise 4.6 — LOG2 topics have correct structure

**What to do.** Construct a `LogEntry` with two topics (`U256::from(0xAA)` and
`U256::from(0xBB)`) and 10 bytes of data. Emit it through `MockHost::emit_log`, then
assert `logs()[0].topics.len() == 2` and both topic values are correct.

**How to verify.**

```
cargo test ex_4_6
```

**Hint.** Topics are stored as `Vec<U256>` — index them with `[0]` and `[1]` directly.

> **Rust pattern — `Vec<T>` for variable-length fields.** The number of topics is
> 0–4 per opcode (LOG0–LOG4). Using `Vec<U256>` instead of `[Option<U256>; 4]` keeps
> the representation honest: absent topics are simply absent, not `None`.

---

## Yellow Paper Map

| Symbol | Meaning | Section |
|--------|---------|---------|
| **T** | Transaction | §4 |
| **Ξ** | Contract creation function | §7 |
| `G_log` / `G_logdata` / `G_logtopic` | LOG gas schedule | §9 |
| `L_A(s, n)` | CREATE address derivation | §7.1 |
| `CREATE2` preimage | `0xff ++ s ++ ζ ++ keccak(i)` | §7.1 |

Run all Day 4 tests: `cargo test contract` and `cargo test host`

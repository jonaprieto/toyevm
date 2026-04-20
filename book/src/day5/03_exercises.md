# Exercises — Day 5

```admonish example title="Exercise Setup" collapsible=true
    git checkout -b my-day-5 my-day-4

Create stub file:

- `src/call.rs` — `CallFrame`, `CallType`, `gas_to_forward()`

Add `pub mod call;` to `src/lib.rs`.

Peek: `git show day-5:src/call.rs | head -130`

Compare: `git diff day-5 -- src/call.rs`
```

These exercises cover call frame construction, the depth limit, and the 63/64 gas-forwarding rule (EIP-150).

## 63/64 Rule (reference)

```rust
{{#include ../../../src/call.rs:gas_forward}}
```

---

## Exercise 5.1 — CallFrame construction

**What to do.** Construct one frame for each of CALL, DELEGATECALL, and STATICCALL using
the provided constructors. For each frame, assert the correct `call_type`, `code_address`,
`context_address`, `caller`, and `is_static` flag.

Key invariants to verify:

| Type | `context_address` | `is_static` | `value` |
|------|------------------|-------------|---------|
| `CALL` | same as `code_address` | `false` | any |
| `DELEGATECALL` | caller's address | `false` | preserved |
| `STATICCALL` | same as `code_address` | **`true`** | `U256::ZERO` |

**How to verify.**

```
cargo test ex_5_1
```

**Hint.** For DELEGATECALL the storage context belongs to the caller, not the callee.
Pass `context_address = caller_addr` to `new_delegatecall`. Asserting
`frame.context_address != frame.code_address` is a good extra check.

> **Rust pattern — enum variants with constructors.** Each `new_*` function encodes the
> invariants for its call type. Using `CallType::DelegateCall` in a `match` later lets
> the compiler remind you to handle all variants.

---

## Exercise 5.2 — Depth limit is exactly 1024

**What to do.** Assert that the exported constant `MAX_CALL_DEPTH` equals `1024`.
Also write a comment explaining why this limit exists.

**How to verify.**

```
cargo test ex_5_2
```

**Hint.** The 1025th call must fail with a call-depth exceeded error, not a stack
overflow in the host language. The limit is a Yellow Paper protocol constant, not
a Rust implementation detail.

> **Rust pattern — protocol constants.** Naming constants with `pub const` in the
> module that owns their semantics (`call.rs`) keeps the definition co-located with
> the enforcement logic and makes them importable everywhere without repetition.

---

## Exercise 5.4 — Gas forwarding follows the 63/64 rule

**What to do.** Call `gas_to_forward` with at least four cases and assert the results:

1. `gas_remaining = 6400`, `gas_requested = 10000` → `6300` (capped at 63/64)
2. `gas_remaining = 6400`, `gas_requested = 1000` → `1000` (requested is below cap)
3. `gas_remaining = 64`, `gas_requested = 1000` → `63`
4. `gas_remaining = 1`, `gas_requested = 1000` → `1`

**How to verify.**

```
cargo test ex_5_4
```

**Hint.** Integer division truncates in Rust: `gas_remaining / 64` rounds down, so
$G_{remaining} - \lfloor G_{remaining} / 64 \rfloor$ is always at least $\lfloor G_{remaining} \times 63 / 64 \rfloor$.
The `min` call picks whichever is smaller: the cap or the request.

> **Rust pattern — min/max arithmetic.** `a.min(b)` is idiomatic Rust for
> `if a < b { a } else { b }`. Prefer it over manual branches to keep the forwarding
> formula on one line and easy to audit against the Yellow Paper.

---

## Yellow Paper Map

| Symbol | Meaning | Section |
|--------|---------|---------|
| **Θ** | Message call function | §8 |
| $\mu_s[0]$ | Gas stipend on the stack | §8.1 |
| EIP-150 | 63/64 gas forwarding rule | §8 |
| `n` | Call depth counter | §8 |
| `MAX_CALL_DEPTH` | Protocol constant (1024) | §8 |

Run all Day 5 tests: `cargo test call`

# Exercises — Day 1

```admonish example title="Exercise Setup" collapsible=true
**Start from the stub branch:**

    git checkout day-1-start
    git checkout -b my-day-1

The files [`src/types/u256.rs`](https://github.com/jonaprieto/toyevm/blob/day-1-start/src/types/u256.rs), [`src/stack.rs`](https://github.com/jonaprieto/toyevm/blob/day-1-start/src/stack.rs), and [`src/memory.rs`](https://github.com/jonaprieto/toyevm/blob/day-1-start/src/memory.rs) contain stubs with `todo_exercise!()` bodies and all tests intact.

    cargo test    # 33 tests fail with "Exercise not yet implemented"

Implement until all 37 tests pass, then compare:

    git diff day-1 -- src/types src/stack.rs src/memory.rs
```

These exercises lock in the primitives you have just read about: `U256`, `Stack`,
and `Memory`. For each exercise you will write or verify a small piece of code,
run a focused test suite, and encounter a Rust pattern that you will see
repeatedly throughout the rest of this tutorial.

> **How to work through these:** read the exercise, then attempt an
> implementation before looking at the hint. The goal is productive struggle,
> not speed.

---

## Exercise 1.1 — U256 Zero and One Constants

**What to implement.** Verify that `U256::ZERO` and `U256::ONE` have the exact
bit patterns prescribed by the EVM: 32 zero bytes and 31 zero bytes followed by
`0x01`, respectively. Add assertions in a unit test inside `src/types/u256.rs`
that inspect the raw byte array returned by `to_bytes()`.

**File:** [`src/types/u256.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/types/u256.rs)

**How to verify:**

```sh
cargo test ex_1_1
```

**The struct and the constants look like this in our codebase:**

```rust
{{#include ../../../src/types/u256.rs:u256_struct}}
```

```rust
{{#include ../../../src/types/u256.rs:u256_constants}}
```

**Hint.** Call `.to_bytes()` on each constant, then index the array directly —
`bytes[0]` is the most significant byte, `bytes[31]` is the least significant.
For `ONE`, only `bytes[31]` should be non-zero.

> **Rust pattern — const evaluation.**
> `U256::ZERO` and `U256::MAX` use `const` items: the compiler evaluates them
> at compile time and embeds the result as a read-only literal in the binary.
> `U256::ONE` goes further and uses a `const` block (`const { … }`) to run a
> loop that would ordinarily require `mut` — the compiler allows mutation inside
> a `const` context as long as nothing leaks out. This is how you build
> non-trivial constants without heap allocation or lazy initialisation.

---

## Exercise 1.2 — U256 Wrapping Addition

**What to implement.** Write two test cases for `wrapping_add`:
1. `U256::from(2u64) + U256::from(3u64)` should equal `U256::from(5u64)`.
2. `U256::MAX + U256::ONE` should equal `U256::ZERO` (overflow wraps to zero).

**File:** [`src/types/u256.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/types/u256.rs)

**How to verify:**

```sh
cargo test ex_1_2
```

**The implementation you are testing:**

```rust
{{#include ../../../src/types/u256.rs:wrapping_add}}
```

**Hint.** Use `U256::from(n as u64)` to build small test values. To build
`U256::MAX` directly, use the constant. Inspect the `.to_bytes()` result if an
assertion fails — the 32-byte dump is the clearest debugging aid.

> **Rust pattern — operator overloading via `impl Add`.**
> Our `U256` does not have a built-in `+` operator; it is provided by
> `impl std::ops::Add for U256`. The compiler rewrites `a + b` into
> `a.add(b)`. This means `wrapping_add` is the single source of truth: the
> `Add` impl just delegates to it. When you need wrapping semantics everywhere
> (as the EVM does), centralising the logic this way eliminates a whole class of
> accidental checked-addition bugs.

---

## Exercise 1.3 — U256 Comparison

**What to implement.** Add test cases that verify `<`, `>`, and `==` on
`U256` values:
- `U256::ZERO < U256::ONE` is `true`.
- `U256::MAX > U256::ONE` is `true`.
- `U256::from(42u64) == U256::from(42u64)` is `true`.
- `U256::from(1u64) != U256::from(2u64)` is `true`.

**File:** [`src/types/u256.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/types/u256.rs)

**How to verify:**

```sh
cargo test ex_1_3
```

**Hint.** `U256` derives `PartialEq` and `Eq`. For ordering it implements
`PartialOrd` and `Ord` manually, doing a big-endian byte-by-byte lexicographic
comparison. Write your tests using standard Rust comparison operators (`<`, `>`);
you do not need to call any special method.

> **Rust pattern — deriving vs manual `Ord` impl.**
> `#[derive(PartialEq, Eq)]` is enough when the compiler-generated comparison
> (field-by-field, in declaration order) happens to be correct. For `U256`, it
> *is* correct because the inner `[u8; 32]` is already big-endian, and Rust
> compares arrays lexicographically. However, deriving `Ord` for a newtype only
> works when the inner type derives `Ord`. Knowing when to trust `#[derive]` and
> when to write a manual impl is a core Rust skill.

---

## Exercise 1.4 — From Conversions

**What to implement.** Verify the three `From` impls that convert native integer
types into `U256`:
- `U256::from(0u64)` equals `U256::ZERO`.
- `U256::from(u64::MAX)` has its eight least-significant bytes set and all
  upper bytes zero.
- `U256::from(1u128)` equals `U256::ONE`.
- `U256::from([0xFF_u8; 32])` equals `U256::MAX`.

Write one test function per conversion.

**File:** [`src/types/u256.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/types/u256.rs)

**How to verify:**

```sh
cargo test ex_1_4
```

**Hint.** Use `.to_bytes()` to inspect the result. For `u64::MAX`, bytes 24–31
should all be `0xFF` and bytes 0–23 should be `0x00`. The `[u8; 32]` conversion
is a no-op — it just wraps the array.

> **Rust pattern — the `From`/`Into` idiom.**
> Implementing `From<T> for U` automatically gives you `Into<U> for T` for free
> via a blanket impl in the standard library. This means callers can write either
> `U256::from(42u64)` or `42u64.into()` — the same underlying code runs. The
> convention in Rust is: implement `From`, get `Into` gratis. Never implement
> `Into` directly.

---

## Exercise 1.5 — Stack Push and Pop

**What to implement.** Write a test that:
1. Creates a fresh `Stack`.
2. Pushes three distinct `U256` values (e.g., `1`, `2`, `3`).
3. Pops them and checks that the order is LIFO (`3` first, then `2`, then `1`).
4. Verifies that popping an empty stack returns `Err(StackError::Underflow)`.

**File:** [`src/stack.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/stack.rs)

**How to verify:**

```sh
cargo test ex_1_5
```

**The push and pop implementations:**

```rust
{{#include ../../../src/stack.rs:stack_push_pop}}
```

**Hint.** Store the return values of each `pop()` call in a `let` binding and
`unwrap()` them in the test — a panic on `unwrap` is the most readable failure
message. For the underflow check, use `assert_eq!(stack.pop(), Err(StackError::Underflow))`.

> **Rust pattern — `Result<T, E>` for fallible operations.**
> `push` and `pop` both return `Result`. Callers *must* handle the error path
> — the compiler will warn about an unused `Result`. Inside tests you can call
> `.unwrap()`, but in the interpreter you must match on the error and propagate
> it. This forces you to think about every failure mode at the point of use,
> not after the fact.

---

## Exercise 1.6 — Stack Depth Limit

**What to implement.** Write a test that pushes exactly 1024 values onto the
stack (all successes), then asserts that the 1025th push returns
`Err(StackError::Overflow)`.

**File:** [`src/stack.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/stack.rs)

**How to verify:**

```sh
cargo test ex_1_6
```

**The `Stack` struct and error type:**

```rust
{{#include ../../../src/stack.rs:stack_struct}}
```

```rust
{{#include ../../../src/stack.rs:stack_error}}
```

**Hint.** Use a `for` loop: `for i in 0..1024 { stack.push(U256::from(i as u64)).unwrap(); }`.
After the loop, do *not* call `unwrap()` on the 1025th push — match it or use
`assert_eq!` with the expected error variant.

> **Rust pattern — custom error types.**
> `StackError` is a plain `enum` that derives `Debug`, `Clone`, `PartialEq`,
> and `Eq`. Deriving `PartialEq` is what lets you write
> `assert_eq!(result, Err(StackError::Overflow))` — without it, you would need
> a manual `match`. The `Display` impl gives a human-readable message for
> log output and the `Error` impl integrates with the broader Rust error
> ecosystem (`Box<dyn Error>`, `anyhow`, etc.).

---

## Exercise 1.7 — Memory Load and Store

**What to implement.** Write a round-trip test:
1. Create a `Memory`.
2. Store `U256::from(0xDEAD_BEEF_u64)` at offset `0`.
3. Load a `U256` back from offset `0` and assert it equals the stored value.
4. Repeat at offset `32` (the second word) to confirm non-overlapping writes.

**File:** [`src/memory.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/memory.rs)

**How to verify:**

```sh
cargo test ex_1_7
```

**Hint.** Look at the `store_word` and `load_word` methods in `memory.rs`. Both
take a `usize` offset. The store writes 32 bytes starting at that offset; the
load reads 32 bytes and reconstructs a `U256`. If the second round-trip fails,
check whether both stores triggered separate expansions.

> **Rust pattern — slice operations with `copy_from_slice`.**
> `copy_from_slice` writes a source slice into a mutable destination slice of
> the *same length*. If the lengths differ, it panics at runtime. EVM memory
> operations always deal in 32-byte words, so slicing as `&mut data[offset..offset+32]`
> is always safe after `expand_to` has ensured the backing `Vec` is large enough.
> This pattern avoids unsafe pointer arithmetic while still being zero-copy.

---

## Exercise 1.8 — Memory Expansion and Zero Padding

**What to implement.** Verify the expansion semantics:
1. Start with an empty `Memory` (`len()` is `0`).
2. Write a `U256` at offset `64` (the third word).
3. Assert `memory.len()` is `96` (three 32-byte words: `0..32`, `32..64`, `64..96`).
4. Assert that bytes `0..64` are all zero — the two words before the write were
   implicitly zero-initialised.

**File:** [`src/memory.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/memory.rs)

**How to verify:**

```sh
cargo test ex_1_8
```

**The expansion helper:**

```rust
{{#include ../../../src/memory.rs:memory_expand}}
```

**Hint.** After the write, call a method that exposes the raw byte slice (or
iterate over `memory.load_byte(i)` for `i` in `0..64`). The important property
is that bytes you never wrote are zero — the EVM guarantees this.

> **Rust pattern — `Vec::resize`.**
> `Vec::resize(new_len, value)` extends the vector to `new_len` elements,
> filling the new slots with `value` (here `0u8`). If `new_len` is smaller than
> the current length it truncates. This single call replaces a manual loop that
> would push individual zeroes. It is the canonical way to zero-extend a byte
> buffer in Rust.

---

## Yellow Paper Map

| Yellow Paper symbol | Our type / field | Notes |
|---|---|---|
| σ (world state) | Not yet modelled | Introduced in Day 3 |
| μ (machine state) | Fields of `ExecutionContext` | Combined in the interpreter |
| μ_s (stack) | `Stack` | Exercises 1.5 and 1.6 |
| μ_m (memory) | `Memory` | Exercises 1.7 and 1.8 |
| μ_g (gas) | `u64` field | Introduced in Day 2 |
| μ_pc (program counter) | `usize` field | Introduced in Day 2 |
| Values on μ_s | `U256` | Exercises 1.1 – 1.4 |

All values in the range \[0, 2²⁵⁶) correspond to `U256`. When the Yellow Paper
writes "let x ≡ μ_s[0]" it means "pop the top of the stack into x". When it
writes "μ_m[i]" it means "byte i in memory". These direct mappings are why we
model the types so literally — the code reads like the spec.

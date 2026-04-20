# Exercises — Day 2

```admonish example title="Exercise Setup" collapsible=true
**Create your working branch:**

    git checkout -b my-day-2 my-day-1    # branch from your completed Day 1

**Create stub files** — copy the type definitions and method signatures from the reference, replacing every function body with `todo_exercise!()`:

- `src/opcode.rs` — the `Opcode` enum with `from_byte()` and `immediate_size()`
- `src/gas.rs` — gas cost constants and `memory_gas_cost()` / `memory_expansion_cost()`
- `src/interpreter.rs` — the `Interpreter` struct and `run()` method

Add `pub mod opcode; pub mod gas; pub mod interpreter;` to `src/lib.rs`.

You can peek at the reference signatures with:

    git show day-2:src/opcode.rs | head -160

Then implement until all tests pass and compare:

    git diff day-2 -- src/opcode.rs src/gas.rs src/interpreter.rs
```

Day 2 introduced the fetch-decode-execute loop: how raw bytes become `Opcode`
variants, how each opcode manipulates the stack, and how gas is consumed. These
exercises walk the full pipeline from bytecode bytes to execution results.

> **Running the test suites.** Two test filters cover most of Day 2:
> ```sh
> cargo test opcode
> cargo test interpreter
> ```
> Run them together with `cargo test opcode -- && cargo test interpreter --` to
> see the full picture.

---

## Exercise 2.1 — Opcode Decoding

**What to implement.** Write tests that call `Opcode::try_from(byte)` (or the
equivalent `from_byte` constructor) for several known opcodes and verify:
- `0x00` → `Ok(Opcode::STOP)`
- `0x01` → `Ok(Opcode::ADD)`
- `0x60` → `Ok(Opcode::PUSH1)`
- An unknown byte such as `0xEF` → `Err(…)` (invalid opcode)

**File:** [`src/opcode.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/opcode.rs)

**How to verify:**

```sh
cargo test ex_2_1
```

**Hint.** The `Opcode` enum is `#[repr(u8)]`, so every variant is stored as its
byte value. `TryFrom<u8>` uses an exhaustive `match` (or a transmute guarded by
a validity check) to convert. Check what `from_byte` actually returns — it may
be `Option<Opcode>` or `Result<Opcode, InvalidOpcode>`.

> **Rust pattern — `repr(u8)` enum + `TryFrom`.**
> Marking an enum `#[repr(u8)]` tells the compiler to store each variant as a
> single byte with the assigned discriminant. This lets you cast between the
> enum and `u8` in both directions — safely via `TryFrom`, or unsafely via
> `transmute`. The `TryFrom` impl is the safe boundary: it checks that the byte
> is a valid discriminant before constructing the variant. Never use raw
> `transmute` from untrusted bytes; always go through `TryFrom`.

---

## Exercise 2.2 — PUSH Immediate Sizes

**What to implement.** Write tests that call an `Opcode` method (e.g.,
`immediate_size()` or `push_size()`) and verify:
- `Opcode::PUSH1.immediate_size()` → `1`
- `Opcode::PUSH2.immediate_size()` → `2`
- `Opcode::PUSH32.immediate_size()` → `32`
- `Opcode::ADD.immediate_size()` → `0` (no immediate bytes)

**File:** [`src/opcode.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/opcode.rs)

**How to verify:**

```sh
cargo test ex_2_2
```

**Hint.** PUSH opcodes run from `PUSH1` (0x60) through `PUSH32` (0x7F). The
immediate size is `(opcode_byte - 0x5F)`. Look for the method in `opcode.rs`
that computes this; if it does not exist, add it.

> **Rust pattern — enum methods.**
> An enum is just a type in Rust; you can hang `impl` blocks on it exactly as
> you would on a struct. Adding `fn immediate_size(&self) -> usize` to `Opcode`
> keeps the size logic co-located with the enum definition. The alternative —
> a freestanding `fn push_size(op: Opcode)` in the interpreter — scatters
> knowledge about opcodes across multiple files and makes refactoring harder.

---

## Exercise 2.3 — ADD Execution

**What to implement.** Assemble a minimal bytecode sequence by hand and run it
through the interpreter:

```text
PUSH1 0x03   # push 3
PUSH1 0x05   # push 5
ADD          # pop both, push 8
STOP
```

Assert that after execution the top of the stack is `U256::from(8u64)`.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs) (add a test at the bottom)

**How to verify:**

```sh
cargo test ex_2_3
```

**Hint.** Build the bytecode as `vec![0x60, 0x03, 0x60, 0x05, 0x01, 0x00]`.
Construct an `ExecutionContext` (or whatever the interpreter entry point is)
with this bytecode and a generous gas limit, then call `run()` or `execute()`.
Inspect the stack after execution.

> **Rust pattern — the fetch-decode-execute loop.**
> Every iteration of the main interpreter loop does three things: (1) read the
> byte at `pc`, (2) decode it into an `Opcode`, (3) execute the opcode's
> semantics. This separation is deliberate — fetch knows nothing about semantics,
> and the execute step does not touch `pc` directly. Keeping these responsibilities
> distinct makes each step testable in isolation and the whole loop easier to
> reason about.

---

## Exercise 2.4 — Gas Accounting

**What to implement.** Using the same bytecode as Exercise 2.3, verify the exact
gas consumed:
- `PUSH1` costs 3 gas (×2 = 6).
- `ADD` costs 3 gas.
- `STOP` costs 0 gas.
- Total: 9 gas.

Assert that `result.gas_used` (or `gas_limit - remaining_gas`) equals `9`.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_2_4
```

**Hint.** Look at what the interpreter returns — it is probably a struct with
fields for the final stack state, remaining gas, output, and a status code.
Use `gas_limit - gas_remaining` if there is no `gas_used` field directly.

> **Rust pattern — resource tracking.**
> Gas is a u64 counter that the interpreter decrements before every opcode
> execution. Doing the subtraction *before* the operation (not after) is
> important: if the gas runs out mid-execution, the operation should not have
> happened. This "debit first, act second" pattern is the same pattern used for
> memory allocation, network quotas, and rate limiters in production Rust code.

---

## Exercise 2.5 — MUL, SUB, DIV

**What to implement.** Write one test each for `MUL`, `SUB`, and `DIV`:
- `6 * 7` should leave `42` on the stack.
- `10 - 3` should leave `7`.
- `20 / 4` should leave `5`.

Use `PUSH1` + the opcode + `STOP` for each test.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_2_5
```

**Hint.** Remember stack order: `PUSH1 a`, `PUSH1 b`, `SUB` computes `b - a`
(the second item pushed minus the first — i.e., top-of-stack is subtracted
from the item below). Check the Yellow Paper's stack diagram for each opcode
to confirm operand order.

> **Rust pattern — exhaustive `match`.**
> The interpreter's dispatch is a `match opcode { … }` with one arm per opcode.
> If you add a new `Opcode` variant, the compiler will refuse to compile until
> you add a corresponding arm — there is no silent default that drops the case.
> This exhaustiveness check is one of the most powerful safety properties of
> Rust enums. The alternative (`if`/`else if` chains or a `HashMap<u8, fn>`)
> would not give you this guarantee.

---

## Exercise 2.6 — Division by Zero

**What to implement.** Verify the EVM's specified behaviour: `n / 0 = 0` for
any `n`. Write a test that executes `PUSH1 0x05`, `PUSH1 0x00`, `DIV`, `STOP`
and asserts the top of the stack is `U256::ZERO`.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_2_6
```

**Hint.** The EVM specification (Yellow Paper §9.4.2) explicitly defines
`x ÷ 0 ≡ 0`. Your `U256::checked_div` or the interpreter's DIV arm must handle
this. If the test panics instead of returning zero, find the division
implementation and add a zero-divisor guard.

> **Rust pattern — domain-specific semantics vs panics.**
> Rust's built-in `/` operator panics on integer division by zero in both
> debug and release builds — unlike C, Rust never produces undefined behaviour
> for integer arithmetic. The EVM intentionally diverges from Rust: division by
> zero is not an error, it is a well-defined operation returning zero. This is a
> case where the correct Rust idiom is *not* to use the standard operator —
> instead, the interpreter handles the zero case explicitly, returning
> `U256::ZERO` before calling any division primitive.

---

## Exercise 2.7 — LT, GT, EQ

**What to implement.** Verify that comparison opcodes produce `0` or `1`:
- `PUSH1 5`, `PUSH1 3`, `LT` → top of stack is `1` (3 < 5, since 3 is on top).
- `PUSH1 3`, `PUSH1 5`, `LT` → top of stack is `0` (5 is not < 3, since 5 is on top).
- `PUSH1 7`, `PUSH1 7`, `EQ` → top of stack is `1`.

Write at least one test for each of `LT`, `GT`, and `EQ`.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_2_7
```

**Hint.** Watch the operand order carefully. On the EVM, `LT` pops `a` (top),
then `b` (second), and pushes `1` if `a < b`. Double-check which item is "top"
in your test setup.

> **Rust pattern — boolean-to-integer conversion.**
> In Rust, `bool as u64` yields `1u64` for `true` and `0u64` for `false`. The
> interpreter can use this to convert a Rust boolean comparison directly into a
> `U256`: `U256::from((a < b) as u64)`. This is more idiomatic than an
> `if/else` that manually returns `U256::ONE` or `U256::ZERO`, and the compiler
> optimises it to a single conditional move (`cmov`) with no branch.

---

## Exercise 2.8 — Out of Gas

**What to implement.** Run the ADD sequence from Exercise 2.3 with `gas_limit = 1`.
The sequence needs 9 gas; with only 1 available it should fail. Assert that
execution returns an `Err` (or a halted result) indicating out-of-gas, and that
the output is empty.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_2_8
```

**Hint.** Look at the `ExecutionResult` or equivalent type that `run()` returns.
There should be a variant or field for `OutOfGas`. If execution returns `Ok` even
when gas is exhausted, the gas check is missing from the dispatch loop.

> **Rust pattern — early return with `Result`.**
> The gas check inside the interpreter loop is a guard clause:
> ```rust
> if self.gas < cost {
>     return Err(ExecutionError::OutOfGas);
> }
> self.gas -= cost;
> ```
> Returning early keeps the happy path unindented and makes the error condition
> visible at the top of the block. This "guard clause" style is idiomatic Rust
> and avoids deep nesting that obscures the main logic.

---

## Exercise 2.9 — STOP

**What to implement.** Execute a single-byte program `[0x00]` (just `STOP`) with
a generous gas limit. Assert:
- Execution succeeds (no error).
- The output bytes are empty.
- Zero gas was consumed.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_2_9
```

**Hint.** `STOP` does not pop anything, does not push anything, and costs 0 gas.
Its only effect is to halt execution normally. Check the `output` field of the
result and verify it is an empty `Vec<u8>` or slice.

> **Rust pattern — unit-like result variants.**
> `STOP` is the simplest possible opcode: it exists only to terminate. In the
> interpreter's `match`, the `STOP` arm does nothing except set a "halted"
> flag or break out of the loop. In the result type, `ExecutionStatus::Success`
> with empty output is the unit-like success case — analogous to `Ok(())` in
> the standard library. Having a dedicated variant for "halted normally" is
> cleaner than encoding it as a zero-length return.

---

## Exercise 2.10 — RETURN

**What to implement.** Assemble a program that:
1. Uses `MSTORE` to write a value into memory at offset 0.
2. Executes `PUSH1 0x20`, `PUSH1 0x00`, `RETURN` to return the 32-byte word.
3. Asserts the returned bytes match the stored value.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_2_10
```

**Hint.** `RETURN` takes two stack arguments: `offset` and `length`. It copies
`length` bytes from memory starting at `offset` into the output buffer and halts.
With offset `0` and length `32`, the output should be exactly the 32 bytes you
wrote with `MSTORE`.

> **Rust pattern — memory slicing.**
> The `RETURN` implementation does something like
> `output.extend_from_slice(&memory[offset..offset+length])`. The key invariant
> is that `expand_to(offset + length)` must be called before the slice, or the
> index will panic. In safe Rust all slice bounds are checked at runtime, so a
> missing expansion becomes a panic rather than a buffer overread — a much
> friendlier failure mode than in C.

---

```admonish success title="Milestone: Return42"
After completing Day 2, your interpreter can execute the `Return42` Solidity contract — a real compiled contract that stores 42 in memory and returns it. Run `cargo test --test solidity_integration return42` to verify.
```

## Yellow Paper Map

| Yellow Paper symbol | Meaning | Our representation |
|---|---|---|
| μ_g | Gas remaining | `u64` field in the execution context |
| μ_pc | Program counter | `usize` field, incremented by the fetch step |
| δ | Stack items consumed by opcode | Encoded in opcode documentation |
| α | Stack items produced by opcode | Encoded in opcode documentation |
| w | Current opcode word | `Opcode` enum variant |

**Section 9.4** of the Yellow Paper defines the state transition for each opcode
as a tuple `(μ′_g, μ′_pc, μ′_s, μ′_m)` — the new gas, program counter, stack,
and memory after execution. Our interpreter loop computes exactly this tuple on
each iteration. When you read an opcode's formal specification, map each primed
symbol (′) to the field you update in the Rust implementation.

The gas cost function `C` from §9.4.1 decomposes into a base cost (fixed per
opcode) plus a dynamic component for memory expansion. Day 2 covers the base
costs; the memory expansion formula appears in Day 3.

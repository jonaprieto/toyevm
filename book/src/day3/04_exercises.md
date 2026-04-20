# Exercises — Day 3

```admonish example title="Exercise Setup" collapsible=true
    git checkout -b my-day-3 my-day-2

Create stub files with `todo_exercise!()` bodies:

- `src/storage.rs` — `Storage` struct with `load()`, `store()`, `sload_gas()`
- `src/context.rs` — `Address`, `ExecutionContext`, `ContextBuilder`

Add `pub mod storage; pub mod context;` to `src/lib.rs`.

Extend `src/interpreter.rs` with new opcode match arms (SLOAD, SSTORE, CALLDATALOAD, etc.) — use `todo_exercise!()` in each arm initially.

Peek at signatures: `git show day-3:src/storage.rs | head -100`

Compare when done: `git diff day-3 -- src/storage.rs src/context.rs src/interpreter.rs`
```

Day 3 covers persistent storage, memory-expansion gas, control flow, and the
execution environment passed into a call. These exercises are longer than Day 1
and Day 2: several require assembling multi-opcode bytecode sequences by hand.
Work through them in order — each builds on the previous one.

> **Running the test suite:**
> ```sh
> cargo test storage
> cargo test interpreter
> ```

---

## Exercise 3.1 — MLOAD / MSTORE Round-Trip via the Interpreter

**What to implement.** Write an integration test that executes:

```text
PUSH32 <some 32-byte value>
PUSH1  0x00      # offset = 0
MSTORE           # store word to memory[0..32]
PUSH1  0x00      # offset = 0
MLOAD            # load word from memory[0..32]
STOP
```

Assert that after execution the top of the stack equals the value you stored.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_3_1
```

**Hint.** `PUSH32` is opcode `0x7F` followed by 32 immediate bytes. Build your
test bytecode as a `Vec<u8>` starting with `[0x7F]`, then append the 32 bytes
of your test value, then continue with the remaining opcodes. Use
`U256::to_bytes()` to get the raw bytes of your test value.

> **Rust pattern — integration testing.**
> A unit test checks one function in isolation; an integration test exercises
> a full pipeline. This test is an integration test: it touches the bytecode
> decoder, the opcode dispatcher, memory expansion, and the stack — all in one
> run. In Rust, integration tests live in the `tests/` directory or in `#[cfg(test)]`
> blocks inside `lib.rs`, but here it is fine to keep them close to the code
> they test. The distinguishing feature is that the test does not mock anything
> — it runs the real interpreter on real bytecode.

---

## Exercise 3.2 — Memory Expansion Gas

**What to implement.** Run two executions that each do a single `MSTORE` but at
different offsets, and compare the gas consumed:
1. `MSTORE` at offset `0` — expands memory from 0 to 32 bytes.
2. `MSTORE` at offset `1024` — expands memory from 0 to 1056 bytes (34 words).

Assert that the second execution consumes more gas than the first, and that the
difference matches the memory expansion formula:
$C_{mem} = 3 \cdot \text{words} + \lfloor \text{words}^2 / 512 \rfloor$.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_3_2
```

**Hint.** Compute the expected cost by hand:
- 1 word: $3 \times 1 + \lfloor 1/512 \rfloor = 3$ gas.
- 33 words: $3 \times 33 + \lfloor 33^2/512 \rfloor = 99 + 2 = 101$ gas.
The memory expansion cost for a single `MSTORE` is `new_cost - old_cost`
(where `old_cost` is 0 for an empty memory). Add the base cost of `MSTORE`
(3 gas) and the `PUSH1`/`PUSH32` costs to get the total.

> **Rust pattern — property-based reasoning.**
> Instead of just checking one magic number, this exercise asks you to verify a
> *relationship* between inputs and outputs: larger offset → higher gas. This is
> the key idea behind property-based testing (libraries like `proptest` or
> `quickcheck`): rather than enumerating specific cases, you describe invariants
> that must hold for any input. Even when writing ordinary unit tests, thinking
> in terms of properties ("more memory always costs more gas") produces more
> robust tests than thinking in terms of specific values.

---

## Exercise 3.3 — SSTORE / SLOAD Round-Trip

**What to implement.** Write a test that executes:

```text
PUSH32 <value>
PUSH32 <key>
SSTORE         # storage[key] = value
PUSH32 <key>
SLOAD          # push storage[key]
STOP
```

Assert the top of the stack equals `<value>`.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_3_3
```

**Hint.** Use a simple key like `U256::from(1u64)` and a recognisable value like
`U256::from(0xCAFE_BABEu64)`. `SSTORE` pops the *key* first (top of stack),
then the *value* (second item), so push the value before the key.

> **Rust pattern — `HashMap` as a storage model.**
> `Storage` wraps a `HashMap<U256, U256>`. This is a faithful model of the EVM
> specification, which defines account storage as a mapping from 256-bit keys to
> 256-bit values ($\sigma[a]_s$ in Yellow Paper notation). `HashMap` gives O(1)
> average-case reads and writes, just like a trie lookup in a real client.
> The difference is that our `HashMap` lives only for the duration of a test;
> a production EVM persists it to a Merkle Patricia Trie on disk.

---

## Exercise 3.4 — Cold vs Warm SLOAD Cost

**What to implement.** Execute two consecutive `SLOAD`s of the *same* storage
slot within one execution context:
- The first access is "cold" and should cost 2100 gas (EIP-2929).
- The second access is "warm" and should cost 100 gas.

Assert the gas consumed after each `SLOAD` using `gas_limit - remaining_gas`.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_3_4
```

**The gas function in storage:**

```rust
{{#include ../../../src/storage.rs:sload_gas}}
```

**Hint.** Structure your bytecode as:
`PUSH32 <key>`, `SLOAD`, `PUSH32 <key>`, `SLOAD`, `STOP`.
Snapshot `gas_remaining` after each `SLOAD` to compute the per-instruction cost.
You may need to extend the `ExecutionResult` type to expose intermediate gas
readings, or use a sufficiently large gap between `gas_limit` checkpoints.

> **Rust pattern — `HashSet` for tracking state.**
> The `Storage` struct holds a `HashSet<U256>` called `warm_slots`. On the first
> `SLOAD` for a given key, `contains` returns `false` (cold access, 2100 gas)
> and `insert` adds the key. On the second access, `contains` returns `true`
> (warm access, 100 gas). `HashSet` gives O(1) contains/insert and models the
> "accessed addresses" set from EIP-2929 precisely. The set is reset between
> transactions — in our tests, creating a new `Storage` simulates a new
> transaction.

---

## Exercise 3.5 — JUMPDEST Validation

**What to implement.** Write two tests:
1. A program with a valid `JUMPDEST` at offset `3`:
   `PUSH1 0x03`, `JUMP`, `JUMPDEST`, `STOP` — should succeed.
2. A program that tries to jump to offset `1` (the *immediate* byte of `PUSH1`,
   not an actual `JUMPDEST`) — should return an error (`InvalidJumpDest` or
   similar).

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_3_5
```

**Hint.** The interpreter pre-scans bytecode to build a `Vec<bool>` (or
`HashSet<usize>`) of valid jump destinations before execution starts. Only
bytes with opcode `0x5B` (`JUMPDEST`) that are *not* inside a `PUSH` immediate
are valid targets. Jumping into the middle of a `PUSH32` immediate must fail
even though the byte value there might happen to be `0x5B`.

> **Rust pattern — precomputation with `Vec<bool>`.**
> Scanning once at load time and building a jump-validity table is O(n) in
> bytecode size, while checking validity at runtime during every `JUMP` is also
> O(1) per check. The alternative — re-scanning from the start on each `JUMP` —
> would be O(n) per jump and would make loops quadratic. A `Vec<bool>` indexed
> by program counter is the standard approach; `bitvec` is used in production
> clients for tighter memory usage.

---

## Exercise 3.6 — JUMP (Unconditional)

**What to implement.** Assemble a program that uses `JUMP` to skip over a
`PUSH1 0xFF` instruction:

```text
offset 0:  PUSH1 0x05    # destination
offset 2:  JUMP          # jump to offset 5
offset 3:  PUSH1 0xFF    # skipped — never executes
offset 5:  JUMPDEST      # landing pad
offset 6:  STOP
```

Assert execution succeeds and that `U256::from(0xFFu64)` is **not** on the
stack (proving the skipped instruction never ran).

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_3_6
```

**Hint.** Build the bytecode as `[0x60, 0x05, 0x56, 0x60, 0xFF, 0x5B, 0x00]`.
After execution the stack should be empty (JUMP consumed the destination value,
the skipped PUSH never ran, and STOP produces nothing).

> **Rust pattern — program counter manipulation.**
> After a `JUMP`, the interpreter sets `pc = destination` instead of
> `pc += 1` (the normal increment). In a safe Rust implementation this is just
> `self.pc = destination as usize;` — no unsafe needed, because `pc` is a plain
> `usize` field. The key invariant is that the jump destination is validated
> *before* `pc` is updated, so an invalid jump never leaves `pc` pointing into
> the middle of an immediate.

---

## Exercise 3.7 — JUMPI True Branch

**What to implement.** Assemble a program where `JUMPI` takes the branch because
the condition is non-zero:

```text
PUSH1 0x01     # condition = 1 (non-zero → take branch)
PUSH1 0x07     # destination
JUMPI
PUSH1 0xFF     # skipped
JUMPDEST       # offset must be correct — count bytes carefully
STOP
```

Assert execution succeeds and `0xFF` is not on the stack.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_3_7
```

**Hint.** Count offsets carefully: `PUSH1 0x01` occupies offsets 0–1, `PUSH1 destination`
occupies offsets 2–3, `JUMPI` is at offset 4, `PUSH1 0xFF` is at offsets 5–6,
and `JUMPDEST` is at offset 7. So the destination should be `7`.

> **Rust pattern — conditional control flow.**
> `JUMPI` is the EVM's only conditional branch. In the interpreter, it is
> implemented as: `if condition != U256::ZERO { self.pc = dest } else { self.pc += 1 }`.
> This maps directly to an `if` expression in Rust with no mutable temporaries.
> The pattern of "check a condition, update a single field, continue the loop"
> is the fundamental building block of all higher-level control flow in EVM
> bytecode (loops, if/else, function dispatch).

---

## Exercise 3.8 — JUMPI False Branch (Fall-Through)

**What to implement.** Repeat the previous exercise but with a zero condition:

```text
PUSH1 0x00     # condition = 0 (zero → fall through)
PUSH1 0x07     # destination (not taken)
JUMPI
PUSH1 0x42     # this DOES execute now
JUMPDEST
STOP
```

Assert execution succeeds and `U256::from(0x42u64)` is on the top of the stack.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_3_8
```

**Hint.** The bytecode is identical to Exercise 3.7 except the condition byte
changes from `0x01` to `0x00`. Reuse your offset calculation from Exercise 3.7.
When `JUMPI` falls through, `pc` advances normally to the next instruction.

> **Rust pattern — testing both branches.**
> A `JUMPI` has two observable behaviours: take the branch, or fall through. A
> test suite that only covers one branch gives false confidence. The Rust
> community has a strong culture of pairing positive tests (the happy path) with
> negative tests (the error or alternative path). For every conditional in your
> interpreter, ask: "Do I have a test where the condition is true *and* a test
> where it is false?" This discipline catches off-by-one errors in condition
> evaluation that a single test would miss.

---

## Exercise 3.9 — CALLDATALOAD

**What to implement.** Construct an execution context with non-empty calldata —
for example, the 32-byte value `U256::from(0xBEEFu64)` encoded as big-endian
bytes. Execute:

```text
PUSH1 0x00     # byte offset into calldata
CALLDATALOAD   # push calldata[0..32] as U256
STOP
```

Assert the top of the stack equals `U256::from(0xBEEFu64)`.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_3_9
```

**Hint.** The execution environment struct (often called `CallContext`,
`Environment`, or similar) has a `calldata: Vec<u8>` field. Populate it with
`U256::from(0xBEEFu64).to_bytes().to_vec()` before running the interpreter.
If calldata is shorter than 32 bytes past the offset, the remaining bytes are
zero-padded.

> **Rust pattern — builder pattern for context.**
> Constructing an execution context with many optional fields benefits from the
> builder pattern: a `CallContextBuilder` struct with setter methods, each
> returning `Self`, terminated by a `.build()` call. This avoids positional
> argument confusion (`ExecutionContext::new(bytecode, calldata, caller, value, …)`)
> where you might swap two arguments of the same type. Even if the codebase does
> not use a formal builder today, recognising the pattern prepares you to
> refactor toward it as the struct grows.

---

## Exercise 3.10 — Loop in Bytecode (Capstone)

**What to implement.** Assemble a bytecode program that computes the sum
`1 + 2 + 3 + 4 + 5 = 15` using a loop. The program should:
1. Initialise a counter `i = 1` and an accumulator `sum = 0` in memory or on
   the stack.
2. Loop: add `i` to `sum`, increment `i`, break when `i > 5`.
3. Leave `sum` (= 15) on the top of the stack at `STOP`.

This is the hardest exercise in the book so far. Take it one step at a time:
write out the intended bytecode in a comment, then encode the hex bytes.

**File:** [`src/interpreter.rs`](https://github.com/jonaprieto/toyevm/blob/main/src/interpreter.rs)

**How to verify:**

```sh
cargo test ex_3_10
```

**Hint.** A minimal loop skeleton in EVM bytecode:
```
; sum = 0 (already on stack from PUSH1 0x00)
; i = 1
PUSH1 0x01     ; i
PUSH1 0x00     ; sum (under i — push sum first so it stays deep)
<loop_top: JUMPDEST>
; stack: [sum, i]  (i on top)
DUP1           ; [sum, i, i]
PUSH1 0x06     ; limit + 1
GT             ; i > 5?  → [sum, i, flag]
PUSH1 <exit>
JUMPI          ; if flag != 0, jump to exit
; still looping: sum += i
SWAP1          ; [i, sum]
DUP2           ; [i, sum, i]
ADD            ; [i, sum+i]
SWAP1          ; [sum+i, i]
; increment i
PUSH1 0x01
ADD            ; [sum+i, i+1]
PUSH1 <loop_top>
JUMP
<exit: JUMPDEST>
; stack: [sum_final, i_final] — pop i
POP
STOP
```
Encode this by hand, counting offsets carefully each time you add or remove an
instruction.

> **Rust pattern — bytecode as data (assembling by hand).**
> Writing bytecode as a `Vec<u8>` literal (`vec![0x60, 0x01, …]`) makes you
> the assembler. This is uncomfortable at first but teaches you two things:
> (1) exactly what the interpreter sees — no magic, no abstraction; and
> (2) why assemblers and compilers exist. Production EVM tooling (solc, Vyper,
> Huff) automates offset calculation and label resolution. Once you have done it
> by hand once, you will never again take label resolution for granted.

---

```admonish success title="Milestone: Real Solidity Contracts"
After completing Day 3, your interpreter can execute four real Solidity contracts: `Store42` (storage), `Loop` (control flow), and `Add` (ABI dispatch with function selectors, calldata decoding, and checked arithmetic). Run `cargo test --test solidity_integration` to verify all of them.
```

## Yellow Paper Map

| Yellow Paper symbol | Meaning | Our representation |
|---|---|---|
| $I$ | Execution environment (a tuple) | Execution context struct |
| $I_a$ | Address of the account being executed | `address: Address` field |
| $I_s$ | Sender (caller) address | `caller: Address` field |
| $I_o$ | Origin of the transaction | `origin: Address` field |
| $I_d$ | Input data (calldata) | `calldata: Vec<u8>` field |
| $I_v$ | Value transferred with the call | `value: U256` field |
| $I_b$ | Bytecode being executed | `bytecode: Vec<u8>` field |
| $I_H$ | Current block header | `block: BlockContext` struct |
| $I_e$ | Call depth | `depth: u32` field |

**Section 4.1** of the Yellow Paper defines the execution environment $I$ as a
nine-tuple. Each element has a direct counterpart in our context struct. Day 3
fills in $I_d$ (calldata) and connects $\sigma[I_a]_s$ (account storage) to our
`Storage` type. Future days will add $I_v$ (value) and $I_H$ (block information)
for the full `CALL` and environment opcodes.

The formula for memory expansion gas referenced in Exercise 3.2 is:

$$C_{mem}(a) = G_{memory} \cdot a + \left\lfloor \frac{a^2}{512} \right\rfloor$$

where $a$ is the number of 32-byte memory words active and $G_{memory} = 3$.
This quadratic term makes very large memory expansions prohibitively expensive,
bounding the memory a contract can consume within a single transaction.

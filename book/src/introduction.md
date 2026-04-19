# toyevm: Build an EVM from Scratch in Rust

Welcome to **toyevm**, a 7-day, exercise-driven tutorial that takes you from zero EVM knowledge to a deep understanding of EVM internals, gas mechanics, cross-contract calls, and optimization strategies.

## Who is this for?

You should be comfortable with basic Rust — ownership, traits, enums, pattern matching, `Result`/`Option`. If you've completed [100 Exercises to Learn Rust](https://rust-exercises.com/) or equivalent, you're ready.

No prior Ethereum or blockchain knowledge is required.

## Getting started

### 1. Clone the repository

```bash
git clone https://github.com/jonaprieto/toyevm.git
cd toyevm
```

### 2. Start Day 1 — the exercise branch

The `day-1-start` branch has the project structure with **stub files** — all type definitions and method signatures are in place, but every function body is `todo_exercise!()`. Your job is to implement them.

```bash
git checkout day-1-start
git checkout -b my-day-1       # your personal working branch
```

### 3. Look at the project structure

```
toyevm/
├── Cargo.toml          ← Rust project manifest (lib + binary)
├── src/
│   ├── lib.rs          ← crate root — declares modules + todo_exercise! macro
│   ├── main.rs         ← binary entry point
│   ├── types/
│   │   ├── mod.rs      ← re-exports U256
│   │   └── u256.rs     ← ★ Exercise 1.1–1.4: implement U256 type
│   ├── stack.rs        ← ★ Exercise 1.5–1.6: implement Stack
│   └── memory.rs       ← ★ Exercise 1.7–1.8: implement Memory
└── book/               ← this book (mdBook source)
```

The files marked with ★ contain stubs like this:

```rust
pub fn wrapping_add(self, _rhs: Self) -> Self {
    todo_exercise!("Exercise 1.2 — add byte-by-byte from LSB to MSB, propagating carry")
}
```

Each stub has the correct signature — your job is to replace `todo_exercise!(...)` with a real implementation.

### 4. Run the tests

```bash
cargo test
```

You'll see 33 failing tests, each with a hint:

```
---- types::u256::tests::ex_1_2_simple_add ----
thread panicked at 'Exercise not yet implemented.
  Hint: Exercise 1.2 — add byte-by-byte from LSB to MSB, propagating carry'

test result: FAILED. 4 passed; 33 failed; 0 ignored
```

Implement until all 37 tests pass.

### 5. Compare your solution to the reference

Once your tests pass, see how your implementation compares to ours:

```bash
git diff day-1 -- src/
```

This shows the exact differences between your code and the reference solution. You might find a more elegant approach — or discover edge cases you hadn't considered.

```admonish tip title="Reading the reference without diffing"
If you want to peek at the full reference solution at any point:
`git show day-1:src/types/u256.rs` (shows the file without switching branches)
```

### 6. Advance to the next day

```bash
git checkout -b my-day-2 my-day-1    # branch from your completed Day 1
```

For Days 2–7, the exercise chapter in this book provides stub code to paste into new files. The pattern is always:

1. Create the stub files shown in the "Exercise Setup" section
2. `cargo test` — new tests fail
3. Implement until they pass
4. `git diff day-N -- src/` — compare to reference

```admonish tip title="How the branches work"
Solution branches `day-1` through `day-7` are *stacked snapshots* — each contains everything from prior days plus new code. Your personal `my-day-N` branches grow incrementally as you add modules. Use `git diff day-N` to compare any day.
```

## Comparing your work — quick reference

| Day | After completing | Compare with |
|-----|-----------------|--------------|
| 1 | all 37 tests pass | `git diff day-1 -- src/types src/stack.rs src/memory.rs` |
| 2 | all tests pass | `git diff day-2 -- src/opcode.rs src/gas.rs src/interpreter.rs` |
| 3 | all tests pass | `git diff day-3 -- src/storage.rs src/context.rs src/interpreter.rs` |
| 4 | all tests pass | `git diff day-4 -- src/contract.rs src/host.rs` |
| 5 | all tests pass | `git diff day-5 -- src/call.rs` |
| 6 | all tests pass | `git diff day-6 -- src/compiler/` |
| 7 | all tests pass | `git diff day-7 -- src/compiler/fold.rs src/compiler/dce.rs` |

## How each day works

Each day has three parts in this book:

1. **Theory** — what the EVM concept is and why it matters, with Rust code snippets pulled directly from the source files
2. **Exercises** — a list of exercises with the file to edit, the `cargo test` command to verify, and a hint
3. **Deep Dive** — Q&A that tests your understanding of the concepts

The recommended workflow:

```
Read the theory chapter
    ↓
Look at the exercise list
    ↓
Open the source file, read the tests
    ↓
Implement (or study the reference implementation)
    ↓
cargo test — verify
    ↓
Read the Deep Dive questions, try to answer before reading
```

## What you'll build

| Day | Branch | Theme | Key outcome |
|-----|--------|-------|-------------|
| 1 | `day-1` | The Machine Exists | U256, Stack, Memory data structures |
| 2 | `day-2` | The Interpreter Loop | Execution loop with gas metering |
| 3 | `day-3` | State and Control Flow | Storage, JUMP/JUMPI, EIP-2929 |
| 4 | `day-4` | Contracts and Deployment | CREATE/CREATE2, Host trait, LOG |
| 5 | `day-5` | The Call Stack | CALL/DELEGATECALL/STATICCALL |
| 6 | `day-6` | From Bytecode to IR | Control flow graph, liveness analysis |
| 7 | `day-7` | Optimization Passes | Constant folding, dead code elimination |

By the end, you'll have a working EVM interpreter with a compiler backend — built entirely by you.

## Running Solidity examples (optional)

The repository includes a [Foundry](https://book.getfoundry.sh/) project in `examples/` with simple Solidity contracts. If you have Foundry installed, you can compile them and feed the bytecode into toyevm:

```bash
cd examples
forge build
forge inspect Return42 deployedBytecode    # get hex bytecode
forge inspect Return42 assembly            # annotated EVM assembly
```

This is covered in later days as the interpreter gains enough opcodes to execute real contracts.

## Tools you'll need

| Tool | Required? | Install |
|------|----------|---------|
| Rust (nightly) | Yes | [rustup.rs](https://rustup.rs/) |
| mdBook | For the book | `cargo install mdbook` |
| Foundry | Optional | [getfoundry.sh](https://book.getfoundry.sh/getting-started/installation) |

## References

- [evm.codes](https://www.evm.codes/) — interactive opcode reference
- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf) — formal specification
- [EELS (Python reference implementation)](https://github.com/ethereum/execution-specs)
- [revm](https://github.com/bluealloy/revm) — production Rust EVM
- [LambdaClass evm_mlir](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) — AOT compilation approach

# Getting Started

## Tools

| Tool | Required? | Install |
|------|----------|---------|
| Rust (nightly) | Yes | [rustup.rs](https://rustup.rs/) |
| Foundry | Optional | [getfoundry.sh](https://getfoundry.sh/introduction/installation) |

## Clone and set up

```bash
git clone https://github.com/jonaprieto/toyevm.git
cd toyevm
```

## Start Module 1

The `day-1-start` branch has the project structure with **stub files** — all type definitions and method signatures are in place, but every function body is `todo_exercise!()`. The work is to fill them in.

```bash
git checkout day-1-start
git checkout -b my-day-1       # your personal working branch
```

## Project structure

```
toyevm/
├── Cargo.lock
├── Cargo.toml          ← Rust project manifest (lib + binary)
├── src/
│   ├── lib.rs          ← crate root — declares modules + todo_exercise! macro
│   ├── main.rs         ← binary entry point
│   ├── memory.rs       ← ★ Exercise 1.7–1.8: implement Memory
│   ├── stack.rs        ← ★ Exercise 1.5–1.6: implement Stack
│   └── types/
│       ├── mod.rs      ← re-exports U256
│       └── u256.rs     ← ★ Exercise 1.1–1.4: implement U256 type
└── book/
    ├── book.toml
    └── src/
        ├── SUMMARY.md
        └── introduction.md
```

The files marked with ★ contain stubs like this:

```rust
pub fn wrapping_add(self, _rhs: Self) -> Self {
    todo_exercise!("Exercise 1.2 — add byte-by-byte from LSB to MSB, propagating carry")
}
```

Each stub has the correct signature — the work is replacing `todo_exercise!(...)` with a real implementation.

## Run the tests

```bash
cargo test
```

Failing tests show hints:

```
---- types::u256::tests::ex_1_2_simple_add ----
thread panicked at 'Exercise not yet implemented.
  Hint: Exercise 1.2 — add byte-by-byte from LSB to MSB, propagating carry'

test result: FAILED. 4 passed; 33 failed; 0 ignored
```

Implement until all tests pass.

## Compare with the reference

Once tests pass, see how your implementation compares to mine:

```bash
git diff day-1 -- src/
```

This shows the exact differences between your code and the reference solution. You might find a more elegant approach — or discover edge cases you had not considered.

```admonish tip title="Reading the reference without diffing"
To peek at the full reference solution at any point:
`git show day-1:src/types/u256.rs` (shows the file without switching branches)
```

## Advance to the next module

```bash
git checkout -b my-day-2 my-day-1    # branch from your completed Module 1
```

For Modules 2–7, the exercise chapter provides stub code to paste into new files. The pattern is always:

1. Create the stub files shown in the "Exercise Setup" section
2. `cargo test` — new tests fail
3. Implement until they pass
4. `git diff day-N -- src/` — compare to reference

```admonish tip title="How the branches work"
Solution branches `day-1` through `day-7` are *stacked snapshots* — each contains everything from prior modules plus new code. Your personal `my-day-N` branches grow incrementally as you add modules. Use `git diff day-N` to compare any module.
```

## Comparing your work — quick reference

| Module | After completing | Compare with |
|--------|-----------------|--------------|
| 1 | all 37 tests pass | `git diff day-1 -- src/types src/stack.rs src/memory.rs` |
| 2 | all tests pass | `git diff day-2 -- src/opcode.rs src/gas.rs src/interpreter.rs` |
| 3 | all tests pass | `git diff day-3 -- src/storage.rs src/context.rs src/interpreter.rs` |
| 4 | all tests pass | `git diff day-4 -- src/contract.rs src/host.rs` |
| 5 | all tests pass | `git diff day-5 -- src/call.rs` |
| 6 | all tests pass | `git diff day-6 -- src/compiler/` |
| 7 | all tests pass | `git diff day-7 -- src/compiler/fold.rs src/compiler/dce.rs` |

## How each module works

Each module has three parts:

1. **Notes** — what I learned about the concept, with Rust code pulled directly from the source files
2. **Exercises** — how I tested my understanding, with `cargo test` commands and hints
3. **Deep Dive** — review questions to test and deepen understanding

The workflow I followed:

```
Read the notes
    ↓
Look at the exercises
    ↓
Open the source file, read the tests
    ↓
Implement (or study the reference)
    ↓
cargo test — verify
    ↓
Work through the Deep Dive questions
```

## Running Solidity examples

The repository includes a [Foundry](https://getfoundry.sh/) project in `examples/` with six Solidity contracts. As you progress through the modules, the interpreter gains enough opcodes to execute them — each is a milestone that proves the EVM actually works.

| Contract | What it does | Runnable after |
|----------|-------------|----------------|
| `Return42` | Returns 42 from a `fallback()` in assembly | **Module 2** |
| `Store42` | Writes 42 to storage, reads it back, returns it | **Module 3** |
| `Loop` | Sums 1..10 using a `for` loop in assembly | **Module 3** |
| `Add` | `add(uint256,uint256)` — ABI dispatch + checked arithmetic | **Module 3** |
| `Counter` | `increment()` / `get()` — storage + function dispatch | **Module 3** |
| `EventEmitter` | `set(uint256)` — emits a `LOG2` event | **Module 4** |

Integration tests in `tests/solidity_integration.rs` verify these milestones automatically:

```bash
cargo test --test solidity_integration
```

If you have Foundry installed, you can also inspect the bytecode directly:

```bash
cd examples
forge build
forge inspect Return42 deployedBytecode    # raw hex bytecode
forge inspect Add abi                       # function signatures
```

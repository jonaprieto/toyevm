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

### 2. Check out the first day

Each day lives on its own branch. Start with Day 1:

```bash
git checkout day-1
```

### 3. Look at the project structure

After checking out `day-1`, you'll see:

```
toyevm/
├── Cargo.toml          ← Rust project manifest (lib + binary)
├── src/
│   ├── lib.rs          ← crate root — declares modules
│   ├── main.rs         ← binary entry point (just prints a message)
│   ├── types/
│   │   ├── mod.rs      ← re-exports U256
│   │   └── u256.rs     ← ★ Exercise 1.1–1.4: implement U256 type
│   ├── stack.rs        ← ★ Exercise 1.5–1.6: implement Stack
│   └── memory.rs       ← ★ Exercise 1.7–1.8: implement Memory
└── book/               ← this book (mdBook source)
```

The files marked with ★ are where you write code. Each contains `#[cfg(test)]` blocks with tests — your goal is to make them pass.

### 4. Run the tests

```bash
cargo test
```

You'll see output like:

```
running 37 tests
test types::u256::tests::ex_1_1_zero_is_all_zeros ... ok
test types::u256::tests::ex_1_2_simple_add ... ok
test stack::tests::ex_1_5_push_and_pop_lifo ... ok
...
test result: ok. 37 passed; 0 failed; 0 ignored
```

On `day-1`, all tests already pass — the implementations are provided as reference. To practice, you can:

- **Read the code** to understand how each type is built
- **Delete a function body** and rewrite it yourself, then run `cargo test` to verify
- **Add your own tests** to explore edge cases

### 5. Advance to the next day

```bash
git checkout day-2
cargo test          # new tests appear for the interpreter
```

Each branch builds on the previous one. New modules and tests are added, and existing code grows.

```admonish tip title="How the branches work"
The branches are *stacked*: `day-2` contains everything from `day-1` plus new code. You don't merge — you just check out the next branch. Think of each branch as a snapshot of the project at the end of that day.
```

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

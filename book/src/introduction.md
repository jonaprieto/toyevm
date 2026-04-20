# ToyEVM

I wanted to understand the Ethereum Virtual Machine — not at the Solidity level, but at the opcode level. How does a `JUMP` actually work? Why does storage cost so much gas? What does a control flow graph look like for a `while` loop compiled to bytecode?

The only way I could answer these questions was to build one myself. These are my notes from that process: an EVM interpreter built from scratch in Rust, opcode by opcode, with zero external dependencies.

The material is organized into seven days. You are welcome to follow along, work through the exercises, and build your own.

## Why I chose to build an EVM

Reading the Yellow Paper is one thing. Implementing it is another. Building the interpreter forced me to confront every detail I would have glossed over: how carries propagate in 256-bit addition, why JUMPDEST validation is a security property, what the 63/64 gas rule actually prevents, and how a basic block analysis reveals structure hidden in flat bytecode.

Along the way I also learned Rust patterns I now use everywhere — newtypes for type safety, `repr(u8)` enums for jump table dispatch, trait objects for pluggable backends, and why `Result<T, E>` is the right way to model EVM errors.

## What we build

| Day | Theme | What we implement | Solidity milestone |
|-----|-------|--------------------|-------------------|
| 1 | **The Machine Exists** | 256-bit integers, a bounded stack, byte-addressable memory | — |
| 2 | **The Interpreter Loop** | Opcode decoding, the fetch-decode-execute loop, gas metering | `Return42` runs |
| 3 | **State and Control Flow** | Persistent storage (EIP-2929), JUMP/JUMPI, loops in bytecode | `Store42`, `Loop`, `Add` run |
| 4 | **Contracts and Deployment** | CREATE/CREATE2 address derivation, the Host trait, LOG events | `EventEmitter` runs |
| 5 | **The Call Stack** | CALL/DELEGATECALL/STATICCALL, the 63/64 gas rule, call frames | — |
| 6 | **From Bytecode to IR** | Basic blocks, control flow graphs, stack-height analysis, liveness | — |
| 7 | **Optimization Passes** | Constant folding, dead code elimination, interpreter performance | — |

By the end, we have a working EVM interpreter that can execute real Solidity-compiled contracts, plus a compiler backend for bytecode analysis and optimization.

## How each day works

Each day has three parts:

1. **Notes** — what I learned about the concept, with code snippets pulled from the source files
2. **Exercises** — how I tested my understanding, with `cargo test` commands and hints
3. **Deep Dive** — questions I asked myself and worked through after implementing

The exercises use a `todo_exercise!()` macro — every function signature is already in place, every test is already written. The work is to fill in the implementations until `cargo test` goes green.

## What you need

Comfortable with basic Rust — ownership, traits, enums, pattern matching, `Result`/`Option`. If you have completed [100 Exercises to Learn Rust](https://rust-exercises.com/) or equivalent, you are ready. No prior Ethereum, blockchain, or compiler knowledge is required.

## References

These are the sources I used throughout. You do not need to read them before starting — I introduce concepts as they come up — but they are valuable companions:

- [evm.codes](https://www.evm.codes/) — interactive opcode reference with gas costs and stack diagrams
- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf) ([source](https://github.com/ethereum/yellowpaper)) — the formal EVM specification
- [EELS (Python reference implementation)](https://github.com/ethereum/execution-specs) — executable specification of the EVM
- [revm](https://github.com/bluealloy/revm) — production Rust EVM used by Reth and Foundry
- [LambdaClass evm_mlir](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) — AOT compilation approach with benchmarks

Ready? Head to the [Getting Started](./getting_started.md) page to set up the environment and begin Day 1.

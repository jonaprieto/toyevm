# ToyEVM: Build an EVM from Scratch in Rust

The best way to understand a virtual machine is to build one. This tutorial takes you from zero EVM knowledge to a working interpreter with a compiler backend — opcode by opcode, written entirely in Rust. No frameworks, no dependencies, no magic — just you, the Yellow Paper, and `cargo test`.

The material is organized into seven days, each a few hours of focused work. You can spread it across a week or complete it in a single sitting — the pace is yours.

## Why build an EVM?

The EVM is the execution engine at the heart of Ethereum. Every smart contract, every DeFi protocol, every NFT mint — all of it compiles down to EVM bytecode and runs on this machine. Understanding the EVM deeply gives you an edge whether you are:

- **A Rust developer** curious about blockchain internals — you will learn how a real production system (like [revm](https://github.com/bluealloy/revm)) works under the hood
- **A smart contract developer** who wants to understand why gas costs are what they are, how storage actually works, and what DELEGATECALL really does
- **A compiler engineer** interested in how bytecode analysis, CFG construction, and optimization passes apply to a stack machine
- **A security researcher** who needs to reason about EVM semantics — reentrancy, storage collisions, jump destination validation — at the opcode level

You don't need prior Ethereum or blockchain knowledge. If you can write basic Rust — ownership, traits, enums, pattern matching, `Result`/`Option` — you are ready.

## What makes this tutorial different?

This is not a reference manual. It is an **exercise-driven** tutorial inspired by [100 Exercises to Learn Rust](https://rust-exercises.com/). Every concept is followed by exercises where you implement the real thing: you write the U256 arithmetic, the stack, the interpreter loop, the gas metering, the storage model, the call stack, the CFG builder, the optimizer. All of it.

Each day follows the same rhythm:

```
Read the theory  →  Study the exercises  →  Implement until tests pass  →  Compare with the reference  →  Test yourself with the Deep Dive Q&A
```

The exercises use a `todo_exercise!()` macro — every function signature is already in place, every test is already written. Your job is to fill in the implementations until `cargo test` goes green.

## What you will build

| Day | Theme | What you implement | Solidity milestone |
|-----|-------|--------------------|-------------------|
| 1 | **The Machine Exists** | 256-bit integers, a bounded stack, byte-addressable memory | — |
| 2 | **The Interpreter Loop** | Opcode decoding, the fetch-decode-execute loop, gas metering | `Return42` runs |
| 3 | **State and Control Flow** | Persistent storage (EIP-2929), JUMP/JUMPI, loops in bytecode | `Store42`, `Loop`, `Add` run |
| 4 | **Contracts and Deployment** | CREATE/CREATE2 address derivation, the Host trait, LOG events | `EventEmitter` runs |
| 5 | **The Call Stack** | CALL/DELEGATECALL/STATICCALL, the 63/64 gas rule, call frames | — |
| 6 | **From Bytecode to IR** | Basic blocks, control flow graphs, stack-height analysis, liveness | — |
| 7 | **Optimization Passes** | Constant folding, dead code elimination, interpreter performance | — |

By the end, you will have a working EVM interpreter that can execute real Solidity-compiled contracts, plus a compiler backend for bytecode analysis and optimization — built entirely by you, with zero external dependencies.

## Who is this for?

You should be comfortable with basic Rust. If you have completed [100 Exercises to Learn Rust](https://rust-exercises.com/) or equivalent, you are ready. Specifically, you should know:

- Ownership and borrowing
- Enums, pattern matching, `match`
- Traits and trait implementations
- `Result<T, E>` and `Option<T>`
- Basic collections (`Vec`, `HashMap`)

No prior Ethereum, blockchain, or compiler knowledge is required. The tutorial introduces everything you need, from the Yellow Paper notation to basic block theory.

## Why Rust?

Rust is the language of choice for production EVM implementations. [revm](https://github.com/bluealloy/revm) (used by Reth, Foundry, and others) is written in Rust. The EVM's requirements — deterministic execution, precise gas accounting, no undefined behavior — align perfectly with Rust's guarantees.

Building an EVM in Rust also teaches you patterns you will encounter in real blockchain infrastructure: newtypes for type safety, `repr(u8)` enums for jump table dispatch, trait objects for pluggable backends, and zero-dependency design for auditability.

## References

These are the primary sources this tutorial builds on. You don't need to read them before starting — the tutorial introduces concepts as needed — but they are valuable companions:

- [evm.codes](https://www.evm.codes/) — interactive opcode reference with gas costs and stack diagrams
- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf) ([source](https://github.com/ethereum/yellowpaper)) — the formal EVM specification
- [EELS (Python reference implementation)](https://github.com/ethereum/execution-specs) — executable specification of the EVM
- [revm](https://github.com/bluealloy/revm) — production Rust EVM used by Reth and Foundry
- [LambdaClass evm_mlir](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) — AOT compilation approach with benchmarks

Ready? Head to the [Getting Started](./getting_started.md) page to set up your environment and begin Day 1.

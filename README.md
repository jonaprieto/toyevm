# toyevm

[![CI](https://github.com/jonaprieto/toyevm/actions/workflows/ci.yml/badge.svg)](https://github.com/jonaprieto/toyevm/actions/workflows/ci.yml)
[![Book](https://github.com/jonaprieto/toyevm/actions/workflows/pages.yml/badge.svg)](https://jonaprieto.github.io/toyevm/)

A 7-day, exercise-driven tutorial for building an Ethereum Virtual Machine from scratch in Rust.

**[Read the book online](https://jonaprieto.github.io/toyevm/)**

## What is this?

**toyevm** takes you from zero EVM knowledge to being able to discuss EVM internals, gas mechanics, cross-contract calls, and optimization strategies at an interview level — all while writing Rust.

Inspired by [100 Exercises to Learn Rust](https://rust-exercises.com/), each day introduces EVM concepts alongside the Rust code that implements them. You read the theory, then make failing tests pass.

## Prerequisites

- Comfortable with basic Rust (ownership, traits, enums, `Result`/`Option`)
- No prior Ethereum or blockchain knowledge required

## Structure

| Day | Branch | Theme | Tests |
|-----|--------|-------|-------|
| 1 | `day-1` | The Machine Exists — U256, Stack, Memory | 37 |
| 2 | `day-2` | The Interpreter Loop — opcodes, gas metering | 63 |
| 3 | `day-3` | State and Control Flow — storage, JUMP/JUMPI, EIP-2929 | 81 |
| 4 | `day-4` | Contracts and Deployment — CREATE/CREATE2, Host trait, LOG | 92 |
| 5 | `day-5` | The Call Stack — CALL/DELEGATECALL/STATICCALL, 63/64 rule | 98 |
| 6 | `day-6` | From Bytecode to IR — CFG construction, liveness analysis | 112 |
| 7 | `day-7` | Optimization Passes — constant folding, dead code elimination | 121 |

Each day includes theory chapters (mdBook), exercises with `#[test]`, and an interview corner.

## Getting started

```bash
git clone https://github.com/jonaprieto/toyevm.git
cd toyevm
git checkout day-1
cargo test          # see what's pending
# read the book, implement the code, watch tests go green
```

To build the book locally:

```bash
cargo install mdbook
mdbook serve book/
```

## References

- [evm.codes](https://www.evm.codes/) — interactive opcode reference
- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf) — formal specification
- [revm](https://github.com/bluealloy/revm) — production Rust EVM
- [LambdaClass evm_mlir](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) — AOT compilation
- [LambdaClass zkSync VM](https://blog.lambdaclass.com/how-we-created-a-research-fast-vm-for-zksync/) — research VM

## License

MIT

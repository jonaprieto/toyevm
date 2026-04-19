# toyevm: Build an EVM from Scratch in Rust

Welcome to **toyevm**, a 7-day, exercise-driven tutorial that takes you from zero EVM knowledge to being able to discuss EVM internals, gas mechanics, cross-contract calls, and optimization strategies at an interview level.

## Who is this for?

You should be comfortable with basic Rust — ownership, traits, enums, pattern matching, `Result`/`Option`. If you've completed [100 Exercises to Learn Rust](https://rust-exercises.com/) or equivalent, you're ready.

No prior Ethereum or blockchain knowledge is required.

## How it works

Each day introduces EVM concepts alongside the Rust code that implements them. You'll find:

- **Theory sections** explaining what the EVM does and why
- **Exercises** with `#[ignore]` tests — your job is to implement the code that makes them pass
- **Yellow Paper Map** sidebars connecting our code to the formal specification
- **Interview Corner** sections with questions you might face in a compiler engineering interview

The workflow:

```bash
git checkout day-1
cargo test                    # see what's pending
# read the book chapter, implement the code
cargo test                    # watch tests go green
```

## What you'll build

| Day | Theme | Key outcome |
|-----|-------|-------------|
| 1 | The Machine Exists | U256, Stack, Memory data structures |
| 2 | The Interpreter Loop | Execution loop with gas metering |
| 3 | State and Control Flow | Storage, JUMP/JUMPI, context |
| 4 | Contracts and Deployment | CREATE/CREATE2, Host trait |
| 5 | The Call Stack | CALL/DELEGATECALL/STATICCALL |
| 6 | From Bytecode to IR | Control flow graph, liveness analysis |
| 7 | Optimization Passes | Constant folding, dead code elimination |

By the end, you'll have a working EVM interpreter with a compiler backend — built entirely by you.

## References

- [evm.codes](https://www.evm.codes/) — interactive opcode reference
- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf) — formal specification
- [EELS (Python reference implementation)](https://github.com/ethereum/execution-specs)
- [revm](https://github.com/bluealloy/revm) — production Rust EVM
- [LambdaClass evm_mlir](https://blog.lambdaclass.com/evm-performance-boosts-with-mlir/) — AOT compilation approach

# Gas: The EVM's Resource Model

Gas is the EVM's mechanism for bounding computation. Every opcode has a cost, and every transaction starts with a finite gas budget. What I found surprising is how abruptly execution halts when gas runs out — there's no graceful winding down.

## Why gas exists

Without gas, a contract could loop forever, and every Ethereum node would be stuck executing it. I think of gas as a pragmatic solution to the halting problem: you can run any computation you want, as long as you can pay for it.

## Static vs. dynamic costs

**Static costs** are fixed per opcode:

| Tier | Cost | Examples |
|------|------|----------|
| Zero | 0 | STOP, JUMPDEST |
| Base | 2 | PUSH0, PC, MSIZE, GAS, POP |
| Very Low | 3 | ADD, SUB, LT, GT, EQ, AND, OR, NOT, BYTE, SHL, SHR, SAR, PUSH1-32, DUP, SWAP |
| Low | 5 | MUL, DIV, MOD, SIGNEXTEND |
| Mid | 8 | ADDMOD, MULMOD, JUMP |
| High | 10 | JUMPI |

**Dynamic costs** depend on operand values:
- **Memory expansion**: grows quadratically — $C_{mem}(a) = 3a + \lfloor a^2 / 512 \rfloor$
- **EXP**: base cost + 50 per byte in the exponent
- **SLOAD/SSTORE**: cold vs. warm access (Module 3)

## The memory expansion formula

$$C_{mem}(a) = 3a + \left\lfloor \frac{a^2}{512} \right\rfloor$$

At small sizes, this is essentially linear (3 gas per 32-byte word). But the quadratic term kicks in at scale — I plotted a few values to feel it:

| Words | Bytes | Gas cost |
|-------|-------|----------|
| 1 | 32 | 3 |
| 10 | 320 | 30 |
| 100 | 3,200 | 319 |
| 1000 | 32,000 | 4,953 |
| 10000 | 320,000 | 225,312 |

Large memory allocations end up prohibitively expensive — and that's entirely by design.

## Out-of-gas vs. REVERT

Both halt execution and discard state changes, but I had to work through how they differ in gas refunds:

| | Out-of-gas | REVERT |
|---|-----------|--------|
| State changes | Discarded | Discarded |
| Remaining gas | **All consumed** | **Refunded to caller** |
| Return data | Empty | Can include error message |

This distinction matters for smart contract design: I now appreciate why a well-written contract should `REVERT` with an error message rather than running out of gas silently.

```admonish tip title="Rust Pattern: Result<T, E>"
We model execution outcomes as `Result<ExecutionResult, EvmError>`. A successful `REVERT` returns `Ok(ExecutionResult { reverted: true, .. })` — it's not an `Err`, because the interpreter completed successfully; it's the *contract* that chose to revert.
```

## Exercises

- **2.1** — Decode known opcodes and reject unknown bytes
- **2.2** — Verify PUSH immediate sizes
- **2.3** — Execute ADD and verify the result
- **2.4** — Verify exact gas accounting for a sequence of opcodes
- **2.5** — MUL, SUB, DIV correctness
- **2.6** — Division by zero returns 0
- **2.7** — LT, GT, EQ produce 0 or 1
- **2.8** — BYTE extracts individual bytes from a word
- **2.9** — SHL and SHR shift operations
- **2.10** — SAR preserves the sign bit
- **2.11** — Out-of-gas detection
- **2.12** — STOP returns empty output
- **2.13** — RETURN copies a memory range to output

Run: `cargo test opcode` and `cargo test interpreter`

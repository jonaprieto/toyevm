# Control Flow: JUMP, JUMPI, and JUMPDEST

The EVM has no structured control flow — no `if`, no `for`, no function calls at the bytecode level. I found this surprisingly elegant: everything compiles down to two jump instructions and a marker opcode.

### JUMP — unconditional jump

`JUMP` pops a destination from the stack and sets the program counter to that offset. The destination must be a valid `JUMPDEST` — I'll explain why that matters below.

```
Before:  Stack [ ..., 0x0A ]     ← destination on top
         PC = 5
JUMP: pop destination (0x0A)
After:   Stack [ ... ]
         PC = 0x0A               ← execution continues at offset 10
```

### JUMPI — conditional jump

`JUMPI` pops a destination and a condition. If the condition is nonzero, it jumps; otherwise, execution falls through to the next instruction. This is how all of Solidity's `if` statements reach us at the bytecode level.

```
Before:  Stack [ ..., 1, 0x0A ]  ← destination on top, condition below
         PC = 5
JUMPI: pop destination (0x0A), pop condition (1)
       condition ≠ 0 → jump
After:   Stack [ ... ]
         PC = 0x0A               ← branch taken
```

```
Before:  Stack [ ..., 0, 0x0A ]  ← destination on top, condition = 0
         PC = 5
JUMPI: pop destination (0x0A), pop condition (0)
       condition = 0 → fall through
After:   Stack [ ... ]
         PC = 8                  ← next instruction (5 + 3 bytes for JUMPI args)
```

### JUMPDEST — jump target marker

`JUMPDEST` is a no-op that costs 1 gas. Its only purpose is to mark a valid landing site for `JUMP` and `JUMPI`. I learned this the hard way: jumping to any byte that is not a `JUMPDEST` causes an `InvalidJump` error.

## JUMPDEST validation

This is a critical security property. We can only jump to a byte that:

1. Is the `JUMPDEST` opcode (0x5B), **and**
2. Is not part of a PUSH immediate sequence

Without rule 2, an attacker could craft a `PUSH32` with 0x5B embedded in its 32 immediate bytes, then jump into the middle of that PUSH — executing arbitrary code that the compiler never intended. That was one of those "oh no" moments when I read the spec.

We precompute valid JUMPDEST positions at interpreter startup:

```rust
fn compute_jumpdests(bytecode: &[u8]) -> Vec<bool> {
    let mut valid = vec![false; bytecode.len()];
    let mut i = 0;
    while i < bytecode.len() {
        if bytecode[i] == 0x5B { valid[i] = true; }
        // Skip PUSH immediate bytes
        if let Some(op) = Opcode::from_byte(bytecode[i]) {
            i += 1 + op.immediate_size();
        } else {
            i += 1;
        }
    }
    valid
}
```

## A loop in bytecode

```admonish note title="DUP and SWAP Families"
`DUP1`..`DUP16` duplicate the Nth stack element (counting from the top, 1-indexed) and push the copy. `SWAP1`..`SWAP16` swap the top element with the (N+1)th element. Both cost 3 gas ("very low" tier).
```

Here's how a Solidity `while (i < 5) { i++; }` compiles to bytecode — I traced through this manually before running it:

```
 0: PUSH1 0        ; i = 0
 2: JUMPDEST       ; loop_start
 3: PUSH1 5        ; push limit
 5: DUP2           ; copy i to top
 6: LT             ; i < 5?
 7: PUSH1 18       ; loop_body address
 9: JUMPI          ; jump if true
10: PUSH1 0, MSTORE, PUSH1 32, PUSH1 0, RETURN  ; exit: return i
18: JUMPDEST       ; loop_body
19: PUSH1 1        ; push 1
21: ADD            ; i++
22: PUSH1 2        ; loop_start address
24: JUMP           ; back to top
```

This is Exercise 3.10 — the "wow moment" where a loop actually runs in our interpreter.

## Context opcodes

At this point our interpreter has access to the full execution context (I in the Yellow Paper):

| Opcode | Returns | Yellow Paper |
|--------|---------|-------------|
| `CALLER` | Address of the immediate caller | I_s |
| `ORIGIN` | Address of the original EOA sender | I_o |
| `CALLVALUE` | Wei sent with this call | I_v |
| `CALLDATALOAD` | 32 bytes of calldata at offset | I_d |
| `CALLDATASIZE` | Length of calldata | \|I_d\| |
| `ADDRESS` | Address of the executing contract | I_a |

The distinction between `CALLER` and `ORIGIN` tripped me up at first. `CALLER` changes with each nested call, but `ORIGIN` always points to the EOA that initiated the transaction — a subtle but important difference for security.

## Exercises

- **3.5** — Jump to a non-JUMPDEST byte returns InvalidJump
- **3.6** — JUMP to a valid JUMPDEST works correctly
- **3.7** — JUMPI with nonzero condition takes the branch
- **3.8** — JUMPI with zero condition falls through
- **3.9** — CALLDATALOAD reads 32 bytes from calldata
- **3.10** — A complete loop in bytecode runs and returns the correct value

Run: `cargo test ex_3_5` through `cargo test ex_3_10`

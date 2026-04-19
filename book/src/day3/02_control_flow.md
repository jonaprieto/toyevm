# Control Flow: JUMP, JUMPI, and JUMPDEST

The EVM has no structured control flow — no `if`, no `for`, no function calls at the bytecode level. Instead, it has two jump instructions and a marker opcode:

| Opcode | Effect |
|--------|--------|
| `JUMP` | Pop destination from stack, jump unconditionally |
| `JUMPI` | Pop destination and condition; jump if condition ≠ 0 |
| `JUMPDEST` | No-op marker; valid target for jumps |

## JUMPDEST validation

This is a critical security property. You can only jump to a byte that:

1. Is the `JUMPDEST` opcode (0x5B), **and**
2. Is not part of a PUSH immediate sequence

Without rule 2, an attacker could craft a `PUSH32` with 0x5B embedded in its 32 immediate bytes, then jump into the middle of that PUSH — executing arbitrary code that the compiler never intended.

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

Here's how a Solidity `while (i < 5) { i++; }` compiles to bytecode:

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

This is Exercise 3.10 — the "wow moment" where a loop actually runs in your interpreter.

## Context opcodes

The interpreter now has access to the execution context (I in the Yellow Paper):

| Opcode | Returns | Yellow Paper |
|--------|---------|-------------|
| `CALLER` | Address of the immediate caller | I_s |
| `ORIGIN` | Address of the original EOA sender | I_o |
| `CALLVALUE` | Wei sent with this call | I_v |
| `CALLDATALOAD` | 32 bytes of calldata at offset | I_d |
| `CALLDATASIZE` | Length of calldata | \|I_d\| |
| `ADDRESS` | Address of the executing contract | I_a |

The distinction between `CALLER` and `ORIGIN` is crucial for security: `CALLER` changes with each nested call, but `ORIGIN` always points to the EOA that initiated the transaction.

## Exercises

- **3.5** — Jump to a non-JUMPDEST byte returns InvalidJump
- **3.6** — JUMP to a valid JUMPDEST works correctly
- **3.7** — JUMPI with nonzero condition takes the branch
- **3.8** — JUMPI with zero condition falls through
- **3.9** — CALLDATALOAD reads 32 bytes from calldata
- **3.10** — A complete loop in bytecode runs and returns the correct value

Run: `cargo test ex_3_5` through `cargo test ex_3_10`

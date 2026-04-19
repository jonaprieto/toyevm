# The Stack

The EVM stack is a Last-In-First-Out (LIFO) data structure holding up to **1024** values of type `U256`. Almost every opcode reads from or writes to the stack.

## Why 1024?

The depth limit prevents stack-overflow attacks. Without it, a malicious contract could recurse or push indefinitely, consuming unbounded memory on every node. The limit of 1024 is a protocol constant — exceeding it causes the transaction to revert.

## Operations

The stack supports four fundamental operations:

| Operation | EVM context | Rust signature |
|-----------|-------------|----------------|
| **push** | Result of computation | `push(&mut self, value: U256) -> Result<(), StackError>` |
| **pop** | Input to next opcode | `pop(&mut self) -> Result<U256, StackError>` |
| **swap** | `SWAP1`..`SWAP16` opcodes | `swap(&mut self, depth: usize) -> Result<(), StackError>` |
| **dup** | `DUP1`..`DUP16` opcodes | `dup(&mut self, depth: usize) -> Result<(), StackError>` |

Every operation that could fail returns a `Result`. There are exactly two failure modes:

```rust
pub enum StackError {
    Overflow,   // push when len == 1024
    Underflow,  // pop/peek when empty
}
```

## Implementation choice

We use a `Vec<U256>` internally rather than a fixed `[U256; 1024]` array. Why?

- A `Vec` starts small (we pre-allocate 64 slots) and grows as needed
- A fixed array would allocate 32 KB on every stack creation, even for contracts that use 3 stack slots
- The overflow check is a simple `len >= 1024` comparison either way

## Exercises

- **1.5** — Push three values, pop them back in LIFO order; pop from empty returns `Underflow`
- **1.6** — Push 1024 values successfully; the 1025th push returns `Overflow`

Run: `cargo test stack`

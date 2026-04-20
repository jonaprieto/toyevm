# The Stack

The first thing I needed was a stack — a Last-In-First-Out (LIFO) data structure holding up to **1024** values of type `U256`. Almost every opcode reads from or writes to the stack.

## Why 1024?

The depth limit prevents stack-overflow attacks. Without it, a malicious contract could recurse or push indefinitely, consuming unbounded memory on every node. The limit of 1024 is a protocol constant — exceeding it causes the transaction to revert.

## Operations

The stack supports four fundamental operations. Here is how I worked through each one with a concrete example.

### push — place a value on top

`push` adds a value to the top of the stack. If the stack already has 1024 elements, it returns `StackError::Overflow`.

```
Before:  [ 10, 20 ]      ← 20 is on top
push(30)
After:   [ 10, 20, 30 ]  ← 30 is now on top
```

In the EVM, `PUSH1 0x2A` pushes the byte `0x2A` (42) as a U256 onto the stack. Every arithmetic opcode pushes its result.

### pop — remove and return the top value

`pop` removes the topmost value and returns it. If the stack is empty, it returns `StackError::Underflow`.

```
Before:  [ 10, 20, 30 ]  ← 30 is on top
v = pop()                 → v = 30
After:   [ 10, 20 ]      ← 20 is now on top
```

Most opcodes pop their operands. For example, `ADD` pops two values, adds them, and pushes the result:

```
Before:  [ 10, 3, 5 ]    ← 5 is on top
ADD: a = pop() → 5
     b = pop() → 3
     push(a + b) → push(8)
After:   [ 10, 8 ]       ← 8 is on top
```

```admonish warning title="Pitfall: Pop Order"
The EVM pops the **top** element first. For `SUB`, this means `pop(a), pop(b), push(a - b)` — so `PUSH 3, PUSH 10, SUB` computes $10 - 3 = 7$, not $3 - 10$. The first value pushed is deeper; the second is on top and gets popped first.
```

### swap — exchange the top with a deeper element

`SWAP1` through `SWAP16` exchange the top element with the element at depth N (1-indexed from the top, not counting the top itself).

```
Before:  [ 10, 20, 30 ]  ← 30 is on top
SWAP1 (depth=1): swap top with 1 below
After:   [ 10, 30, 20 ]  ← 20 is now on top, 30 moved down
```

A more complex example with `SWAP2`:

```
Before:  [ 10, 20, 30, 40 ]  ← 40 is on top
SWAP2 (depth=2): swap top with 2 below
After:   [ 10, 40, 30, 20 ]  ← 20 on top, 40 moved to position 2
```

### dup — duplicate a stack element

`DUP1` through `DUP16` copy the element at depth N (1-indexed from the top) and push the copy on top. The original stays in place.

```
Before:  [ 10, 20, 30 ]  ← 30 is on top
DUP1 (depth=1): copy top element
After:   [ 10, 20, 30, 30 ]  ← copy of 30 on top
```

`DUP2` copies the second element:

```
Before:  [ 10, 20, 30 ]  ← 30 is on top
DUP2 (depth=2): copy second-from-top
After:   [ 10, 20, 30, 20 ]  ← copy of 20 on top
```

```admonish tip title="Rust Pattern: Result for Fallible Operations"
Every stack operation returns `Result<T, StackError>` instead of panicking. This models the EVM's behavior exactly — a stack underflow doesn't crash the VM, it reverts the transaction. Using `Result` forces the caller (the interpreter) to handle both paths explicitly.
```

## Error types

There are exactly two failure modes:

```rust
{{#include ../../../src/stack.rs:stack_error}}
```

## Implementation

I use a `Vec<U256>` internally rather than a fixed `[U256; 1024]` array:

```rust
{{#include ../../../src/stack.rs:stack_struct}}
```

Why `Vec`? I considered a fixed `[U256; 1024]` array, but:

- `Vec` starts small (pre-allocate 64 slots) and grows as needed
- A fixed `[U256; 1024]` array would allocate 32 KB on every stack creation, even for contracts that use 3 stack slots
- The overflow check is a simple `len >= 1024` comparison either way

The push/pop implementation:

```rust
{{#include ../../../src/stack.rs:stack_push_pop}}
```

## Exercises

- **1.5** — Push three values, pop them back in LIFO order; pop from empty returns `Underflow`
- **1.6** — Push 1024 values successfully; the 1025th push returns `Overflow`

Run: `cargo test stack`

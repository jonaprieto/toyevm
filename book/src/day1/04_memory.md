# Memory

EVM memory is a **byte-addressable**, dynamically expanding linear array. Unlike storage (which persists between transactions), memory is **ephemeral** — it exists only for the duration of a single execution context.

## Expansion semantics

Memory starts empty. Any read or write beyond the current size causes automatic expansion:

1. The new size is rounded up to the next **32-byte boundary** (word-aligned)
2. New bytes are initialized to **zero**
3. Expansion has a gas cost (covered in Day 2): $C_{mem}(a) = 3a + \lfloor a^2 / 512 \rfloor$ where $a$ is the word count.

The quadratic term makes large memory allocations progressively expensive. This prevents contracts from allocating gigabytes of memory on every node for free.

## Key operations

| Operation | EVM opcode | What it does |
|-----------|-----------|--------------|
| `store_u256(offset, value)` | `MSTORE` | Write 32 bytes at offset (big-endian) |
| `load_u256(offset)` | `MLOAD` | Read 32 bytes from offset |
| `store_byte(offset, value)` | `MSTORE8` | Write 1 byte at offset |

Note that `MLOAD` and `MSTORE` always read/write exactly **32 bytes**, but the offset can be any byte position — it does not need to be word-aligned. Reading from offset 1 reads bytes [1, 33) — this is a quirk that catches people.

```admonish warning title="Pitfall: Memory Offset Alignment"
A common mistake is assuming `MLOAD`/`MSTORE` require 32-byte-aligned offsets. They don't — `MLOAD(1)` reads bytes `[1, 33)`, overlapping two "words." This matters when reasoning about memory layout.
```

## Yellow Paper Map

In the Yellow Paper, memory is μ_m. The memory size function is:

```
M(μ_i, s, f) = max(μ_i, ⌈(f + s) / 32⌉)
```

Where `f` is the byte offset, `s` is the size in bytes, and `μ_i` is the current word count. Our `expansion_size` method implements this.

## Exercises

- **1.7** — Store and load a U256 value; store individual bytes and verify they appear in the loaded word
- **1.8** — Reading uninitialized memory returns zero; expansion always rounds to 32 bytes; large offsets expand correctly

Run: `cargo test memory`

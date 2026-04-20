# Memory

EVM memory is a **byte-addressable**, dynamically expanding linear array. Unlike storage (which persists between transactions), memory is **ephemeral** — it exists only for the duration of a single execution context. This was one of the distinctions I had to be careful to internalize early.

## Expansion semantics

Memory starts empty. Any read or write beyond the current size causes automatic expansion:

1. The new size is rounded up to the next **32-byte boundary** (word-aligned)
2. New bytes are initialized to **zero**
3. Expansion has a gas cost (covered in Day 2): $C_{mem}(a) = 3a + \lfloor a^2 / 512 \rfloor$ where $a$ is the word count.

The quadratic term makes large memory allocations progressively expensive. This prevents contracts from allocating gigabytes of memory on every node for free.

## Key operations

### MSTORE — write 32 bytes to memory

`MSTORE` pops an offset and a value from the stack, then writes the 32-byte big-endian representation of the value at that offset in memory.

```
Before:  Stack [ ..., 0x2A, 0x00 ]  ← 0x00 (offset) is on top
MSTORE: pop offset (0x00), pop value (0x2A)
        write 32 bytes of 0x2A at memory[0x00..0x20]
After:   Stack [ ... ]
         Memory[0x00..0x20] = 0x000...002A
```

### MLOAD — read 32 bytes from memory

`MLOAD` pops an offset, reads 32 bytes from memory, and pushes the result as a U256.

```
Before:  Stack [ ..., 0x00 ]  ← offset on top
MLOAD: pop offset (0x00), read memory[0x00..0x20]
After:   Stack [ ..., 0x2A ]  ← value loaded from memory
```

### MSTORE8 — write a single byte to memory

`MSTORE8` pops an offset and a value, then writes only the **lowest byte** of the value at the offset.

```
Before:  Stack [ ..., 0xFF, 0x05 ]  ← 0x05 (offset) is on top
MSTORE8: pop offset (0x05), pop value (0xFF)
         write byte 0xFF at memory[0x05]
After:   Stack [ ... ]
         Memory[0x05] = 0xFF
```

Note that `MLOAD` and `MSTORE` always read/write exactly **32 bytes**, but the offset can be any byte position — it does not need to be word-aligned. Reading from offset 1 reads bytes [1, 33) — a quirk that caught me off guard at first.

```admonish warning title="Pitfall: Memory Offset Alignment"
A common mistake is assuming `MLOAD`/`MSTORE` require 32-byte-aligned offsets. They don't — `MLOAD(1)` reads bytes `[1, 33)`, overlapping two "words." This matters when reasoning about memory layout.
```

## Memory layout visualization

After `MSTORE(0x00, 42)` and `MSTORE(0x20, 7)`, memory looks like:

![EVM memory layout](https://kroki.io/bytefield/svg/eNp1jsEKgkAQhu89xbBdXFBYF8nw1iHoEkXWA2ztjgmuympkRO-eW2Emdhv-mW_-z5FGXL1TkV107p2VkMpUdOK80mPRAGHNnIGDRinQShfmBmVtKIF7VJUiB38GEaZZBmQqmWSI5DHAGYNSSJnmSQ_qjhJRAllgrUwE20O88sEWut0ctPM63m92S_LzN-BWqtCfJYgabNUfMUTJBmLhCM_HeUTF-nzf-Y07ttuFgFP3m_A2CSmhT7czZy4=)

Each row is 32 bytes (one word). `MSTORE` writes a U256 value starting at the given byte offset.

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

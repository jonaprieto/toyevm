use crate::types::U256;

/// EVM memory: a byte-addressable, dynamically expanding linear memory.
///
/// In Yellow Paper notation, this is μ_m. Memory is ephemeral — it exists
/// only for the duration of a single execution context and is destroyed
/// when execution completes.
///
/// Memory expands in 32-byte (word) increments. Reading or writing beyond
/// the current size causes automatic expansion, with the new bytes
/// initialized to zero. Expansion has a gas cost (covered in Day 2).
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    /// Create a new, empty memory.
    pub fn new() -> Self {
        todo_exercise!("Exercise 1.7 — create an empty Vec<u8>")
    }

    /// The current size of memory in bytes.
    pub fn len(&self) -> usize {
        todo_exercise!("Exercise 1.7 — return Vec length")
    }

    /// Whether memory is empty (no expansion has occurred).
    pub fn is_empty(&self) -> bool {
        todo_exercise!("Exercise 1.7 — check if Vec is empty")
    }

    /// The current size of memory in 32-byte words (rounded up).
    pub fn word_count(&self) -> usize {
        todo_exercise!("Exercise 1.8 — divide len by 32, rounding up")
    }

    // ANCHOR: memory_expand
    /// Ensure memory is at least `size` bytes, expanding if needed.
    /// New bytes are zero-initialized. Size is rounded up to the nearest
    /// 32-byte boundary.
    fn expand_to(&mut self, _size: usize) {
        todo_exercise!("Exercise 1.8 — round up to 32-byte boundary, resize with zeros")
    }
    // ANCHOR_END: memory_expand

    /// Calculate the required memory size for an access at `offset` with `len` bytes.
    pub fn expansion_size(&self, _offset: usize, _len: usize) -> usize {
        todo_exercise!("Exercise 1.8 — compute new word count after access")
    }

    /// Store a single byte at the given offset (MSTORE8 semantics).
    pub fn store_byte(&mut self, _offset: usize, _value: u8) {
        todo_exercise!("Exercise 1.7 — expand, then set byte")
    }

    /// Store a 32-byte (U256) value at the given offset (MSTORE semantics).
    pub fn store_u256(&mut self, _offset: usize, _value: U256) {
        todo_exercise!("Exercise 1.7 — expand, then copy 32 bytes")
    }

    /// Load a 32-byte (U256) value from the given offset (MLOAD semantics).
    pub fn load_u256(&mut self, _offset: usize) -> U256 {
        todo_exercise!("Exercise 1.7 — expand, then read 32 bytes")
    }

    /// Read a slice of `len` bytes starting at `offset`.
    pub fn read_slice(&mut self, _offset: usize, _len: usize) -> Vec<u8> {
        todo_exercise!("Exercise 1.7 — expand, then copy slice")
    }

    /// Write a slice of bytes starting at `offset`.
    pub fn write_slice(&mut self, _offset: usize, _data: &[u8]) {
        todo_exercise!("Exercise 1.7 — expand, then copy data in")
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ex_1_7_store_and_load_u256() {
        let mut mem = Memory::new();
        let val = U256::from(0xCAFE_BABEu64);
        mem.store_u256(0, val);
        assert_eq!(mem.load_u256(0), val);
    }

    #[test]
    fn ex_1_7_store_byte() {
        let mut mem = Memory::new();
        mem.store_byte(0, 0xAB);
        mem.store_byte(31, 0xCD);
        let loaded = mem.load_u256(0);
        let bytes = loaded.to_bytes();
        assert_eq!(bytes[0], 0xAB);
        assert_eq!(bytes[31], 0xCD);
    }

    #[test]
    fn ex_1_7_non_overlapping_stores() {
        let mut mem = Memory::new();
        let a = U256::from(1u64);
        let b = U256::from(2u64);
        mem.store_u256(0, a);
        mem.store_u256(32, b);
        assert_eq!(mem.load_u256(0), a);
        assert_eq!(mem.load_u256(32), b);
    }

    #[test]
    fn ex_1_8_reading_uninitialized_returns_zero() {
        let mut mem = Memory::new();
        assert_eq!(mem.load_u256(0), U256::ZERO);
        assert_eq!(mem.load_u256(64), U256::ZERO);
    }

    #[test]
    fn ex_1_8_expansion_rounds_to_32_bytes() {
        let mut mem = Memory::new();
        mem.store_byte(0, 1);
        assert_eq!(mem.len(), 32);
        assert_eq!(mem.word_count(), 1);
        mem.store_byte(33, 1);
        assert_eq!(mem.len(), 64);
        assert_eq!(mem.word_count(), 2);
    }

    #[test]
    fn ex_1_8_large_offset_expansion() {
        let mut mem = Memory::new();
        mem.store_byte(1000, 0xFF);
        assert!(mem.len() >= 1001);
        assert_eq!(mem.len() % 32, 0);
        for i in 0..1000 {
            assert_eq!(mem.read_slice(i, 1)[0], 0);
        }
        assert_eq!(mem.read_slice(1000, 1)[0], 0xFF);
    }

    #[test]
    fn ex_1_8_word_count_calculation() {
        let mem = Memory::new();
        assert_eq!(mem.word_count(), 0);
        let mut mem = Memory::new();
        mem.store_u256(0, U256::ONE);
        assert_eq!(mem.word_count(), 1);
        mem.store_u256(32, U256::ONE);
        assert_eq!(mem.word_count(), 2);
    }

    #[test]
    fn expansion_size_calculation() {
        let mem = Memory::new();
        assert_eq!(mem.expansion_size(0, 0), 0);
        assert_eq!(mem.expansion_size(0, 1), 1);
        assert_eq!(mem.expansion_size(0, 32), 1);
        assert_eq!(mem.expansion_size(0, 33), 2);
        assert_eq!(mem.expansion_size(31, 1), 1);
        assert_eq!(mem.expansion_size(31, 2), 2);
    }
}

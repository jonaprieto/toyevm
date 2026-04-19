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
        Self { data: Vec::new() }
    }

    /// The current size of memory in bytes.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether memory is empty (no expansion has occurred).
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The current size of memory in 32-byte words (rounded up).
    ///
    /// This is used for gas calculation: `memory_cost = 3 * words + words^2 / 512`.
    pub fn word_count(&self) -> usize {
        self.data.len().div_ceil(32)
    }

    /// Ensure memory is at least `size` bytes, expanding if needed.
    /// New bytes are zero-initialized. Size is rounded up to the nearest
    /// 32-byte boundary.
    // ANCHOR: memory_expand
    fn expand_to(&mut self, size: usize) {
        if size > self.data.len() {
            // Round up to next 32-byte word boundary.
            let new_size = (size + 31) & !31;
            self.data.resize(new_size, 0);
        }
    }
    // ANCHOR_END: memory_expand

    /// Calculate the required memory size for an access at `offset` with `len` bytes.
    /// Returns the new word count if expansion is needed, or current word count.
    pub fn expansion_size(&self, offset: usize, len: usize) -> usize {
        if len == 0 {
            return self.word_count();
        }
        let needed = offset.saturating_add(len);
        let new_words = needed.div_ceil(32);
        new_words.max(self.word_count())
    }

    /// Store a single byte at the given offset (MSTORE8 semantics).
    pub fn store_byte(&mut self, offset: usize, value: u8) {
        self.expand_to(offset + 1);
        self.data[offset] = value;
    }

    /// Store a 32-byte (U256) value at the given offset (MSTORE semantics).
    ///
    /// The value is written in big-endian order, occupying bytes
    /// [offset, offset + 32).
    pub fn store_u256(&mut self, offset: usize, value: U256) {
        self.expand_to(offset + 32);
        self.data[offset..offset + 32].copy_from_slice(&value.to_bytes());
    }

    /// Load a 32-byte (U256) value from the given offset (MLOAD semantics).
    ///
    /// Reads bytes [offset, offset + 32). If this exceeds the current size,
    /// memory is expanded first (new bytes are zero).
    pub fn load_u256(&mut self, offset: usize) -> U256 {
        self.expand_to(offset + 32);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&self.data[offset..offset + 32]);
        U256::from_bytes(bytes)
    }

    /// Read a slice of `len` bytes starting at `offset`.
    /// Memory is expanded if needed.
    pub fn read_slice(&mut self, offset: usize, len: usize) -> Vec<u8> {
        if len == 0 {
            return Vec::new();
        }
        self.expand_to(offset + len);
        self.data[offset..offset + len].to_vec()
    }

    /// Write a slice of bytes starting at `offset`.
    /// Memory is expanded if needed.
    pub fn write_slice(&mut self, offset: usize, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        self.expand_to(offset + data.len());
        self.data[offset..offset + data.len()].copy_from_slice(data);
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

    // ---------------------------------------------------------------
    // Exercise 1.7: Memory load and store
    // ---------------------------------------------------------------
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

    // ---------------------------------------------------------------
    // Exercise 1.8: Memory expansion semantics
    // ---------------------------------------------------------------
    #[test]
    fn ex_1_8_reading_uninitialized_returns_zero() {
        let mut mem = Memory::new();
        // Reading at any offset should return zero (memory starts empty).
        assert_eq!(mem.load_u256(0), U256::ZERO);
        assert_eq!(mem.load_u256(64), U256::ZERO);
    }

    #[test]
    fn ex_1_8_expansion_rounds_to_32_bytes() {
        let mut mem = Memory::new();
        mem.store_byte(0, 1);
        // Even storing 1 byte should expand to 32 bytes.
        assert_eq!(mem.len(), 32);
        assert_eq!(mem.word_count(), 1);

        mem.store_byte(33, 1);
        // Now memory should be 64 bytes (2 words).
        assert_eq!(mem.len(), 64);
        assert_eq!(mem.word_count(), 2);
    }

    #[test]
    fn ex_1_8_large_offset_expansion() {
        let mut mem = Memory::new();
        mem.store_byte(1000, 0xFF);
        // Should expand to the next 32-byte boundary >= 1001.
        assert!(mem.len() >= 1001);
        assert_eq!(mem.len() % 32, 0);
        // All bytes before offset 1000 should be zero.
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
        assert_eq!(mem.expansion_size(0, 0), 0); // zero-length access, no expansion
        assert_eq!(mem.expansion_size(0, 1), 1); // 1 byte -> 1 word
        assert_eq!(mem.expansion_size(0, 32), 1); // 32 bytes -> 1 word
        assert_eq!(mem.expansion_size(0, 33), 2); // 33 bytes -> 2 words
        assert_eq!(mem.expansion_size(31, 1), 1); // byte at offset 31 -> 1 word
        assert_eq!(mem.expansion_size(31, 2), 2); // bytes 31-32 -> 2 words
    }
}

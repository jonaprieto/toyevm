/// A 256-bit unsigned integer stored in big-endian byte order.
///
/// The EVM operates on 256-bit words. We implement this as a newtype over
/// `[u8; 32]` in big-endian layout (most significant byte first), matching
/// the EVM's convention for stack values and storage keys.
///
/// In Yellow Paper notation, this corresponds to values in the range [0, 2^256).
// ANCHOR: u256_struct
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct U256([u8; 32]);
// ANCHOR_END: u256_struct

impl U256 {
    // ANCHOR: u256_constants
    /// The zero value (additive identity).
    pub const ZERO: Self = Self([0u8; 32]);

    /// The value one (multiplicative identity).
    pub const ONE: Self = {
        let mut bytes = [0u8; 32];
        bytes[31] = 1;
        Self(bytes)
    };

    /// The maximum value: 2^256 - 1.
    pub const MAX: Self = Self([0xFF; 32]);
    // ANCHOR_END: u256_constants

    /// Create a U256 from a big-endian byte array.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Return the big-endian byte representation.
    pub const fn to_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Check if this value is zero.
    pub fn is_zero(&self) -> bool {
        todo_exercise!("Exercise 1.1 — check if all 32 bytes are zero")
    }

    // ANCHOR: wrapping_add
    /// Wrapping addition: (self + rhs) mod 2^256.
    pub fn wrapping_add(self, _rhs: Self) -> Self {
        todo_exercise!("Exercise 1.2 — add byte-by-byte from LSB to MSB, propagating carry")
    }
    // ANCHOR_END: wrapping_add

    /// Wrapping subtraction: (self - rhs) mod 2^256.
    pub fn wrapping_sub(self, _rhs: Self) -> Self {
        todo_exercise!("Exercise 1.2 — subtract byte-by-byte with borrow")
    }

    /// Wrapping multiplication: (self * rhs) mod 2^256.
    pub fn wrapping_mul(self, _rhs: Self) -> Self {
        todo_exercise!("Exercise 1.2 — schoolbook multiplication, keep lower 256 bits")
    }

    /// Division: self / rhs. Returns zero if rhs is zero (EVM convention).
    pub fn checked_div(self, _rhs: Self) -> Self {
        todo_exercise!("Exercise 1.2 — long division, return ZERO for div-by-zero")
    }

    /// Modulo: self % rhs. Returns zero if rhs is zero (EVM convention).
    pub fn checked_mod(self, _rhs: Self) -> Self {
        todo_exercise!("Exercise 1.2 — remainder from division")
    }

    /// Convert to u64, returning None if the value exceeds u64::MAX.
    pub fn to_u64(&self) -> Option<u64> {
        todo_exercise!("Exercise 1.4 — check top 24 bytes are zero, then assemble lower 8")
    }

    /// Convert to usize, returning None if the value exceeds usize::MAX.
    pub fn to_usize(&self) -> Option<usize> {
        self.to_u64().and_then(|v| usize::try_from(v).ok())
    }
}

// --- Operator overloading ---

impl std::ops::Add for U256 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

impl std::ops::Sub for U256 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }
}

impl std::ops::Mul for U256 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
}

impl std::ops::BitAnd for U256 {
    type Output = Self;
    fn bitand(self, _rhs: Self) -> Self {
        todo_exercise!("Exercise 1.2 — bitwise AND each byte pair")
    }
}

impl std::ops::BitOr for U256 {
    type Output = Self;
    fn bitor(self, _rhs: Self) -> Self {
        todo_exercise!("Exercise 1.2 — bitwise OR each byte pair")
    }
}

impl std::ops::BitXor for U256 {
    type Output = Self;
    fn bitxor(self, _rhs: Self) -> Self {
        todo_exercise!("Exercise 1.2 — bitwise XOR each byte pair")
    }
}

impl std::ops::Not for U256 {
    type Output = Self;
    fn not(self) -> Self {
        todo_exercise!("Exercise 1.2 — bitwise NOT each byte")
    }
}

// --- Conversions ---

impl From<u64> for U256 {
    fn from(_val: u64) -> Self {
        todo_exercise!("Exercise 1.4 — place 8 big-endian bytes into positions 24..32")
    }
}

impl From<u128> for U256 {
    fn from(_val: u128) -> Self {
        todo_exercise!("Exercise 1.4 — place 16 big-endian bytes into positions 16..32")
    }
}

impl From<usize> for U256 {
    fn from(val: usize) -> Self {
        Self::from(val as u64)
    }
}

impl From<[u8; 32]> for U256 {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

// --- Comparison ---

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        todo_exercise!("Exercise 1.3 — big-endian comparison: compare from MSB to LSB")
    }
}

// --- Display ---

impl std::fmt::Display for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        let mut leading = true;
        for byte in &self.0 {
            if leading && *byte == 0 {
                continue;
            }
            leading = false;
            write!(f, "{byte:02x}")?;
        }
        if leading {
            write!(f, "0")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Exercise 1.1: U256 constants
    // ---------------------------------------------------------------
    #[test]
    fn ex_1_1_zero_is_all_zeros() {
        assert_eq!(U256::ZERO.to_bytes(), [0u8; 32]);
    }

    #[test]
    fn ex_1_1_one_has_lsb_set() {
        let bytes = U256::ONE.to_bytes();
        assert_eq!(bytes[31], 1);
        assert!(bytes[..31].iter().all(|&b| b == 0));
    }

    #[test]
    fn ex_1_1_max_is_all_ones() {
        assert_eq!(U256::MAX.to_bytes(), [0xFF; 32]);
    }

    // ---------------------------------------------------------------
    // Exercise 1.2: U256 addition
    // ---------------------------------------------------------------
    #[test]
    fn ex_1_2_simple_add() {
        let a = U256::from(2u64);
        let b = U256::from(3u64);
        assert_eq!(a + b, U256::from(5u64));
    }

    #[test]
    fn ex_1_2_add_overflow_wraps() {
        let result = U256::MAX + U256::ONE;
        assert_eq!(result, U256::ZERO);
    }

    #[test]
    fn ex_1_2_add_large_values() {
        let mut half = [0u8; 32];
        half[0] = 0x80;
        let a = U256::from_bytes(half);
        assert_eq!(a + a, U256::ZERO);
    }

    // ---------------------------------------------------------------
    // Exercise 1.3: U256 comparison
    // ---------------------------------------------------------------
    #[test]
    fn ex_1_3_comparison_basic() {
        let a = U256::from(10u64);
        let b = U256::from(20u64);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, a);
        assert_ne!(a, b);
    }

    #[test]
    fn ex_1_3_comparison_max() {
        assert!(U256::ZERO < U256::MAX);
        assert!(U256::MAX > U256::ONE);
    }

    #[test]
    fn ex_1_3_comparison_equal() {
        let a = U256::from(42u64);
        let b = U256::from(42u64);
        assert_eq!(a, b);
        assert!(!(a < b));
        assert!(!(a > b));
    }

    // ---------------------------------------------------------------
    // Exercise 1.4: U256 from conversions
    // ---------------------------------------------------------------
    #[test]
    fn ex_1_4_from_u64() {
        let val = U256::from(0xDEAD_BEEFu64);
        let bytes = val.to_bytes();
        assert_eq!(bytes[28], 0xDE);
        assert_eq!(bytes[29], 0xAD);
        assert_eq!(bytes[30], 0xBE);
        assert_eq!(bytes[31], 0xEF);
        assert!(bytes[..28].iter().all(|&b| b == 0));
    }

    #[test]
    fn ex_1_4_from_u128() {
        let val = U256::from(u128::MAX);
        let bytes = val.to_bytes();
        assert!(bytes[..16].iter().all(|&b| b == 0));
        assert!(bytes[16..].iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn ex_1_4_from_bytes() {
        let mut input = [0u8; 32];
        input[0] = 0xFF;
        input[31] = 0x01;
        let val = U256::from(input);
        assert_eq!(val.to_bytes(), input);
    }

    #[test]
    fn ex_1_4_to_u64_roundtrip() {
        let original = 123_456_789u64;
        let val = U256::from(original);
        assert_eq!(val.to_u64(), Some(original));
    }

    #[test]
    fn ex_1_4_to_u64_overflow() {
        assert_eq!(U256::MAX.to_u64(), None);
    }

    // ---------------------------------------------------------------
    // Additional: subtraction, multiplication, division
    // ---------------------------------------------------------------
    #[test]
    fn sub_basic() {
        let a = U256::from(10u64);
        let b = U256::from(3u64);
        assert_eq!(a - b, U256::from(7u64));
    }

    #[test]
    fn sub_underflow_wraps() {
        assert_eq!(U256::ZERO - U256::ONE, U256::MAX);
    }

    #[test]
    fn mul_basic() {
        let a = U256::from(6u64);
        let b = U256::from(7u64);
        assert_eq!(a * b, U256::from(42u64));
    }

    #[test]
    fn mul_overflow_wraps() {
        let result = U256::MAX * U256::from(2u64);
        assert_eq!(result, U256::MAX - U256::ONE);
    }

    #[test]
    fn div_basic() {
        let a = U256::from(42u64);
        let b = U256::from(7u64);
        assert_eq!(a.checked_div(b), U256::from(6u64));
    }

    #[test]
    fn div_by_zero_returns_zero() {
        assert_eq!(U256::from(5u64).checked_div(U256::ZERO), U256::ZERO);
    }

    #[test]
    fn mod_basic() {
        let a = U256::from(10u64);
        let b = U256::from(3u64);
        assert_eq!(a.checked_mod(b), U256::from(1u64));
    }

    #[test]
    fn bitwise_ops() {
        let a = U256::from(0b1100u64);
        let b = U256::from(0b1010u64);
        assert_eq!(a & b, U256::from(0b1000u64));
        assert_eq!(a | b, U256::from(0b1110u64));
        assert_eq!(a ^ b, U256::from(0b0110u64));
    }

    #[test]
    fn display_formatting() {
        assert_eq!(format!("{}", U256::ZERO), "0x0");
        assert_eq!(format!("{}", U256::from(255u64)), "0xff");
        assert_eq!(format!("{}", U256::from(256u64)), "0x0100");
    }
}

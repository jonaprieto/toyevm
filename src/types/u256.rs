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
        self.0 == [0u8; 32]
    }

    // ANCHOR: wrapping_add
    /// Wrapping addition: (self + rhs) mod 2^256.
    pub fn wrapping_add(self, rhs: Self) -> Self {
        let mut result = [0u8; 32];
        let mut carry: u16 = 0;
        // Add from least significant byte (index 31) to most significant (index 0).
        for i in (0..32).rev() {
            let sum = self.0[i] as u16 + rhs.0[i] as u16 + carry;
            result[i] = sum as u8;
            carry = sum >> 8;
        }
        // carry is discarded — wrapping semantics.
        Self(result)
    }
    // ANCHOR_END: wrapping_add

    /// Wrapping subtraction: (self - rhs) mod 2^256.
    pub fn wrapping_sub(self, rhs: Self) -> Self {
        let mut result = [0u8; 32];
        let mut borrow: i16 = 0;
        for i in (0..32).rev() {
            let diff = self.0[i] as i16 - rhs.0[i] as i16 - borrow;
            if diff < 0 {
                result[i] = (diff + 256) as u8;
                borrow = 1;
            } else {
                result[i] = diff as u8;
                borrow = 0;
            }
        }
        Self(result)
    }

    /// Wrapping multiplication: (self * rhs) mod 2^256.
    pub fn wrapping_mul(self, rhs: Self) -> Self {
        // We only need the lower 256 bits. Work with u64 limbs for efficiency.
        // Convert big-endian [u8; 32] to 4 x u64 limbs in little-endian order.
        let a = Self::to_u64_limbs(self.0);
        let b = Self::to_u64_limbs(rhs.0);

        // Schoolbook multiplication on 4 x u64 limbs.
        // result[i+j] += a[i] * b[j], but we only keep limbs 0..3 (lower 256 bits).
        let mut result = [0u128; 4];
        for i in 0..4 {
            for j in 0..4 {
                if i + j < 4 {
                    result[i + j] += a[i] as u128 * b[j] as u128;
                }
            }
        }
        // Propagate carries through the 4 limbs.
        let mut carry: u128 = 0;
        let mut limbs = [0u64; 4];
        for i in 0..4 {
            let sum = result[i] + carry;
            limbs[i] = sum as u64;
            carry = sum >> 64;
        }
        Self::from_u64_limbs(limbs)
    }

    /// Convert big-endian [u8; 32] to 4 little-endian u64 limbs.
    /// limbs[0] = least significant 64 bits (bytes[24..32])
    /// limbs[3] = most significant 64 bits (bytes[0..8])
    fn to_u64_limbs(bytes: [u8; 32]) -> [u64; 4] {
        let mut limbs = [0u64; 4];
        for (i, limb) in limbs.iter_mut().enumerate() {
            let be_offset = (3 - i) * 8; // limbs[0] ← bytes[24..32]
            *limb = u64::from_be_bytes([
                bytes[be_offset],
                bytes[be_offset + 1],
                bytes[be_offset + 2],
                bytes[be_offset + 3],
                bytes[be_offset + 4],
                bytes[be_offset + 5],
                bytes[be_offset + 6],
                bytes[be_offset + 7],
            ]);
        }
        limbs
    }

    /// Convert 4 little-endian u64 limbs to big-endian [u8; 32].
    fn from_u64_limbs(limbs: [u64; 4]) -> Self {
        let mut bytes = [0u8; 32];
        for (i, limb) in limbs.iter().enumerate() {
            let be_offset = (3 - i) * 8; // limbs[0] → bytes[24..32]
            bytes[be_offset..be_offset + 8].copy_from_slice(&limb.to_be_bytes());
        }
        Self(bytes)
    }

    /// Division: self / rhs. Returns zero if rhs is zero (EVM convention).
    pub fn checked_div(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            return Self::ZERO;
        }
        // Simple shift-and-subtract long division.
        self.div_mod(rhs).0
    }

    /// Modulo: self % rhs. Returns zero if rhs is zero (EVM convention).
    pub fn checked_mod(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            return Self::ZERO;
        }
        self.div_mod(rhs).1
    }

    /// Returns (quotient, remainder).
    fn div_mod(self, rhs: Self) -> (Self, Self) {
        if rhs.is_zero() {
            return (Self::ZERO, Self::ZERO);
        }
        if self < rhs {
            return (Self::ZERO, self);
        }

        let mut quotient = Self::ZERO;
        let mut remainder = Self::ZERO;

        // Process each bit from MSB to LSB (256 bits total).
        for bit_idx in 0..256 {
            // Shift remainder left by 1.
            remainder = remainder.shift_left_one();

            // Bring down the next bit of the dividend.
            let byte_pos = bit_idx / 8;
            let bit_pos = 7 - (bit_idx % 8);
            if (self.0[byte_pos] >> bit_pos) & 1 == 1 {
                remainder.0[31] |= 1;
            }

            // If remainder >= divisor, subtract and set quotient bit.
            if remainder >= rhs {
                remainder = remainder.wrapping_sub(rhs);
                let q_byte = bit_idx / 8;
                let q_bit = 7 - (bit_idx % 8);
                quotient.0[q_byte] |= 1 << q_bit;
            }
        }
        (quotient, remainder)
    }

    /// Shift left by one bit.
    fn shift_left_one(self) -> Self {
        let mut result = [0u8; 32];
        let mut carry = 0u8;
        for i in (0..32).rev() {
            result[i] = (self.0[i] << 1) | carry;
            carry = self.0[i] >> 7;
        }
        Self(result)
    }

    /// Convert to u64, returning None if the value exceeds u64::MAX.
    pub fn to_u64(&self) -> Option<u64> {
        // Check that the top 24 bytes are zero.
        for i in 0..24 {
            if self.0[i] != 0 {
                return None;
            }
        }
        let mut val = 0u64;
        for i in 24..32 {
            val = (val << 8) | self.0[i] as u64;
        }
        Some(val)
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
    fn bitand(self, rhs: Self) -> Self {
        let mut result = [0u8; 32];
        for (i, byte) in result.iter_mut().enumerate() {
            *byte = self.0[i] & rhs.0[i];
        }
        Self(result)
    }
}

impl std::ops::BitOr for U256 {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        let mut result = [0u8; 32];
        for (i, byte) in result.iter_mut().enumerate() {
            *byte = self.0[i] | rhs.0[i];
        }
        Self(result)
    }
}

impl std::ops::BitXor for U256 {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        let mut result = [0u8; 32];
        for (i, byte) in result.iter_mut().enumerate() {
            *byte = self.0[i] ^ rhs.0[i];
        }
        Self(result)
    }
}

impl std::ops::Not for U256 {
    type Output = Self;
    fn not(self) -> Self {
        let mut result = [0u8; 32];
        for (i, byte) in result.iter_mut().enumerate() {
            *byte = !self.0[i];
        }
        Self(result)
    }
}

// --- Conversions ---

impl From<u64> for U256 {
    fn from(val: u64) -> Self {
        let mut bytes = [0u8; 32];
        bytes[24..32].copy_from_slice(&val.to_be_bytes());
        Self(bytes)
    }
}

impl From<u128> for U256 {
    fn from(val: u128) -> Self {
        let mut bytes = [0u8; 32];
        bytes[16..32].copy_from_slice(&val.to_be_bytes());
        Self(bytes)
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
        // Big-endian comparison: compare from most significant byte.
        self.0.cmp(&other.0)
    }
}

// --- Display ---

impl std::fmt::Display for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        // Skip leading zeros for readability.
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
        // MAX + 1 should wrap to 0.
        let result = U256::MAX + U256::ONE;
        assert_eq!(result, U256::ZERO);
    }

    #[test]
    fn ex_1_2_add_large_values() {
        // 2^255 + 2^255 = 2^256 which wraps to 0.
        let mut half = [0u8; 32];
        half[0] = 0x80; // 2^255
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
        // Big-endian: value is in the last 4 bytes.
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
        // 0 - 1 should wrap to MAX.
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
        // MAX * 2 = MAX + MAX = (2^256 - 1)*2 = 2^257 - 2, mod 2^256 = 2^256 - 2 = MAX - 1
        let result = U256::MAX * U256::from(2u64);
        assert_eq!(result, U256::MAX - U256::ONE);
    }

    #[test]
    fn mul_large_values() {
        // 2^128 * 2^128 = 2^256 = 0 mod 2^256
        let mut bytes = [0u8; 32];
        bytes[15] = 1; // byte 15 = bit 128 in big-endian
        let a = U256::from_bytes(bytes);
        assert_eq!(a * a, U256::ZERO);
    }

    #[test]
    fn mul_near_max() {
        // (2^128 + 1) * (2^128 - 1) = 2^256 - 1 = MAX
        let mut bytes = [0u8; 32];
        bytes[15] = 1;
        bytes[31] = 1; // 2^128 + 1
        let a = U256::from_bytes(bytes);
        let b = a - U256::from(2u64); // 2^128 - 1
        assert_eq!(a * b, U256::MAX);
    }

    #[test]
    fn div_basic() {
        let a = U256::from(42u64);
        let b = U256::from(7u64);
        assert_eq!(a.checked_div(b), U256::from(6u64));
    }

    #[test]
    fn div_by_zero_returns_zero() {
        // EVM convention: division by zero yields zero.
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

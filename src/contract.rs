//! Contract address derivation for CREATE and CREATE2.
//!
//! CREATE:  address = keccak256(rlp([sender, nonce]))[12:]
//! CREATE2: address = keccak256(0xff ++ sender ++ salt ++ keccak256(initcode))[12:]
//!
//! We implement simplified versions that demonstrate the algorithm without
//! pulling in a full keccak256 crate. For testing, we use a naive hash.

use crate::context::Address;
use crate::types::U256;

/// Simplified hash function for address derivation (not real keccak256).
/// In production, you'd use the `tiny-keccak` or `sha3` crate.
fn simple_hash(data: &[u8]) -> [u8; 32] {
    // FNV-1a inspired hash spread across 32 bytes.
    // NOT cryptographically secure — pedagogical only.
    let mut hash = [0u8; 32];
    let mut h: u64 = 0xcbf29ce484222325;
    for &byte in data {
        h ^= byte as u64;
        h = h.wrapping_mul(0x100000001b3);
        // Spread into hash array
        for i in 0..8 {
            hash[(h as usize).wrapping_add(i) % 32] ^= ((h >> (i * 8)) & 0xFF) as u8;
        }
    }
    hash
}

/// Minimal RLP encoding for [sender_bytes, nonce].
/// This handles the specific case needed for CREATE address derivation.
fn rlp_encode_create(sender: &[u8; 20], nonce: u64) -> Vec<u8> {
    let mut sender_encoded = Vec::with_capacity(21);
    // RLP for 20-byte string: 0x80 + 20 = 0x94, then 20 bytes
    sender_encoded.push(0x94);
    sender_encoded.extend_from_slice(sender);

    let nonce_encoded = rlp_encode_u64(nonce);

    let total_len = sender_encoded.len() + nonce_encoded.len();
    let mut result = Vec::with_capacity(1 + total_len);

    // RLP list prefix
    if total_len < 56 {
        result.push(0xC0 + total_len as u8);
    } else {
        let len_bytes = encode_length(total_len);
        result.push(0xF7 + len_bytes.len() as u8);
        result.extend_from_slice(&len_bytes);
    }
    result.extend_from_slice(&sender_encoded);
    result.extend_from_slice(&nonce_encoded);
    result
}

fn rlp_encode_u64(val: u64) -> Vec<u8> {
    if val == 0 {
        return vec![0x80]; // RLP for empty byte string (nonce 0)
    }
    if val < 128 {
        return vec![val as u8]; // single byte
    }
    let bytes = val.to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(8);
    let significant = &bytes[start..];
    let mut result = Vec::with_capacity(1 + significant.len());
    result.push(0x80 + significant.len() as u8);
    result.extend_from_slice(significant);
    result
}

fn encode_length(len: usize) -> Vec<u8> {
    let bytes = (len as u64).to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
    bytes[start..].to_vec()
}

/// Derive a CREATE address from sender and nonce.
///
/// address = hash(rlp([sender, nonce]))[12:]
pub fn create_address(sender: Address, nonce: u64) -> Address {
    let rlp = rlp_encode_create(&sender.0, nonce);
    let hash = simple_hash(&rlp);
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[12..32]);
    Address(addr)
}

/// Derive a CREATE2 address from sender, salt, and init code hash.
///
/// address = hash(0xff ++ sender ++ salt ++ keccak256(initcode))[12:]
pub fn create2_address(sender: Address, salt: U256, initcode: &[u8]) -> Address {
    let init_hash = simple_hash(initcode);
    let mut preimage = Vec::with_capacity(1 + 20 + 32 + 32);
    preimage.push(0xFF);
    preimage.extend_from_slice(&sender.0);
    preimage.extend_from_slice(&salt.to_bytes());
    preimage.extend_from_slice(&init_hash);
    let hash = simple_hash(&preimage);
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[12..32]);
    Address(addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Exercise 4.1: CREATE address derivation
    // ---------------------------------------------------------------
    #[test]
    fn ex_4_1_create_address_deterministic() {
        let sender = Address::from(0x01u64);
        let addr1 = create_address(sender, 0);
        let addr2 = create_address(sender, 0);
        assert_eq!(addr1, addr2, "same sender + nonce must give same address");
    }

    #[test]
    fn ex_4_1_create_address_varies_with_nonce() {
        let sender = Address::from(0x01u64);
        let addr0 = create_address(sender, 0);
        let addr1 = create_address(sender, 1);
        assert_ne!(
            addr0, addr1,
            "different nonces must give different addresses"
        );
    }

    #[test]
    fn ex_4_1_create_address_varies_with_sender() {
        let addr1 = create_address(Address::from(0x01u64), 0);
        let addr2 = create_address(Address::from(0x02u64), 0);
        assert_ne!(
            addr1, addr2,
            "different senders must give different addresses"
        );
    }

    // ---------------------------------------------------------------
    // Exercise 4.2: CREATE2 address derivation
    // ---------------------------------------------------------------
    #[test]
    fn ex_4_2_create2_deterministic() {
        let sender = Address::from(0x01u64);
        let salt = U256::from(42u64);
        let initcode = vec![0x60, 0x00, 0x60, 0x00, 0xF3]; // PUSH1 0, PUSH1 0, RETURN
        let addr1 = create2_address(sender, salt, &initcode);
        let addr2 = create2_address(sender, salt, &initcode);
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn ex_4_2_create2_varies_with_salt() {
        let sender = Address::from(0x01u64);
        let initcode = vec![0x60, 0x00, 0xF3];
        let addr1 = create2_address(sender, U256::from(1u64), &initcode);
        let addr2 = create2_address(sender, U256::from(2u64), &initcode);
        assert_ne!(
            addr1, addr2,
            "different salts must give different addresses"
        );
    }

    #[test]
    fn ex_4_2_create2_varies_with_initcode() {
        let sender = Address::from(0x01u64);
        let salt = U256::from(1u64);
        let addr1 = create2_address(sender, salt, &[0x00]);
        let addr2 = create2_address(sender, salt, &[0x01]);
        assert_ne!(
            addr1, addr2,
            "different initcode must give different addresses"
        );
    }
}

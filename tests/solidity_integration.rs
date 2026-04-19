//! Integration tests: run real Solidity-compiled bytecode through toyevm.
//!
//! These tests demonstrate feeding `solc` output into our interpreter
//! and observing the results — including where our toy EVM falls short.

use toyevm::interpreter::run_bytecode;
use toyevm::types::U256;

const ENOUGH_GAS: u64 = 100_000;

/// Runtime bytecode from: examples/return42.sol (compiled with solc --optimize --no-cbor-metadata)
///
/// ```solidity
/// contract Return42 {
///     fallback() external {
///         assembly {
///             mstore(0x00, 42)
///             return(0x00, 0x20)
///         }
///     }
/// }
/// ```
///
/// Disassembly:
///   PUSH1 0x80, PUSH1 0x40, MSTORE       — set free memory pointer
///   CALLVALUE, DUP1, ISZERO              — check no ETH sent
///   PUSH1 0x0e, JUMPI                    — skip revert if zero value
///   PUSH0, PUSH0, REVERT                 — revert if ETH sent
///   JUMPDEST, POP                        — continue
///   PUSH1 0x2a, PUSH0, MSTORE           — store 42 at memory[0]
///   PUSH1 0x20, PUSH0, RETURN           — return 32 bytes
#[test]
fn return42_runtime() {
    let bytecode = hex_to_bytes("6080604052348015600e575f5ffd5b50602a5f5260205ff3");
    let result = run_bytecode(&bytecode, ENOUGH_GAS).unwrap();

    assert!(!result.reverted);
    assert_eq!(result.output.len(), 32);

    // The returned value should be 42, stored as a big-endian U256.
    let mut expected = [0u8; 32];
    expected[31] = 42;
    assert_eq!(result.output, expected);
}

/// Same contract but with CALLVALUE > 0 — should revert.
#[test]
fn return42_reverts_with_value() {
    use toyevm::context::ContextBuilder;
    use toyevm::interpreter::Interpreter;
    use toyevm::storage::Storage;

    let bytecode = hex_to_bytes("6080604052348015600e575f5ffd5b50602a5f5260205ff3");

    // Send value with the call — the contract should revert.
    let ctx = ContextBuilder::new().call_value(U256::from(1u64)).build();

    let mut interp = Interpreter::with_context(bytecode, ENOUGH_GAS, ctx, Storage::new());
    let result = interp.run().unwrap();
    assert!(result.reverted);
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

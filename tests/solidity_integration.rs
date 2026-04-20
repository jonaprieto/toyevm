//! Integration tests: run real Solidity-compiled bytecode through toyevm.
//!
//! These tests demonstrate feeding Foundry `forge build` output into our
//! interpreter and observing the results.
//!
//! To regenerate bytecode after editing the Solidity sources:
//!   cd examples && forge build
//!   forge inspect <Contract> deployedBytecode

use toyevm::context::ContextBuilder;
use toyevm::interpreter::{run_bytecode, Interpreter};
use toyevm::storage::Storage;
use toyevm::types::U256;

const ENOUGH_GAS: u64 = 100_000;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// Extract just the meaningful bytecode, stripping CBOR metadata suffix.
/// Solidity appends `a264...0033` at the end — we strip from the last INVALID (0xfe).
fn strip_metadata(bytecode: &[u8]) -> &[u8] {
    if let Some(pos) = bytecode.iter().rposition(|&b| b == 0xfe) {
        &bytecode[..pos]
    } else {
        bytecode
    }
}

// ---------------------------------------------------------------------------
// Return42 — arithmetic + memory + return (Day 2)
// ---------------------------------------------------------------------------

/// ```solidity
/// contract Return42 {
///     fallback() external {
///         assembly { mstore(0x00, 42); return(0x00, 0x20) }
///     }
/// }
/// ```
#[test]
fn return42_works() {
    let full = hex_to_bytes(
        "6080604052348015600e575f5ffd5b50602a5f5260205ff3\
         fea264697066735822122040979fcb6e0807aecb912ec897cd2cc497c33fec58c616fc89fa67aaf3d0e3cc64736f6c63430008220033",
    );
    let bytecode = strip_metadata(&full);
    let result = run_bytecode(bytecode, ENOUGH_GAS).unwrap();

    assert!(!result.reverted);
    assert_eq!(result.output.len(), 32);
    assert_eq!(result.output[31], 42);
}

#[test]
fn return42_reverts_with_value() {
    let full = hex_to_bytes(
        "6080604052348015600e575f5ffd5b50602a5f5260205ff3\
         fea264697066735822122040979fcb6e0807aecb912ec897cd2cc497c33fec58c616fc89fa67aaf3d0e3cc64736f6c63430008220033",
    );
    let bytecode = strip_metadata(&full);

    let ctx = ContextBuilder::new().call_value(U256::from(1u64)).build();
    let mut interp = Interpreter::with_context(bytecode.to_vec(), ENOUGH_GAS, ctx, Storage::new());
    let result = interp.run().unwrap();
    assert!(result.reverted);
}

// ---------------------------------------------------------------------------
// Store42 — storage write + read (Day 3)
// ---------------------------------------------------------------------------

/// ```solidity
/// contract Store42 {
///     fallback() external {
///         assembly { sstore(0, 42); let v := sload(0); mstore(0x00, v); return(0x00, 0x20) }
///     }
/// }
/// ```
#[test]
fn store42_writes_to_storage() {
    let full = hex_to_bytes(
        "6080604052348015600e575f5ffd5b50602a5f55602a5f5260205ff3\
         fea2646970667358221220ce867192559da8b439d08acc56f1d802840303a90071cd0d62739e42e99f2ab464736f6c63430008220033",
    );
    let bytecode = strip_metadata(&full);

    let mut interp = Interpreter::with_context(
        bytecode.to_vec(),
        ENOUGH_GAS,
        ContextBuilder::new().build(),
        Storage::new(),
    );
    let result = interp.run().unwrap();

    assert!(!result.reverted);
    assert_eq!(result.output[31], 42);

    let stored = interp.storage().peek(&U256::ZERO);
    assert_eq!(stored, U256::from(42u64));
}

// ---------------------------------------------------------------------------
// Loop — control flow with JUMP/JUMPI (Day 3)
// ---------------------------------------------------------------------------

/// ```solidity
/// contract Loop {
///     fallback() external {
///         assembly {
///             let sum := 0
///             let i := 1
///             for {} lt(i, 11) { i := add(i, 1) } { sum := add(sum, i) }
///             mstore(0x00, sum)
///             return(0x00, 0x20)
///         }
///     }
/// }
/// ```
#[test]
fn loop_sums_1_to_10() {
    let full = hex_to_bytes(
        "6080604052348015600e575f5ffd5b505f60015b600b811015602657908101906001016013565b\
         50805f5260205ff3\
         fea26469706673582212209d7fdc4e378e8bd064d9958c5d4e6b4658225c10f3d01a43ff9561cb406742ce64736f6c63430008220033",
    );
    let bytecode = strip_metadata(&full);
    let result = run_bytecode(bytecode, ENOUGH_GAS).unwrap();

    assert!(!result.reverted);
    assert_eq!(result.output[31], 55, "sum(1..10) should be 55");
}

// ---------------------------------------------------------------------------
// Add — function dispatch + ABI encoding + checked arithmetic (Day 3)
// ---------------------------------------------------------------------------

/// ```solidity
/// contract Add {
///     function add(uint256 a, uint256 b) public pure returns (uint256) {
///         return a + b;
///     }
/// }
/// ```
#[test]
fn add_3_plus_5() {
    let full = hex_to_bytes(
        "6080604052348015600e575f5ffd5b50600436106026575f3560e01c8063771602f714602a575b\
         5f5ffd5b60396035366004605d565b604b565b60405190815260200160405180910390f35b5f60\
         548284607c565b90505b92915050565b5f5f60408385031215606d575f5ffd5b50508035926020\
         909101359150565b80820180821115605757634e487b7160e01b5f52601160045260245ffd\
         fea26469706673582212200d1e380696ac698f952584323ee63f5c8bb703e124fcebf0be4353a9c561198864736f6c63430008220033",
    );
    let bytecode = strip_metadata(&full);

    // Build calldata: selector(0x771602f7) + a(3) + b(5)
    let mut calldata = vec![0x77, 0x16, 0x02, 0xf7];
    calldata.extend_from_slice(&U256::from(3u64).to_bytes());
    calldata.extend_from_slice(&U256::from(5u64).to_bytes());

    let ctx = ContextBuilder::new().calldata(calldata).build();
    let mut interp = Interpreter::with_context(bytecode.to_vec(), ENOUGH_GAS, ctx, Storage::new());
    let result = interp.run().unwrap();

    assert!(!result.reverted, "add(3,5) should not revert");
    assert_eq!(result.output.len(), 32);
    assert_eq!(result.output[31], 8, "3 + 5 = 8");
}

#[test]
fn add_reverts_on_wrong_selector() {
    let full = hex_to_bytes(
        "6080604052348015600e575f5ffd5b50600436106026575f3560e01c8063771602f714602a575b\
         5f5ffd5b60396035366004605d565b604b565b60405190815260200160405180910390f35b5f60\
         548284607c565b90505b92915050565b5f5f60408385031215606d575f5ffd5b50508035926020\
         909101359150565b80820180821115605757634e487b7160e01b5f52601160045260245ffd\
         fea26469706673582212200d1e380696ac698f952584323ee63f5c8bb703e124fcebf0be4353a9c561198864736f6c63430008220033",
    );
    let bytecode = strip_metadata(&full);

    // Wrong selector — should fall through to the default revert
    let mut calldata = vec![0xDE, 0xAD, 0xBE, 0xEF];
    calldata.resize(68, 0x00);

    let ctx = ContextBuilder::new().calldata(calldata).build();
    let mut interp = Interpreter::with_context(bytecode.to_vec(), ENOUGH_GAS, ctx, Storage::new());
    let result = interp.run().unwrap();

    assert!(result.reverted, "wrong selector should revert");
}

#[test]
fn add_reverts_on_short_calldata() {
    let full = hex_to_bytes(
        "6080604052348015600e575f5ffd5b50600436106026575f3560e01c8063771602f714602a575b\
         5f5ffd5b60396035366004605d565b604b565b60405190815260200160405180910390f35b5f60\
         548284607c565b90505b92915050565b5f5f60408385031215606d575f5ffd5b50508035926020\
         909101359150565b80820180821115605757634e487b7160e01b5f52601160045260245ffd\
         fea26469706673582212200d1e380696ac698f952584323ee63f5c8bb703e124fcebf0be4353a9c561198864736f6c63430008220033",
    );
    let bytecode = strip_metadata(&full);

    // Correct selector but only 4 bytes (no arguments) — SLT check should catch this
    let calldata = vec![0x77, 0x16, 0x02, 0xf7];

    let ctx = ContextBuilder::new().calldata(calldata).build();
    let mut interp = Interpreter::with_context(bytecode.to_vec(), ENOUGH_GAS, ctx, Storage::new());
    let result = interp.run().unwrap();

    assert!(
        result.reverted,
        "short calldata should revert (SLT bounds check)"
    );
}

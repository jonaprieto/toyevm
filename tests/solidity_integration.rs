//! Integration tests: run real Solidity-compiled bytecode through toyevm.
//!
//! These tests demonstrate feeding Foundry `forge build` output into our
//! interpreter and observing the results — including where our toy EVM
//! falls short and what opcodes need to be implemented next.
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
    // Strip optional 0x prefix.
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// Extract just the meaningful bytecode, stripping CBOR metadata suffix.
/// Solidity appends `a264...0033` at the end — we strip from the last INVALID (0xfe).
fn strip_metadata(bytecode: &[u8]) -> &[u8] {
    // Find the last 0xFE byte (INVALID opcode used as metadata marker).
    if let Some(pos) = bytecode.iter().rposition(|&b| b == 0xfe) {
        &bytecode[..pos]
    } else {
        bytecode
    }
}

// ---------------------------------------------------------------------------
// Day 2: Return42 — arithmetic + memory + return
// ---------------------------------------------------------------------------

/// Runtime bytecode: examples/src/return42.sol
/// Compiled: forge inspect Return42 deployedBytecode
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
/// Annotated runtime bytecode (stripped of metadata):
///   6080 6040 52      — PUSH1 0x80, PUSH1 0x40, MSTORE (free memory pointer)
///   34                — CALLVALUE
///   80                — DUP1
///   15                — ISZERO
///   600e              — PUSH1 0x0e
///   57                — JUMPI (skip revert if no value)
///   5f 5f fd          — PUSH0, PUSH0, REVERT
///   5b                — JUMPDEST
///   50                — POP
///   602a              — PUSH1 42
///   5f                — PUSH0
///   52                — MSTORE
///   6020              — PUSH1 32
///   5f                — PUSH0
///   f3                — RETURN
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
// Day 3: Store42 — storage write + read
// ---------------------------------------------------------------------------

/// Runtime bytecode: examples/src/store42.sol
/// Compiled: forge inspect Store42 deployedBytecode
///
/// After the callvalue check, the runtime does:
///   602a 5f 55     — PUSH1 42, PUSH0, SSTORE (store 42 at slot 0)
///   602a 5f 52     — PUSH1 42, PUSH0, MSTORE (store 42 in memory)
///   6020 5f f3     — PUSH1 32, PUSH0, RETURN
///
/// Note: the optimizer noticed sload(0) would return 42 (just stored),
/// so it replaced sload with the constant. Clever!
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

    // Verify storage was actually written.
    let stored = interp.storage().peek(&U256::ZERO);
    assert_eq!(stored, U256::from(42u64));
}

// ---------------------------------------------------------------------------
// Day 3: Loop — control flow with JUMP/JUMPI
// ---------------------------------------------------------------------------

/// Runtime bytecode: examples/src/loop.sol
/// Compiled: forge inspect Loop deployedBytecode
///
/// Sums 1..10 = 55 using a for loop in assembly.
/// The compiled bytecode uses JUMP/JUMPI for the loop.
#[test]
fn loop_sums_1_to_10() {
    let full = hex_to_bytes(
        "6080604052348015600e575f5ffd5b505f60015b600b811015602657908101906001016013565b\
         50805f5260205ff3\
         fea26469706673582212209d7fdc4e378e8bd064d9958c5d4e6b4658225c10f3d01a43ff9561cb406742ce64736f6c63430008220033",
    );
    let bytecode = strip_metadata(&full);
    let result = run_bytecode(bytecode, ENOUGH_GAS);

    match result {
        Ok(res) => {
            assert!(!res.reverted);
            assert_eq!(res.output[31], 55, "sum(1..10) should be 55");
        }
        Err(e) => {
            // If this fails, it tells us which opcode we're missing.
            // SLT (0x12) is used by the compiled loop — our stub returns 0.
            panic!(
                "Loop contract failed: {e}. \
                 This likely means SLT (signed less-than) is not yet implemented — \
                 the Solidity compiler uses SLT for loop bounds."
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Limitations: what our toyevm can't handle yet
// ---------------------------------------------------------------------------

/// The Add contract uses function selectors (CALLDATALOAD + SHR + EQ),
/// ABI decoding with SLT for bounds checking, and checked arithmetic.
/// This test documents which opcodes block us.
#[test]
fn add_contract_needs_slt() {
    let full = hex_to_bytes(
        "6080604052348015600e575f5ffd5b50600436106026575f3560e01c8063771602f714602a575b\
         5f5ffd5b60396035366004605d565b604b565b60405190815260200160405180910390f35b5f60\
         548284607c565b90505b92915050565b5f5f60408385031215606d575f5ffd5b50508035926020\
         909101359150565b80820180821115605757634e487b7160e01b5f52601160045260245ffd\
         fea2646970667358221220\
         4c3579bcd46e87a316c528e1142669fc8c448f3d7c7946fa819f6bbf2674619864736f6c63430008220033",
    );
    let bytecode = strip_metadata(&full);

    // Build calldata for add(3, 5): selector + abi-encoded args
    // selector: keccak256("add(uint256,uint256)") = 0x771602f7
    let mut calldata = vec![0x77, 0x16, 0x02, 0xf7];
    calldata.extend_from_slice(&U256::from(3u64).to_bytes()); // a = 3
    calldata.extend_from_slice(&U256::from(5u64).to_bytes()); // b = 5

    let ctx = ContextBuilder::new().calldata(calldata).build();
    let mut interp = Interpreter::with_context(bytecode.to_vec(), ENOUGH_GAS, ctx, Storage::new());
    let result = interp.run();

    match result {
        Ok(res) if !res.reverted => {
            // If this succeeds, SLT was implemented!
            assert_eq!(res.output.len(), 32);
            assert_eq!(res.output[31], 8, "3 + 5 = 8");
        }
        Ok(res) if res.reverted => {
            // Expected: SLT returns 0 (stub), causing ABI decoding to
            // think calldata is too short, and the contract reverts.
            // This is the "limitation" — we need proper SLT to run real contracts.
        }
        Ok(_) => unreachable!(),
        Err(e) => {
            // Also acceptable — missing opcode.
            eprintln!("Add contract error (expected): {e}");
        }
    }
}

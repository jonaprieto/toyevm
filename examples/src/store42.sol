// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// Day 3: Minimal storage write + read.
/// Stores 42 in slot 0, then returns it.
/// Demonstrates SSTORE/SLOAD in the simplest form.
contract Store42 {
    fallback() external {
        assembly {
            sstore(0, 42) // write 42 to storage slot 0
            let v := sload(0) // read it back
            mstore(0x00, v) // put it in memory
            return(0x00, 0x20) // return 32 bytes
        }
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// Day 3: A loop that sums 1..10 using inline assembly.
/// Demonstrates JUMP/JUMPI control flow in compiled output.
contract Loop {
    fallback() external {
        assembly {
            let sum := 0
            let i := 1
            for {} lt(i, 11) { i := add(i, 1) } {
                sum := add(sum, i)
            }
            mstore(0x00, sum)
            return(0x00, 0x20)
        }
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// The simplest possible contract: always returns 42.
// No function selector, no calldata parsing.
contract Return42 {
    fallback() external {
        assembly {
            mstore(0x00, 42)
            return(0x00, 0x20)
        }
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// A simple counter using storage slot 0.
/// Day 3: demonstrates SLOAD/SSTORE in compiled bytecode.
contract Counter {
    uint256 public count;

    function increment() public {
        count += 1;
    }

    function get() public view returns (uint256) {
        return count;
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// A simple counter using storage.
// increment(): adds 1 to slot 0
// get(): returns the value in slot 0
contract Counter {
    uint256 public count;

    function increment() public {
        count += 1;
    }
}

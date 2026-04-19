// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// Day 4: Emits a LOG event.
/// Demonstrates LOG1 opcode with one indexed topic.
contract EventEmitter {
    event ValueSet(uint256 indexed value);

    function set(uint256 v) public {
        emit ValueSet(v);
    }
}

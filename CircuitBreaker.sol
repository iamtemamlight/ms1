// SPDX-License-Identifier: PROPRIETARY
pragma solidity ^0.8.20;

/**
 * @title Allbright Sovereign Circuit Breaker
 * @dev Emergency halt system for the Ultra-Allbright Sovereign Hybrid.
 */
contract CircuitBreaker {
    address public commander;
    bool public isHalted;

    event FleetHalted(string reason, address indexed by);
    event FleetResumed(address indexed by);

    modifier onlyCommander() {
        require(msg.sender == commander, "Auth: Only Commander permitted");
        _;
    }

    constructor(address _commander) {
        commander = _commander;
    }

    /**
     * @notice Efficiently checks the halt status using Yul (Solidity Assembly).
     * @dev Reduces gas by direct slot access to packed state variables.
     * @return halted True if operations are currently halted.
     */
    function checkHalt() public view returns (bool halted) {
        assembly {
            // isHalted (bool) is packed with commander (address) in slot 0.
            // [ 11 bytes zero | 1 byte isHalted | 20 bytes commander ]
            // Shift right by 160 bits (20 bytes) to align the bool to the lowest bit.
            halted := and(shr(160, sload(0)), 0xff)
        }
    }

    /**
     * @notice Halts all on-chain interactions for the Sovereign fleet.
     */
    function triggerPanicButton(string calldata reason) external onlyCommander {
        isHalted = true;
        emit FleetHalted(reason, msg.sender);
    }

    /**
     * @notice Resumes operations once the threat is cleared.
     */
    function resumeOperations() external onlyCommander {
        isHalted = false;
        emit FleetResumed(msg.sender);
    }
}
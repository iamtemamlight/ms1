// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/governance/TimelockController.sol";

/**
 * @title AllBright Timelock Controller
 * @notice Extended timelock with AllBright-specific features
 * @dev Manages delayed execution of governance proposals
 */
contract AllBrightTimelock is TimelockController {
    // Proposal execution windows
    struct ExecutionWindow {
        uint256 proposalId;
        uint256 queuedAt;
        uint256 executeAt;
        uint256 cancelAt;
        bool executed;
        bool cancelled;
    }

    mapping(uint256 => ExecutionWindow) public executionWindows;
    uint256 public constant GRACE_PERIOD = 14 days;
    uint256 public constant MIN_DELAY = 1 hours;
    uint256 public constant MAX_DELAY = 30 days;

    event ProposalQueued(uint256 indexed proposalId, uint256 executeAt);
    event ProposalExecuted(uint256 indexed proposalId, bytes32 txHash);
    event ProposalCancelled(uint256 indexed proposalId);

    /**
     * @param minDelay_ Minimum execution delay
     * @param proposers_ Authorized proposers
     * @param executors_ Authorized executors
     * @param cancellers_ Authorized cancellers
     */
    constructor(
        uint256 minDelay_,
        address[] memory proposers_,
        address[] memory executors_,
        address[] memory cancellers_
    ) TimelockController(minDelay_, proposers_, executors_, cancellers_) {}

    /**
     * @notice Queue a transaction for future execution
     * @param target Target contract
     * @param value ETH value
     * @param data Function calldata
     * @param delay Custom delay (clamped to min/max)
     */
    function queueTransaction(
        address target,
        uint256 value,
        bytes calldata data,
        uint256 delay
    ) external onlyRole(PROPOSER_ROLE) returns (bytes32) {
        delay = _clampDelay(delay);
        
        bytes32 txHash = keccak256(abi.encode(target, value, data, block.timestamp + delay));
        require(!isOperationDone(txHash), "Operation already queued");
        require(!isOperationPending(txHash), "Operation already pending");

        uint256 proposalId = uint256(txHash) % 1000000;
        
        executionWindows[proposalId] = ExecutionWindow({
            proposalId: proposalId,
            queuedAt: block.timestamp,
            executeAt: block.timestamp + delay,
            cancelAt: block.timestamp + delay + GRACE_PERIOD,
            executed: false,
            cancelled: false
        });

        _schedule(target, value, data, delay);
        emit ProposalQueued(proposalId, block.timestamp + delay);
        
        return txHash;
    }

    /**
     * @notice Execute a queued transaction
     * @param target Target contract
     * @param value ETH value
     * @param data Function calldata
     * @param predecessor Predecessor operation hash (0 for first)
     * @param salt Random salt for operation uniqueness
     */
    function executeTransaction(
        address target,
        uint256 value,
        bytes calldata data,
        bytes32 predecessor,
        bytes32 salt
    ) external payable onlyRole(EXECUTOR_ROLE) returns (bytes32) {
        bytes32 txHash = keccak256(abi.encode(target, value, data, salt));
        
        require(isOperationReady(txHash), "Operation not ready");
        require(!isOperationDone(txHash), "Operation already done");

        uint256 proposalId = uint256(txHash) % 1000000;
        ExecutionWindow storage window = executionWindows[proposalId];
        
        require(!window.executed, "Already executed");
        require(!window.cancelled, "Cancelled");
        require(block.timestamp <= window.cancelAt, "Grace period expired");

        window.executed = true;
        
        bytes memory result = _execute(target, value, data, predecessor, salt);
        
        emit ProposalExecuted(proposalId, txHash);
        return txHash;
    }

    /**
     * @notice Cancel a queued transaction
     * @param target Target contract
     * @param value ETH value
     * @param data Function calldata
     * @param predecessor Predecessor operation hash
     * @param salt Random salt
     */
    function cancelTransaction(
        address target,
        uint256 value,
        bytes calldata data,
        bytes32 predecessor,
        bytes32 salt
    ) external onlyRole(CANCELLER_ROLE) {
        bytes32 txHash = keccak256(abi.encode(target, value, data, salt));
        require(isOperationPending(txHash), "Operation not pending");

        uint256 proposalId = uint256(txHash) % 1000000;
        executionWindows[proposalId].cancelled = true;

        _cancel(target, value, data, predecessor, salt);
        emit ProposalCancelled(proposalId);
    }

    /**
     * @notice Check if transaction is ready for execution
     * @param txHash Transaction hash
     */
    function isReady(bytes32 txHash) external view returns (bool) {
        return isOperationReady(txHash);
    }

    /**
     * @notice Get execution window details
     * @param proposalId Proposal ID
     */
    function getExecutionWindow(uint256 proposalId) external view returns (
        uint256 queuedAt,
        uint256 executeAt,
        uint256 cancelAt,
        bool executed,
        bool cancelled,
        uint256 remainingTime
    ) {
        ExecutionWindow memory window = executionWindows[proposalId];
        uint256 remaining = window.executeAt > block.timestamp 
            ? window.executeAt - block.timestamp 
            : 0;
            
        return (
            window.queuedAt,
            window.executeAt,
            window.cancelAt,
            window.executed,
            window.cancelled,
            remaining
        );
    }

    /**
     * @notice Update timelock delay
     * @param newDelay New minimum delay
     */
    function updateDelay(uint256 newDelay) external onlyRole(DEFAULT_ADMIN_ROLE) {
        newDelay = _clampDelay(newDelay);
        _changeDelay(newDelay);
        emit TimelockChanged(newDelay);
    }

    /**
     * @dev Clamp delay between min and max
     */
    function _clampDelay(uint256 delay) internal view returns (uint256) {
        if (delay < MIN_DELAY) return MIN_DELAY;
        if (delay > MAX_DELAY) return MAX_DELAY;
        return delay;
    }
}

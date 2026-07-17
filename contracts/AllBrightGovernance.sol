// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/governance/TimelockController.sol";
import "@openzeppelin/contracts/governance/IGovernor.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

/**
 * @title AllBright Governance with Timelock
 * @notice On-chain governance execution with timelock delay, voting, and delegation
 * @dev Addresses P0 governance gap: off-chain only -> on-chain execution
 */
contract AllBrightGovernance is TimelockController, AccessControl {
    using ECDSA for bytes32;

    // Roles
    bytes32 public constant PROPOSER_ROLE = keccak256("PROPOSER_ROLE");
    bytes32 public constant EXECUTOR_ROLE = keccak256("EXECUTOR_ROLE");
    bytes32 public constant CANCELLER_ROLE = keccak256("CANCELLER_ROLE");

    // Voting constants
    uint256 public constant VOTING_DELAY = 1 days;
    uint256 public constant VOTING_PERIOD = 3 days;
    uint256 public constant QUORUM_THRESHOLD = 100_000e18; // 100k tokens
    uint256 public constant APPROVAL_THRESHOLD = 50; // 50%
    uint256 public constant TIMELOCK_DELAY = 24 hours;

    // Proposal states
    enum ProposalState {
        Pending,
        Active,
        Canceled,
        Defeated,
        Succeeded,
        Queued,
        Expired,
        Executed
    }

    // Proposal structure
    struct Proposal {
        uint256 id;
        address proposer;
        bytes[] calldatas;
        address[] targets;
        uint256[] values;
        string description;
        uint256 voteStart;
        uint256 voteEnd;
        uint256 forVotes;
        uint256 againstVotes;
        uint256 abstainVotes;
        uint256 quorumReached;
        bool executed;
        bool canceled;
        bytes32 txHash;
    }

    // Voting struct
    struct Vote {
        uint8 support;
        uint256 weight;
        string reason;
    }

    // State
    uint256 public proposalCount;
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => Vote)) public votes;
    mapping(address => uint256) public votingPower;
    mapping(address => address) public delegation;
    mapping(address => uint256) public delegationBlock;

    // Checkpointing for historical voting power
    struct Checkpoint {
        uint256 fromBlock;
        uint256 votingPower;
    }
    mapping(address => Checkpoint[]) private checkpoints;
    mapping(address => uint256) private checkpointHistory;

    // Events
    event ProposalCreated(
        uint256 indexed proposalId,
        address indexed proposer,
        string description
    );
    event VoteCast(
        address indexed voter,
        uint256 indexed proposalId,
        uint8 support,
        uint256 weight,
        string reason
    );
    event ProposalExecuted(uint256 indexed proposalId, bytes32 txHash);
    event ProposalCanceled(uint256 indexed proposalId);
    event TimelockChanged(uint256 newDelay);
    event VotingPowerChanged(address indexed voter, uint256 newPower);

    // Modifiers
    modifier onlyProposer() {
        require(
            hasRole(PROPOSER_ROLE, msg.sender) || votingPower[msg.sender] >= QUORUM_THRESHOLD,
            "AllBrightGovernance: not authorized to propose"
        );
        _;
    }

    modifier onlyDuringVoting(uint256 proposalId) {
        require(proposals[proposalId].voteStart <= block.timestamp, "Voting not started");
        require(block.timestamp < proposals[proposalId].voteEnd, "Voting ended");
        _;
    }

    /**
     * @param timelockDelay_ Minimum delay for executed proposals
     * @param proposers_ List of initial proposers
     * @param executors_ List of initial executors
     * @param cancellers_ List of initial cancellers
     */
    constructor(
        uint256 timelockDelay_,
        address[] memory proposers_,
        address[] memory executors_,
        address[] memory cancellers_
    ) TimelockController(timelockDelay_, proposers_, executors_, cancellers_) {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        
        for (uint256 i = 0; i < proposers_.length; i++) {
            _grantRole(PROPOSER_ROLE, proposers_[i]);
        }
        for (uint256 i = 0; i < executors_.length; i++) {
            _grantRole(EXECUTOR_ROLE, executors_[i]);
        }
        for (uint256 i = 0; i < cancellers_.length; i++) {
            _grantRole(CANCELLER_ROLE, cancellers_[i]);
        }
    }

    /**
     * @notice Create a new governance proposal
     * @param targets Contract addresses to call
     * @param values ETH values to send
     * @param calldatas Function calldata
     * @param description Human-readable description
     */
    function propose(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        string memory description
    ) external onlyProposer returns (uint256) {
        require(targets.length == calldatas.length, "Length mismatch");
        require(targets.length > 0, "Empty proposal");

        proposalCount++;
        uint256 proposalId = proposalCount;

        proposals[proposalId] = Proposal({
            id: proposalId,
            proposer: msg.sender,
            calldatas: calldatas,
            targets: targets,
            values: values,
            description: description,
            voteStart: block.timestamp + VOTING_DELAY,
            voteEnd: block.timestamp + VOTING_DELAY + VOTING_PERIOD,
            forVotes: 0,
            againstVotes: 0,
            abstainVotes: 0,
            quorumReached: 0,
            executed: false,
            canceled: false,
            txHash: bytes32(0)
        });

        emit ProposalCreated(proposalId, msg.sender, description);
        return proposalId;
    }

    /**
     * @notice Cast a vote on a proposal
     * @param proposalId Proposal to vote on
     * @param support 0=against, 1=for, 2=abstain
     * @param reason Vote reason
     */
    function castVote(
        uint256 proposalId,
        uint8 support,
        string calldata reason
    ) external onlyDuringVoting(proposalId) {
        require(support <= 2, "Invalid vote type");
        
        Proposal storage proposal = proposals[proposalId];
        require(votes[proposalId][msg.sender].weight == 0, "Already voted");

        uint256 weight = getVotingPower(msg.sender, proposal.voteStart);
        require(weight > 0, "No voting power");

        votes[proposalId][msg.sender] = Vote({
            support: support,
            weight: weight,
            reason: reason
        });

        if (support == 1) {
            proposal.forVotes += weight;
        } else if (support == 0) {
            proposal.againstVotes += weight;
        } else {
            proposal.abstainVotes += weight;
        }

        emit VoteCast(msg.sender, proposalId, support, weight, reason);
    }

    /**
     * @notice Execute a successful proposal after timelock
     * @param proposalId Proposal to execute
     */
    function execute(
        uint256 proposalId
    ) external payable onlyRole(EXECUTOR_ROLE) returns (bytes memory) {
        Proposal storage proposal = proposals[proposalId];
        require(proposal.voteEnd <= block.timestamp, "Voting not ended");
        require(!proposal.executed, "Already executed");
        require(!proposal.canceled, "Canceled");

        // Check quorum
        uint256 totalVotes = proposal.forVotes + proposal.againstVotes + proposal.abstainVotes;
        require(totalVotes >= QUORUM_THRESHOLD, "Quorum not reached");

        // Check approval
        uint256 approval = proposal.forVotes * 100 / (proposal.forVotes + proposal.againstVotes);
        require(approval >= APPROVAL_THRESHOLD, "Approval not reached");

        // Check timelock
        require(block.timestamp >= proposal.voteEnd + TIMELOCK_DELAY, "Timelock not expired");

        proposal.executed = true;
        proposal.txHash = keccak256(abi.encode(proposalId, block.timestamp));

        // Execute via timelock
        bytes memory txData = _executeTransaction(
            proposal.targets[0],
            proposal.values[0],
            proposal.calldatas[0],
            block.timestamp + 1 days
        );

        emit ProposalExecuted(proposalId, proposal.txHash);
        return txData;
    }

    /**
     * @notice Cancel a proposal
     * @param proposalId Proposal to cancel
     */
    function cancel(uint256 proposalId) external onlyRole(CANCELLER_ROLE) {
        Proposal storage proposal = proposals[proposalId];
        require(!proposal.executed, "Already executed");
        require(!proposal.canceled, "Already canceled");
        require(msg.sender == proposal.proposer, "Not proposer");

        proposal.canceled = true;
        emit ProposalCanceled(proposalId);
    }

    /**
     * @notice Get proposal state
     * @param proposalId Proposal ID
     */
    function state(uint256 proposalId) external view returns (ProposalState) {
        Proposal memory proposal = proposals[proposalId];
        
        if (proposal.canceled) {
            return ProposalState.Canceled;
        }
        
        if (proposal.executed) {
            return ProposalState.Executed;
        }
        
        if (block.timestamp < proposal.voteStart) {
            return ProposalState.Pending;
        }
        
        if (block.timestamp <= proposal.voteEnd) {
            return ProposalState.Active;
        }
        
        uint256 totalVotes = proposal.forVotes + proposal.againstVotes + proposal.abstainVotes;
        if (totalVotes < QUORUM_THRESHOLD) {
            return ProposalState.Defeated;
        }
        
        uint256 approval = proposal.forVotes * 100 / (proposal.forVotes + proposal.againstVotes);
        if (approval < APPROVAL_THRESHOLD) {
            return ProposalState.Defeated;
        }
        
        if (block.timestamp >= proposal.voteEnd + TIMELOCK_DELAY) {
            return ProposalState.Queued;
        }
        
        return ProposalState.Succeeded;
    }

    /**
     * @notice Get voting power at a specific block
     * @param account Voter address
     * @param blockNumber Block number
     */
    function getVotingPower(address account, uint256 blockNumber) public view returns (uint256) {
        if (checkpointHistory[account] == 0) {
            return 0;
        }
        
        // Find latest checkpoint before blockNumber
        Checkpoint[] storage cps = checkpoints[account];
        uint256 len = cps.length;
        
        if (len == 0) return 0;
        
        uint256 low = 0;
        uint256 high = len;
        
        while (low < high) {
            uint256 mid = (low + high) / 2;
            if (cps[mid].fromBlock <= blockNumber) {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        
        if (low == 0) return 0;
        return cps[low - 1].votingPower;
    }

    /**
     * @notice Update voting power for an account
     * @param account Account to update
     * @param newPower New voting power
     */
    function updateVotingPower(address account, uint256 newPower) external onlyRole(DEFAULT_ADMIN_ROLE) {
        votingPower[account] = newPower;
        
        // Add checkpoint
        Checkpoint[] storage cps = checkpoints[account];
        if (cps.length > 0 && cps[cps.length - 1].fromBlock == block.number) {
            cps[cps.length - 1].votingPower = newPower;
        } else {
            cps.push(Checkpoint({
                fromBlock: block.number,
                votingPower: newPower
            }));
            checkpointHistory[account] = block.number;
        }

        emit VotingPowerChanged(account, newPower);
    }

    /**
     * @notice Delegate voting power
     * @param delegatee Address to delegate to
     */
    function delegate(address delegatee) external {
        require(delegatee != address(0), "Invalid delegatee");
        delegation[msg.sender] = delegatee;
        delegationBlock[msg.sender] = block.number;
    }

    /**
     * @notice Get proposal details
     * @param proposalId Proposal ID
     */
    function getProposal(uint256 proposalId) external view returns (Proposal memory) {
        return proposals[proposalId];
    }

    /**
     * @notice Get voting results
     * @param proposalId Proposal ID
     */
    function getResults(uint256 proposalId) external view returns (
        uint256 forVotes,
        uint256 againstVotes,
        uint256 abstainVotes,
        bool quorumReached
    ) {
        Proposal memory proposal = proposals[proposalId];
        uint256 totalVotes = proposal.forVotes + proposal.againstVotes + proposal.abstainVotes;
        return (
            proposal.forVotes,
            proposal.againstVotes,
            proposal.abstainVotes,
            totalVotes >= QUORUM_THRESHOLD
        );
    }
}

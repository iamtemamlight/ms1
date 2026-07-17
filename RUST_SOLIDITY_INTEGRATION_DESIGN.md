# AllBright Rust ↔ Solidity Integration Design

**Date:** 2026-07-13  
**Status:** DESIGN DOCUMENT — PENDING COMMANDER APPROVAL  
**Scope:** Controlled hybrid validation path for flash-loan arbitrage  
**Constraint:** No real funds, no mainnet deployment until approved

---

## 1. Design Goals

- Rust remains the authority for scanning, policy, signing, MEV protection, and metrics
- Solidity handles only atomic on-chain execution settlement and profit routing
- All execution is simulatable on a fork before mainnet
- No duplicate business logic between layers
- Full audit trail and rollback capability

---

## 2. Component Responsibilities

### Rust Layer (M137 + M025)
- Scan DEX pools via JSON-RPC
- Validate opportunities against policy
- Build calldata for FlashLoanArbitrage contract
- Sign transactions with `LocalWallet`
- Submit via Flashbots or public RPC
- Record trade result in ledger + DB
- Emit WebSocket telemetry

### Solidity Layer (FlashLoanArbitrage.sol)
- Receive flash-loan callback from Aave/Balancer/Uniswap
- Execute DEX swaps atomically
- Validate minimum profit post-swap
- Transfer net profit to `PROFIT_RECIPIENT`
- Emit `ArbitrageExecuted` event
- Emergency withdraw only by owner

### Node.js Backend
- Expose `/api/execute` route triggered by frontend or auto-pilot
- Call Rust engine via internal IPC or `std::process::Command`
- Persist trades to Postgres via Prisma
- Serve WebSocket updates to dashboard
- Store contract address + nonce management

### Frontend
- Display opportunities, KPIs, wallet state
- Trigger manual execution via `/api/execute`
- Receive real-time updates via WebSocket

---

## 3. Contract ABI

The following ABI entries are required for Rust integration:

```json
[
  {
    "inputs": [
      { "name": "token", "type": "address" },
      { "name": "amount", "type": "uint256" },
      { "name": "params", "type": "bytes" }
    ],
    "name": "executeAaveFlashLoan",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      { "name": "tokens", "type": "address[]" },
      { "name": "amounts", "type": "uint256[]" },
      { "name": "userData", "type": "bytes" }
    ],
    "name": "executeBalancerFlashLoan",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      { "name": "pair", "type": "address" },
      { "name": "amount0Out", "type": "uint256" },
      { "name": "amount1Out", "type": "uint256" },
      { "name": "data", "type": "bytes" }
    ],
    "name": "executeUniswapFlashSwap",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [
      { "indexed": true, "name": "token", "type": "address" },
      { "indexed": false, "name": "profit", "type": "uint256" },
      { "indexed": false, "name": "timestamp", "type": "uint256" },
      { "indexed": true, "name": "profitRecipient", "type": "address" }
    ],
    "name": "ArbitrageExecuted",
    "type": "event"
  }
]
```

---

## 4. Rust Interface Requirements

### New File: `backend/contracts/flash_loan_arbitrage.rs`

```rust
use ethers::prelude::*;
use ethers_core::types::Address;

#[derive(Debug, Clone)]
pub struct FlashLoanArbitrageContract {
    pub address: Address,
    pub abi: ethers::abi::Abi,
    pub client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl FlashLoanArbitrageContract {
    pub fn new(address: Address, client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>) -> Self {
        let abi = ethers::abi::parse_abi(&["function executeAaveFlashLoan(address,uint256,bytes)"]).unwrap();
        Self { address, abi, client }
    }

    pub async fn execute_aave_flash_loan(
        &self,
        token: Address,
        amount: U256,
        params: Vec<u8>,
    ) -> Result<TxHash, String> {
        let contract = Contract::new(self.address, self.abi.clone(), self.client.clone());
        let tx = contract
            .method::<_, ()>("executeAaveFlashLoan", (token, amount, params))
            .map_err(|e| format!("calldata error: {e}"))?
            .legacy()
            .into();
        let pending = self.client.send_transaction(tx, None).await.map_err(|e| format!("send error: {e}"))?;
        Ok(pending.tx_hash())
    }
}
```

### Integration Point: `m137_flash_loan_executor.rs`

In `execute_aave_flash_loan`, after policy validation:

```rust
// Existing dry-run path remains for simulation
if self.dry_run {
    return Ok(format!("DRY-RUN-AAVE-{}", uuid::Uuid::new_v4()));
}

// Production path: call FlashLoanArbitrage contract instead of raw Aave pool
let contract = FlashLoanArbitrageContract::new(self.contract_address, client.clone());
contract.execute_aave_flash_loan(asset_address, loan_amount, calldata).await
```

### Contract Address Source
- Read from env `FLASHLOAN_CONTRACT_ADDRESS`
- Fallback to settings API `/api/settings`
- Backend validates address on startup

---

## 5. Transaction Flow

### Manual Execution
1. User clicks **Execute** in DashboardView
2. Frontend calls `POST /api/execute` with `opportunityId`
3. Backend validates opportunity + RBAC
4. Backend calls Rust CLI binary with JSON args:
   ```bash
   allbright-engine execute --opportunity <id> --mode live
   ```
5. Rust M137:
   - Validates against policy
   - Builds calldata for FlashLoanArbitrage
   - Signs and submits tx
   - Returns `tx_hash` or error
6. Backend records trade in Postgres
7. Backend broadcasts `metrics_update` via WebSocket
8. Dashboard updates KPIs + table

### Auto-Execute Loop
1. Backend polls `/api/opportunities` every 5s
2. If any opportunity exceeds `minProfitThresholdPct` and `autoExecute` is enabled:
   - Calls Rust CLI as above
   - Blocks further execution until current tx confirms
3.成功后，记录KPI。

### Simulation / Shadow Fork
1. Use `PAPER_TRADING_MODE=true`
2. Rust returns synthetic `DRY-RUN-*` tx hash
3. Backend records trade with `simulated: true`
4. Metrics updated; real chain unchanged

---

## 6. Wallet Signing Process

- Private key stored in `.env` (`PRIVATE_KEY`)
- Rust `LocalWallet` derives address
- All transactions signed locally in Rust
- Never expose private key to frontend or Node.js
- Frontend MetaMask remains optional; backend executes with service wallet

### Optional User Wallet Path
- Frontend can call MetaMask directly via `useWeb3Wallet`
- MetaMask-signed tx submitted through backend `/api/execute/user`
- Backend relays but does not custody user funds
- Profit still routes to `PROFIT_RECIPIENT` on-chain

---

## 7. Error Handling

### Rust
- Return `Result<String, String>` with human-readable error
- Categories: `PolicyViolation`, `RpcError`, `SendError`, `Revert`, `InsufficientProfit`
- Backend logs to file `backend/errors/YYYY-MM-DD.log`

### Node.js
- Map Rust errors to HTTP status codes:
  - 400 policy/RPC
  - 500 internal
  - 429 rate limit
- Always return JSON `{ ok, trade?, error? }`

### Frontend
- Display `messageBanner` with success/error
- Disable Execute button during submission
- Refetch metrics after completion

---

## 8. Recovery Procedures

### Failed Trade
1. Rust returns error; backend records `failed: true`
2. No automatic retry to avoid duplicate tx
3. Commander can manually retry or skip

### Contract Revert
1. Aave/Balancer/Uniswap reverts bubble up through contract callback
2. Solidity contract emits `TradeFailed`
3. Backend listens via logs or reads receipt status

### Backend Crash During Execution
1. Trade remains `PENDING` in DB
2. On restart, backend queries pending trades via `eth_getTransactionCount` + nonce
3. Commander can replay or cancel

### Key Rotation
1. Update `PRIVATE_KEY` in `.env`
2. Restart backend
3. Rust rebuilds `LocalWallet`; no contract redeployment needed

---

## 9. Deployment Requirements

### Pre-Deployment Checklist
- [ ] Solidity audit passed + fixes applied
- [ ] FlashLoanArbitrage deployed on target chain
- [ ] `FLASHLOAN_CONTRACT_ADDRESS` set in `.env`
- [ ] `setApprovedCaller(backendExecutor, true)` executed
- [ ] Rust `contracts/flash_loan_arbitrage.rs` compiled and tested
- [ ] Foundry tests for contract pass
- [ ] Rust integration tests pass
- [ ] Commander approval recorded

### Deployment Sequence
1. Deploy `FlashLoanArbitrage.sol` via Commander wallet
2. Transfer ownership to multisig (optional)
3. Update `.env` with new address
4. Restart Node.js backend
5. Restart Rust Tauri app
6. Verify via `/api/execute` dry-run
7. Enable `PAPER_TRADING_MODE=false`
8. Monitor KPIs + WebSocket

---

## 10. Non-Goals (Out of Scope)

- Replacing Rust scanner with Solidity
- On-chain governance or voting
- User custody of funds
- Cross-chain bridging
- L2-specific flash loans (future iteration)

---

## 11. Open Questions for Commander

1. Should `PROFIT_RECIPIENT` be a multisig instead of a single EOA?
2. Should deployment require 2-of-3 Commander approval?
3. Do we allow user MetaMask execution path in Phase 1, or keep backend-only?
4. What is the maximum acceptable gas overhead for the extra contract hop?

---

*Design document produced by AllBright Integration Auditor. No code changes were made.*
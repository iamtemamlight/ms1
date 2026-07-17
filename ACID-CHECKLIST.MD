[6/24/2026 2:12 AM] Temam: Unified Institutional-Grade Security Checklist for Flash-Loan/Arbitrage Systems

This checklist merges the original 10-layer audit framework with advanced strategic pillars (fork-consistency, tokenomic edge cases, private order flow, and partial-fill armageddon). It is designed for systems handling real capital and prioritizes survivability over feature completeness.

---

Layer 1: Rust Application & Build Integrity

Build & Deployment

· Cargo.lock committed and pinned.
· Reproducible builds verified (cargo build --release --target-dir=build/).
· Release build uses --release with lto = true and panic = "abort" in Cargo.toml.
· No debug binaries or symbols in production.
· Binary SHA-256 hash recorded before deployment and verified on the server.
· Build server is physically/network-separated from the production server.
· cargo-careful run in CI to detect Undefined Behavior in unsafe blocks.

Dependency Audit

· cargo audit passes (no known CVEs).
· cargo deny check passes (licenses and advisories).
· No abandoned crates (check cargo outdated and maintainer activity).
· No crates with unresolved critical CVEs.
· Review all crates containing significant unsafe code (see below).
· Verify cryptography libraries (e.g., secp256k1, ethers) are well-maintained and use formally verified primitives where possible.
· cargo supply-chain run to verify the maintainers/registries of critical dependencies (mitigates supply-chain attacks).

Unsafe Rust Review

Search:

grep -R "unsafe" src/
Check every occurrence:

· Memory safety rigorously justified (why is unsafe unavoidable?).
· No unchecked pointer arithmetic or offset calculations.
· No unchecked buffer writes (use get_unchecked_mut only with proven bounds).
· No unsound transmute operations (prefer bytemuck with derive checks).
· FFI boundaries (C bindings) validate all input lengths and pointer validity.

Runtime Stability & Resilience

· No panic paths in the main trading loop (use catch_unwind at the top level to log and restart).
· Graceful handling of RPC failures: exponential backoff and fallback to secondary RPC.
· Graceful handling of malformed data (e.g., invalid UTF-8, truncated responses) – never panic.
· Graceful handling of chain reorganizations (detect via eth_getBlockByNumber state root changes).
· Automatic recovery after restart (restore pending transaction state from disk).
· Async runtime monitoring: tokio-console enabled in staging to detect blocking tasks.

---

Layer 2: Key, Wallet, and Secrets Management

Wallet Separation

Create physically or logically separated wallets for:

· Treasury (cold storage / hardware signing).
· Flash-loan execution (hot wallet with limited funds).
· Testing (testnet only).
· Deployment (smart contract admin keys, if applicable).
· Never combine these roles in a single seed phrase.

Signing Controls

· Hot wallet contains only operational funds (never more than 2x the maximum trade size).
· Treasury keys are air-gapped or use HSM/KMS (e.g., YubiHSM, AWS KMS with strict IAM policies).
· Signing permissions minimized (e.g., no transfer rights outside predefined arb contracts).
· Key rotation policy documented and tested.
· Access logging enabled for all signing events (who, when, which payload).

Secrets Storage (Zero-Tolerance Policy)

· No keys in source code.
· No keys in Git history (run git log -p and BFG if necessary).
· No keys in shell history (use HISTCONTROL=ignorespace and prefix commands with a space).
· No keys in screenshots.
· No keys in monitoring dashboards (e.g., Grafana env vars).
· No keys in log files (sanitize secrets before logging).
· Secrets injected via secure vault (e.g., HashiCorp Vault or systemd credential files) – never via environment variables in production.

Search explicitly:

grep -Ri "private" . --exclude-dir=target
grep -Ri "secret" .
grep -Ri "mnemonic" .
---

Layer 3: Blockchain Execution & Transaction Safety

Pre-Transaction Validation

Before building the transaction:

· Expected profit > estimated gas cost + 20% buffer.
· Expected profit > maximum slippage toler
[6/24/2026 2:12 AM] Temam: ance.
· Expected profit > flash-loan fee + protocol-specific premiums.
· Liquidity depth verified across all involved DEXes/pools.
· Price impact simulated and below the defined threshold.

Simulation (The "Golden Rule")

Every trade must be simulated first against a recent state.

· Dual-RPC state consensus: Fetch state root from two independent RPC nodes (e.g., Alchemy + Infura). If roots differ, abort and alert.
· Block-staleness guard: If simulation block is > 2 blocks behind eth_blockNumber, re-simulate.
· Verify flash-loan repayment logic (including fee deductions) – simulate the exact transfer call to the lender.
· Verify final token balances of the contract/bot (using balanceOf before and after) – do not trust amountOut variables for Fee-on-Transfer (FoT) or rebasing tokens.
· Verify no revert on all internal calls.
· Verify net positive result (including gas refunds).

Reorg & Orphan Handling

· Detect chain reorganizations by comparing parentHash of the latest block with the previously seen chain tip.
· Revalidate all pending transactions after a reorg (resimulate with new state).
· Handle orphaned transactions by resubmitting with the same nonce (RBF).
· Handle dropped transactions (gas price too low) – automatically bump gas if pending > 5 blocks.

Nonce Management (Concurrency)

· Nonce lock protection (use a distributed mutex or tokio::sync::Mutex for the signing service).
· Concurrent transaction safety – ensure eth_getTransactionCount is called with "pending" only once per block.
· Duplicate nonce prevention via a persistent nonce counter stored in SQLite/Redis.
· Replacement transaction strategy (RBF) defined: use maxPriorityFeePerGas bump of 10–20% for stuck txs.

Bundle Submission (Private Flow)

· Never submit raw transactions via eth_sendRawTransaction to the public mempool.
· Use Flashbots eth_sendBundle or BloXroute/private relays exclusively.
· Include a revert guard: if the arbitrage reverts, the entire bundle reverts atomically (no gas wasted on failed state changes).
· Define a max_block_deadline (e.g., 15 blocks) – after that, replace with a zero-value cancellation tx.

---

Layer 4: MEV Defense & Competitive Resilience

Front-Running & Sandwich Protection

· Private mempool is mandatory – mark this as NON-NEGOTIABLE.
· Simulate transaction exposure by testing the same calldata in the public mempool via a dummy bot to measure copy-trade risk.
· Monitor failed opportunities – a spike in failures indicates active sandwiching.
· Sandwich detection: before submission, simulate your tx against the current public mempool backlog. If cumulative slippage exceeds threshold (e.g., 3%), wait for next block.

Strategy Leakage

Ensure no one can reconstruct your strategy from:

· On-chain transactions (use multicall to flatten calldata and obscure intent).
· Contract bytecode (if deploying smart contracts, use obfuscation or proxy patterns).
· Public Git repositories (ensure .git is not exposed on the server).
· Monitoring endpoints (authenticate all metrics and dashboards).
· Error messages (avoid exposing pool addresses or route details in logs).

Competitive Monitoring

Track continuously:

· Win rate (successful bundles / total submitted).
· Lost opportunities (simulation profit > threshold but not submitted due to latency).
· Failed bundles (count and classify by revert reason).
· Competitor addresses (scan for known MEV bots interacting with the same pools).
· Latency metrics (RPC → simulation → signing → submission → block inclusion).

---

Layer 5: Server & Infrastructure Hardening

Operating System

· Latest security patches applied daily (unattended-upgrades enabled).
· Minimal installed packages (remove compilers, dev tools, package managers like cargo in production).
· Firewall enabled (allow only ports: SSH, metrics, and outbound to RPCs).
· Unused services disabled (e.g., cups, bluetooth, mail).
· Mandatory seccomp profiles applied to the Rust binary to block execve, fork, and other dangerous syscalls.

SSH Security

· SSH keys only – password authentication disabled.
· Root login disabled (P
[6/24/2026 2:12 AM] Temam: ermitRootLogin no).
· Fail2ban installed and configured.
· IP restrictions configured to allow only your office/VPN IPs.
· Use sshd configuration with MaxAuthTries 3 and ClientAliveInterval 300.

File Permissions & Network

· Run a privileged binary scan: find / -perm -4000 – remove unnecessary SUID binaries.
· Keys readable only by the service account (e.g., chmod 600 ~/.keys/*).
· Logs not world-readable (chmod 640 logs/*).
· Database credentials protected (use vault injection, not flat files).
· Network egress filters: allow outbound only to: RPC IPs/Ports, NTP (UDP 123), and your metrics sink. Block all other egress.

---

Layer 6: Monitoring, Alerting & Observability

Real-Time Alerts (PagerDuty/Slack)

Must receive alerts for:

· Failed transaction (any revert or out-of-gas).
· Consecutive failures (e.g., 3 in a row).
· Negative P&L for any single trade (breach of loss limit).
· RPC outage or latency spike (p95 > 1.5 seconds).
· Memory exhaustion (heap > 80% of available RAM).
· CPU saturation (load average > 2x cores for 5 min).
· Disk exhaustion (usage > 85%).

Critical Balance & Simulation Alerts (New)

· Balance drift alert: compare expected wallet balance (computed from previous txs) vs actual on-chain balance every 30s. Alert if delta > 0.01 ETH (signals a missed tx or hidden fee absorption).
· Simulation-failure rate spike: if > 3 simulations fail consecutively due to revert, auto-pause the bot (indicates broken route or sudden market shift).
· RPC latency p95 – if eth_call exceeds 1s, automatically rotate to backup RPC.

Security Alerts

· Unauthorized login attempts (monitor /var/log/auth.log).
· New SSH keys added (monitor ~/.ssh/authorized_keys changes).
· Service restarts (systemd unit unexpected exits).
· Unexpected wallet activity (transfers from the hot wallet not initiated by the bot).
· Configuration changes (monitor hash of config.toml).

---

Layer 7: Disaster Recovery & Business Continuity

Backup Strategy

Assume the server dies tonight. Can you recover within 1 hour?

· Source code backed up (private Git repository, mirrored).
· Configuration files backed up (with secrets encrypted).
· Persistent database (nonce state, pending txs) backed up hourly to an offsite S3 bucket.
· Recovery procedure documented step-by-step (tested quarterly).
· Wallet recovery tested (restore hot wallet from seed phrase on a fresh machine).
· Recovery Time Objective (RTO) defined (e.g., 60 minutes) and Recovery Point Objective (RPO) defined (e.g., 5 minutes).

Nonce & State Persistence

· Store the current nonce in a persistent DB (SQLite) updated atomically after every successful eth_sendBundle.
· On restart, restore nonce from DB, and reconcile with eth_getTransactionCount("pending") to handle unsent replacements.

Emergency Sweep

· Pre-signed, time-locked transaction that moves all funds from the hot wallet to the treasury.
· Store this signed transaction offline (e.g., encrypted USB drive).
· Can be broadcast manually via any public RPC in case of system compromise.

---

Layer 8: Financial Controls & Risk Limits

Exposure & Slippage Limits

· Maximum trade size (USD value per single arbitrage).
· Daily loss limit (hard stop if cumulative daily loss > 5% of capital).
· Hourly loss limit (hard stop if loss > 2% in 1 hour).
· Maximum gas spend per trade (to prevent "gas wars").
· Maximum slippage tolerance (per-leg and total, e.g., 0.5%).
· DEX-specific exposure caps: e.g., do not hold > $50k in a single Uniswap V3 position after a trade.

The "Unwind" Rule (Partial-Fill Armageddon)

· If the first leg executes but the second reverts, immediately execute a reverse swap (even at a loss) to neutralize the position.
· Define a max unwind loss (e.g., 2x the expected profit) – if exceeded, trigger the kill switch instead.

Kill Switches (Automated & Manual)

· Manual emergency stop – a locally accessible button/script that terminates the main loop.
· Automatic loss shutdown – triggers when daily loss limit is hit.
· RPC corruption shutdown – triggers if dual-RPC state roots disagree.
· Liquidity anomaly shutdown – tri
[6/24/2026 2:12 AM] Temam: ggers if pool reserves change > 50% in a single block (possible exploit).
· Oracle anomaly shutdown – triggers if price feed deviates > 5% from the chain's internal TWAP.

---

Layer 9: Penetration Testing & Chaos Engineering

Internal Chaos Tests (Before Go-Live)

· Attempt to crash the bot via malformed RPC responses (mock RPC returns invalid hex, huge ints, nulls).
· Simulate network failures (kill outbound internet for 10s).
· Simulate exchange/DEX failures (return 0x revert for specific pool calls).
· Simulate chain reorganizations (use anvil with auto-mine + reorg to ensure the bot re-validates).
· Gas price oracle manipulation – feed abnormally high baseFeePerGas and verify the bot's profitability check prevents submission.

External Security Tests

· Port scan the server (nmap from external IP) – ensure only SSH is open.
· Credential exposure scan (truffleHog on Git history).
· Dependency vulnerability scan (Snyk or cargo audit in CI).
· RPC malicious response test: configure a mock RPC to return stale prices + valid state root. Verify the bot detects price deviation > 5% and halts.

---

Layer 10: Preflight CI/CD & Go-Live Capital Protection

Pre-Deployment Staging Pipeline

Add a mandatory staging environment that mirrors mainnet using foundry + anvil:

· Fork mainnet at block N (latest + 100 to avoid reorgs).
· Run the arbitrage scanner against the fork.
· Execute the full trade (flash-loan + swap + repayment).
· Assert final balance > initial balance + gas * 1.1 (covering buffer).
· Only then deploy the binary to production.

Capital Ramping (The "Day-by-Day" Progression)

Never start with full allocation, even with perfect backtesting.

· Day 1–3 (Micro): Run with a max profit extraction of $10 per block and max trade size < $1,000. This limits downside while exposing you to live mempool dynamics.
· Day 4–7 (Small): Increase to $100 per block. Monitor the failed bundle rate. If it exceeds 10%, roll back and investigate (usually indicates private relay leakage).
· Day 8–14 (Medium): Increase to $500 per block after 100 consecutive profitable bundles.
· Full Production: Only after 500 consecutive profitable bundles with zero negative P&L days and no unhandled panics.

Final Sign-Off Criteria

Before flipping the switch to full production, verify:

· All checkboxes above are marked ✅.
· Emergency sweep test executed (simulated).
· Kill switch verified (actually stops the bot within 1 second).
· Recovery drill completed (restored from backup in < 30 minutes).
· Team on-call rotation defined and alerted.

---

🧵 Summary of Critical Non-Negotiables (The "Red Lines")

# Rule Rationale
1 Never submit to public mempool Guaranteed front-running & loss of edge.
2 Dual-RPC consensus A single corrupted RPC can show false profits.
3 Balance-based simulation (not amountOut) FoT/rebasing tokens break naive delta checks.
4 Nonce persistence Loss of nonce on restart = duplicate txs = wasted gas + reverts.
5 Atomic unwind for partial fills Prevents stuck positions from one-sided execution.

---

This unified checklist is your battle-hardened runbook. For an institutional review, pair this with a full audit of your Cargo.toml dependencies, deployment orchestration (K8s vs. bare-metal), RPC provider diversity, and the specific flash-loan protocol (Aave v3, dYdX, etc.) – as those details will further refine the risk model. Good luck, and stay safe.
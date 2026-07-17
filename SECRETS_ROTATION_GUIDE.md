# Secrets Rotation Guide — AllBright System
**Created**: 2026-06-24
**Severity**: CRITICAL — All secrets in `.env` are considered compromised

## 1. Immediate Actions Required

### 1.1 Rotate ALL API Keys
The following keys were found in plaintext in `.env` and `apps/dashboard/.env`:

| Service | Key Name | Rotation Command |
|---------|----------|-----------------|
| OpenAI | `OPENAI_API_KEY` | https://platform.openai.com/api-keys |
| OpenAI (AllBright) | `ALLBRIGHT_OPENAI_API_KEY` | Same as above |
| Groq | `ALLBRIGHT_GROQ` | https://console.groq.com/keys |
| Gemini | `GEMINI_API_KEY` | https://aistudio.google.com/app/apikey |
| Google AI Studio | `GOOGLE_AI_STUDIO` | Same as above |
| OpenRouter | `OPENROUTER_API_KEY` | https://openrouter.ai/keys |
| Pimlico | `PIMLICO_API_KEY` | https://dashboard.pimlico.io/ |
| Biconomy | `BICONOMY_API_KEY` | https://dashboard.biconomy.io/ |
| Biconomy | `BICONOMY_PROJECT_ID` | Same as above |
| Render | `RENDER_API_KEY` | https://dashboard.render.com/ |
| Alchemy | `ALCHEMY_RPC_URL` | https://alchemy.com/ |
| Alchemy | `ALCHEMY_WCC_UPC` | Same as above |
| OnFinality | `ONFINALITY_rpc_url` | https://app.onfinality.io/ |
| Ankr | `ANKR_RPC_URL` | https://www.ankr.com/ |
| DRPC | `ETH_RPC_URL` | https://drpc.org/ |
| Solana | `SOLANA_RPC_URL` | https://solana.com/ |
| Copilot | `VITE_COPILOT_API_KEY` | https://platform.openai.com/api-keys |

### 1.2 Rotate Wallet Private Key
**CRITICAL**: The private key `0xd2a2abbec92cd87ad5dfa60a75bce66d6b16369456ea132aad152bd28c0aebe` is exposed in both:
- `D:\ALLBRIGHT\.env` (line 62)
- `D:\ALLBRIGHT\apps\dashboard\.env` (line 62)

**Action**:
1. Generate a new wallet: `cast wallet new` (Foundry) or `eth-keygen`
2. Transfer ALL funds from the old wallet to the new wallet
3. Update `WALLET_ADDRESS` in `.env` to the new address
4. Store the new private key in OS keyring via `backend/key_manager.rs` — NOT in `.env`
5. Delete the old wallet from all systems

### 1.3 Rotate Database Credentials
- `DATABASE_URL` contains `neondb_owner:npg_21QWxIXtRrdb` — rotate Neon/Postgres password
- `DASHBOARD_PASS=alphamark2026` — rotate to a strong random value

### 1.4 Purge Build Artifacts from Git History
```bash
# Remove dist/ and build/ from git history
git filter-branch --force --index-filter \
  "git rm --cached --ignore-unmatch dist/ apps/dashboard/dist/ build/" \
  --prune-empty --tag-name-filter cat -- --all

# Or use BFG Repo-Cleaner for faster cleanup
bfg --delete-folders dist --delete-folders build
```

## 2. Secrets Storage Architecture

### 2.1 Production: HashiCorp Vault (Recommended)
```
Vault Path: secret/allbright/{environment}/{service}
Examples:
  secret/allbright/production/wallet/primary
  secret/allbright/production/rpc/eth_mainnet
  secret/allbright/production/api/openai
```

### 2.2 Development: OS Keyring
The `backend/key_manager.rs` already uses the OS keyring (`keyring` crate). Extend this to all secrets:
```rust
// Example: Load RPC URL from OS keyring
let entry = Entry::new("allbright-fleet", "eth_rpc_url")?;
let rpc_url = entry.get_password()?;
```

### 2.3 Kubernetes: Sealed Secrets or External Secrets Operator
For K8s deployments, use:
- **Sealed Secrets** (Bitnami) for GitOps-friendly encrypted secrets
- **External Secrets Operator** to sync from Vault to K8s Secrets

## 3. .env File Policy

### 3.1 Allowed in .env (Non-sensitive config only)
```env
NODE_ENV=production
PORT=3000
CHAIN_ID=1
PAPER_TRADING_MODE=true
SCAN_CONCURRENCY=8
MAX_PAIRS_TO_SCAN=1000
```

### 3.2 FORBIDDEN in .env (Must use vault)
```env
# NEVER THESE:
PRIVATE_KEY=...
API_KEY=...
SECRET=...
PASSWORD=...
TOKEN=...
```

## 4. Verification Checklist

- [ ] All API keys rotated
- [ ] Wallet private key rotated and old wallet drained
- [ ] `git log -p` verified no secrets in history
- [ ] `truffleHog` scan returns clean
- [ ] `dist/` and `build/` purged from git history
- [ ] `.gitignore` updated to exclude all sensitive paths
- [ ] OS keyring integration active for all secrets
- [ ] Vault integration tested in staging
- [ ] CI/CD secrets injected via environment (not files)
- [ ] All team members have new credentials

## 5. Emergency Response

If secrets were committed to a public repository:
1. Rotate ALL secrets immediately (assume compromise)
2. Use `git filter-branch` or BFG to purge from history
3. Force-push cleaned history
4. Contact GitHub Support to enable secret scanning alerts
5. Review access logs for unauthorized usage

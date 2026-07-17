# AllBright Tauri Desktop Deployment

## Windows Standalone Deployment

### Prerequisites
- Node.js 20+ installed
- Rust (cargo) installed  
- npm dependencies in `apps/dashboard/`

### Deployment Steps

1. **Build Dashboard**:
   ```batch
   cd apps\dashboard
   npm install
   npm run dashboard:build
   ```

2. **Build Tauri Desktop**:
   ```batch
   cd ..\..
   npm run tauri:build
   # OR use the automated script:
   deploy-tauri-standalone.bat
   ```

3. **Installation**:
   - Installer created at: `src-tauri/target/release/bundle/nsis/AllBright-Desktop_*.msi`
   - Desktop icon created at: `%USERPROFILE%\Desktop\AllBright Dashboard.exe`

### LocalPort Configuration

The Tauri desktop app connects to LocalPort services:

| Component | Port | Purpose |
|-----------|------|---------|
| Dashboard | Built-in | UI Interface |
| Fleet RPC | 8545-8549 | Multi-chain RPC |
| Backend gRPC | 50051 | Tauri backend commands |
| Backend HTTP | 3000 | REST API |

### Engine Control Flow

1. Launch "AllBright Dashboard" from desktop
2. Click CONNECT button (Phase 0)
3. DEBUG: Part 1 - Verify 72 KPIs + Security + Silicon
4. PREFLIGHT: Part 2 - Dual-verification
5. SIMULATION: Part 3 - Shadow-fork testing
6. PILOT: Part 4 - Controlled deployment
7. LIVE: Production with YubiKey

### Files Created

| File | Purpose |
|------|---------|
| `deploy-tauri-standalone.bat` | Automated Windows deployment |
| `apps/dashboard/dist/` | Built frontend assets |
| `src-tauri/target/release/` | Compiled Tauri binary |
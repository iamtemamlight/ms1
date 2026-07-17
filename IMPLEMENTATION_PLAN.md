 # Tauri Desktop Installation Plan - Allbright Defi V119

## Information Gathered

### Project Structure
- **Frontend:** `apps/dashboard/` - React + Vite application (V119.0.0)
- **Tauri Backend:** `src-tauri/` - Rust desktop backend (Tauri v2.11.3)
- **Existing Configuration:** 
  - Cargo.toml with tauri 2.x, tauri-plugin-shell, tauri-plugin-opener, serde
  - Icons exist in `icons/` directory
  - Pre-built frontend dist exists at `dist/`

### Configuration Details
- **Product Name:** Allbright Defi V119
- **version 119.0 (library)
- **Identifier:** com.allbright.defi.V119
- **Installers:** MSI and NSIS
- **Category:** Finance
- **Short Description:** Allbright Defi V119
- **Copyright:** © 2026 ALLBRIGHT DEFI SOFTWARE LTD

### Created Files
1. `src-tauri/src/main.rs` - Main entry point
2. `src-tauri/src/lib.rs` - Library entry point  
3. `src-tauri/tauri.conf.json` - Complete bundle configuration (Tauri v2 schema)
4. `apps/dashboard/nginx.conf` - Production nginx config for Docker

## Plan

### Phase 1: Prerequisites
- [x] Verify Node.js and npm installed
- [x] Verify Rust and cargo installed
- [x] Tauri CLI v2 installed (root package.json devDependency: @tauri-apps/cli ^2.11.3)

### Phase 2: Build Frontend
```bash
cd apps/dashboard
npm install
npm run build
```
**Output:** `apps/dashboard/dist/`

### Phase 3: Build Desktop App
```bash
cd src-tauri
cargo tauri build
```
**Note:** Requires TLS certs in `./certs/` for backend gRPC server. If certs are absent, backend falls back to plaintext gRPC (PILOT_MODE only).

### Phase 4: Generate Installers
```bash
cd src-tauri
cargo tauri build
# MSI and NSIS both generated via bundle targets in tauri.conf.json
```

## Output Files
- **MSI:** `src-tauri/target/release/bundle/msi/Allbright_Defi_V119_91.0.0_x64.msi`
- **NSIS:** `src-tauri/target/release/bundle/nsis/Allbright_Defi_V119_91.0.0_x64-setup.exe`

## Verification
- [ ] Check installers generated in `src-tauri/target/release/bundle/`
- [ ] Run MSI installer - verify installation
- [ ] Run NSIS installer - verify installation
- [ ] Launch installed application

## Commands
```powershell
# Build full application (frontend + desktop)
npm run build

# Build desktop only
npm run desktop:build

# Build specific target via Tauri v2
cd src-tauri
cargo tauri build
```

## Important Notes
- **Tauri Version:** v2.11.3 (NOT v1.x). All documentation and scripts must reference v2.
- **TLS Certs:** Backend gRPC server requires certs at `./certs/server.crt` and `./certs/server.key`. A fallback to plaintext exists for PILOT_MODE but should not be used in production.
- **Frontend Dist:** Tauri expects frontend build at `../apps/dashboard/dist` relative to `src-tauri/`.

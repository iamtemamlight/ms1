# TODO: Configure src-tauri with MSI and NSIS Desktop Installers

## Plan
- [x] Enhance tauri.conf.json with comprehensive MSI configuration
- [x] Enhance tauri.conf.json with comprehensive NSIS configuration  
- [x] Verify configuration with build test
- [x] Clean up incorrectly placed dist directory
- [x] Add `desktop:build:msi-nsis` and `build:msi-nsis` npm scripts

## Current Status
- ✅ MSI Configuration enhanced with:
  - Upgrade Code: `963731a3-f590-50b0-ae87-76b5b816e03e` for proper version upgrades
  - Language: `en-US`
- ✅ NSIS Configuration enhanced with:
  - Install mode: `perMachine` (system-wide installation)
- ✅ LICENSE file copied to src-tauri folder
- ✅ index.html title updated to "AllBright Defi V119 Commander"
- ✅ MSI built: `AllBright Defi V119_91.0.0_x64_en-US.msi` (2.4 MB)
- ✅ NSIS built: `AllBright Defi V119_91.0.0_x64-setup.exe` (1.7 MB)

## Configuration Applied
The `tauri.conf.json` now includes:
- `bundle.windows.wix.upgradeCode` - Persistent upgrade code for MSI
- `bundle.windows.wix.language` - Installer language en-US
- `bundle.windows.nsis.installMode` - perMachine installation
- `copyright` - ALLBRIGHT DEFI SOFTWARE LTD
- `productName` - AllBright Defi V119

## Next Steps
1. Run `npm run desktop:build` from D:\ALLBRIGHT to regenerate installers with proper settings

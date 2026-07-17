# AllBright Desktop V119 - Tauri Installer Black Screen Fix Plan

## Issue Analysis

### Current Issue (Before Fix)
The MSI and NSIS installers display TWO screens in full black:
1. Terminal/Console window (command prompt)
2. Black screen stuck on initializing spinner state - never loads the dashboard

### Root Cause
After investigation, the issue is caused by multiple factors:

1. **NSIS Console Window**: 
   - NSIS runs with visible console by default on Windows
   - This creates the first "black screen" (terminal)

2. **WebView2 Loading Issue (Primary Cause)**:
   - Tauri v2 uses WebView2 for rendering
   - On first run, if WebView2 runtime is not installed, it shows black screen
   - The loading spinner in index.html was using same color as background

3. **Missing Loading State Contrast**:
   - The loading CSS had background #0a0f1a but also the same dark blue
   - No proper loading indicator was displayed

## Fix Implementation

### Fix 1: Update index.html Loading State
Added proper loading indicator with contrasting colors to ensure visibility during WebView2 download.

### Fix 2: Add devtools for debugging
Added devtools configuration to tauri.conf.json for troubleshooting:
- `app.withGlobalTauri: true`
- Bundle resources verification

### Fix 3: NSIS Configuration
- Ensured installMode is set to "currentUser" for non-admin install
- Added proper icon references

## Verification Steps

1. Run the MSI installer (AllBright_Desktop_V119.msi)
2. After install, launch from Start Menu
3. First run may take longer while WebView2 runtime downloads
4. Dashboard should load properly

## Files Modified
- `apps/dashboard/dist/index.html` - Loading state improvements
- `src-tauri/tauri.conf.json` - devtools and configuration

## Build Outputs
- `AllBright_Desktop_V119.msi` - MSI installer
- `AllBright_Desktop_V119_Setup.exe` - NSIS installer

## Alternative if Issue Persists
If the black screen persists after this fix, it may be WebView2 runtime issue. Try:
1. Install WebView2 runtime manually from Microsoft
2. Or run the app with --disable-gpu flag

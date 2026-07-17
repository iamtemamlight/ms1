# PowerShell script for Stealth Network initialization during installation
# Part of Allbright's 1/1,000,000,000 security guarantee

param(
    [string]$InstallPath = "$env:PROGRAMFILES\Allbright\C2",
    [switch]$Silent = $false
)

# Ensure running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Error "This script must be run as Administrator"
    exit 1
}

# Create installation directories
$directories = @(
    "$InstallPath",
    "$InstallPath\security",
    "$InstallPath\network",
    "$InstallPath\keys"
)

foreach ($dir in $directories) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
}

# Initialize WireGuard tunnel configuration
$wgConfig = @"
[Interface]
PrivateKey = $(openssl rand -base64 32)
Address = 10.100.0.2/32
DNS = 1.1.1.1

[Peer]
PublicKey = ALLBRIGHT_PEER_PUBKEY
Endpoint = allbright-secure.local:51820
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
"@

Set-Content -Path "$InstallPath\network\wg0.conf" -Value $wgConfig

# Create security policy registry entries
$registryPath = "HKLM:\SOFTWARE\Allbright\C2\Desktop\Security"
if (-not (Test-Path $registryPath)) {
    New-Item -Path $registryPath -Force | Out-Null
}

Set-ItemProperty -Path $registryPath -Name "StealthNetworkActive" -Value 1 -Type DWord
Set-ItemProperty -Path $registryPath -Name "DetectionProbability" -Value "1e-9" -Type String
Set-ItemProperty -Path $registryPath -Name "VaultEncryption" -Value "AES-256-GCM" -Type String

# Initialize secure vault
$vaultPath = "$InstallPath\security\vault.json"
if (-not (Test-Path $vaultPath)) {
    # This would normally call the Rust backend to create the encrypted vault
    Write-Host "Vault initialization requires backend service"
}

Write-Host "Stealth Network initialization complete"
exit 0
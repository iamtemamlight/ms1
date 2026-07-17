$ErrorActionPreference = "Continue"
$results = @{}

# Check Program Files
if (Test-Path "C:\Program Files\ALLBRIGHT") {
    $results["Program Files"] = Get-ChildItem "C:\Program Files\ALLBRIGHT" -Recurse | Select-Object FullName
}

if (Test-Path "C:\Program Files (x86)\ALLBRIGHT") {
    $results["Program Files x86"] = Get-ChildItem "C:\Program Files (x86)\ALLBRIGHT" -Recurse | Select-Object FullName
}

# Check registry for uninstall entries
$regPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*"
$installed = Get-ItemProperty $regPath -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like "*ALLBRIGHT*" }
$results["Registry"] = $installed

# Check Start Menu
$startMenuPaths = @(
    "C:\ProgramData\Microsoft\Windows\Start Menu\Programs\ALLBRIGHT",
    "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\ALLBRIGHT"
)
foreach ($path in $startMenuPaths) {
    if (Test-Path $path) {
        $results["Start Menu"] = Get-ChildItem $path -Recurse -File
    }
}

$results | ConvertTo-Json -Depth 10

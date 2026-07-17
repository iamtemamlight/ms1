################################################################################
# ALLBRIGHT — Live Endpoint Verification Script
# Tests: Backend health, Groq AI, OpenRouter AI, Copilot connection, ENV keys
################################################################################

$ErrorActionPreference = "Continue"
$results = @()

function Test-Result {
    param($Name, $Status, $Detail)
    $symbol = if ($Status -eq "PASS") { "✅" } elseif ($Status -eq "WARN") { "⚠️ " } else { "❌" }
    $results += [PSCustomObject]@{ Test = $Name; Status = $symbol + " " + $Status; Detail = $Detail }
    Write-Host "$symbol [$Status] $Name — $Detail"
}

$backendUrl = "http://localhost:3000"
$groqKey    = $env:GROQ_API_KEY
$orKey      = $env:OPENROUTER_API_KEY

Write-Host ""
Write-Host "========================================================"
Write-Host " ALLBRIGHT LIVE ENDPOINT VERIFICATION"
Write-Host "========================================================"
Write-Host ""

# ── 1. ENV KEY CHECKS ────────────────────────────────────────────
Write-Host "── ENV KEYS ──"

if ($groqKey -and $groqKey.Length -gt 10) {
    Test-Result "GROQ_API_KEY" "PASS" "Set (gsk_...${groqKey.Substring($groqKey.Length-6)})"
} else {
    Test-Result "GROQ_API_KEY" "FAIL" "NOT SET or empty"
}

if ($orKey -and $orKey.Length -gt 10) {
    Test-Result "OPENROUTER_API_KEY" "PASS" "Set (sk-or-...${orKey.Substring($orKey.Length-6)})"
} else {
    Test-Result "OPENROUTER_API_KEY" "FAIL" "NOT SET or empty"
}

Write-Host ""

# ── 2. BACKEND HEALTH ────────────────────────────────────────────
Write-Host "── BACKEND ──"
try {
    $health = Invoke-RestMethod -Uri "$backendUrl/healthz" -Method GET -TimeoutSec 5
    Test-Result "Backend /healthz" "PASS" "Response: $health"
} catch {
    Test-Result "Backend /healthz" "FAIL" $_.Exception.Message
}

try {
    $ready = Invoke-RestMethod -Uri "$backendUrl/readyz" -Method GET -TimeoutSec 5
    Test-Result "Backend /readyz" "PASS" "Response: $ready"
} catch {
    Test-Result "Backend /readyz" "FAIL" $_.Exception.Message
}

Write-Host ""

# ── 3. GROQ AI ENDPOINT ─────────────────────────────────────────
Write-Host "── GROQ AI (via Backend /api/ai/ask) ──"
try {
    $body = @{
        provider      = "groq"
        system_prompt = "You are a DeFi trading assistant for AllBright V91."
        user_prompt   = "Reply with exactly: GROQ_LIVE_OK"
    } | ConvertTo-Json

    $groqResp = Invoke-RestMethod -Uri "$backendUrl/api/ai/ask" `
        -Method POST `
        -ContentType "application/json" `
        -Body $body `
        -TimeoutSec 30

    if ($groqResp.response) {
        Test-Result "Groq /api/ai/ask" "PASS" "Response: $($groqResp.response.Substring(0, [Math]::Min(80, $groqResp.response.Length)))..."
        Test-Result "Groq Provider Used" "PASS" "provider_used=$($groqResp.provider_used)"
    } else {
        Test-Result "Groq /api/ai/ask" "WARN" "Empty response body"
    }
} catch {
    Test-Result "Groq /api/ai/ask" "FAIL" $_.Exception.Message
}

Write-Host ""

# ── 4. OPENROUTER AI ENDPOINT ────────────────────────────────────
Write-Host "── OPENROUTER AI (via Backend /api/ai/ask) ──"
try {
    $body2 = @{
        provider      = "openrouter"
        system_prompt = "You are a DeFi trading assistant for AllBright V91."
        user_prompt   = "Reply with exactly: OPENROUTER_LIVE_OK"
    } | ConvertTo-Json

    $orResp = Invoke-RestMethod -Uri "$backendUrl/api/ai/ask" `
        -Method POST `
        -ContentType "application/json" `
        -Body $body2 `
        -TimeoutSec 30

    if ($orResp.response) {
        Test-Result "OpenRouter /api/ai/ask" "PASS" "Response: $($orResp.response.Substring(0, [Math]::Min(80, $orResp.response.Length)))..."
        Test-Result "OpenRouter Provider Used" "PASS" "provider_used=$($orResp.provider_used)"
    } else {
        Test-Result "OpenRouter /api/ai/ask" "WARN" "Empty response body"
    }
} catch {
    Test-Result "OpenRouter /api/ai/ask" "FAIL" $_.Exception.Message
}

Write-Host ""

# ── 5. DIRECT GROQ API (bypass backend) ─────────────────────────
Write-Host "── GROQ DIRECT API (https://api.groq.com) ──"
try {
    $groqHeaders = @{
        "Authorization" = "Bearer $groqKey"
        "Content-Type"  = "application/json"
    }
    $groqBody = @{
        model    = "llama-3.3-70b-versatile"
        messages = @(
            @{ role = "user"; content = "Reply with exactly: DIRECT_GROQ_OK" }
        )
        max_tokens = 10
    } | ConvertTo-Json -Depth 5

    $directGroq = Invoke-RestMethod -Uri "https://api.groq.com/openai/v1/chat/completions" `
        -Method POST `
        -Headers $groqHeaders `
        -Body $groqBody `
        -TimeoutSec 20

    $msg = $directGroq.choices[0].message.content
    Test-Result "Groq Direct API" "PASS" "Model: $($directGroq.model) | Reply: $msg"
} catch {
    Test-Result "Groq Direct API" "FAIL" $_.Exception.Message
}

Write-Host ""

# ── 6. DIRECT OPENROUTER API ────────────────────────────────────
Write-Host "── OPENROUTER DIRECT API (https://openrouter.ai) ──"
try {
    $orHeaders = @{
        "Authorization" = "Bearer $orKey"
        "Content-Type"  = "application/json"
    }
    $orBody = @{
        model    = "deepseek/deepseek-chat-v3-0324:free"
        messages = @(
            @{ role = "user"; content = "Reply with exactly: DIRECT_OPENROUTER_OK" }
        )
        max_tokens = 10
    } | ConvertTo-Json -Depth 5

    $directOR = Invoke-RestMethod -Uri "https://openrouter.ai/api/v1/chat/completions" `
        -Method POST `
        -Headers $orHeaders `
        -Body $orBody `
        -TimeoutSec 30

    $orMsg = $directOR.choices[0].message.content
    Test-Result "OpenRouter Direct API" "PASS" "Model: $($directOR.model) | Reply: $orMsg"
} catch {
    Test-Result "OpenRouter Direct API" "FAIL" $_.Exception.Message
}

Write-Host ""

# ── 7. ENGINE MODE ENV CHECK ─────────────────────────────────────
Write-Host "── ENGINE MODE CONFIGURATION ──"
$engineMode = $env:VITE_ENGINE_MODE
if ($engineMode) {
    Test-Result "VITE_ENGINE_MODE" "PASS" "Mode = $engineMode"
} else {
    Test-Result "VITE_ENGINE_MODE" "WARN" "Not set (defaults to simulation)"
}

$backendApiUrl = $env:VITE_BACKEND_API_URL
if ($backendApiUrl) {
    Test-Result "VITE_BACKEND_API_URL" "PASS" "URL = $backendApiUrl"
} else {
    Test-Result "VITE_BACKEND_API_URL" "WARN" "Not set"
}

Write-Host ""

# ── SUMMARY ─────────────────────────────────────────────────────
Write-Host "========================================================"
Write-Host " VERIFICATION SUMMARY"
Write-Host "========================================================"
$results | Format-Table -AutoSize

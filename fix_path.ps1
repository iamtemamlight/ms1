$m = [Environment]::GetEnvironmentVariable('PATH', 'Machine')
[Environment]::SetEnvironmentVariable('PATH', $m, 'User')
$env:PATH = $m
"FIXED: User PATH reset to Machine PATH. New PATH length = $($env:PATH.Length)"
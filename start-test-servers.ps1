# Starts a few test servers so there is something to kill.
$ErrorActionPreference = 'Stop'

cargo build --quiet --example test-server
$exe = Join-Path $PSScriptRoot 'target\debug\examples\test-server.exe'

$servers = @{ 3000 = 'Server-1'; 3001 = 'Server-2'; 3002 = 'Server-3'; 8080 = 'API-Server' }
foreach ($port in $servers.Keys | Sort-Object) {
    Start-Process -NoNewWindow $exe -ArgumentList $port, $servers[$port]
}
Start-Sleep -Milliseconds 500

Write-Host ""
Write-Host "Try it with:  cargo run -- 3000" -ForegroundColor Yellow
Write-Host "Stop all:     .\stop-test-servers.ps1" -ForegroundColor Yellow

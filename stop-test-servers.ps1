# Stops all servers started by start-test-servers.ps1.
$servers = Get-Process test-server -ErrorAction SilentlyContinue
if (-not $servers) {
    Write-Host "No test servers running." -ForegroundColor Yellow
    return
}
$servers | Stop-Process -Force
Write-Host "Stopped $(@($servers).Count) test server(s)." -ForegroundColor Green

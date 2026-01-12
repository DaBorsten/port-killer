# Stoppe alle bun Prozesse die test-server.ts ausfuehren
Write-Host "Stoppe alle Test-Server..." -ForegroundColor Red
Write-Host ""

# Finde alle bun Prozesse
$bunProcesses = Get-Process bun -ErrorAction SilentlyContinue

if ($bunProcesses) {
    $killed = 0
    $bunProcesses | ForEach-Object {
        try {
            Write-Host "   Stoppe Prozess $($_.Id) ($($_.ProcessName))..." -ForegroundColor Gray
            Stop-Process -Id $_.Id -Force -ErrorAction Stop
            $killed++
        } catch {
            Write-Host "   Konnte Prozess $($_.Id) nicht stoppen" -ForegroundColor Yellow
        }
    }
    Write-Host ""
    if ($killed -gt 0) {
        Write-Host "$killed Prozess(e) gestoppt!" -ForegroundColor Green
    }
} else {
    Write-Host "Keine Bun-Prozesse gefunden" -ForegroundColor Yellow
}



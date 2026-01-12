# Starte mehrere Test-Server auf verschiedenen Ports
Write-Host "🚀 Starte Test-Server..." -ForegroundColor Cyan
Write-Host ""

# Starte Server auf verschiedenen Ports
Write-Host "   Server-1 auf Port 3000..." -ForegroundColor Gray
Start-Process -NoNewWindow bun -ArgumentList "test-server.ts", "3000", "Server-1"
Start-Sleep -Seconds 1

Write-Host "   Server-2 auf Port 3001..." -ForegroundColor Gray
Start-Process -NoNewWindow bun -ArgumentList "test-server.ts", "3001", "Server-2"
Start-Sleep -Seconds 1

Write-Host "   Server-3 auf Port 3002..." -ForegroundColor Gray
Start-Process -NoNewWindow bun -ArgumentList "test-server.ts", "3002", "Server-3"
Start-Sleep -Seconds 1

Write-Host "   API-Server auf Port 8080..." -ForegroundColor Gray
Start-Process -NoNewWindow bun -ArgumentList "test-server.ts", "8080", "API-Server"
Start-Sleep -Seconds 1

Write-Host ""
Write-Host "✅ Test-Server gestartet!" -ForegroundColor Green
Write-Host ""
Write-Host "Verfügbare Ports zum Testen:" -ForegroundColor Yellow
Write-Host "  • 3000 - Server-1" -ForegroundColor White
Write-Host "  • 3001 - Server-2" -ForegroundColor White
Write-Host "  • 3002 - Server-3" -ForegroundColor White
Write-Host "  • 8080 - API-Server" -ForegroundColor White
Write-Host ""
Write-Host "Starte Port Killer mit:" -ForegroundColor Yellow
Write-Host "  bun start" -ForegroundColor Cyan
Write-Host ""
Write-Host "Zum Beenden aller Server:" -ForegroundColor Yellow
Write-Host "  .\stop-test-servers.ps1" -ForegroundColor White

Write-Host "🚀 Initializing Mainframe Modernization Pipeline..." -ForegroundColor Cyan

# 1. Spin up the containers in the background (-d)
Write-Host "📦 Building and Starting Containers..." -ForegroundColor Yellow
docker compose up --build -d

# 2. Wait for the Agents to be 'Healthy'
Write-Host "⏳ Waiting for Green Agent to wake up..." -ForegroundColor Yellow
while (!(Test-NetConnection -ComputerName 127.0.0.1 -Port 8080 -WarningAction SilentlyContinue).TcpTestSucceeded) {
    Start-Sleep -Seconds 2
}

Write-Host "✅ Agents are Online!" -ForegroundColor Green

# 3. Trigger the Demo
Write-Host "📡 Injecting Modernization Task..." -ForegroundColor Cyan
.\demo.ps1

# 4. Show the Logs (Follow mode)
Write-Host "📋 Attaching to Logs (Press Ctrl+C to exit)..." -ForegroundColor Gray
docker compose logs -f green-agent
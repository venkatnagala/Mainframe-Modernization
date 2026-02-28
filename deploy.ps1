# ============================================================
# Mainframe Modernization Pipeline - Kubernetes Deployment
# Author: Venkat Nagala
# Usage: .\deploy.ps1
# ============================================================

Write-Host "🚀 Mainframe Modernization Pipeline - Kubernetes Deployment" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

# ============================================================
# STEP 1: Verify Prerequisites
# ============================================================
Write-Host "`n📋 Step 1: Verifying Prerequisites..." -ForegroundColor Yellow

# Check kubectl
if (!(Get-Command kubectl -ErrorAction SilentlyContinue)) {
    Write-Host "❌ kubectl not found! Please install kubectl first." -ForegroundColor Red
    exit 1
}
Write-Host "✅ kubectl found" -ForegroundColor Green

# Check cluster connection
$nodeStatus = kubectl get nodes --no-headers 2>$null
if (!$nodeStatus) {
    Write-Host "❌ Cannot connect to Kubernetes cluster!" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Kubernetes cluster connected" -ForegroundColor Green

# ============================================================
# STEP 2: Load environment variables from .env
# ============================================================
Write-Host "`n📋 Step 2: Loading environment variables..." -ForegroundColor Yellow

$envFile = ".env"
if (!(Test-Path $envFile)) {
    Write-Host "❌ .env file not found! Please create .env from .env.example" -ForegroundColor Red
    exit 1
}

# Parse .env file
Get-Content $envFile | ForEach-Object {
    if ($_ -match "^\s*([^#][^=]+)=(.*)$") {
        $name = $matches[1].Trim()
        $value = $matches[2].Trim()
        Set-Item -Path "env:$name" -Value $value
    }
}
Write-Host "✅ Environment variables loaded" -ForegroundColor Green

# ============================================================
# STEP 3: Apply Namespace and RBAC
# ============================================================
Write-Host "`n📋 Step 3: Creating Namespace and RBAC..." -ForegroundColor Yellow
kubectl apply -f k8s/base/00-namespace-rbac.yaml
if ($LASTEXITCODE -ne 0) { Write-Host "❌ Failed to apply namespace/RBAC!" -ForegroundColor Red; exit 1 }
Write-Host "✅ Namespace and RBAC created" -ForegroundColor Green

# ============================================================
# STEP 4: Create Secrets with real values from .env
# ============================================================
Write-Host "`n📋 Step 4: Creating Kubernetes Secrets..." -ForegroundColor Yellow

# Delete existing secrets (ignore errors if not exist)
kubectl delete secret gateway-jwt-secret -n mainframe-modernization 2>$null
kubectl delete secret green-agent-credentials -n mainframe-modernization 2>$null
kubectl delete secret purple-agent-credentials -n mainframe-modernization 2>$null
kubectl delete secret ai-mcp-credentials -n mainframe-modernization 2>$null
kubectl delete secret s3-mcp-credentials -n mainframe-modernization 2>$null

# Create secrets with real values
kubectl create secret generic gateway-jwt-secret `
    --from-literal=jwt-secret="$env:JWT_SECRET" `
    -n mainframe-modernization

kubectl create secret generic green-agent-credentials `
    --from-literal=api-key="$env:AGENT_API_KEY" `
    --from-literal=aws-access-key-id="$env:AWS_ACCESS_KEY_ID" `
    --from-literal=aws-secret-access-key="$env:AWS_SECRET_ACCESS_KEY" `
    --from-literal=aws-region="us-east-1" `
    -n mainframe-modernization

kubectl create secret generic purple-agent-credentials `
    --from-literal=api-key="purple-agent-dev-key-change-in-prod" `
    --from-literal=claude-api-key="$env:CLAUDE_API_KEY" `
    -n mainframe-modernization

kubectl create secret generic ai-mcp-credentials `
    --from-literal=claude-api-key="$env:CLAUDE_API_KEY" `
    -n mainframe-modernization

kubectl create secret generic s3-mcp-credentials `
    --from-literal=aws-access-key-id="$env:AWS_ACCESS_KEY_ID" `
    --from-literal=aws-secret-access-key="$env:AWS_SECRET_ACCESS_KEY" `
    --from-literal=aws-region="us-east-1" `
    -n mainframe-modernization

if ($LASTEXITCODE -ne 0) { Write-Host "❌ Failed to create secrets!" -ForegroundColor Red; exit 1 }
Write-Host "✅ Secrets created" -ForegroundColor Green

# ============================================================
# STEP 5: Apply ConfigMap and Secrets YAML
# ============================================================
Write-Host "`n📋 Step 5: Applying ConfigMap..." -ForegroundColor Yellow
kubectl apply -f k8s/base/01-secrets-config.yaml
Write-Host "✅ ConfigMap applied" -ForegroundColor Green

# ============================================================
# STEP 6: Deploy Agent Gateway
# ============================================================
Write-Host "`n📋 Step 6: Deploying Agent Gateway..." -ForegroundColor Yellow
kubectl apply -f k8s/base/02-agent-gateway.yaml
if ($LASTEXITCODE -ne 0) { Write-Host "❌ Failed to deploy agent-gateway!" -ForegroundColor Red; exit 1 }
Write-Host "✅ Agent Gateway deployed" -ForegroundColor Green

# Wait for agent-gateway to be ready
Write-Host "⏳ Waiting for Agent Gateway to be ready..." -ForegroundColor Yellow
kubectl rollout status deployment/agent-gateway -n mainframe-modernization --timeout=120s
Write-Host "✅ Agent Gateway is ready!" -ForegroundColor Green

# ============================================================
# STEP 7: Deploy MCP Servers
# ============================================================
Write-Host "`n📋 Step 7: Deploying MCP Servers..." -ForegroundColor Yellow
kubectl apply -f k8s/base/05-mcp-servers.yaml
if ($LASTEXITCODE -ne 0) { Write-Host "❌ Failed to deploy MCP servers!" -ForegroundColor Red; exit 1 }
Write-Host "✅ MCP Servers deployed" -ForegroundColor Green

# ============================================================
# STEP 8: Deploy Agents
# ============================================================
Write-Host "`n📋 Step 8: Deploying Agents..." -ForegroundColor Yellow
kubectl apply -f k8s/base/03-agents.yaml
if ($LASTEXITCODE -ne 0) { Write-Host "❌ Failed to deploy agents!" -ForegroundColor Red; exit 1 }
Write-Host "✅ Agents deployed" -ForegroundColor Green

# ============================================================
# STEP 9: Apply Network Policies
# ============================================================
Write-Host "`n📋 Step 9: Applying Zero-Trust Network Policies..." -ForegroundColor Yellow
kubectl apply -f k8s/base/04-network-policy.yaml
if ($LASTEXITCODE -ne 0) { Write-Host "❌ Failed to apply network policies!" -ForegroundColor Red; exit 1 }
Write-Host "✅ Network Policies applied" -ForegroundColor Green

# ============================================================
# STEP 10: Port Forward for local access
# ============================================================
Write-Host "`n📋 Step 10: Waiting for all pods to be ready..." -ForegroundColor Yellow
kubectl rollout status deployment/agent-gateway -n mainframe-modernization --timeout=180s
kubectl rollout status deployment/green-agent -n mainframe-modernization --timeout=180s
kubectl rollout status deployment/purple-agent -n mainframe-modernization --timeout=180s
kubectl rollout status deployment/s3-mcp -n mainframe-modernization --timeout=180s
kubectl rollout status deployment/ai-mcp -n mainframe-modernization --timeout=180s
kubectl rollout status deployment/cobol-mcp -n mainframe-modernization --timeout=180s
kubectl rollout status deployment/rust-mcp -n mainframe-modernization --timeout=180s

# ============================================================
# STEP 11: Show deployment status
# ============================================================
Write-Host "`n============================================================" -ForegroundColor Cyan
Write-Host "🎉 DEPLOYMENT COMPLETE!" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Cyan

Write-Host "`n📊 Pod Status:" -ForegroundColor Yellow
kubectl get pods -n mainframe-modernization

Write-Host "`n🌐 Services:" -ForegroundColor Yellow
kubectl get services -n mainframe-modernization

# ============================================================
# STEP 12: Start Port Forward and Run Demo
# ============================================================
Write-Host "`n📋 Step 12: Starting Port Forward for Green Agent..." -ForegroundColor Yellow

# Kill any existing kubectl port-forward on 8080
$existing = netstat -ano | findstr :8080 | Select-String "LISTENING"
if ($existing) {
    $pid = ($existing -split '\s+')[-1]
    taskkill /PID $pid /F 2>$null
    Write-Host "✅ Cleared existing process on port 8080" -ForegroundColor Green
}

# Start port-forward in background
$portForward = Start-Process -FilePath "kubectl" `
    -ArgumentList "port-forward svc/green-agent 8080:8080 -n mainframe-modernization" `
    -PassThru -WindowStyle Hidden

Write-Host "✅ Port forward started (PID: $($portForward.Id))" -ForegroundColor Green

# Give it 3 seconds to establish
Write-Host "⏳ Waiting for port forward to establish..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

# ============================================================
# STEP 13: Run Demo
# ============================================================
Write-Host "`n📋 Step 13: Running Demo Pipeline..." -ForegroundColor Yellow
.\demo.ps1

# ============================================================
# STEP 14: Cleanup port forward
# ============================================================
Write-Host "`n📋 Step 14: Cleaning up port forward..." -ForegroundColor Yellow
Stop-Process -Id $portForward.Id -Force 2>$null
Write-Host "✅ Port forward stopped" -ForegroundColor Green

Write-Host "`n✅ Mainframe Modernization Pipeline is running on Kubernetes!" -ForegroundColor Green
Write-Host "`n💡 To run again manually:" -ForegroundColor Cyan
Write-Host "   kubectl port-forward svc/green-agent 8080:8080 -n mainframe-modernization" -ForegroundColor White
Write-Host "   .\demo.ps1" -ForegroundColor White
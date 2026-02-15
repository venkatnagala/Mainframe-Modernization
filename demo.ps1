# 1. Configuration
$TaskID = "MODERN-DEMO-2026"
$S3Bucket = "mainframe-refactor-lab-venkatnagala"  # <--- UPDATE THIS
$S3Key = "programs/interest_calc.cbl"

# 2. Build the Payload
$Payload = @{
    task_id = $TaskID
    source_location = @{
        bucket = $S3Bucket
        key = $S3Key
    }
} | ConvertTo-Json

# 3. Trigger the Green Agent
Write-Host "📡 Sending Modernization Task to Green Agent..." -ForegroundColor Cyan
try {
    $Response = # Change this line:
Invoke-RestMethod -Uri "http://127.0.0.1:8080/evaluate" -Method Post -Body $Payload -ContentType "application/json"
    Write-Host "✅ Task accepted! Monitor your Docker logs for the ASCII Verification report." -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to reach Green Agent. Is 'docker-compose up' running?" -ForegroundColor Red
}
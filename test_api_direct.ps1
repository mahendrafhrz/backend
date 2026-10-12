# Test POST langsung ke API (bypass MQTT)

$body = @{
    sample_id = 99
    score = 95.5
    accuracy = 94.2
    predicted_class = "Test Coffee"
    confidence = 0.99
    grade = "high_grade"
    device_id = "TEST-API"
    captured_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
} | ConvertTo-Json

Write-Host "Sending test measurement via API POST..."
Write-Host "URL: https://enose-dashboard-app.azurewebsites.net/api/v1/measurements"
Write-Host "Payload: $body"

try {
    $response = Invoke-RestMethod -Uri "https://enose-dashboard-app.azurewebsites.net/api/v1/measurements" -Method POST -Body $body -ContentType "application/json"
    Write-Host "`n✅ SUCCESS! Response:" -ForegroundColor Green
    $response | ConvertTo-Json
    
    Write-Host "`nChecking total measurements..."
    $summary = Invoke-RestMethod -Uri "https://enose-dashboard-app.azurewebsites.net/api/v1/analytics/summary"
    Write-Host "Total measurements: $($summary.total_measurements)" -ForegroundColor Cyan
    
    if ($summary.total_measurements -gt 0) {
        Write-Host "🎉 Data berhasil masuk ke database!" -ForegroundColor Green
        Write-Host "Cek dashboard: https://enose-dashboard-app.azurewebsites.net/dashboard"
    }
} catch {
    Write-Host "`n❌ ERROR: $($_.Exception.Message)" -ForegroundColor Red
}

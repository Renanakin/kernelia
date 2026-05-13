$ErrorActionPreference = 'Stop'

Write-Host '[1/5] Levantando infraestructura...' -ForegroundColor Cyan
docker compose -f docker-compose-ollama.yml up -d

Write-Host '[2/5] Esperando proxy en 11435...' -ForegroundColor Cyan
$ok = $false
for ($i = 0; $i -lt 20; $i++) {
  try {
    $health = Invoke-WebRequest -Uri 'http://localhost:11435/health' -UseBasicParsing -TimeoutSec 3
    if ($health.StatusCode -eq 200) { $ok = $true; break }
  } catch {}
  Start-Sleep -Seconds 2
}
if (-not $ok) { throw 'Proxy 11435 no disponible' }

Write-Host '[3/5] Verificando modelos...' -ForegroundColor Cyan
Invoke-WebRequest -Uri 'http://localhost:11435/v1/models' -UseBasicParsing | Select-Object -ExpandProperty Content | Write-Host

Write-Host '[4/5] Enviando prompt de prueba...' -ForegroundColor Cyan
$body = @{ model='gemma3'; messages=@(@{ role='user'; content='Responde solo: OK REAL' }) } | ConvertTo-Json -Depth 6
Invoke-WebRequest -Uri 'http://localhost:11435/v1/chat/completions' -Method POST -ContentType 'application/json' -Body $body -UseBasicParsing | Select-Object -ExpandProperty Content | Write-Host

Write-Host '[5/5] Listo. App debe usar /api/local-chat -> localhost:11435/v1' -ForegroundColor Green

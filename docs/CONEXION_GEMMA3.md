# Conectar Gemma3 a KernelIA (Desktop App)

## Estado Actual
- **App**: Kernelia (Tauri + Svelte) corre en tu PC como aplicación de escritorio
- **Modelo**: Gemma3 está en Docker Desktop
- **Puerto Configurado**: `http://localhost:11435/v1` (ya definido en el código Rust)

## Arquitectura de Conexión

```
KernelIA (Tauri/Rust Backend)  
    ↓ (HTTP POST)
    → localhost:11435/v1/chat/completions
            ↓
    Proxy OpenAI Compatible (FastAPI)
    puerto 11435
            ↓
    Ollama en Docker Desktop
    puerto 11434
            ↓
    Modelo: gemma3
```

## Pasos para Conectar

### 1. Iniciar Ollama en Docker Desktop

```powershell
# En PowerShell o CMD:
docker run -d `
  --name kernelia-ollama `
  -p 11434:11434 `
  -v ollama_data:/root/.ollama `
  -e OLLAMA_HOST=0.0.0.0:11434 `
  ollama/ollama
```

**Verificar que está corriendo:**
```powershell
docker ps | findstr ollama
# O visita: http://localhost:11434/api/tags
```

### 2. Descargar Gemma3 en Ollama

```powershell
docker exec kernelia-ollama ollama pull gemma:2b
```

Espera a que termine (~5-10 minutos según tu conexión).

**Verificar descarga:**
```powershell
docker exec kernelia-ollama ollama list
# Deberías ver: gemma:2b
```

### 3. Iniciar el Proxy OpenAI (FastAPI)

El proxy convierte Ollama a API compatible con OpenAI.

**Opción A: Dentro de Docker (recomendado)**

```powershell
cd G:\DESARROLLOS\kernelia

docker run -d `
  --name kernelia-proxy `
  -p 11435:11435 `
  --link kernelia-ollama `
  -v ${PWD}:/app `
  -e OLLAMA_BASE_URL=http://kernelia-ollama:11434 `
  -e OLLAMA_MODEL=gemma3 `
  python:3.11-slim `
  sh -c "pip install --no-cache-dir fastapi uvicorn httpx ; uvicorn ollama_proxy:app --host 0.0.0.0 --port 11435"
```

**Opción B: En tu host (si Python está instalado)**

```powershell
cd G:\DESARROLLOS\kernelia
pip install fastapi uvicorn httpx
set OLLAMA_BASE_URL=http://localhost:11434
set OLLAMA_MODEL=gemma3
uvicorn ollama_proxy:app --host 0.0.0.0 --port 11435
```

**Verificar que funciona:**
```powershell
# En otra PowerShell:
curl http://localhost:11435/health
# Deberías ver JSON con "status": "ok"
```

### 4. Inicia KernelIA

Abre la aplicación desktop Kernelia. Debería detectar automáticamente Gemma3 en localhost:11435.

---

## Test de Conectividad

**Desde PowerShell:**

```powershell
# Test 1: Proxy accesible
curl http://localhost:11435/health

# Test 2: Ver modelos
curl http://localhost:11435/v1/models

# Test 3: Chat simple
$body = @{
    model = "gemma3"
    messages = @(@{role = "user"; content = "Hola"})
    stream = $false
} | ConvertTo-Json

curl -Method POST `
  -Uri http://localhost:11435/v1/chat/completions `
  -Body $body `
  -ContentType "application/json"
```

---

## Troubleshooting

### Error: "Connection refused" en port 11435
- Verifica que el proxy está corriendo: `docker ps | findstr proxy`
- Si no, relanza: `docker run -d --name kernelia-proxy ...` (ver arriba)

### Error: "Cannot connect to Ollama"
- Verifica Ollama: `docker ps | findstr ollama`
- Verifica que gemma3 está descargado: `docker exec kernelia-ollama ollama list`

### KernelIA no detecta el modelo
- Abre Settings en la app → Busca "Ollama" o "localhost:11435"
- Si está en blanco, ingresa: `http://localhost:11435/v1`
- Reinicia la app

### Lento o freezing
- Verifica RAM disponible (Gemma3 usa ~4-6GB)
- Reduce el contexto en settings si es necesario

---

## Parar los Contenedores

```powershell
docker stop kernelia-ollama kernelia-proxy
docker rm kernelia-ollama kernelia-proxy
docker volume rm ollama_data  # Solo si quieres borrar modelos
```

---

## Variables de Entorno (Opcional)

En `G:\DESARROLLOS\kernelia\.env`:

```
KERNELIA_GEMMA3_BASE_URL=http://localhost:11435/v1
KERNELIA_RAG_ENABLED=true
KERNELIA_RAG_DEBUG_PANEL=false
```

Luego reinicia KernelIA para que cargue.

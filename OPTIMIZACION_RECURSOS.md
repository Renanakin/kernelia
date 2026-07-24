# Optimización de Recursos - Gemma 3 en Docker

## Cambios Realizados

Tu `docker-compose-ollama.yml` fue optimizado de la siguiente forma:

### 1. **Ollama Container**

#### Reduce paralelismo:
```yaml
OLLAMA_NUM_PARALLEL=1          # Antes: 2 (ahora: 1)
```
- Solo procesa 1 request a la vez en lugar de 2
- Reduce picos de memoria significativamente

#### Keep-alive más corto:
```yaml
OLLAMA_KEEP_ALIVE=1m            # Antes: 5m (ahora: 1m)
```
- Descarga el modelo de memoria después de 1 minuto sin usar
- Libera ~600MB si no hay actividad

#### Limites de CPU y RAM:
```yaml
deploy:
  resources:
    limits:
      cpus: '2'                 # Máximo 2 CPUs
      memory: 2G                # Máximo 2GB RAM
```
- Evita que Ollama consuma todos los recursos disponibles

---

### 2. **Proxy OpenAI Container**

#### Un solo worker:
```yaml
command: ... uvicorn ollama_proxy:app --workers 1
```
- Reduce procesos paralelos del proxy

#### Parámetros reducidos:
```yaml
OLLAMA_DEFAULT_MAX_TOKENS=300      # Antes: 500
OLLAMA_MAX_TOKENS_CAP=800          # Antes: 1200
OLLAMA_NUM_THREAD=4                # Antes: 6
OLLAMA_NUM_CTX=2048                # Antes: 4096
OLLAMA_PARALLELISM=1               # Antes: 2
OLLAMA_KEEP_ALIVE=1m               # Antes: 5m
```

#### Limites de proxy:
```yaml
deploy:
  resources:
    limits:
      cpus: '1'                     # Máximo 1 CPU
      memory: 512M                  # Máximo 512MB
```

---

## Impacto en Rendimiento

| Métrica | Antes | Después | Cambio |
|---------|-------|---------|--------|
| Memoria Ollama (en reposo) | ~600MB | ~100-200MB | -70% |
| CPU max | 800% | 200% | -75% |
| Contexto max | 4096 tokens | 2048 tokens | -50% |
| Tokens por request | 500 | 300 | -40% |

---

## Recomendaciones por Caso de Uso

### Uso Mínimo (Mi recomendación - actual):
✅ Actual - Bajo consumo, respuestas rápidas, adecuado para desktop

### Uso Medio (Si necesitas más contexto):
```yaml
OLLAMA_NUM_CTX=3072
OLLAMA_DEFAULT_MAX_TOKENS=400
OLLAMA_NUM_PARALLEL=2
```

### Uso Máximo (Para servidor dedicado):
```yaml
OLLAMA_NUM_CTX=4096
OLLAMA_DEFAULT_MAX_TOKENS=1000
OLLAMA_NUM_PARALLEL=4
deploy:
  resources:
    limits:
      memory: 8G
      cpus: '4'
```

---

## Cómo Aplicar Cambios

1. **Modificar** `docker-compose-ollama.yml` (ya hecho ✅)

2. **Reiniciar contenedores:**
```powershell
docker-compose -f docker-compose-ollama.yml down
docker-compose -f docker-compose-ollama.yml up -d
```

3. **Verificar consumo en Docker Desktop:**
   - Abre Dashboard
   - Observa la memoria de `kernelia-ollama`
   - Debe estar ~200-600MB en uso

---

## Si Quieres Optimizar Más

### Opción 1: Usar modelo más pequeño
```yaml
# En lugar de gemma:2b, usar:
OLLAMA_MODEL=gemma:2b-instruct-q4_0  # Cuantizado (más pequeño)
```

### Opción 2: Reducir contexto aún más
```yaml
OLLAMA_NUM_CTX=1024  # Muy bajo, respuestas cortas solamente
```

### Opción 3: Keep-alive = 0 (descargar inmediatamente)
```yaml
OLLAMA_KEEP_ALIVE=0   # CUIDADO: más lento en primer request
```

---

## Monitoreo Recomendado

Verifica periódicamente con:
```powershell
docker stats --no-stream kernelia-ollama kernelia-openai-proxy
```

Deberías ver:
- **Ollama**: 100-600MB (depende de carga)
- **Proxy**: 50-100MB
- **CPU**: Spike cuando procesa, luego 0%

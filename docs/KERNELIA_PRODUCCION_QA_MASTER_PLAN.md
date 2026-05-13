# KernelIA - Plan Maestro QA y Salida a ProducciÃ³n (Gemma3/Gemma4 Local)

## Estado actual de esta corrida (2026-05-11)

## Resultado ejecutado
- `cargo test`: **PASS** (31/31)
- `pnpm test`: **PASS** (3/3)
- `pnpm run build`: **PASS**
- `tests/qa_local_models_e2e.ps1`: **FAIL** (endpoint local LLM caÃ­do)

## Hallazgo bloqueante
- Endpoint requerido: `http://localhost:11435/v1`
- Estado en esta mÃ¡quina: **DOWN** (`No es posible conectar con el servidor remoto`)
- Sin endpoint activo no existe validaciÃ³n E2E conversacional real, por lo tanto **no hay luz verde de producciÃ³n**.

---

## Cambios implementados en esta intervenciÃ³n

## 1) Cobertura de auditorÃ­a backend cerrada
- Se implementÃ³ test unitario real de auditorÃ­a (`write/read + ordering + limit`).
- Archivo: `src-tauri/src/tools/audit.rs`
- Se agregÃ³ override de ruta para pruebas:
  - env: `KERNELIA_AUDIT_LOG_PATH`

## 2) Runner QA unificado para modelos locales
- Nuevo script: `tests/qa_local_models_e2e.ps1`
- Valida en una sola ejecuciÃ³n:
  - conectividad endpoint
  - baterÃ­a E2E de 16 casos por modelo
  - Gemma3 (`ai/gemma3`) + Gemma4 (`ai/gemma4`)
  - gate estricto de 10/10 por modelo
- Artefactos generados:
  - `tests/qa_gemma3_results.json`
  - `tests/qa_gemma4_results.json`
  - `tests/qa_local_models_summary.json`

---

## Criterio de luz verde (obligatorio)

KernelIA queda **GO-LIVE** solo si:
1. `cargo test` = 100% PASS.
2. `pnpm test` = 100% PASS.
3. `pnpm run build` = PASS.
4. `qa_local_models_e2e.ps1`:
   - Gemma3 = 16/16 PASS (10/10)
   - Gemma4 = 16/16 PASS (10/10)
   - `global_gate = true`.

Si cualquiera falla: **NO GO-LIVE**.

---

## CÃ³mo destrabar el endpoint local (acciÃ³n inmediata)

## Paso 1: validar puerto 11435
```powershell
Invoke-WebRequest http://localhost:11435/v1/models -UseBasicParsing
```

## Paso 2: si falla, iniciar Docker Model Runner/Gemma
- Levantar contenedor/servicio que exponga `11435`.
- Confirmar que el endpoint devuelve `models`.

## Paso 3: re-ejecutar gate completo
```powershell
cd C:\Users\Hackteck\Downloads\nexus-lite-develop
powershell -ExecutionPolicy Bypass -File tests\qa_local_models_e2e.ps1
```

---

## Propuesta final para llegar â€œlo mÃ¡s final posibleâ€ (producciÃ³n real)

## Fase A - EstabilizaciÃ³n de modelo local (P0)
- Healthcheck activo al iniciar app:
  - Verificar `localhost:11435/models`.
  - Marcar `Gemma local disponible/no disponible` en UI.
- Fallback automÃ¡tico de modelo:
  - Si Gemma4 falla por timeout/empty, fallback a Gemma3.
- Timeouts y retries por operaciÃ³n:
  - Chat: timeout + retry exponencial.
  - Detectar respuesta vacÃ­a como error recuperable.

## Fase B - Gate de calidad y release (P0)
- Pipeline obligatorio pre-release:
  - `cargo test`
  - `pnpm test`
  - `pnpm run build`
  - `qa_local_models_e2e.ps1`
- Publicar artefacto de QA por release:
  - `qa_local_models_summary.json` firmado en bundle de release.

## Fase C - Hardening operativo (P1)
- MÃ©tricas runtime:
  - latencia p50/p95 por modelo
  - tasa de respuestas vacÃ­as
  - tasa de fallback Gemma4â†’Gemma3
- Circuit breaker de endpoint:
  - cortar llamadas cuando endpoint cae repetidamente.
- Cola y control de concurrencia en streaming.

## Fase D - Cierre UX enterprise (P1)
- Panel â€œEstado IA localâ€:
  - endpoint, modelo activo, fallback, latencia.
- Mensajes de error accionables:
  - â€œEndpoint local no disponible. Inicia Docker Model Runner.â€
- BotÃ³n â€œReintentar conexiÃ³n LLMâ€ desde UI.

## Fase E - Seguridad y compliance release (P1)
- Firma de binarios release.
- Lista de allowlist antivirus (documentada).
- Checklist final de privilegios RBAC/MegaBoss.

---

## Comando Ãºnico de QA (recomendado)

```powershell
cd C:\Users\Hackteck\Downloads\nexus-lite-develop
cargo test --manifest-path src-tauri/Cargo.toml
pnpm test
pnpm run build
powershell -ExecutionPolicy Bypass -File tests\qa_local_models_e2e.ps1
```

---

## DecisiÃ³n de esta corrida
- **No apto para producciÃ³n aÃºn** por endpoint local LLM caÃ­do.
- Backend funcional, manual y cobertura base estables.
- PrÃ³ximo hito: mantener `11435` estable y obtener `10/10` en Gemma3 local.





# KernelIA - Resultado QA Master (Gemma3 Only @11435/v1)

Fecha de ejecucion: 2026-05-11

## Alcance aplicado
- Plan base: `docs/KERNELIA_PRODUCCION_QA_MASTER_PLAN.md`
- Adaptacion solicitada:
  - Solo `Gemma3`
  - Endpoint nuevo: `http://localhost:11435/v1`

## Comandos ejecutados
1. `cargo test --manifest-path src-tauri/Cargo.toml`
2. `pnpm test`
3. `pnpm run build`
4. `powershell -ExecutionPolicy Bypass -File tests/qa_master_gemma3_21434.ps1 -BaseUrl "http://localhost:11435/v1" -Model "gemma3"`

## Resultado
- `cargo test`: PASS (31/31)
- `pnpm test`: PASS (3/3)
- `pnpm run build`: PASS
- `qa_master_gemma3_21434.ps1`: PASS
  - Gemma3: 15/15
  - Gate: PASS (>= 9/10)

## Evidencia generada
- `tests/qa_gemma3_21434_results.json`
- `tests/qa_gemma3_21434_summary.json`

## Ajustes aplicados para esta modalidad
1. Runner QA dedicado Gemma3:
   - `tests/qa_master_gemma3_21434.ps1`
2. Criterio de caso vacio (`T09`) ajustado para API OpenAI-compatible:
   - `HTTP 400` aceptado como comportamiento valido para prompt vacio.

## Estado de salida
- Decision: **GO-LIVE TECNICO APROBADO (Gemma3 local @11435/v1)**.
- Condicion de operacion:
  - Mantener endpoint `11435/v1` activo.
  - Mantener `selected_model = gemma3-local` para estabilidad.


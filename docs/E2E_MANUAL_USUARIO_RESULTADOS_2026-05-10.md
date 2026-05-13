# E2E - Manual de Usuario KernelIA

Fecha: 2026-05-10
Workspace: nexus-lite-develop

## Resumen ejecutivo

- Suite Rust ejecutada: `cargo test` en `src-tauri/`
- Resultado: **29/29 PASS**
- Tiempo suite Rust: ~3.74s (sin contar compilacion previa)
- Suite LLM HTTP (`tests/qa_gemma3_e2e.ps1`): **0/16 PASS** por endpoint no disponible en `http://localhost:21434/engines/llama.cpp/v1`

## Cobertura por funciones completas del manual (`docs/MANUAL_USUARIO_KERNELIA.md`)

### 2.1 Nucleo de conversacion y ejecucion
- Cobertura parcial:
  - `ai::function_calling::tests::shortlist_for_network_query_is_bounded`
  - `ai::function_calling::tests::local_normalization_trims_long_history`
  - `ai::router::tests::detects_network_adapter_question`
  - `ai::router::tests::detects_top_process_question`
  - `ai::intent_engine::tests::*`
- Brecha:
  - No se pudo validar E2E de chat/streaming/modelo cloud-local por endpoint LLM remoto caido.

### 2.2 Telemetria del equipo
- Cobertura:
  - `tools::phase2::tests::health_overview_returns_score`
  - `tools::phase7::tests::latency_probe_returns_metrics`
  - `tools::phase7::tests::performance_kpis_returns_structure`

### 2.3 Diagnostico de red
- Cobertura:
  - `tools::phase6::tests::slowpc_diagnostic_returns_causes` (incluye contexto de rendimiento)
  - `tools::phase7::tests::noc_global_status_returns_sla`
  - `tools::phase3::tests::phase3_smoke_returns_steps_payload` (smoke multi-modulo)
- Nota:
  - Diagnostico de red por LLM en script HTTP no validado por endpoint no disponible.

### 2.4 Mantenimiento operativo
- Cobertura:
  - `tools::phase4::tests::proactive_scheduler_returns_json`
  - `tools::phase9::tests::self_healing_cycle_simulation_works`

### 2.5 Drivers y hardware
- Cobertura:
  - `tools::phase3::tests::driver_issue_counter_handles_plain_text`

### 2.6 Seguridad y cumplimiento local
- Cobertura:
  - `tools::phase6::tests::guardrails_blocks_destructive_patterns`
  - `tools::phase10::tests::controls_verification_returns_flags`

### 2.7 Auditoria y trazabilidad
- Cobertura indirecta:
  - smokes de fases con persistencia de evidencias (phase3/phase9/phase10)
- Brecha:
  - no hay test dedicado que verifique lectura de `audit` extremo a extremo en esta corrida.

### 2.8 Reportes
- Cobertura:
  - `tools::phase5::tests::enterprise_dashboard_returns_kpis`
  - `tools::phase10::tests::go_live_readiness_returns_score`
  - `tools::phase10::tests::phase10_smoke_runs_successfully`

### 2.9 Automatizacion y fases avanzadas
- Cobertura:
  - `tools::phase3::tests::phase3_smoke_returns_steps_payload`
  - `tools::phase4::tests::model_route_returns_rationale`
  - `tools::phase5::tests::support_case_creation_returns_json`
  - `tools::phase5::tests::support_case_deduplicates_open_ticket`
  - `tools::phase8::tests::anomalies_detection_returns_json`
  - `tools::phase8::tests::root_cause_explainer_returns_hypothesis`
  - `tools::phase8::tests::sla_status_returns_structure`
  - `tools::phase9::tests::readiness_returns_structure`
  - `tools::phase10::tests::phase10_smoke_runs_successfully`

## Evidencia de ejecucion

### 1) Suite backend
Comando:

```powershell
cd src-tauri
cargo test
```

Resultado:
- 29 passed; 0 failed; 0 ignored

### 2) Suite HTTP LLM (Gemma3)
Comando:

```powershell
powershell -ExecutionPolicy Bypass -File tests/qa_gemma3_e2e.ps1
```

Resultado:
- 16 failed por conexion a endpoint remoto no disponible

## Conclusiones

- La cobertura funcional backend de las capacidades del manual es alta y estable en esta corrida.
- La validacion E2E conversacional real (chat+streaming+seleccion de modelo con endpoint LLM) quedo bloqueada por indisponibilidad del endpoint local HTTP.

## Recomendacion inmediata

1. Levantar endpoint LLM en `localhost:21434` y repetir `tests/qa_gemma3_e2e.ps1`.
2. Agregar test Rust dedicado para `audit` (lectura/escritura) para cerrar la brecha de 2.7.
3. Mantener `cargo test` como gate obligatorio previo a release candidate.


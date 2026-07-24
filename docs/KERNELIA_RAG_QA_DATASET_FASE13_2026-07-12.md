# KERNELIA RAG QA DATASET FASE 13

Fecha: 2026-07-12

## Casos obligatorios cubiertos

1. Consulta corta ambigua
- `ai::query_analyzer::tests::marks_ambiguous_queries_for_clarification`
- `ai::confidence_engine::tests::lowers_confidence_for_ambiguous_queries`
- `ai::decision_engine::tests::chooses_clarify_for_low_confidence_queries`

2. Comando sensible solicitado por usuario Viewer
- `tools::rbac::tests::denies_sensitive_command_for_viewer_role`

3. Retrieval con chunks conflictivos
- `ai::knowledge_retriever::tests::prefers_exact_dns_chunk_over_conflicting_generic_chunk`

4. Accion R3 sin snapshot
- `ai::decision_engine::tests::escalates_r3_action_without_snapshot`

5. Respuesta con confidence baja
- `ai::decision_engine::tests::chooses_clarify_for_low_confidence_queries`

6. Servicio reiniciado pero no recuperado
- `ai::tool_verifier::tests::marks_service_recovery_as_failed_when_service_does_not_return`

7. Memoria vieja contradiciendo snapshot nuevo
- `ai::live_state_retriever::tests::detects_service_conflict_against_snapshot`
- `ai::memory_engine::tests::stores_conflict_when_memory_is_contradicted_by_live_state`

## Casos de refuerzo

- retrieval semantico DNS con lenguaje abierto;
- conflicto de performance vs snapshot;
- driver code 43;
- policy retrieval para spooler;
- latencia de retrieval sobre corpus curado.

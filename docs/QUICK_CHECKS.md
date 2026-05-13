# Quick Checks (Diagnosticos Rapidos) - Nexus-Lite

## 1. Concepto
Los Quick Checks son diagnósticos preconfigurados que permiten ejecutar capacidades técnicas con un clic y respuesta inmediata.

Tipos disponibles:
1. **DirectTool**: Ejecuta tools backend directamente (latencia baja, sin razonamiento LLM).
2. **LlmPrompt**: Inyecta prompt al asistente para un análisis ejecutivo guiado.

## 2. Catalogo Vigente

### Base operativa
- `system_health`: Salud del sistema (`get_system_info`) - DirectTool.
- `slow_pc`: Diagnóstico IA de lentitud - LlmPrompt.
- `junk_scan`: Escaneo de basura (`analyze_junk`) - DirectTool.
- `network_check`: Chequeo de red (`run_network_diagnostic`) - DirectTool.
- `autostart_audit`: Auditoría de inicio (`analyze_registry`) - DirectTool.

### Fase 2 (Operacion y valor visible)
- `health_score`: Riesgo/anomalías/tendencia (`health_overview`) - DirectTool.
- `automation_plan`: Reglas SI/ENTONCES (`run_automation_cycle`) - DirectTool.
- `asset_inventory`: Inventario endpoint (`scan_asset_inventory`) - DirectTool.
- `incident_tickets`: Tickets IA (`list_incident_tickets`) - DirectTool.
- `ops_docs`: Runbook operacional (`generate_operational_documentation`) - DirectTool.

### Fase 3 (Escala empresarial)
- `multi_tenant`: Consolidado multiempresa (`cloud_multi_tenant_overview`) - DirectTool.
- `multiagent`: Diagnóstico coordinado (`run_multiagent_diagnosis`) - DirectTool.
- `remote_support`: Sesiones remotas (`list_remote_support_sessions`) - DirectTool.
- `rollback_ops`: Snapshots de rollback (`list_rollback_snapshots`) - DirectTool.
- `trusted_exec`: Integridad de artefacto (`verify_release_attestation`) - DirectTool.
- `phase3_smoke`: Validación E2E de Fase 3 (`run_phase3_smoke`) - DirectTool.

### Fase 4 (Autonomia proactiva y multimodelo)
- `proactive_maintenance`: Ciclo preventivo (`run_proactive_maintenance`) - DirectTool.
- `proactive_alerts`: Alertas operativas (`list_proactive_alerts`) - DirectTool.
- `multimodel_route`: Enrutamiento de modelo IA (`recommend_model_route`) - DirectTool.
- `phase4_smoke`: Validación E2E de Fase 4 (`run_phase4_smoke`) - DirectTool.

### Fase 5 (Conectividad cloud y soporte enterprise)
- `cloud_sync`: Sincronización cloud (`upload_cloud_report`) - DirectTool.
- `enterprise_dashboard`: KPIs ejecutivos (`get_enterprise_dashboard`) - DirectTool.
- `enterprise_cases`: Casos de soporte (`list_support_cases`) - DirectTool.
- `phase5_smoke`: Validación E2E de Fase 5 (`run_phase5_smoke`) - DirectTool.

### Fase 6 (Diagnósticos KernelIA y guardrails)
- `kernel_slowpc`: Diagnóstico de PC lenta (`run_kernel_slowpc_diagnostic`) - DirectTool.
- `kernel_network`: Playbook de red (`run_kernel_network_playbook`) - DirectTool.
- `kernel_guardrails`: Validación de bloqueos (`validate_kernel_guardrails`) - DirectTool.
- `kernel_readiness`: Reporte de readiness (`generate_kernelia_readiness_report`) - DirectTool.
- `phase6_smoke`: Validación E2E de Fase 6 (`run_phase6_smoke`) - DirectTool.

### Fase 7 (Rendimiento y latencia)
- `latency_probe`: Probing de latencia (`run_latency_probe`) - DirectTool.
- `tool_benchmark`: Benchmark de tools (`run_tool_benchmark`) - DirectTool.
- `performance_kpis`: KPIs de performance (`get_performance_kpis`) - DirectTool.
- `phase7_smoke`: Validación E2E de Fase 7 (`run_phase7_smoke`) - DirectTool.

### Fase 8 (Fiabilidad y SLA)
- `sla_status`: Estado de SLA (`calculate_sla_status`) - DirectTool.
- `reliability_anomalies`: Anomalías de fiabilidad (`detect_performance_anomalies`) - DirectTool.
- `reliability_report`: Reporte de fiabilidad (`generate_reliability_report`) - DirectTool.
- `phase8_smoke`: Validación E2E de Fase 8 (`run_phase8_smoke`) - DirectTool.

### Fase 9 (Autocuracion y prevencion)
- `self_healing_readiness`: Readiness de autocuracion (`assess_self_healing_readiness`) - DirectTool.
- `self_healing_plan`: Plan de autocuracion (`generate_self_healing_plan`) - DirectTool.
- `self_healing_runs`: Historial autocuracion (`list_self_healing_runs`) - DirectTool.
- `phase9_smoke`: Validación E2E de Fase 9 (`run_phase9_smoke`) - DirectTool.

### Fase 10 (Go-Live AAA y compliance)
- `go_live_readiness`: Readiness go-live (`assess_go_live_readiness`) - DirectTool.
- `go_live_controls`: Controles de hardening (`verify_go_live_controls`) - DirectTool.
- `go_live_bundle`: Evidencias ejecutivas (`generate_go_live_bundle`) - DirectTool.
- `phase10_smoke`: Validación E2E de Fase 10 (`run_phase10_smoke`) - DirectTool.

## 3. Seguridad y permisos
- Todos los checks pasan por validación RBAC en backend.
- Los checks de tools owner-only requieren desbloqueo MegaBoss cuando corresponde.
- El frontend solo muestra checks permitidos por rol activo.

## 4. Beneficios
- **Accesibilidad**: Diagnósticos complejos en un clic.
- **Eficiencia**: Menor latencia y menor consumo de tokens cuando aplica DirectTool.
- **Governanza**: Auditoría y permisos consistentes por rol.

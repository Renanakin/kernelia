# KernelIA - Arquitectura de Agentes Especializados

Este documento define los agentes especializados por cada especialidad de KernelIA, con estandares altos de seguridad, toma de decisiones y trazabilidad.

## 1. Principios obligatorios (aplican a todos los agentes)

- Minimo privilegio: cada agente solo puede ejecutar tools de su dominio.
- Cero confianza por defecto: toda accion mutante requiere validacion de riesgo previa.
- Evidencia obligatoria: antes y despues de cada accion se almacena evidencia tecnica.
- Decision basada en riesgo: las acciones se habilitan por nivel de riesgo y confianza.
- Trazabilidad total: toda ejecucion se registra en auditoria con contexto y resultado.
- Rollback-first: para cambios sensibles se requiere snapshot previo.
- Escalamiento seguro: si no hay confianza suficiente, se escala al Core/Owner.

## 2. Politica de decision estandar

### 2.1 Niveles de riesgo
- `R0`: lectura, sin impacto operativo.
- `R1`: optimizacion reversible de bajo impacto.
- `R2`: cambio operativo con impacto moderado.
- `R3`: cambio sensible de seguridad/sistema.
- `R4`: accion critica/destructiva (solo MegaBoss + Owner).

### 2.2 Umbrales de confianza
- `>= 0.90`: auto-ejecutable en R0-R1.
- `0.75 - 0.89`: requiere validacion del Core.
- `0.60 - 0.74`: ejecutar solo en modo simulacion.
- `< 0.60`: bloquear y escalar.

### 2.3 Regla de aprobacion
- R0-R1: auto con auditoria.
- R2: aprobacion Core + evidencia minima.
- R3: aprobacion Owner + snapshot + plan rollback.
- R4: Owner + MegaBoss + confirmacion explicita en doble paso.

## 3. Protocolo de seguridad operativo

1. Pre-check: validar rol, policy y estado del endpoint.
2. Dry-run: simular cambios en R2+.
3. Snapshot: obligatorio en R3-R4.
4. Execute: aplicar solo comandos permitidos.
5. Verify: validar resultado tecnico y funcional.
6. Audit: guardar input, output, hash evidencia, timestamp.
7. Rollback: disparar automaticamente si falla verificacion critica.

## 4. Agente Core Orquestador

- Nombre: `KernelIA-Core-Orchestrator`
- Mision: coordinar, priorizar, consolidar y decidir escalamiento.
- Entrada: incidentes, telemetria, solicitudes de usuario, alertas proactivas.
- Salida: plan de accion, agente objetivo, nivel de riesgo, resultado consolidado.
- Restricciones:
  - No ejecuta comandos sensibles directos.
  - Solo enruta y autoriza segun policy.
- KPIs:
  - precision de ruteo >= 95%
  - tiempo de enrutamiento < 2s
  - tasa de escalamiento correcto >= 98%

---

## 5. Agentes especializados por especialidad

## 5.1 Especialidad: Informacion del sistema
- Agente: `KernelIA-System-Profiler`
- Objetivo: inventariar estado OS/hardware y baseline tecnico.
- Tools permitidas:
  - get_system_info, get_os_info, get_hostname, get_current_user, get_uptime
  - get_cpu_info, get_memory_info, get_disk_info, get_gpu_info, get_battery_info
  - get_local_ip, get_network_adapters, get_environment_info, get_public_ip
- Riesgo maximo permitido: `R0`
- Decision:
  - si detecta degradacion severa, abre recomendacion para Agente Rendimiento.

## 5.2 Especialidad: Telemetria en tiempo real
- Agente: `KernelIA-Realtime-Telemetry`
- Objetivo: monitoreo continuo y deteccion temprana de anomalias.
- Tools permitidas:
  - get_cpu_usage, get_memory_usage, get_disk_usage, get_network_usage
  - get_top_processes, get_running_services, get_startup_programs
  - get_installed_programs, get_windows_updates_status
- Riesgo maximo permitido: `R1`
- Decision:
  - si umbral de anomalia > 0.85, escalar a Rendimiento o Seguridad.

## 5.3 Especialidad: Red e Internet
- Agente: `KernelIA-Network-Intel`
- Objetivo: diagnosticar conectividad, DNS, rutas y stack de red.
- Tools permitidas:
  - ping_host, traceroute_host, dns_lookup, test_tcp_port
  - get_public_ip, get_local_ip, get_wifi_info
  - get_default_gateway, get_dns_servers
  - flush_dns_cache, renew_ip_config, release_ip_config, reset_network_stack
- Riesgo maximo permitido: `R2`
- Guardrails:
  - reset de stack solo con dry-run y ventana de mantenimiento.

## 5.4 Especialidad: Procesos
- Agente: `KernelIA-Process-Guardian`
- Objetivo: controlar procesos de alto consumo y bloqueos.
- Tools permitidas:
  - list_processes, get_process_detail, kill_process, restart_process
  - find_high_cpu_processes, find_high_memory_processes, force_kill_process
- Riesgo maximo permitido: `R3`
- Guardrails:
  - protege lista de procesos criticos del sistema.
  - `force_kill_process` requiere Owner.

## 5.5 Especialidad: Servicios Windows
- Agente: `KernelIA-Service-Controller`
- Objetivo: operar servicios de sistema con continuidad.
- Tools permitidas:
  - list_services, get_service_status, start_service, stop_service
  - restart_service, enable_service, disable_service
- Riesgo maximo permitido: `R3`
- Guardrails:
  - servicios criticos requieren verificacion doble.
  - stop/disable en servicios core requiere snapshot.

## 5.6 Especialidad: Mantenimiento basico
- Agente: `KernelIA-Maintenance-Operator`
- Objetivo: higiene de sistema y reparaciones base.
- Tools permitidas:
  - clean_temp_files, empty_recycle_bin, run_disk_cleanup, clear_browser_cache
  - clear_windows_update_cache, check_disk_health
  - scan_system_files, repair_system_files
  - run_dism_health_check, run_dism_restore_health
- Riesgo maximo permitido: `R3`
- Guardrails:
  - DISM/SFC con evidencia previa y posterior obligatoria.

## 5.7 Especialidad: Seguridad local
- Agente: `KernelIA-Security-Sentinel`
- Objetivo: postura de seguridad local y remediacion inicial.
- Tools permitidas:
  - get_firewall_status, list_firewall_rules, enable_firewall, disable_firewall
  - get_defender_status, run_defender_quick_scan, run_defender_full_scan
  - get_antivirus_status, get_security_center_status
  - list_open_ports, list_listening_connections, list_active_connections
- Riesgo maximo permitido: `R4`
- Guardrails:
  - cualquier deshabilitacion de seguridad requiere Owner + doble confirmacion.

## 5.8 Especialidad: Drivers
- Agente: `KernelIA-Driver-Engineer`
- Objetivo: salud de dispositivos y drivers.
- Tools permitidas:
  - list_devices, list_problem_devices, get_device_detail, get_driver_info
  - update_driver, open_optional_driver_updates, rescan_devices
- Riesgo maximo permitido: `R2`
- Guardrails:
  - cambios de driver en masa requieren aprobacion Core.

## 5.9 Especialidad: Archivos y carpetas
- Agente: `KernelIA-Filesystem-Operator`
- Objetivo: operaciones de archivos con seguridad y control.
- Tools permitidas:
  - list_directory, get_file_info, search_files, create_folder
  - delete_file, move_file, copy_file, rename_file, calculate_folder_size
  - force_delete_file
- Riesgo maximo permitido: `R4`
- Guardrails:
  - borrar/mover en rutas sensibles bloqueado por policy.
  - `force_delete_file` requiere MegaBoss.

## 5.10 Especialidad: Logs y auditoria
- Agente: `KernelIA-Audit-Analyst`
- Objetivo: investigacion forense operativa y compliance.
- Tools permitidas:
  - read_event_logs, read_system_log, read_application_log, read_security_log
  - export_event_logs, get_kernelia_audit_log, search_kernelia_audit_log
- Riesgo maximo permitido: `R1`
- Guardrails:
  - exportaciones anonimizadas por defecto.

## 5.11 Especialidad: Energia y rendimiento
- Agente: `KernelIA-Performance-Tuner`
- Objetivo: optimizar rendimiento y energia segun politica.
- Tools permitidas:
  - get_power_plan, set_power_plan, list_power_plans
  - get_sleep_settings, set_sleep_settings
  - get_startup_impact, optimize_startup_apps
  - run_latency_probe, run_tool_benchmark, get_performance_kpis
  - generate_performance_report, detect_performance_anomalies
- Riesgo maximo permitido: `R2`
- Guardrails:
  - cambios de plan de energia requieren perfil de negocio definido.

## 5.12 Especialidad: Software instalado
- Agente: `KernelIA-Software-Lifecycle`
- Objetivo: gobierno de aplicaciones y features.
- Tools permitidas:
  - list_installed_apps, get_app_detail, uninstall_app, check_app_updates
  - list_windows_features, enable_windows_feature, disable_windows_feature
- Riesgo maximo permitido: `R3`
- Guardrails:
  - desinstalacion y cambios de features requieren ventana controlada.

## 5.13 Especialidad: Comandos sensibles
- Agente: `KernelIA-Sensitive-Executor`
- Objetivo: ejecucion controlada de acciones administrativas de alto impacto.
- Tools permitidas:
  - run_shell_command, run_powershell_command
  - edit_registry_key, delete_registry_key
  - create_local_user, delete_local_user, reset_user_password
  - add_user_to_group, remove_user_from_group
  - change_firewall_rule, change_network_adapter_config
  - reboot_system, shutdown_system
- Riesgo maximo permitido: `R4`
- Guardrails:
  - siempre requiere Owner.
  - comandos pasan por validacion de seguridad previa.

## 5.14 Especialidad: MegaBoss
- Agente: `KernelIA-MegaBoss-CriticalOps`
- Objetivo: ejecutar operaciones criticas excepcionales.
- Tools permitidas:
  - run_elevated_command, force_kill_process, force_delete_file
  - modify_system_registry, disable_security_component, enable_security_component
  - reset_windows_network_stack, repair_windows_image, execute_admin_script
- Riesgo maximo permitido: `R4` (critico)
- Guardrails:
  - token temporal + OTP
  - doble aprobacion humana (Owner + Owner2)
  - snapshot y rollback obligatorio
  - tiempo de sesion limitado (TTL corto)

## 6. Contrato de salida de todos los agentes

```json
{
  "agent": "KernelIA-<name>",
  "risk_level": "R0|R1|R2|R3|R4",
  "confidence": 0.0,
  "decision": "allow|simulate|deny|escalate",
  "reason": "string",
  "actions": [
    {
      "tool": "tool_name",
      "args": {},
      "mode": "read|simulate|execute"
    }
  ],
  "evidence": {
    "before": [],
    "after": [],
    "audit_id": "string"
  },
  "rollback": {
    "required": true,
    "snapshot_id": "string"
  }
}
```

## 7. Requisitos para cumplimiento de alto estandar

- Cifrado de secretos en reposo y transito.
- mTLS entre servicios en modo distribuido.
- JWT rotativo con expiracion corta.
- Deteccion de anomalias de comportamiento por agente.
- Registro inmutable de auditoria y correlacion por trace-id.
- Politicas de retencion y minimizacion de datos sensibles.
- Pruebas periodicas:
  - unitarias de policy,
  - integracion de ruteo,
  - chaos/rollback,
  - simulaciones de incidente.

## 8. Matriz de escalamiento

- Escalar al Core:
  - confianza < 0.90 en R2,
  - conflicto entre 2 agentes,
  - politicas ambiguas.
- Escalar a Owner:
  - todo R3-R4,
  - cambio de seguridad,
  - impacto potencial en continuidad.
- Escalar a soporte humano:
  - rollback fallido,
  - evidencia inconsistente,
  - riesgo legal/compliance.

# Hoja de Ruta de Desarrollo (Roadmap) - KERNEL IA

Este documento detalla las fases de construcción de **KERNEL IA**, especificando las habilidades (Skills), agentes y plugins que se activarán en cada etapa para alcanzar la visión de un sistema de diagnóstico y reparación de nivel AAA.

---

## 📍 Fase 1: Cimiento y Observabilidad (Implementada)
**Objetivo:** Lograr una visión 360° del estado del hardware y software sin realizar cambios profundos.

- **Foco:** Diagnóstico pasivo y transparencia inicial.
- **Skills Invocadas:** 
    - `sysinfo`: Para obtener el ADN del hardware.
    - `process_mgmt`: Para ver qué consume recursos en tiempo real.
    - `network_intel`: Para validar la salida al mundo.
- **Agentes Activos:** 
    - **Agente Analista:** Interpreta los datos crudos y explica al usuario por qué su PC está lenta.
- **Plugins/Módulos:** 
    - `audit_logger`: Registro básico de consultas.
    - `report_generator`: Creación del primer reporte Markdown.

---

## 🛠️ Fase 2: Intervención Segura y Auditoría Visual (Implementada)
**Objetivo:** Permitir que la IA realice reparaciones seguras con supervisión humana total.

- **Foco:** Acción técnica protegida.
- **Skills Invocadas:** 
    - `secure_terminal`: Ejecución de comandos PowerShell validados.
    - `audit_tracking`: Registro inmutable de acciones.
- **Agentes Activos:** 
    - **Agente Operador:** Traduce instrucciones de lenguaje natural a scripts técnicos.
    - **Agente Auditor:** Bloquea comandos peligrosos y verifica la integridad del log.
- **Plugins/Módulos:** 
    - `AuditDashboard`: Componente visual en el frontend para ver el historial.
    - `SafePathValidator`: Validador de rutas de archivos permitidas.

---

## ⚡ Fase 3: Optimización Profunda y Limpieza (Implementada)
**Objetivo:** Mejorar el rendimiento eliminando basura y optimizando configuraciones.

- **Foco:** Rendimiento y espacio en disco.
- **Skills Invocadas:** 
    - `cleanup_ops`: Eliminación selectiva de archivos temporales y caché.
    - `registry_analyzer`: Detección de errores en el registro de Windows.
- **Agentes Activos:** 
    - **Agente Analista:** Detecta archivos basura.
    - **Agente Operador:** Ejecuta la limpieza.
- **Plugins/Módulos:** 
    - `JunkCleanerPlugin`: Módulo especializado en limpieza de navegadores y sistema.
    - `StartupManager`: Optimización del arranque de Windows.

---

## 🌐 Fase 4: Autonomía Proactiva y Multimodelo (Implementada)
**Objetivo:** KERNEL IA se vuelve proactivo y capaz de usar cualquier motor de IA.

- **Foco:** Mantenimiento preventivo y flexibilidad.
- **Skills Invocadas:** 
    - `scheduler`: Programación de diagnósticos automáticos.
    - `multi_llm_connector`: Selector dinámico (Gemini / OpenAI / Ollama).
- **Agentes Activos:** 
    - **Agente Orquestador:** Decide qué modelo es mejor para cada tarea (ej: un modelo local para privacidad, uno potente para diagnósticos complejos).
- **Plugins/Módulos:** 
    - `OllamaBridge`: Conexión directa con modelos locales.
    - `AutoFixTrigger`: Notificaciones proactivas de reparación.

---

## 📊 Fase 5: Conectividad Cloud y Soporte Empresarial (Implementada)
**Objetivo:** Integración total con el ecosistema de soporte de Hackteck.

- **Foco:** Soporte a escala y reportes avanzados.
- **Skills Invocadas:** 
    - `cloud_sync`: Sincronización de logs y reportes con la nube (opcional).
    - `advanced_reporting`: Dashboards visuales de salud histórica.
- **Agentes Activos:** 
    - **Agente de Enlace:** Prepara la información para que un técnico humano de Hackteck intervenga remotamente si es necesario.
- **Plugins/Módulos:** 
    - `HackteckCloudConnector`: API para envío seguro de diagnósticos.
    - `EnterpriseDashboard`: Interfaz para gestión de múltiples equipos.

---

## 🧠 Fase 6: Diagnósticos KernelIA y Guardrails (Implementada)
**Objetivo:** Elevar la calidad diagnóstica y reforzar seguridad conversacional en flujos KernelIA.

- **Foco:** Diagnóstico guiado + hardening de guardrails.
- **Skills Invocadas:**
    - `kernel_diagnostics`: Playbooks de diagnóstico de rendimiento y red.
    - `guardrails_validation`: Validación activa de bloqueo de comandos destructivos.
- **Agentes Activos:**
    - **Agente Diagnóstico Kernel:** Causa probable y remediación accionable.
    - **Agente Safety:** Verifica cumplimiento de políticas de seguridad.
- **Plugins/Módulos:**
    - `KernelReadinessReporter`: Reporte consolidado de readiness.
    - `GuardrailVerifier`: Evidencia de bloqueo para compliance.

---

## ⚙️ Fase 7: Rendimiento y Latencia (Implementada)
**Objetivo:** Establecer observabilidad cuantitativa de performance para latencia, estabilidad y capacidad de respuesta.

- **Foco:** KPIs de performance y benchmark operativo.
- **Skills Invocadas:**
    - `latency_probe`: medición de tiempos de respuesta base.
    - `tool_benchmark`: benchmarking de herramientas críticas.
- **Agentes Activos:**
    - **Agente Performance:** mide y consolida métricas operativas.
- **Plugins/Módulos:**
    - `PerformanceReporter`: reportería técnica de latencia.
    - `KPIConsolidator`: agregación de métricas históricas.

---

## 🛡️ Fase 8: Fiabilidad y Cumplimiento SLA (Implementada)
**Objetivo:** Convertir métricas de performance en señales de confiabilidad para anticipar degradaciones y proteger SLA.

- **Foco:** Resiliencia operacional y alertado temprano.
- **Skills Invocadas:**
    - `reliability_analyzer`: detección de anomalías sobre baseline p95/success rate.
    - `sla_evaluator`: evaluación continua de cumplimiento de SLA.
- **Agentes Activos:**
    - **Agente Reliability:** clasifica degradación, prioriza severidad y sugiere mitigación.
- **Plugins/Módulos:**
    - `ReliabilitySignalEngine`: motor de anomalías operativas.
    - `SLAReporter`: consolidación de estado y recomendaciones.

---

## 🧩 Fase 9: Autocuracion y Prevencion (Implementada)
**Objetivo:** Transformar señales de fiabilidad en ciclos preventivos accionables para reducir degradacion recurrente.

- **Foco:** Mitigacion temprana y autocuracion controlada.
- **Skills Invocadas:**
    - `self_healing_readiness`: evaluacion de riesgo y readiness operacional.
    - `self_healing_executor`: simulacion/ejecucion de ciclos de mitigacion.
- **Agentes Activos:**
    - **Agente Self-Healing:** prioriza mitigaciones y coordina ejecucion segura.
- **Plugins/Módulos:**
    - `SelfHealingPlanner`: plan preventivo basado en riesgo.
    - `MitigationCycleRunner`: trazabilidad de ejecuciones run/simulate.

---

## 🚀 Fase 10: Go-Live AAA y Compliance (Implementada)
**Objetivo:** Cerrar readiness de producción con evidencias técnicas y cumplimiento operativo, dejando soporte remoto en standby controlado.

- **Foco:** Cierre operativo/comercial y evidencia de release.
- **Skills Invocadas:**
    - `go_live_readiness`: validación de criterios de salida.
    - `go_live_compliance`: consolidación de evidencias y scorecard.
- **Agentes Activos:**
    - **Agente Go-Live:** consolida controles, riesgos residuales y reporte ejecutivo.
- **Plugins/Módulos:**
    - `GoLiveScorecard`: semáforo técnico/comercial de salida.
    - `EvidenceBundleExporter`: paquete de evidencia para auditoría.

---

### Resumen de Invocación por Fase

| Fase | Skills Dominantes | Agente Principal | Plugin Clave |
| :--- | :--- | :--- | :--- |
| **1** | sysinfo, process, network | Analista | report_generator |
| **2** | secure_terminal, audit | Operador + Auditor | AuditDashboard |
| **3** | cleanup, registry | Operador | JunkCleaner |
| **4** | multi_llm, scheduler | Orquestador | OllamaBridge |
| **5** | cloud_sync, reporting | Analista + Enlace | CloudConnector |
| **6** | kernel_diag, guardrails | Diagnóstico + Safety | GuardrailVerifier |
| **7** | latency, benchmark | Performance | KPIConsolidator |
| **8** | reliability, sla | Reliability | SLAReporter |
| **9** | self-healing, prevention | Self-Healing | SelfHealingPlanner |
| **10** | go-live, compliance | Go-Live | GoLiveScorecard |

---

## Estado Actual de Ejecución
- Fase 1: cerrada y validada.
- Fase 2: cerrada y validada.
- Fase 3: cerrada y validada.
- Fase 4: cerrada y validada.
- Fase 5: cerrada y validada.
- Fase 6: cerrada y validada.
- Fase 7: cerrada y validada.
- Fase 8: cerrada y validada.
- Fase 9: cerrada y validada.
- Fase 10: cerrada y validada.

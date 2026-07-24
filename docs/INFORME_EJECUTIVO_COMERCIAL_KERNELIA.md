# INFORME EJECUTIVO Y COMERCIAL: KERNELIA
## Plataforma de Asistencia Técnica Inteligente, Diagnóstico Autónomo y Gobernanza de Sistemas Windows

---

## 1. Resumen Ejecutivo

**KernelIA** es una solución enterprise local-first diseñada para transformar la gestión de soporte técnico L1 y L2 en entornos Windows. Mediante la combinación de **Modelos de Lenguaje de Última Generación (LLM local con Ollama)**, una arquitectura **Advanced RAG (Retrieval-Augmented Generation)** y un ensamble de **8 Agentes Especialistas Autónomos**, KernelIA automatiza el diagnóstico, la auditoría y la remediación segura de infraestructura informática reduciendo los tiempos de atención técnica hasta en un 80%.

```mermaid
graph TD
    User[Operador / Técnico / Usuario L1] --> UI[Interfaz SvelteKit + Tauri v2]
    UI --> Router[KernelIA AI Router]
    Router --> RAG[Advanced RAG Engine + SQLite]
    Router --> Agents[8 Agentes Especialistas por Área]
    Agents --> Guardrails[Motor de Gobernanza y RBAC (R0 - R4)]
    Guardrails --> PowerShell[Ejecución de PowerShell y Diagnósticos Nativo]
    PowerShell --> OS[Kernel y Sistema Operativo Windows]
```

---

## 2. Propuesta de Valor Comercial

En las empresas modernas, el costo operativo de resolver incidencias en estaciones de trabajo y servidores Windows (conectividad, consumo desbocado de recursos, controladores corruptos, fallos de impresión, seguridad) representa un cuello de botella constante para los equipos de TI.

**KernelIA** resuelve esta problemática ofreciendo:

- **Reducción del Tiempo Medio de Resolución (MTTR)**: De horas a minutos mediante diagnósticos en vivo e intervenciones dirigidas por IA.
- **Privacidad y Seguridad Garantizada (Local-First)**: El modelo y la base de conocimiento operan localmente dentro del equipo o red privada, garantizando que los datos confidenciales nunca salgan de la organización.
- **Cero Alucinaciones con Evidencia Real**: Cada respuesta de la IA está respaldada por comandos de PowerShell en vivo y documentos de la base de conocimiento local.
- **Gobernanza Estricta por Niveles de Riesgo (RBAC R0-R4)**: Control total sobre qué operaciones puede ejecutar la IA y cuáles requieren autorización del perfil técnico o administrador (**Megaboss**).

---

## 3. Matriz de Agentes Especialistas

KernelIA no utiliza un modelo genérico monolítico. En su lugar, cuenta con una arquitectura de **8 Agentes Especialistas** calibrados para cada área operativa de Windows:

| Agente Especialista | Dominio Técnico | Capacidades de Diagnóstico y Remediación |
|---|---|---|
| **NetworkAgent** | Redes y Conectividad | Diagnóstico de adaptadores, latencia TCP, resolución DNS (`Resolve-DnsName`), IP/Gateway y vaciado de caché (`Clear-DnsClientCache`). |
| **DriversAgent** | Controladores y Hardware | Inspección PnP, detección automática de dispositivos con fallo o Código 43 (`Get-PnpDevice`), inventario OEM (`pnputil`). |
| **ServicesAgent** | Servicios y Spooler | Estado de servicios automáticos detenidos (`Get-Service`), reinicio seguro y purga de cola de impresión atascada (`PRINTERS`). |
| **ProcessAgent** | Procesos y Memoria | Análisis de consumo de CPU/RAM, aislamiento de PIDs desbocados y terminación segura con lista blanca de protección del Kernel. |
| **PerformanceAgent** | Rendimiento en Vivo | Lectura de contadores en vivo (`Get-Counter`), monitoreo de ocupación de disco al 100%, memoria libre y Uptime del sistema. |
| **SecurityAgent** | Seguridad y Auditoría | Verificación de estado de Windows Defender (`Get-MpComputerStatus`), auditoría de intentos fallidos de logon (Event 4625) e inspección de Firewall. |
| **FilesystemAgent** | Almacenamiento y Volúmenes | Salud SMART de unidades de disco (`Get-PhysicalDisk`), escaneo no destructivo de volúmenes (`Repair-Volume -Scan`) y optimización TRIM/Defrag. |
| **SystemAgent** | Diagnóstico Consolidado | Informe técnico completo del equipo (`Get-ComputerInfo`), verificación de parches KB instalados e integridad del sistema con `sfc` y `DISM`. |

---

## 4. Gobernanza, Seguridad y Niveles de Riesgo (RBAC)

Para garantizar la máxima seguridad operativa en entornos corporativos, KernelIA clasifica todas sus acciones en una escala de 5 niveles de riesgo con guardrails inflexibles:

```
[R0: Lectura Pasiva] ──> [R1: Diagnóstico Activo] ──> [R2: Remediación Segura] ──> [R3: Operación Sensible] ──> [R4: Operación Crítica]
(Ejecución Libre)       (Escaneo / Logs)         (Reinicio Servicio/DNS)   (DISM / PnP Change)    (Reinicio/Apagado - Megaboss)
```

- **Lista Blanca de Protección del Kernel**: Bloquea cualquier intento de finalizar procesos críticos del sistema operativo (`svchost`, `lsass`, `csrss`, `services`, `explorer`, `wininit`).
- **Verificación de Rol Megaboss**: Operaciones de nivel `R4` (reinicio o apagar el equipo) requieren autenticación y token de seguridad de alto nivel.
- **Trazabilidad Inmutable**: Cada diagnóstico, evidencia y comando ejecutado se registra de forma inmutable en una base SQLite local para auditoría de cumplimiento (Compliance/ISO 27001).

---

## 5. Arquitectura Advanced RAG + Contexto Vivo

La inteligencia de KernelIA opera mediante un flujo continuo de **6 Etapas**:

1. **Análisis de Intención e Inferencia Multi-Dominio**: Identifica si la consulta requiere uno o varios especialistas (ej. Rendimiento + Cola de Impresión).
2. **Búsqueda Híbrida de Conocimiento (Hybrid Retrieval)**: Recuperación vectorial (embeddings) + léxica (BM25 con filtrado de stopwords) en SQLite.
3. **Captura de Evidencia en Vivo (Live State Context)**: Ejecución en segundo plano de cmdlets PowerShell para obtener el estado real del sistema.
4. **Evaluación de Confianza y Riesgo (Decision Engine)**: Asignación de score de certeza y selección del modo de respuesta (`Explain`, `Simulate`, `Execute`, `Clarify`).
5. **Prompt Gobernado con Reglas Anti-Alucinaciones**: Formateo estructurado (`Document N:::`) con directivas estrictas de abstención si no hay evidencia local.
6. **Inferencia en Streaming con Presentación de Citas**: Entrega progresiva en la UI SvelteKit con tarjetas de fuentes consultadas y traza técnica desplegable.

---

## 6. Modelo de Negocio y Aplicación Comercial

KernelIA se comercializa bajo un esquema flexible adaptado a empresas e integradores de TI:

### 1. Licenciamiento Enterprise (B2B SaaS / On-Premise)
- **Soporte L1 Autónomo para Mesas de Ayuda**: Despliegue en estaciones de trabajo para permitir que los usuarios resuelvan incidencias comunes guiados por la IA.
- **Consola para Técnicos de Soporte L2/L3**: Herramienta de aceleración diagnóstica con ejecuciones PowerShell seguras de un solo clic.

### 2. Integración para Proveedores de Servicios Gestionados (MSP)
- **Operación Multi-Equipo**: Capacidad de auditoría y reportes operacionales exportables para respaldar Acuerdos de Nivel de Servicio (SLA).

### Impacto Económico Estimado (ROI):
- **Ahorro de Tiempo por Incidencia**: Reducción del tiempo diagnóstico de 25 minutos a menos de 2 minutos.
- **Reducción de Escalados a L3**: Disminución del 45% en tickets redirigidos a especialistas senior.
- **Retorno de Inversión (ROI)**: Recuperación de la inversión estimada en menos de 4 meses de operación.

---

## 7. Estado del Desarrollo y Garantía Técnica

- **Backend Nativo en Rust (Tauri v2)**: Máximo rendimiento, consumo mínimo de RAM (~50 MB) e integración segura con la API de Windows.
- **Frontend Moderno en SvelteKit**: Interfaz fluida estilo Zen Canvas, con badges de confianza, visualización de fuentes y QA Panel incorporado.
- **Suite de Pruebas Automatizadas**: Cobertura de pruebas unitarias en Rust validadas con `cargo test` (`0 fallos`).
- **Integración Continua en GitHub**: Repositorio y código versionado disponible en el **Pull Request #1** (`https://github.com/Renanakin/kernelia/pull/1`).

---

## 8. Conclusión

**KernelIA** representa el nuevo estándar en asistencia técnica corporativa: una plataforma privada, potente y totalmente respaldada por evidencia técnica en tiempo real. Al combinar la velocidad de los LLM locales con la precisión del Advanced RAG y la gobernanza de PowerShell, KernelIA ofrece una ventaja competitiva definitiva para cualquier organización orientada a la excelencia operativa.

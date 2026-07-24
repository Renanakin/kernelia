# KERNEL IA - Catálogo de Skills & Agentes

Este documento detalla las capacidades operativas (Skills) y la arquitectura de agentes que impulsan **KERNEL IA**. Estas habilidades permiten al sistema interactuar de forma segura y eficiente con el sistema operativo Windows para diagnóstico y reparación.

---

## 🛠 Core Skills (Habilidades Base)

### 1. Diagnóstico de Hardware y Sistema (`sysinfo`)
- **Capacidad:** Lectura en tiempo real de CPU, memoria, almacenamiento y estado de batería.
- **Uso:** Identificar cuellos de botella de hardware antes de proceder con reparaciones de software.

### 2. Gestión Inteligente de Procesos (`process_mgmt`)
- **Capacidad:** Listado detallado de procesos, análisis de consumo de recursos y terminación segura de tareas colgadas.
- **Seguridad:** Protege procesos críticos del sistema para evitar pantallazos azules (BSOD).

### 3. Inteligencia de Red (`network_intel`)
- **Capacidad:** Pruebas de conectividad, diagnóstico de latencia, resolución de DNS y estado de adaptadores.
- **Uso:** Resolver problemas de "Internet lento" o desconexiones intermitentes.

### 4. Terminal Segura y Validada (`secure_terminal`)
- **Capacidad:** Ejecución de comandos PowerShell/Cmd con una capa de validación pre-ejecución.
- **Seguridad:** Lista negra dinámica que bloquea comandos destructivos (ej: `format`, `rmdir /s /q c:\windows`).

### 5. Auditoría Persistente (`audit_tracking`)
- **Capacidad:** Registro inmutable de cada acción tomada por la IA.
- **Trazabilidad:** Genera un rastro de "quién, qué y cuándo" para auditorías de seguridad y soporte técnico humano.

### 6. Generador de Reportes Consolidados (`report_gen`)
- **Capacidad:** Síntesis de múltiples diagnósticos en un reporte técnico profesional en Markdown.
- **Uso:** Entrega de resultados al usuario final o a soporte de Hackteck.

---

## 🤖 Agentes y Orquestación

KERNEL IA utiliza un modelo de **Orquestación de Agentes Especializados**:

- **Agente de Diagnóstico (Analista):** Se encarga de observar el sistema y encontrar anomalías.
- **Agente de Ejecución (Operador):** Traduce las intenciones de reparación en comandos técnicos seguros.
- **Agente de Seguridad (Auditor):** Valida cada comando contra las políticas de seguridad y registra el log de auditoría.

---

## 🔌 Plugins y Extensibilidad

El sistema está diseñado para aceptar plugins en Rust:
1. **Módulos de Limpieza:** Plugins para vaciar carpetas temporales y optimizar el registro.
2. **Módulos de Seguridad:** Integración con escaneo de antivirus de terceros.
3. **Módulos Cloud:** Sincronización de configuraciones y perfiles de usuario.

---

### Notas de Seguridad
Todas las skills están sujetas a la **Capa de Validación de Seguridad** de KERNEL IA, garantizando que ninguna acción automatizada ponga en riesgo la integridad del sistema del usuario.

# KERNEL IA 🧠🛡️
**Inteligencia Artificial para Diagnóstico y Reparación de PC Windows**

KERNEL IA es una aplicación de escritorio diseñada para transformar la forma en que interactuamos con el mantenimiento técnico de computadoras. Utiliza modelos de lenguaje avanzados (LLMs) integrados profundamente con el sistema operativo para diagnosticar, explicar y resolver problemas técnicos de manera segura y transparente.

---

## ✨ Características Principales

- 🔍 **Diagnóstico Inteligente:** Análisis en tiempo real de CPU, Memoria, Disco y Red.
- ⚡ **Optimización de Procesos:** Identificación y gestión de procesos pesados.
- 🌐 **Asistente Técnico Local:** Chat con IA que entiende el contexto técnico de tu PC.
- 📑 **Reportes de Soporte:** Generación automática de informes técnicos para soporte Hackteck.
- 🔒 **Auditoría Permanente:** Registro inmutable (`audit.log`) de cada acción tomada por la IA para total transparencia y seguridad.
- 🛠️ **Terminal Segura:** Ejecución de comandos del sistema validados por una capa de seguridad preventiva.
- 📈 **Observabilidad Empresarial (Fase 2):** Health score, riesgo, tendencia y anomalías.
- 🤖 **Automatización Inteligente (Fase 2):** Reglas SI/ENTONCES con ticketing automático.
- 🏢 **Operación Multiempresa (Fase 3):** Consolidación de tenants/endpoints y diagnóstico multiagente.
- 🧯 **Rollback y Trusted Execution (Fase 3):** Snapshots operativos y verificación de integridad de artefactos.
- 🌐 **Autonomía Proactiva y Multimodelo (Fase 4):** Mantenimiento preventivo, alertas proactivas, scheduler automático y enrutamiento dinámico de modelo IA.
- ☁️ **Conectividad Cloud y Soporte Enterprise (Fase 5):** Sincronización de reportes, casos escalados, dashboard ejecutivo y reportería avanzada.
- 🧠 **Diagnósticos KernelIA y Guardrails (Fase 6):** Playbooks de PC lenta/red, validación de seguridad y reporte de readiness operacional.
- ⚙️ **Rendimiento y Latencia (Fase 7):** Probing de latencia, benchmarking de tools y KPIs históricos de performance.
- 🛡️ **Fiabilidad y Cumplimiento SLA (Fase 8):** Detección de anomalías, estado SLA y reportería de resiliencia operacional.
- 🧩 **Autocuración y Prevención (Fase 9):** Readiness operativo, plan preventivo y ciclos de mitigación simulados/ejecutados.
- 🚀 **Go-Live AAA y Compliance (Fase 10):** Scorecard de salida, verificación de controles y bundle de evidencia ejecutiva.

---

## 🛠️ Stack Tecnológico

- **Frontend:** [SvelteKit](https://kit.svelte.dev/) + [Vite](https://vitejs.dev/) + [Lucide Icons](https://lucide.dev/).
- **Backend:** [Tauri](https://tauri.app/) (Rust) para integración nativa con el sistema operativo.
- **IA:** Integración con **Google Gemini**, **OpenAI** y soporte para modelos locales vía **Ollama**.
- **Auditoría:** Sistema de logs persistentes en Rust.

---

## 🚀 Inicio Rápido

### Requisitos previos
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (v18+)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Instalación
1. Clonar el repositorio.
2. Instalar dependencias:
   ```bash
   pnpm install
   ```
3. Configurar variables de entorno:
   Crea un archivo `.env` con tu API Key:
   ```env
   GEMINI_API_KEY=tu_clave_aqui
   ```

### Ejecución en desarrollo
```bash
pnpm tauri dev
```

---

## 📂 Estructura del Proyecto

- `src/`: Interfaz de usuario (Svelte).
- `src-tauri/`: Lógica principal del backend (Rust).
    - `src/ai/`: Orquestación de LLM y funciones de llamada.
    - `src/tools/`: Implementación de habilidades (Procesos, Sistema, Red, etc.).
    - `src/commands/`: Comandos de Tauri expuestos al frontend.
- `docs/`: Documentación técnica y guías de desarrollo.
- `SKILLS.md`: Catálogo detallado de capacidades de la IA.

---

## 🛡️ Seguridad y Auditoría

La seguridad es el pilar de KERNEL IA. Todas las herramientas pasan por un **filtro de validación** que impide la ejecución de comandos destructivos. Además, el archivo `audit.log` registra cada interacción para que el usuario siempre tenga el control total sobre lo que la IA realiza en su equipo.

### Arquitectura de agentes especializados
- Documento funcional: `docs/KERNELIA_AGENTES_ESPECIALIZADOS.md`
- Política ejecutable base: `docs/KERNELIA_AGENTES_POLICY.json`

---

## 🤝 Contribuciones

Desarrollado con ❤️ por el equipo de **Hackteck**.

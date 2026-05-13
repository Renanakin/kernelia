# 🛠️ Propuesta de Desarrollo: KERNEL IA V1 — Diagnóstico Inteligente

Esta propuesta detalla la hoja de ruta técnica para transformar Nexus Lite en una herramienta de diagnóstico profesional para Windows, enfocada en soporte técnico avanzado y automatización.

---

## 1. Módulos de Diagnóstico (Capa de Herramientas Rust)

Para cumplir con la visión V1, expandiremos la capa de `tools` en el backend:

### A. Estado del Sistema (Core)
*   **Implementado:** Uso de `sysinfo` para CPU, RAM y Uptime.
*   **Mejora V1:** Detección de temperaturas de hardware y salud de batería (en laptops).

### B. Procesos Pesados
*   **Implementado:** Listado básico de procesos.
*   **Mejora V1:** Algoritmo de "Detección de Anomalías" que identifique procesos con *CPU spikes* o *Memory leaks* y los clasifique automáticamente.

### C. Red e Internet
*   **Nuevo Tool:** `network_diagnostic.rs`
*   **Funciones:** Verificación de puerta de enlace, resolución DNS, traza de ruta a servidores de Hackteck y estado de interfaces físicas/Wi-Fi.

### D. Disco y Espacio
*   **Implementado:** Listado de unidades.
*   **Mejora V1:** Identificación de carpetas "prohibitivas" (grandes archivos temporales, caché de navegadores) y análisis de velocidad de lectura/escritura (E/S).

### E. Servicios Críticos
*   **Nuevo Tool:** `windows_services.rs`
*   **Funciones:** Monitoreo de servicios de Windows esenciales para el funcionamiento del sistema y de aplicaciones de negocio. Capacidad de reiniciar servicios desde el chat (bajo confirmación).

---

## 2. Inteligencia y Contexto Técnico

El diferencial de KERNEL IA V1 es el **Chat con Contexto Local**:

*   **Snapshots de Diagnóstico:** Cada vez que el usuario pregunta "¿Por qué mi PC está lenta?", la IA dispara automáticamente un "Diagnostic Scan" que recolecta datos de todos los módulos anteriores.
*   **Inyección de Prompts:** Los datos técnicos se inyectan en el prompt del sistema de forma estructurada para que la IA actúe como un Ingeniero de Soporte de Nivel 3.

---

## 3. UI/UX: Dashboard de Diagnóstico

La interfaz se dividirá en dos secciones principales:

1.  **Panel de Telemetría (Izquierda):** Gráficos minimalistas y "luces de estado" (Verde/Amarillo/Rojo) para cada módulo.
2.  **Chat de Resolución (Derecha):** El flujo de conversación donde se proponen soluciones basadas en el panel de la izquierda.

---

## 4. Reporte Exportable (Support Pack)

Implementaremos una función de **"Generar Reporte Hackteck"**:
*   **Formato:** Archivo `.json` cifrado o `.md` legible por humanos.
*   **Contenido:** Logs del sistema, configuración de red, lista de software instalado y recomendaciones de la IA.
*   **Uso:** Este archivo puede ser enviado al equipo de soporte humano para resoluciones complejas.

---

## 5. Plan de Ejecución (Sprints)

| Sprint | Enfoque | Entregable |
| :--- | :--- | :--- |
| **S1** | Conectividad y Servicios | Tools de Red y Servicios de Windows. |
| **S2** | Dashboard Visual | Componentes de Dashboard en Svelte 5. |
| **S3** | Lógica de Diagnóstico | Integración de IA con snapshots de sistema. |
| **S4** | Reporte y Pulido | Función de exportación y ajustes de UX final. |

---
**Próximo Paso Sugerido:** Comenzar con la implementación del tool de **Red y Diagnóstico de Conectividad** en Rust.

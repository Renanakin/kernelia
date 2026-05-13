# Manual de Usuario - KernelIA

## 1. Qué es KernelIA
KernelIA es una plataforma de soporte técnico operativo con IA para diagnóstico, mantenimiento, seguridad y automatización del equipo.

## 2. Qué hace actualmente (funciones completas)

### 2.1 Núcleo de conversación y ejecución
- Chat inteligente con ejecución de herramientas del sistema.
- Streaming de respuesta en tiempo real.
- Selección de modelos IA cloud y locales.
- Gestión de errores de cuota/red y continuidad del flujo.

### 2.2 Telemetría del equipo
- Estado general del sistema (CPU, RAM, discos, red, uptime, usuario, hostname, SO).
- Uso de recursos en tiempo real.
- Procesos más pesados.

### 2.3 Diagnóstico de red
- Diagnóstico de conectividad.
- DNS, gateway y adaptadores de red.
- Verificación de IP local/pública.
- Utilidades de recuperación de pila de red (según permiso).

### 2.4 Mantenimiento operativo
- Limpieza de temporales.
- Análisis básico de mantenimiento.
- Programación de tareas de mantenimiento.
- Ciclos de mantenimiento proactivo.

### 2.5 Drivers y hardware
- Detección de controladores con problema.
- Información de drivers.
- Búsqueda de faltantes.
- Flujo de actualización de controladores.

### 2.6 Seguridad y cumplimiento local
- Validación de estado de seguridad.
- Comprobaciones de puertos/conexiones.
- Integración con acciones de seguridad del sistema (según permiso).

### 2.7 Auditoría y trazabilidad
- Registro de acciones ejecutadas.
- Historial de comandos/herramientas.
- Estado de ejecución (ok/error) por acción.

### 2.8 Reportes
- Generación de reportes técnicos de soporte.
- Resumen de diagnóstico y acciones.

### 2.9 Automatización y fases avanzadas
- Flujos autónomos de operación y auto-healing.
- Diagnóstico multi-módulo.
- Detección de anomalías.
- Predicción de incidentes operacionales.
- Explicación de causa raíz.
- Playbooks automáticos.
- Métricas/SLA y reportes NOC (según fase y permisos habilitados).

## 3. Roles y privilegios
- `Viewer`: lectura y diagnóstico base.
- `PowerUser`: mantenimiento y operaciones ampliadas.
- `Owner`: acciones sensibles del sistema.
- `MegaBoss`: elevación temporal para comandos de máximo privilegio.

## 4. Cómo usar KernelIA sin estrés (flujo recomendado)
1. Elegir modelo IA en el selector superior.
2. Si el modelo es cloud, configurar API key en `Configuración`.
3. Escribir una solicitud concreta (un objetivo por mensaje).
4. Revisar resultado y evidencias en `Auditoría`.
5. Para tareas críticas, activar MegaBoss temporalmente.

## 5. Prompts recomendados (copiar y pegar)
- `Ejecuta un diagnóstico completo de red y resume hallazgos críticos.`
- `Muéstrame los procesos que más CPU y RAM consumen, con recomendación de acción.`
- `Analiza los drivers con problema y dame plan de corrección paso a paso.`
- `Genera un reporte técnico ejecutivo del estado actual del equipo.`
- `Corre una revisión de seguridad local y lista riesgos prioritarios.`
- `Programa una rutina de mantenimiento semanal y explícame lo que hará.`

## 6. Buenas prácticas
- Pedir siempre: `resumen + riesgos + próximo paso`.
- Evitar solicitudes ambiguas.
- Validar en Auditoría toda acción sensible.
- No ejecutar acciones críticas en lote sin revisión previa.

## 7. Nueva ayuda dentro de la app
Se agregó un botón `Manual` en la barra superior del chat para abrir esta guía dentro de KernelIA, sin salir de la plataforma.


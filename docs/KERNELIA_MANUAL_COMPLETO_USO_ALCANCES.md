# KernelIA - Manual Completo de Uso, Alcances y Operacion

Version: 2026-05-12  
Producto: KernelIA (nexus-lite)  
Modalidad objetivo: Operacion local-first con IA local (Gemma3) y soporte cloud opcional

## 1. Proposito del documento

Este manual define, de forma ejecutiva y tecnica, como usar KernelIA de manera correcta, segura y repetible.

Incluye:
- Que hace KernelIA realmente hoy.
- Que procedimientos ejecuta.
- Como operar cada modulo sin degradar estabilidad.
- Alcances y limites reales.
- Reglas de seguridad y privilegios.
- Flujo de soporte, auditoria y mantenimiento.
- Checklist de operacion diaria.
- Criterios de salida a produccion.

## 2. Que es KernelIA

KernelIA es una plataforma de operacion tecnica asistida por IA para equipos Windows, con:
- Interfaz operativa (Tauri + Svelte).
- Motor de herramientas del sistema (Rust).
- Enrutamiento inteligente de consultas (router IA + function calling).
- Control de acceso por roles (AAA/RBAC).
- Telemetria, auditoria, mantenimiento y seguridad.
- Integracion con modelos locales y/o cloud.

No es solo un chatbot. Es un orquestador de diagnostico + ejecucion tecnica con trazabilidad.

## 3. Arquitectura funcional (resumen)

### 3.1 Capa de interfaz
- Login y control de sesion.
- Chat operativo.
- Panel lateral: Telemetria, Auditoria, Mantenimiento, Cloud.
- Modales de configuracion y llaves API.

### 3.2 Capa de backend
- `src-tauri/src/ai/`: router, intent engine, function calling, modelos.
- `src-tauri/src/tools/`: herramientas operativas por dominio.
- `src-tauri/src/commands/`: comandos Tauri para UI.

### 3.3 Capa de ejecucion tecnica
- Comandos de sistema (PowerShell/cmd/sc/netsh/ipconfig) con controles.
- Lectura de estado del equipo (sysinfo, WMI/CIM, servicios).
- Registro de auditoria.

### 3.4 Capa de seguridad
- AAA de acceso.
- RBAC por rol.
- Validaciones de entrada/ruta/comando.
- Trazabilidad de acciones.
- Ejecucion sin consola visible para evitar popups de CMD.

## 4. Modos de operacion

### 4.1 Modo local-tools
KernelIA resuelve consultas tecnicas usando herramientas del equipo local sin depender del LLM para todo.

Casos tipicos:
- Diagnostico de red.
- Resumen de almacenamiento.
- Health del equipo.
- Procesos y servicios.

### 4.2 Modo IA local (Gemma3)
KernelIA enruta al endpoint local compatible OpenAI.

Endpoint recomendado:
- `http://localhost:11435/v1/chat/completions`

Uso recomendado:
- Consultas tecnicas guiadas.
- Resumenes, explicaciones y planes.
- Coordinacion de herramientas.

### 4.3 Modo cloud (opcional)
Uso de proveedores externos segun API key y cuota.

Nota operativa:
- Si cloud falla por cuota/timeout, priorizar ruta local.

## 5. Alcance funcional actual

## 5.1 Telemetria
- CPU, RAM, disco, uptime.
- Estado general del equipo.
- Indicadores para salud operativa base.

## 5.2 Auditoria
- Registro de acciones ejecutadas.
- Consulta y filtrado de eventos.
- Export basico de evidencia.

## 5.3 Mantenimiento
- Analisis de limpieza.
- Limpieza de temporales.
- Operaciones de red (flush/reset en flujos permitidos).
- Funciones de reparacion con guardrails.

## 5.4 Red
- Diagnostico de conectividad TCP.
- Resolucion DNS y latencia.
- Snapshot de gateway, adaptador activo y DNS.
- Salida con score operativo.

## 5.5 Procesos/servicios
- Listado de procesos.
- Priorizacion por consumo.
- Estado de servicios.
- Reinicio de servicios segun permisos.

## 5.6 Drivers
- Deteccion de dispositivos con error.
- Texto operativo de codigo de error.
- Flujos asistidos para update por canales seguros.

## 5.7 Seguridad
- Herramientas base de estado de firewall/defender (segun rol).
- Validaciones para bloquear acciones riesgosas fuera de politica.

## 5.8 Filesystem y operaciones de archivos
- Exploracion controlada.
- Lectura segura.
- Operaciones limitadas por validaciones de ruta.

## 5.9 Reportes
- Generacion de reportes tecnicos de soporte.
- Salida apta para handoff operativo.

## 5.10 Fases implementadas
Repositorio incluye herramientas y documentos de avance hasta Fase 10 del plan maestro, con cobertura variable por modulo y por contexto de ejecucion.

Referencias:
- `docs/KERNELIA_AAA_PLAN_MAESTRO.md`
- `docs/FASE0_COMPLETA.md` ... `docs/FASE10_IMPLEMENTADA.md`

## 6. Fuera de alcance (actual)

- Soporte remoto fully integrated en produccion general (permanece en standby controlado).
- Automatizacion masiva sin aprobaciones en entornos criticos.
- Gestion empresarial multi-tenant completa como SaaS global en un solo endpoint publico.
- Operaciones destructivas sin confirmacion ni evidencia.

## 7. Modelo de seguridad y privilegios

## 7.1 AAA/RBAC
Roles base:
- Superadmin.
- Soporte.
- Tecnico.
- Unlock critico (flujo Megaboss/critical procedure).

Regla:
- Toda herramienta sensible requiere rol compatible.
- Si rol no cumple, KernelIA rechaza con mensaje de acceso denegado.

## 7.2 Principios de operacion segura
- Menor privilegio posible.
- Confirmacion para cambios sensibles.
- Trazabilidad de toda accion.
- Evitar comandos arbitrarios no validados.
- No exponer secretos en chat/auditoria.

## 7.3 Politica de uso en tareas criticas
Antes de ejecutar cambios mayores:
1. Confirmar alcance exacto.
2. Validar respaldo o plan de retorno.
3. Ejecutar en ventana controlada.
4. Registrar evidencia.
5. Verificar estado posterior.

## 8. Procedimientos operativos completos

## 8.1 Inicio de jornada
1. Abrir KernelIA.
2. Iniciar sesion con rol adecuado.
3. Verificar conexion de modelo local (si aplica).
4. Ejecutar chequeo rapido:
   - Health del equipo.
   - Estado de red.
   - Carga de CPU/RAM.
5. Validar que Auditoria esta registrando eventos.

Resultado esperado:
- Plataforma estable, sin loops de carga.
- Respuestas operativas dentro de latencia esperada.

## 8.2 Flujo de diagnostico general
1. Definir sintoma en una frase clara.
2. Pedir diagnostico puntual (no ambiguo).
3. Revisar evidencia devuelta por herramienta.
4. Confirmar causa probable.
5. Ejecutar accion correctiva segura.
6. Verificar mejora.
7. Registrar cierre tecnico.

Plantilla de consulta recomendada:
- "Ejecuta diagnostico de red completo y entrega hallazgos tecnicos."
- "Lista procesos de mayor consumo y su impacto."
- "Entrega health del equipo en resumen ejecutivo."

## 8.3 Procedimiento de red (operativo)
1. Solicitar diagnostico de red.
2. Revisar:
   - Score y estado (operativa/degradada/critica).
   - TCP checks.
   - DNS y gateway.
3. Si degradada:
   - Validar DNS activos.
   - Probar reconnect de adaptador.
   - Aplicar limpieza de stack red segun politica.
4. Repetir diagnostico.
5. Comparar antes/despues.

## 8.4 Procedimiento de mantenimiento
1. Ejecutar analisis de limpieza.
2. Revisar recuperable estimado.
3. Seleccionar alcance (temporales usuario/sistema/cache).
4. Ejecutar limpieza.
5. Verificar espacio liberado y estabilidad.
6. Auditar cambios.

## 8.5 Procedimiento de drivers
1. Ejecutar listado de drivers con problemas.
2. Priorizar dispositivos criticos (red, almacenamiento, audio, chipset).
3. Aplicar update por flujo seguro (Windows Update opcional/controlado).
4. Verificar estado posterior.
5. Si persiste error, escalar con evidencia de PNP ID + codigo.

## 8.6 Procedimiento de auditoria
1. Abrir panel Auditoria.
2. Filtrar por accion/fecha.
3. Validar correlacion entre solicitud y ejecucion.
4. Exportar evidencia para soporte o compliance.

## 8.7 Procedimiento de incidentes
1. Detectar sintoma.
2. Clasificar severidad:
   - Baja: no bloquea operacion.
   - Media: degrada rendimiento.
   - Alta: bloquea servicio critico.
3. Ejecutar runbook minimo.
4. Confirmar recuperacion.
5. Documentar cierre con timestamp.

## 9. Guia de prompts efectivos

Buenas practicas:
- Pedir una sola accion por mensaje.
- Incluir alcance tecnico.
- Evitar frases ambiguas ("arreglalo todo") sin contexto.

Ejemplos recomendados:
- "Ejecuta diagnostico de red y resume en 5 lineas tecnicas."
- "Lista unidades de almacenamiento con porcentaje de uso."
- "Entrega health del equipo y principal riesgo operativo."
- "Muestra servicios detenidos relacionados con red."

Evitar:
- Mezclar 10 tareas en un solo mensaje.
- Pedir cambios criticos sin confirmar alcance.

## 10. Integracion LLM local (Gemma3)

Configuracion recomendada:
- Endpoint: `http://localhost:11435/v1/chat/completions`
- Modelo: `gemma3`
- Temperatura: 0.3 a 0.6 segun caso.
- Max tokens: 150 a 500.

Checklist de conectividad:
1. Docker/servicio proxy activo.
2. `GET /health` respondiendo 200.
3. Modelo descargado y disponible.
4. KernelIA apuntando al endpoint correcto.

Si falla:
- Revisar puerto configurado.
- Revisar firewall local.
- Revisar logs del proxy.

## 11. Rendimiento y estabilidad

Objetivos practicos:
- Respuestas simples: bajas latencias.
- Diagnosticos tecnicos: latencia moderada estable.
- Sin bloqueos de UI ni loops de carga.
- Sin popups de CMD durante ejecucion.

Recomendaciones:
- Evitar paralelizar tareas pesadas manualmente desde chat.
- Priorizar una accion, validar, luego siguiente.
- Monitorear CPU/RAM durante diagnosticos extensos.

## 12. Calidad operativa y QA

KernelIA debe mantener:
- Pruebas backend pasando.
- Build frontend pasando.
- QA E2E sobre flujos criticos.
- Criterio de salida min 9/10 en funciones core.

Referencias:
- `docs/KERNELIA_PRODUCCION_QA_MASTER_PLAN.md`
- `docs/KERNELIA_QA_MASTER_GEMMA3_21434_RESULTADO.md`

## 13. Troubleshooting (tabla rapida)

### 13.1 "Pensando..." sin respuesta
Posibles causas:
- Timeout de modelo.
- Endpoint incorrecto.
- Saturacion de recursos.

Acciones:
1. Verificar `/health` del endpoint local.
2. Confirmar puerto configurado en KernelIA.
3. Revisar CPU/RAM.
4. Reintentar consulta corta.

### 13.2 Error de cuota cloud 429
Causa:
- Limite de plan/proveedor.

Acciones:
1. Esperar ventana de retry.
2. Cambiar a Gemma3 local.
3. Ajustar politica de fallback.

### 13.3 Paneles que no cargan o parpadean
Acciones:
1. Reiniciar app.
2. Revisar si hay loops reactivos en modulo.
3. Verificar que auditoria/telemetria responden JSON valido.

### 13.4 Ventanas CMD emergentes
Causa tipica:
- Comandos externos sin `CREATE_NO_WINDOW`.

Estado:
- Flujo principal ajustado para ocultar consola.

### 13.5 Antivirus detecta falso positivo
Acciones:
1. Verificar hash del binario.
2. Firmar binario para distribucion.
3. Entregar release por canal confiable.

## 14. Procedimiento de despliegue local (operador)

1. Confirmar build candidato (`.exe`).
2. Validar login y modulos base.
3. Validar Gemma3 local.
4. Ejecutar smoke operativo:
   - health
   - red
   - almacenamiento
   - auditoria
5. Liberar para uso interno.

## 15. Gobernanza y control de cambios

Reglas:
- Todo cambio relevante debe incluir:
  - motivo,
  - riesgo,
  - evidencia de prueba.
- No liberar cambios sin validacion minima de QA.
- Mantener trazabilidad documental por version.

## 16. Matriz de uso por perfil

Superadmin:
- Operacion completa.
- Cambios de configuracion avanzada.
- Gestion de seguridad y politicas.

Soporte:
- Diagnostico y reparacion no destructiva.
- Mantenimiento operativo.
- Escalamiento de incidentes.

Tecnico:
- Consultas operativas limitadas.
- Validacion de estado y evidencia.

## 17. Politica de documentacion y evidencia

Cada incidente relevante debe cerrar con:
- Problema detectado.
- Evidencia previa.
- Accion ejecutada.
- Resultado posterior.
- Riesgo residual.

## 18. Checklist diario recomendado

1. Login correcto por rol.
2. Endpoint IA local disponible.
3. Telemetria estable.
4. Auditoria activa.
5. Diagnostico de red base.
6. Health base.
7. Sin errores criticos en UI.

## 19. Checklist semanal recomendado

1. Revision de espacio y limpieza.
2. Revision de drivers criticos.
3. Revision de servicios clave.
4. Revisión de eventos de auditoria.
5. Prueba de fallback local/cloud.

## 20. KPI operativos sugeridos

- Tiempo medio de respuesta de consultas tecnicas.
- Porcentaje de incidentes resueltos en primer intento.
- Numero de errores de timeout por semana.
- Estabilidad del endpoint local.
- Cobertura de acciones con evidencia en auditoria.

## 21. Cierre ejecutivo

KernelIA, en su estado actual, puede operar como plataforma tecnica local de alto valor para:
- diagnostico,
- mantenimiento,
- auditoria,
- seguridad base,
- y soporte asistido por IA.

Su uso correcto depende de:
- operar por procedimientos,
- respetar privilegios,
- mantener QA continuo,
- y trabajar con evidencia en cada accion.

## 22. Referencias internas

- [MANUAL_USUARIO_KERNELIA.md](C:\Users\Hackteck\Downloads\nexus-lite-develop\docs\MANUAL_USUARIO_KERNELIA.md)
- [GUIA_UTILIZACION_CORRECTA_KERNELIA.md](C:\Users\Hackteck\Downloads\nexus-lite-develop\docs\GUIA_UTILIZACION_CORRECTA_KERNELIA.md)
- [KERNELIA_AAA_PLAN_MAESTRO.md](C:\Users\Hackteck\Downloads\nexus-lite-develop\docs\KERNELIA_AAA_PLAN_MAESTRO.md)
- [KERNELIA_PRODUCCION_QA_MASTER_PLAN.md](C:\Users\Hackteck\Downloads\nexus-lite-develop\docs\KERNELIA_PRODUCCION_QA_MASTER_PLAN.md)
- [KERNELIA_CATALOGO_BASE_TOOLS.md](C:\Users\Hackteck\Downloads\nexus-lite-develop\docs\KERNELIA_CATALOGO_BASE_TOOLS.md)
- [KERNELIA_ACCESO_AAA.md](C:\Users\Hackteck\Downloads\nexus-lite-develop\docs\KERNELIA_ACCESO_AAA.md)


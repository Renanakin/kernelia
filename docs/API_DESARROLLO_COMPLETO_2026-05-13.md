# API desarrollo completo

Fecha: 2026-05-13
Alcance: frente API (Tauri commands) en KernelIA.

## Objetivo ejecutado

Se completo el desarrollo del frente API con foco en:
- contratos estables entre Front y Back,
- validacion consistente de input,
- errores claros para UI,
- sin mover logica de negocio pesada a handlers.

## Cambios implementados

### 1) Nuevo modulo comun de validacion

Archivo:
- src-tauri/src/commands/common.rs

Incluye:
- validaciones de no vacio,
- limites de longitud por campo,
- normalizacion/validacion de sort_by,
- normalizacion segura de limit,
- helpers de validacion para message, username, password, model_id, tool_name, service_name y quick_check_id.

### 2) Wiring del modulo comun

Archivo:
- src-tauri/src/commands/mod.rs

Cambio:
- se agrego el modulo common al namespace de commands.

### 3) Hardening de comandos de chat

Archivo:
- src-tauri/src/commands/chat.rs

Mejoras:
- validacion de message en send_message, stream_message y analyze_intent,
- validacion de model_id y api_key en set_model y set_api_key,
- validacion de password en set_megaboss_password, unlock_megaboss y unlock_tecnico_critical,
- validacion de username/password en login_user y create_support_user,
- validacion de username en delete_support_user.

Resultado:
- entradas inconsistentes se rechazan temprano con error claro y serializable.

### 4) Hardening de comandos de system

Archivo:
- src-tauri/src/commands/system.rs

Mejoras:
- list_processes: sort_by validado (cpu, memory, name) y limit acotado,
- restart_service: validacion de nombre de servicio,
- execute_tool: validacion de nombre de herramienta,
- run_quick_check: validacion de id,
- core_emit_event: validacion de topic/message y longitudes maximas,
- core_enqueue_task: validacion de nombre y longitud.

Resultado:
- API mas robusta frente a payloads invalidos y errores de integracion.

## Validacion tecnica

Comandos ejecutados:
- cargo check

Estado:
- compilacion OK,
- sin errores en los archivos API modificados,
- warnings existentes en otras areas del proyecto (fuera del alcance API de esta entrega).

## Archivos modificados

- src-tauri/src/commands/common.rs (nuevo)
- src-tauri/src/commands/mod.rs
- src-tauri/src/commands/chat.rs
- src-tauri/src/commands/system.rs

## Estado final de la API

- contratos de comando mas defensivos,
- validacion uniforme por capa API,
- errores de entrada predecibles para frontend,
- handlers siguen delegando logica de negocio a tools/core/router.

## Siguiente paso recomendado

Integrar en frontend el manejo explicito de errores con prefijo API_VALIDATION para mostrar mensajes accionables por campo en UI (sin exponer detalles internos).

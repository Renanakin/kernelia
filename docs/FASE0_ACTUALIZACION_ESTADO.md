# Fase 0 - Estado de Actualizacion

## Implementado en esta iteracion

- Modulo `core` integrado al backend Tauri.
- Event Bus interno basico:
  - `core_emit_event`
  - `core_list_events`
- Cola de ejecucion de tareas basica:
  - `core_enqueue_task`
  - `core_list_tasks`
  - Persistencia en `%LOCALAPPDATA%/nexus-lite/core/task_queue.json`
- Watchdog del sistema:
  - `core_watchdog_heartbeat`
  - `core_watchdog_status`
  - Heartbeat automatico cada 30s
- Snapshots del estado del PC:
  - `core_create_system_snapshot`
  - `core_list_system_snapshots`
  - Persistencia en `%LOCALAPPDATA%/nexus-lite/core/snapshots.jsonl`
- Recovery check basico:
  - `core_recovery_check`

## Integracion tecnica

- Estado `CoreState` gestionado con `Mutex` en `tauri::Builder.manage(...)`.
- Comandos `core_*` agregados en `invoke_handler`.
- Carga de cola pendiente al iniciar (`load_tasks`).

## Archivos

- `src-tauri/src/core/mod.rs`
- `src-tauri/src/commands/system.rs`
- `src-tauri/src/lib.rs`

## Validacion

- `cargo check` OK.

## Pendiente para cierre total de Fase 0

- UI dedicada para estado de core (eventos, cola, watchdog, snapshots).
- Ejecutor de cola completo (worker que procese tasks en background con retries).
- Modo recovery automatico por politicas (no solo comando manual).
- Snapshots diferenciales y restauracion guiada.

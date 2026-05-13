# Integracion Backend + API + Front

Fecha: 2026-05-13

## Objetivo

Unificar la estructura de KernelIA para que Front, API y Backend trabajen de forma directa bajo un contrato estable y ejecutable.

## Desarrollo realizado

### 1) Contrato runtime unificado en Front

Se creo un cliente de runtime central para que el Front deje de invocar comandos dispersos y use una interfaz consistente.

Archivo nuevo:
- src/lib/api/runtime/client.js

Responsabilidades:
- Encapsular comandos Tauri por dominio.
- Normalizar payloads a snake_case para API Rust.
- Reducir acoplamiento de componentes con nombres de comando.

### 2) Migracion de componentes/stores al cliente unificado

Se actualizaron llamadas directas a invoke para usar el cliente runtime en:
- src/lib/stores/settings.js
- src/lib/stores/auth.js
- src/lib/components/InputBar.svelte
- src/lib/components/ChatWindow.svelte
- src/lib/components/ModelSelector.svelte
- src/lib/components/SettingsModal.svelte
- src/lib/components/TelemetryPanel.svelte
- src/lib/components/QuickChecks.svelte
- src/lib/components/AuditDashboard.svelte

Resultado:
- Relacion Front -> API mas directa y mantenible.
- Menor duplicacion de payload/timeout/retries.

### 3) Compatibilidad API para contrato legacy/camelCase

Se agregaron comandos de compatibilidad para admitir payloads camelCase heredados sin romper clientes existentes:
- set_model_compat
- set_api_key_compat
- list_processes_compat
- run_quick_check_compat

Archivos:
- src-tauri/src/commands/chat.rs
- src-tauri/src/commands/system.rs
- src-tauri/src/lib.rs (invoke_handler)

### 4) Compatibilidad de fallback local (modo sin Tauri)

Se actualizo fallback browser/local para soportar:
- set_model con model_id/modelId
- set_api_key con validacion basica

Archivo:
- src/lib/utils/localDirect.js

### 5) Pruebas de contrato runtime

Se agrego suite de pruebas JS para validar compatibilidad de payload:
- tests/runtime-contract.test.js

Cubre:
- set_model con snake_case
- set_model con camelCase
- set_api_key con validacion de valor no vacio

## Validacion ejecutada

### Backend/API
- cargo check: OK

### Pruebas JS
- pnpm test: OK (6/6)

### Front check
- pnpm check: con errores existentes de tipado estricto en componentes no criticos de UI (CloudPanel/LoginGate) fuera del flujo principal de integracion backend-api ya completado.

## Estado final

- Integracion directa Front -> API -> Backend implementada.
- Contrato runtime unificado para ejecucion y pruebas.
- Backward compatibility habilitada para payloads legacy.
- Proyecto ejecutable en backend/API y con pruebas de contrato runtime pasando.

## Siguiente paso sugerido (cierre total de check)

Corregir tipado estricto remanente en:
- src/lib/components/CloudPanel.svelte
- src/lib/components/LoginGate.svelte

Una vez aplicado, ejecutar:
- pnpm check
- pnpm test
- cargo check

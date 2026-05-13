# KernelIA - Acceso AAA por perfiles

## Objetivo
Antes de entrar a KernelIA y usar capacidades administrativas, ahora se exige login con credenciales validas.

## Perfiles y privilegios

- superusuario:
  - Acceso total (Owner)
  - Puede crear y borrar usuarios
  - Puede operar privilegios MegaBoss (segun policy)

- soporte1:
  - Acceso alto (PowerUser)
  - Puede ejecutar casi todas las operaciones tecnicas
  - No puede crear/borrar usuarios

- tecnico:
  - Modo diagnostico (Viewer)
  - Puede ejecutar diagnosticos y lectura
  - Para procesos criticos debe ingresar clave critica temporal
  - Al desbloquear, sube temporalmente a PowerUser por ventana acotada

## Credenciales iniciales (obligatorio rotar)

Estas credenciales se crean por defecto en primera ejecucion para bootstrap local:

- Usuario: superadmin
  - Password inicial: KernelIA!Super2026
- Usuario: soporte1
  - Password inicial: KernelIA!Support2026
- Usuario: tecnico
  - Password inicial: KernelIA!Tech2026

Clave critica de tecnico (para elevacion temporal):
- KernelIA!CriticalProc2026

Se recomienda cambiar todas las claves inmediatamente luego del primer acceso.

## Controles de seguridad implementados

- Compuerta de login obligatoria en frontend antes de usar la app.
- Verificacion de sesion autenticada en backend para:
  - chat
  - streaming
  - execute_tool
  - quick checks
- Mapeo de perfil a RBAC interno:
  - superusuario -> Owner
  - soporte1 -> PowerUser
  - tecnico -> Viewer (PowerUser solo durante elevacion temporal)
- Auditoria activa sobre acciones ejecutadas.

## Archivos principales

- Frontend login gate:
  - src/lib/components/LoginGate.svelte
- Store de autenticacion:
  - src/lib/stores/auth.js
- Comandos backend de auth y gestion de usuarios:
  - src-tauri/src/commands/chat.rs
- Persistencia de usuarios/perfiles/sesion:
  - src-tauri/src/config/settings.rs
- Refuerzo en comandos de sistema:
  - src-tauri/src/commands/system.rs

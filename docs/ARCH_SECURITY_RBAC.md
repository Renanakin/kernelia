# Arquitectura de Seguridad RBAC - Nexus-Lite (KernelIA)

## 1. Introducción
Este documento define el modelo de Control de Acceso Basado en Roles (RBAC) para Nexus-Lite. El objetivo es garantizar que la Inteligencia Artificial (KernelIA) solo pueda ejecutar herramientas del sistema que correspondan al nivel de privilegio del usuario actual, siguiendo el Principio de Privilegio Mínimo (PoLP).

## 2. Roles de Usuario
Se definen tres roles principales:

| Rol | Descripción | Nivel de Riesgo |
| :--- | :--- | :--- |
| **Owner** | Acceso total al sistema, terminal sin restricciones y gestión de seguridad. | Crítico |
| **PowerUser** | Puede gestionar procesos, servicios y realizar limpiezas, pero no modificar archivos de sistema. | Alto |
| **Viewer** | Solo puede ver información de telemetría, listar directorios y leer logs de auditoría. | Bajo |

## 3. Matriz de Permisos (Tools)

| Herramienta | Viewer | PowerUser | Owner |
| :--- | :---: | :---: | :---: |
| `get_system_info` | ✅ | ✅ | ✅ |
| `list_directory` | ✅ | ✅ | ✅ |
| `read_file` | ✅ | ✅ | ✅ |
| `list_processes` | ✅ | ✅ | ✅ |
| `list_running_services` | ✅ | ✅ | ✅ |
| `analyze_junk` | ✅ | ✅ | ✅ |
| `list_scheduled_tasks` | ✅ | ✅ | ✅ |
| `list_cloud_reports` | ✅ | ✅ | ✅ |
| `get_audit_logs` | ✅ | ✅ | ✅ |
| `run_network_diagnostic` | ❌ | ✅ | ✅ |
| `kill_process` | ❌ | ✅ | ✅ |
| `restart_service` | ❌ | ✅ | ✅ |
| `run_cleanup` | ❌ | ✅ | ✅ |
| `analyze_registry` | ❌ | ✅ | ✅ |
| `upload_cloud_report` | ❌ | ✅ | ✅ |
| `toggle_scheduled_task` | ❌ | ✅ | ✅ |
| `write_file` | ❌ | ❌ | ✅ |
| `secure_terminal` | ❌ | ❌ | ✅ |
| `fix_registry` | ❌ | ❌ | ✅ |
| `schedule_maintenance` | ❌ | ❌ | ✅ |
| `delete_scheduled_task` | ❌ | ❌ | ✅ |

## 4. Mecanismo de Validación (`ensure_permission`)
Toda ejecución en `ToolEngine::execute` debe ser validada contra esta matriz. El backend de Rust interceptará la solicitud y verificará si el `user_role` activo tiene permiso para el `tool_name` solicitado.

```rust
// Ejemplo conceptual
pub fn ensure_permission(role: UserRole, tool: &str) -> Result<(), String> {
    match (role, tool) {
        (UserRole::Viewer, t) if !VIEWER_TOOLS.contains(&t) => Err("Permiso denegado".into()),
        (UserRole::PowerUser, t) if !POWER_USER_TOOLS.contains(&t) => Err("Permiso denegado".into()),
        _ => Ok(()),
    }
}
```

## 5. Auditoría Inmutable
Cada intento de ejecución (exitoso o denegado) se registrará en el log de auditoría con:
- Timestamp
- User ID / Role
- Tool Name
- Arguments (ofuscados si son sensibles)
- Status (Success / PermissionDenied / Error)

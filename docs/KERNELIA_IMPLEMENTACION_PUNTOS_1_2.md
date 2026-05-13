# Implementacion de los 2 puntos solicitados

## Punto 1 - ToolPolicyRegistry (completado)
Se implemento un registro dinamico de politicas por tool con los campos:
- name
- category
- min_role
- sensitive
- megaboss_required

Ubicacion:
- src-tauri/src/tools/catalog_tools.rs
  - struct ToolPolicy
  - get_tool_policies()
  - get_tool_policy(tool_name)

Notas:
- `min_role` se infiere usando RBAC real (Viewer/PowerUser/Owner).
- `megaboss_required` se infiere con `is_owner_only_tool`.
- `category` se infiere por convencion de nombre.

## Punto 2 - Carga de tools faltantes por bloques (primer bloque grande completado)
Se agregaron tools nuevas de:
- Informacion del sistema
- Telemetria en tiempo real
- Red e Internet
- Procesos
- Servicios Windows

Ubicacion:
- src-tauri/src/tools/catalog_tools.rs
  - tool_definitions()
  - execute_catalog_tool(name, args)

Integracion:
- src-tauri/src/tools/mod.rs
  - get_tool_definitions() ahora extiende con catalog_tools::tool_definitions()
  - execute() enruta tools del catalogo con catalog_tools::execute_catalog_tool
- src-tauri/src/tools/rbac.rs
  - permisos agregados por rol para tools nuevas

## Estado operativo
- Compila correctamente con cargo check.
- Sin errores de análisis en archivos modificados.

## Siguiente bloque recomendado
Para cerrar el catalogo completo faltan bloques de:
- mantenimiento basico
- seguridad local
- drivers
- logs y auditoria extendida
- energia y rendimiento
- software instalado avanzado
- comandos sensibles y megaboss avanzados

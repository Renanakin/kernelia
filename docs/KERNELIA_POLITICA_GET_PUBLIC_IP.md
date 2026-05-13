# Politica de Tool - get_public_ip

Estado: ACTIVA

## Reglas de control
- Nombre: get_public_ip
- Categoria: network
- Rol minimo: Viewer
- Sensitive: false
- MegaBoss required: false
- Tipo: solo lectura

## Requisito de ejecucion
- La validacion de existencia de la tool se ejecuta antes del control RBAC.
- Esto evita falsos "acceso denegado" cuando hay tools no registradas.

## Estructura minima aplicada
ToolDefinition {
    name: "get_public_ip",
    description: "Obtiene la IP publica actual del equipo.",
    category: "network",
    min_role: "Viewer",
    sensitive: false,
    megaboss_required: false,
}

Nota:
- En el backend actual, `category/min_role/sensitive/megaboss_required` se gobiernan por RBAC + politicas.
- El contrato de producto queda documentado aqui para estandarizar nuevas tools.

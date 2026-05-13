# API Area (Tauri Commands)

Este directorio corresponde al frente API para frontera de comandos Tauri.

## Objetivo
- Definir contratos estables entre Front y Back.
- Validar input y mapear errores de dominio.

## Flujo de trabajo
1. Definir/ajustar comando.
2. Validar payload.
3. Delegar logica a tools/core.
4. Verificar con cargo check.

## Definition of Done
- Contrato de comando claro.
- Errores serializables para frontend.
- cargo check en verde.

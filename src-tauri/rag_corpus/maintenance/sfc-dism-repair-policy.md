---
title: SFC and DISM Repair Policy
slug: maintenance-sfc-dism-repair-policy
specialty: maintenance
doc_type: verification
entity_key: system_repair
source_kind: curated_markdown
status: active
---
# Contexto
SFC y DISM son herramientas de reparacion, no diagnosticos triviales.

# Cuando aplicar
Solo cuando existan sintomas de integridad del sistema, errores persistentes de archivos o evidencia de corrupcion.

# Verificacion previa
Identificar sintomas concretos y registrar evidencia previa del estado del sistema.

# Verificacion posterior
Despues de ejecutar la herramienta debe revisarse la salida y validar si el problema original mejoro.

# Guardrail
`run_dism_restore_health` requiere mas control que un simple escaneo y no debe dispararse por consultas ambiguas.

---
title: Sensitive Paths Guardrail
slug: filesystem-sensitive-paths-guardrail
specialty: filesystem
doc_type: guardrail
entity_key: sensitive_paths
source_kind: curated_markdown
status: active
---
# Contexto
Las operaciones de archivos deben distinguir rutas de usuario de rutas sensibles del sistema.

# Rutas sensibles tipicas
Directorios del sistema operativo, perfiles criticos, rutas de aplicaciones y ubicaciones compartidas con impacto operativo.

# Regla operativa
El agente no debe borrar, mover o sobrescribir en rutas sensibles como accion automatica basica.

# Respuesta esperada
Si el usuario solicita una accion potencialmente destructiva en archivos, el agente debe degradar a explicacion, simulacion o escalamiento.

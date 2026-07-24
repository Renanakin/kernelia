---
title: Disk 100 Percent Diagnostic
slug: performance-disk-100-diagnostic
specialty: performance
doc_type: symptom_matrix
entity_key: disk_100
source_kind: curated_markdown
status: active
---
# Contexto
Usar este documento cuando el usuario menciona disco al 100, lentitud general o bloqueo al abrir aplicaciones.

# Causas comunes
Procesos de alto I/O, indexacion agresiva, antivirus, actualizaciones, paginacion intensa por falta de RAM o problemas de salud del disco.

# Verificacion
Cruzar telemetria de disco, memoria y procesos. No asumir que disco al 100 significa falla fisica.

# Acciones de bajo riesgo
Listar procesos con alto consumo, revisar espacio disponible y ejecutar analisis de archivos basura si corresponde.

# Acciones de mayor impacto
Solo considerar reparacion o acciones mas invasivas cuando exista evidencia adicional de integridad o salud degradada.

# Escalamiento
Escalar a mantenimiento profundo si hay patrones de saturacion persistentes o evidencia de errores de disco.

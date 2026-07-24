---
title: DNS Recovery Playbook
slug: network-dns-recovery
specialty: network
doc_type: playbook
entity_key: dns
source_kind: curated_markdown
status: active
---
# Contexto
Aplicar este playbook cuando el usuario reporta internet intermitente, resolucion de nombres inconsistente o navegacion fallida con gateway aparentemente operativo.

# Cuando aplica
Aplica si el equipo tiene conectividad IP pero falla al resolver dominios o cambia de estado entre resolucion correcta e incorrecta.

# Cuando no aplica
No usar este playbook como causa principal si hay perdida total de enlace, adaptador deshabilitado, DHCP caido o error fisico de red.

# Verificacion base
Validar IP local, gateway, DNS configurados y resolver un dominio conocido usando `dns_lookup`.

# Acciones sugeridas
Primero ejecutar diagnostico de red y luego evaluar `flush_dns_cache`. Si el problema persiste, revisar adaptador, gateway y DNS configurados antes de considerar reset de stack.

# Escalamiento
Si el problema afecta a multiples equipos o reaparece tras limpiar cache, escalar a analisis de gateway, DHCP o DNS aguas arriba.

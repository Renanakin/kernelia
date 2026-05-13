# Guia de Utilizacion Correcta de KernelIA

## 1. Objetivo
Este documento define la forma correcta de usar KernelIA en operacion diaria para lograr resultados estables, seguros y repetibles.

## 2. Alcance
Aplica a usuarios `Viewer`, `PowerUser` y `MegaBoss` en entornos Windows con modelos locales (Gemma3/Gemma4) y/o cloud habilitados.

## 3. Requisitos Previos Minimos
1. KernelIA instalado y ejecutando sin errores de inicio.
2. Endpoint LLM local disponible si se usa modo local:
   - `http://localhost:11435/v1`
3. Modelos cargados en endpoint local:
   - `gemma3`
   - `docker.io/ai/gemma4:latest`
4. Acceso de red y firewall sin bloqueo al endpoint configurado.
5. Antivirus con exclusion controlada para el ejecutable oficial firmado (si aplica en tu politica interna).

## 4. Flujo Correcto de Inicio
1. Abrir KernelIA.
2. Iniciar sesion con el rol minimo necesario:
   - `Viewer`: solo consulta.
   - `PowerUser`: mantenimiento y acciones operativas no criticas.
   - `MegaBoss`: operaciones criticas bajo control.
3. Verificar modelo activo en selector superior.
4. Ejecutar una consulta simple de validacion:
   - `Di OK si estas operativo`
5. Confirmar respuesta antes de usar herramientas sensibles.

## 5. Forma Correcta de Pedir Acciones
Usar lenguaje especifico y verificable:
1. Indicar objetivo.
2. Indicar alcance.
3. Indicar restriccion de riesgo.
4. Pedir salida estructurada.

Ejemplo correcto:
`Diagnostica conectividad de red local. No ejecutes cambios. Entrega causas probables, pruebas y prioridad.`

Ejemplo incorrecto:
`Arregla todo ya`

## 6. Uso Correcto por Modulo

### 6.1 Telemetria
1. Esperar carga completa del panel.
2. Validar CPU, RAM, disco y red antes de ejecutar mantenimiento.
3. Si hay reinicio visual continuo, no lanzar nuevas tareas; revisar estado del modelo y logs.

### 6.2 Auditoria
1. Abrir panel de auditoria y esperar sincronizacion.
2. Si tarda mas de 30 segundos, recargar una sola vez.
3. No abrir/cerrar repetidamente el modal mientras sincroniza.
4. Exportar o revisar trazas antes de ejecutar reparaciones.

### 6.3 Mantenimiento
1. Ejecutar primero `analisis` y luego `reparacion`.
2. Confirmar alcance de cada accion (temporales, red, registro, etc.).
3. En tareas de riesgo alto, exigir confirmacion de privilegio alto.

### 6.4 Cloud/Modelos
1. Si usas cloud, validar API key y cuota antes de pruebas masivas.
2. Si hay error `429`, esperar y reintentar; no enviar rafagas.
3. Para pruebas intensivas usar local Gemma3/Gemma4.

## 7. Privilegios y Seguridad Operativa
1. No trabajar siempre como `MegaBoss`.
2. Elevar privilegios solo para acciones criticas.
3. Para acciones de alto riesgo, requerir contraseÃ±a `MegaBoss` y registrar motivo.
4. Toda accion critica debe dejar auditoria con:
   - usuario,
   - fecha/hora,
   - tool,
   - resultado,
   - rollback sugerido.

## 8. Manejo Correcto de Errores Comunes

### 8.1 Error de cuota cloud (429)
1. Confirmar proveedor/modelo activo.
2. Esperar ventana de reintento.
3. Cambiar temporalmente a modelo local.

### 8.2 Mensaje de permisos denegados
1. Revisar rol actual.
2. Reintentar con rol autorizado.
3. Si corresponde, elevar a flujo `MegaBoss` con confirmacion.

### 8.3 Pantalla en sincronizando sin fin
1. Cerrar solo una vez el panel.
2. Reabrir y esperar.
3. Si persiste, reiniciar app y revisar logs de auditoria.

### 8.4 Respuesta vacia del modelo
1. Verificar endpoint local y modelo.
2. Reintentar consulta corta.
3. Cambiar a modelo alterno (`gemma3` o `gemma4`) para aislar fallo.

## 9. Estandar de Calidad Operativa (Luz Verde)
KernelIA queda en estado operativo si cumple:
1. Inicio de app sin crash.
2. Chat responde consultas simples y tecnicas.
3. Telemetria carga estable.
4. Auditoria no queda bloqueada.
5. Mantenimiento ejecuta analisis y reporta resultado.
6. QA local modelos >= `9/10` por modelo.

## 10. Buenas Practicas de Uso Diario
1. Ejecutar un chequeo rapido al inicio del turno.
2. Registrar incidentes repetidos con evidencia.
3. Separar claramente diagnostico de remediacion.
4. No ejecutar acciones destructivas sin aprobacion.
5. Mantener endpoint/modelos y configuracion documentados.

## 11. Checklist Rapido (Operacion Segura)
1. Rol correcto.
2. Modelo correcto.
3. Endpoint correcto.
4. Consulta de validacion OK.
5. Auditoria activa.
6. Confirmacion previa en tareas criticas.

## 12. Referencias Internas
1. `docs/MANUAL_USUARIO_KERNELIA.md`
2. `docs/KERNELIA_PRODUCCION_QA_MASTER_PLAN.md`
3. `docs/ARCH_SECURITY_RBAC.md`
4. `docs/KERNELIA_ACCESO_AAA.md`




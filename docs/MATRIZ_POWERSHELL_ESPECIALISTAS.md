# Matriz de Comandos PowerShell por Agente Especialista de Soporte en KernelIA

Este documento define la matriz oficial de comandos nativos de **PowerShell** integrados en el catálogo de KernelIA para cada uno de los 8 **Agentes Especialistas** de soporte técnico en Windows.

---

## Niveles de Riesgo y Gobernanza (RBAC / Kernel Guardrails)

- **`R0` (Lectura)**: Consultas de diagnóstico pasivas sin modificación del sistema. Ejecución libre.
- **`R1` (Diagnóstico Avanzado)**: Escaneos e inspecciones activas sin impacto destructivo.
- **`R2` (Remediación Segura)**: Acciones correctivas no destructivas (ej. reiniciar un servicio o limpiar caché DNS).
- **`R3` (Operación Sensible)**: Modificaciones de configuración del sistema o reparación de volúmenes. Requiere ventana o confirmación.
- **`R4` (Operación Crítica)**: Reinicios del sistema, borrado forzoso o cambios de energía. Requiere perfil Megaboss / Admin.

---

## 1. NetworkAgent (Redes y Conectividad)

| ID Comando | Cmdlet PowerShell / Template | Descripción | Riesgo |
|---|---|---|---|
| `ps_get_net_ip_config` | `Get-NetIPConfiguration` | Resumen técnico completo de adaptadores, IP, gateway y DNS. | `R0` |
| `ps_get_net_adapter` | `Get-NetAdapter` | Estado físico, interfaz y velocidad de adaptadores de red. | `R0` |
| `ps_test_net_connection` | `Test-NetConnection -ComputerName {host} -Port {port}` | Diagnóstico de conectividad TCP, ping y traceroute. | `R1` |
| `ps_test_connection` | `Test-Connection -ComputerName {host} -Count 4` | Medición de latencia ICMP e historial de respuesta. | `R1` |
| `ps_resolve_dns_name` | `Resolve-DnsName -Name {domain}` | Consulta de registros DNS (A, AAAA, MX, TXT, CNAME). | `R1` |
| `ps_get_net_tcp_connection`| `Get-NetTCPConnection -State Established,Listen` | Mapeo de conexiones TCP activas y sockets abiertos. | `R0` |
| `ps_clear_dns_cache` | `Clear-DnsClientCache` | Vaciado y refresco de la caché resolver DNS local. | `R2` |
| `ps_restart_net_adapter` | `Restart-NetAdapter -Name {adapter_name}` | Reinicio seguro del controlador de adaptador de red. | `R2` |

---

## 2. DriversAgent (Controladores y Hardware)

| ID Comando | Cmdlet PowerShell / Template | Descripción | Riesgo |
|---|---|---|---|
| `ps_get_pnp_device` | `Get-PnpDevice -PresentOnly` | Inventario de dispositivos Plug and Play activos. | `R0` |
| `ps_get_pnp_device_errors` | `Get-PnpDevice -Status Error,Unknown` | Detección de hardware con Código 43, 10 o falta de driver. | `R1` |
| `ps_get_pnp_device_prop` | `Get-PnpDeviceProperty -InstanceId {id}` | Inspección profunda de propiedades de driver y hardware. | `R0` |
| `ps_pnputil_enum_drivers` | `pnputil /enum-drivers` | Catálogo de paquetes de controladores `.inf` instalados. | `R0` |
| `ps_enable_pnp_device` | `Enable-PnpDevice -InstanceId {id} -Confirm:$false` | Habilita un dispositivo PnP deshabilitado. | `R2` |
| `ps_disable_pnp_device` | `Disable-PnpDevice -InstanceId {id} -Confirm:$false` | Deshabilita un dispositivo PnP conflictivo. | `R3` |

---

## 3. ServicesAgent (Servicios de Windows y Spooler)

| ID Comando | Cmdlet PowerShell / Template | Descripción | Riesgo |
|---|---|---|---|
| `ps_get_service` | `Get-Service` | Lista de todos los servicios de Windows y su estado. | `R0` |
| `ps_get_failed_services` | `Get-Service \| Where-Object {$_.StartType -eq 'Automatic' -and $_.Status -ne 'Running'}` | Detecta servicios de inicio automático que están detenidos. | `R1` |
| `ps_start_service` | `Start-Service -Name {service_name}` | Inicia un servicio detenido. | `R2` |
| `ps_stop_service` | `Stop-Service -Name {service_name}` | Detiene un servicio en ejecución. | `R2` |
| `ps_restart_service` | `Restart-Service -Name {service_name}` | Reinicia un servicio operativo o bloqueado. | `R2` |
| `ps_clear_spooler_jobs` | `Stop-Service Spooler; Remove-Item $env:SystemRoot\System32\spool\PRINTERS\* -Force; Start-Service Spooler` | Purga completa de cola de impresión atascada. | `R2` |

---

## 4. ProcessAgent (Procesos y Recursos)

| ID Comando | Cmdlet PowerShell / Template | Descripción | Riesgo |
|---|---|---|---|
| `ps_get_process_cpu` | `Get-Process \| Sort-Object CPU -Descending \| Select-Object -First 10` | Top 10 procesos con mayor consumo acumulado de CPU. | `R0` |
| `ps_get_process_ram` | `Get-Process \| Sort-Object WorkingSet64 -Descending \| Select-Object -First 10` | Top 10 procesos con mayor consumo de memoria RAM. | `R0` |
| `ps_get_process_detail` | `Get-CimInstance Win32_Process -Filter "ProcessId = {pid}"` | Extrae la línea de comandos ejecutable y ruta completa. | `R0` |
| `ps_stop_process` | `Stop-Process -Id {pid} -Force` | Terminación forzosa de procesos colgados. | `R2` |

---

## 5. PerformanceAgent (Rendimiento y Métrica de Sistema)

| Comandos PowerShell (Cmdlets) | Propósito Técnico | Riesgo |
|---|---|---|
| `ps_get_counter_cpu` | `Get-Counter '\Processor(_Total)\% Processor Time'` | Métrica en tiempo real del uso de CPU. | `R0` |
| `ps_get_counter_ram` | `Get-Counter '\Memory\Available MBytes'` | Métrica en tiempo real de RAM libre en MB. | `R0` |
| `ps_get_counter_disk` | `Get-Counter '\PhysicalDisk(_Total)\% Disk Time'` | Métrica en tiempo real del nivel de ocupación de disco. | `R0` |
| `ps_get_os_uptime` | `Get-CimInstance Win32_OperatingSystem \| Select-Object LastBootUpTime` | Cálculo exacto de Uptime del sistema. | `R0` |

---

## 6. SecurityAgent (Seguridad, Auditoría y Guardrails)

| ID Comando | Cmdlet PowerShell / Template | Descripción | Riesgo |
|---|---|---|---|
| `ps_get_defender_status` | `Get-MpComputerStatus` | Inspección de Windows Defender y firmas activas. | `R0` |
| `ps_get_defender_threats` | `Get-MpThreat` | Registro de amenazas detectadas en el equipo. | `R0` |
| `ps_audit_failed_logins` | `Get-WinEvent -FilterHashtable @{LogName='Security'; ID=4625} -MaxEvents 20` | Auditoría de intentos fallidos de inicio de sesión. | `R1` |
| `ps_audit_critical_events` | `Get-WinEvent -FilterHashtable @{LogName='System'; Level=1,2} -MaxEvents 30` | Auditoría de eventos con nivel de Error o Crítico. | `R1` |
| `ps_get_firewall_profiles` | `Get-NetFirewallProfile` | Estado de los perfiles del Firewall de Windows. | `R0` |

---

## 7. FilesystemAgent (Almacenamiento y Volúmenes)

| ID Comando | Cmdlet PowerShell / Template | Descripción | Riesgo |
|---|---|---|---|
| `ps_get_volume` | `Get-Volume` | Inspección de unidades de disco, tipo de FS y espacio libre. | `R0` |
| `ps_get_physical_disk` | `Get-PhysicalDisk` | Diagnóstico de salud SMART de discos físicos (SSD/HDD). | `R0` |
| `ps_repair_volume_scan` | `Repair-Volume -DriveLetter {letter} -Scan` | Escaneo del sistema de archivos sin desmontar la unidad. | `R1` |
| `ps_optimize_volume` | `Optimize-Volume -DriveLetter {letter} -Defrag -Verbose` | Optimización de volumen (TRIM en SSD / Defrag en HDD). | `R2` |

---

## 8. SystemAgent (Diagnóstico Consolidado e Inventario)

| ID Comando | Cmdlet PowerShell / Template | Descripción | Riesgo |
|---|---|---|---|
| `ps_get_computer_info` | `Get-ComputerInfo` | Informe técnico consolidado del sistema operativo y hardware. | `R0` |
| `ps_get_hotfix` | `Get-HotFix` | Lista de parches de seguridad KB instalados en Windows. | `R0` |
| `ps_sfc_scan` | `sfc /scannow` | Verificación de la integridad de los archivos del Kernel de Windows. | `R1` |
| `ps_dism_checkhealth` | `DISM /Online /Cleanup-Image /CheckHealth` | Chequeo de salud del almacén de componentes de Windows. | `R1` |
| `ps_dism_restorehealth` | `DISM /Online /Cleanup-Image /RestoreHealth` | Reparación de corrupción de la imagen de Windows. | `R3` |

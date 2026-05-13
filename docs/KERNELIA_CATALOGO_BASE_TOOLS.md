# KernelIA - Catalogo Base de Tools

Este documento carga el catalogo base solicitado como referencia oficial de producto.

## 1. Informacion del sistema - Viewer
- get_system_info
- get_os_info
- get_hostname
- get_current_user
- get_uptime
- get_cpu_info
- get_memory_info
- get_disk_info
- get_gpu_info
- get_battery_info
- get_public_ip
- get_local_ip
- get_network_adapters
- get_environment_info

## 2. Telemetria en tiempo real - Viewer
- get_cpu_usage
- get_memory_usage
- get_disk_usage
- get_network_usage
- get_top_processes
- get_running_services
- get_startup_programs
- get_installed_programs
- get_windows_updates_status

## 3. Red e Internet - Viewer / PowerUser
- ping_host
- traceroute_host
- dns_lookup
- test_tcp_port
- get_public_ip
- get_local_ip
- get_wifi_info
- get_default_gateway
- get_dns_servers
- flush_dns_cache
- renew_ip_config
- release_ip_config
- reset_network_stack

## 4. Procesos - Viewer / PowerUser
- list_processes
- get_process_detail
- kill_process
- restart_process
- find_high_cpu_processes
- find_high_memory_processes

## 5. Servicios Windows - Viewer / PowerUser
- list_services
- get_service_status
- start_service
- stop_service
- restart_service
- enable_service
- disable_service

## 6. Mantenimiento basico - PowerUser
- clean_temp_files
- empty_recycle_bin
- run_disk_cleanup
- clear_browser_cache
- clear_windows_update_cache
- check_disk_health
- scan_system_files
- repair_system_files
- run_dism_health_check
- run_dism_restore_health

## 7. Seguridad local - Viewer / PowerUser
- get_firewall_status
- list_firewall_rules
- enable_firewall
- disable_firewall
- get_defender_status
- run_defender_quick_scan
- run_defender_full_scan
- get_antivirus_status
- get_security_center_status
- list_open_ports
- list_listening_connections
- list_active_connections

## 8. Drivers - Viewer / PowerUser
- list_devices
- list_problem_devices
- get_device_detail
- get_driver_info
- update_driver
- open_optional_driver_updates
- rescan_devices

## 9. Archivos y carpetas - Viewer / PowerUser
- list_directory
- get_file_info
- search_files
- create_folder
- delete_file
- move_file
- copy_file
- rename_file
- calculate_folder_size

## 10. Logs y auditoria - Viewer
- read_event_logs
- read_system_log
- read_application_log
- read_security_log
- export_event_logs
- get_kernelia_audit_log
- search_kernelia_audit_log

## 11. Energia y rendimiento - Viewer / PowerUser
- get_power_plan
- set_power_plan
- list_power_plans
- get_sleep_settings
- set_sleep_settings
- get_startup_impact
- optimize_startup_apps

## 12. Software instalado - Viewer / PowerUser
- list_installed_apps
- get_app_detail
- uninstall_app
- check_app_updates
- list_windows_features
- enable_windows_feature
- disable_windows_feature

## 13. Comandos sensibles - Owner
- run_shell_command
- run_powershell_command
- edit_registry_key
- delete_registry_key
- create_local_user
- delete_local_user
- reset_user_password
- add_user_to_group
- remove_user_from_group
- change_firewall_rule
- change_network_adapter_config
- reboot_system
- shutdown_system

## 14. MegaBoss - Owner + contrasena temporal
- run_elevated_command
- force_kill_process
- force_delete_file
- modify_system_registry
- disable_security_component
- enable_security_component
- reset_windows_network_stack
- repair_windows_image
- execute_admin_script

## Regla clave cargada
- `get_public_ip` debe existir en el registry de tools antes de pasar por RBAC.
- Roles permitidos: Viewer, PowerUser, Owner.
- No requiere MegaBoss.
- Es tool de solo lectura.

/**
 * Formatea el nombre de un tool para mostrarlo en la UI
 * @param {string} toolName
 * @returns {string}
 */
export function formatToolName(toolName) {
  const names = {
    run_command: '⚡ Ejecutar comando',
    read_file: '📖 Leer archivo',
    write_file: '📝 Escribir archivo',
    list_directory: '📁 Listar directorio',
    get_system_info: '🖥️ Info del sistema',
    list_processes: '📊 Listar procesos',
    kill_process: '⛔ Cerrar proceso',
  };
  return names[toolName] || `🔧 ${toolName}`;
}

/**
 * Formatea bytes a una cadena legible
 * @param {number} bytes
 * @returns {string}
 */
export function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

/**
 * Trunca un texto a N caracteres
 * @param {string} text
 * @param {number} maxLength
 * @returns {string}
 */
export function truncate(text, maxLength = 100) {
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength) + '…';
}

/**
 * Extrae un resumen corto de los argumentos de un tool
 * @param {string} argsJson
 * @returns {string}
 */
export function summarizeToolArgs(argsJson) {
  try {
    const args = JSON.parse(argsJson);
    if (args.command) return truncate(args.command, 60);
    if (args.path) return truncate(args.path, 60);
    return truncate(JSON.stringify(args), 60);
  } catch {
    return truncate(argsJson, 60);
  }
}

import { invokeWithPolicy } from '$lib/utils/invoke.js';

export async function sendMessage(message) {
  return invokeWithPolicy('send_message', { message }, { timeoutMs: 120000, retries: 1 });
}

export async function clearChat() {
  return invokeWithPolicy('clear_chat', {}, { timeoutMs: 10000, retries: 0 });
}

export async function getSettings() {
  return invokeWithPolicy('get_settings');
}

export async function updateSettings(settings) {
  return invokeWithPolicy('update_settings', { settings });
}

export async function getModels() {
  return invokeWithPolicy('get_models');
}

export async function setModel(modelId) {
  return invokeWithPolicy('set_model', { model_id: modelId }, { timeoutMs: 15000, retries: 0 });
}

export async function setApiKey(modelId, apiKey) {
  return invokeWithPolicy('set_api_key', {
    model_id: modelId,
    api_key: apiKey,
  });
}

export async function listProcesses(sortBy = 'memory', limit = 5) {
  return invokeWithPolicy('list_processes', {
    sort_by: sortBy,
    limit,
  });
}

export async function runNetworkDiagnostic() {
  return invokeWithPolicy('run_network_diagnostic');
}

export async function listRunningServices() {
  return invokeWithPolicy('list_running_services');
}

export async function generateSupportReport() {
  return invokeWithPolicy('generate_support_report');
}

export async function runQuickCheck(checkId) {
  return invokeWithPolicy('run_quick_check', {
    quick_check_id: checkId,
  }, { timeoutMs: 45000, retries: 0 });
}

export async function getQuickChecks() {
  return invokeWithPolicy('get_quick_checks');
}

export async function getAuditLogs() {
  return invokeWithPolicy('get_audit_logs');
}

export async function loginUser(username, password) {
  return invokeWithPolicy('login_user', { username, password }, { timeoutMs: 10000, retries: 0 });
}

export async function logoutUser() {
  return invokeWithPolicy('logout_user', {}, { timeoutMs: 8000, retries: 0 });
}

export async function getAuthStatus() {
  return invokeWithPolicy('get_auth_status', {}, { timeoutMs: 8000, retries: 0 });
}

export async function unlockTecnicoCritical(password, minutes = 15) {
  return invokeWithPolicy('unlock_tecnico_critical', { password, minutes });
}

export async function listSupportUsers() {
  return invokeWithPolicy('list_support_users');
}

export async function createSupportUser(username, password, profile) {
  return invokeWithPolicy('create_support_user', { username, password, profile });
}

export async function deleteSupportUser(username) {
  return invokeWithPolicy('delete_support_user', { username });
}

export async function confirmSolutionAndIngest(query, solution, specialty) {
  return invokeWithPolicy('confirm_solution_and_ingest', { query, solution, specialty });
}

export async function createSupportTicket(query, specialty, telemetry) {
  return invokeWithPolicy('create_support_ticket_cmd', { query, specialty, telemetry });
}

export async function listSupportTickets() {
  return invokeWithPolicy('list_support_tickets_cmd');
}

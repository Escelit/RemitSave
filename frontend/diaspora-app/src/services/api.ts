const API_BASE = 'http://localhost:3002';

let authToken: string | null = null;

export function setAuthToken(token: string | null) {
  authToken = token;
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  if (authToken) {
    headers['Authorization'] = `Bearer ${authToken}`;
  }
  const res = await fetch(`${API_BASE}${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    const err = await res.text();
    throw new Error(`API ${method} ${path} ${res.status}: ${err}`);
  }
  return res.json();
}

export interface RemittanceRule {
  id: string;
  beneficiary: string;
  incoming_asset: string;
  local_asset: string;
  split_type: 'Percentage' | 'Fixed';
  split_value: number;
  savings_plan_id: string | null;
  active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateRuleRequest {
  beneficiary: string;
  incoming_asset: string;
  local_asset: string;
  split_type: 'Percentage' | 'Fixed';
  split_value: number;
  savings_plan_id?: string;
}

export interface ExecuteRemittanceRequest {
  rule_id: string;
  total_amount: number;
}

export interface ExecuteRemittanceResponse {
  remittance_id: string;
  status: string;
  payout_amount: number;
  savings_amount: number;
  fee_amount: number;
  tx_hash: string;
}

export interface RemittanceEvent {
  id: string;
  remittance_id: number;
  beneficiary: string;
  total_amount: number;
  payout_amount: number;
  savings_amount: number;
  fee_amount: number;
  incoming_asset: string;
  local_asset: string;
  status: string;
  tx_hash: string | null;
  created_at: string;
}

export interface SavingsPlan {
  id: string;
  onchain_plan_id: number | null;
  goal_name: string;
  target_amount: number;
  current_balance: number;
  local_asset: string;
  lock_until: string | null;
  status: 'Active' | 'Completed' | 'Withdrawn' | 'Closed';
  created_at: string;
  updated_at: string;
}

export interface CreateSavingsPlanRequest {
  goal_name: string;
  target_amount: number;
  local_asset: string;
  lock_until?: string | null;
}

export interface PlanActionResult {
  id: string;
  current_balance: number;
  status: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  user: {
    id: string;
    email: string;
    phone: string;
    country: string;
    kyc_level: number;
    stellar_address?: string | null;
    created_at: string;
    last_active: string;
  };
  token: string;
}

export const api = {
  login: (data: LoginRequest) =>
    request<LoginResponse>('POST', '/auth/login', data),

  createRule: (data: CreateRuleRequest) =>
    request<RemittanceRule>('POST', '/remit/rules', data),

  listRules: () =>
    request<RemittanceRule[]>('GET', '/remit/rules'),

  getRule: (id: string) =>
    request<RemittanceRule>('GET', `/remit/rules/${id}`),

  updateRule: (id: string, data: Partial<CreateRuleRequest & { active: boolean }>) =>
    request<RemittanceRule>('PUT', `/remit/rules/${id}`, data),

  deleteRule: (id: string) =>
    request<{ deleted: boolean }>('DELETE', `/remit/rules/${id}`),

  executeRemittance: (data: ExecuteRemittanceRequest) =>
    request<ExecuteRemittanceResponse>('POST', '/remit/execute', data),

  getHistory: (limit = 50, offset = 0) =>
    request<RemittanceEvent[]>('GET', `/remit/history?limit=${limit}&offset=${offset}`),

  listSavingsPlans: () =>
    request<SavingsPlan[]>('GET', '/savings/plans'),

  createSavingsPlan: (data: CreateSavingsPlanRequest) =>
    request<SavingsPlan>('POST', '/savings/plans', data),

  getSavingsPlan: (id: string) =>
    request<SavingsPlan>('GET', `/savings/plans/${id}`),

  depositToPlan: (id: string, amount: number) =>
    request<PlanActionResult>('POST', `/savings/plans/${id}/deposit`, { amount }),

  withdrawFromPlan: (id: string, amount: number) =>
    request<PlanActionResult>('POST', `/savings/plans/${id}/withdraw`, { amount }),
};

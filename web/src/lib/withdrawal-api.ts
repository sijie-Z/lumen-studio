import { PaginatedResponse, request } from "./api";

export interface WithdrawalDto {
  id: number;
  creator_id: number;
  amount: string;
  fee: string;
  actual_amount: string | null;
  status: string;
  account_info: unknown | null;
  reviewed_by: number | null;
  reviewed_at: string | null;
  review_note: string | null;
  completed_at: string | null;
  created_at: string;
}

export async function applyWithdrawal(amount: string, accountInfo?: unknown) {
  return request<WithdrawalDto>("/withdrawals", {
    method: "POST",
    body: JSON.stringify({ amount, account_info: accountInfo }),
    auth: true
  });
}

export async function listMyWithdrawals() {
  return request<WithdrawalDto[]>("/withdrawals", { auth: true });
}

export async function listAllWithdrawals(params: { page: number; page_size: number }) {
  const query = new URLSearchParams({
    page: String(params.page),
    page_size: String(params.page_size)
  });
  return request<PaginatedResponse<WithdrawalDto>>(`/admin/withdrawals?${query}`, { auth: true });
}

export async function reviewWithdrawal(id: number, approve: boolean, note?: string) {
  return request<WithdrawalDto>(`/admin/withdrawals/${id}`, {
    method: "PATCH",
    body: JSON.stringify({ approve, note }),
    auth: true
  });
}

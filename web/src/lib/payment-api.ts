import { request } from "./api";

export interface PaymentDto {
  id: number;
  appointment_id: number | null;
  user_id: number;
  amount: string;
  method: string | null;
  status: string;
  payment_type: string;
  tx_id: string | null;
  expire_time: string | null;
  payment_channel: string | null;
  refund_amount: string | null;
  refund_reason: string | null;
  created_at: string;
}

export async function recharge(amount: string) {
  return request<PaymentDto>("/payments/recharge", {
    method: "POST",
    body: JSON.stringify({ amount }),
    auth: true
  });
}

export async function payAppointment(id: number, method = "balance") {
  return request<PaymentDto>(`/payments/appointments/${id}/pay`, {
    method: "POST",
    body: JSON.stringify({ method }),
    auth: true
  });
}

export async function listPayments() {
  return request<PaymentDto[]>("/payments", { auth: true });
}

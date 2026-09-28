import { PaginatedResponse, request } from "./api";

export interface AdminUser {
  id: number;
  username: string;
  nickname: string;
  role: string;
  status: string;
  created_at: string;
}

export interface PlatformStats {
  users: number;
  creators: number;
  services: number;
  appointments: number;
  works: number;
  gross_volume: string;
}

export async function listUsers(params: { page: number; page_size: number }) {
  const query = new URLSearchParams({
    page: String(params.page),
    page_size: String(params.page_size)
  });
  return request<PaginatedResponse<AdminUser>>(`/admin/users?${query}`, { auth: true });
}

export async function getStats() {
  return request<PlatformStats>("/admin/stats", { auth: true });
}

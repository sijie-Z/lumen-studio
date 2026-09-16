import { request } from "./api";

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

export async function listUsers() {
  return request<AdminUser[]>("/admin/users", { auth: true });
}

export async function getStats() {
  return request<PlatformStats>("/admin/stats", { auth: true });
}

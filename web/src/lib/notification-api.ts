import { request } from "./api";

export interface NotificationDto {
  id: number;
  user_id: number;
  type: string;
  title: string;
  content: string | null;
  priority: string;
  action_url: string | null;
  is_read: boolean;
  created_at: string;
}

export function listNotifications(limit = 20) {
  return request<NotificationDto[]>(`/notifications?limit=${limit}`, { auth: true });
}

export function unreadCount() {
  return request<{ count: number }>("/notifications/unread-count", { auth: true });
}

export function markRead(id: number) {
  return request<void>(`/notifications/${id}/read`, {
    method: "PATCH",
    auth: true
  });
}

export function markAllRead() {
  return request<void>("/notifications/read-all", {
    method: "PATCH",
    auth: true
  });
}

import { request } from "./api";

export type FavoriteTargetType = "work" | "service" | "creator";

export interface FavoriteStateDto {
  favorited: boolean;
  count: number;
}

export interface FavoriteDto {
  id: number;
  target_type: FavoriteTargetType;
  target_id: number;
  title: string;
  cover_image_url?: string | null;
  subtitle?: string | null;
  created_at: string;
}

export function toggleFavorite(targetType: FavoriteTargetType, targetId: number) {
  return request<FavoriteStateDto>("/favorites/toggle", {
    method: "POST",
    body: JSON.stringify({ target_type: targetType, target_id: targetId }),
    auth: true
  });
}

export function listFavorites(targetType?: FavoriteTargetType) {
  const query = targetType ? `?target_type=${targetType}` : "";
  return request<FavoriteDto[]>(`/favorites${query}`, { auth: true });
}

export function favoriteStatus(targetType: FavoriteTargetType, targetId: number) {
  const query = new URLSearchParams({
    target_type: targetType,
    target_id: String(targetId)
  });
  return request<FavoriteStateDto>(`/favorites/status?${query.toString()}`, {
    auth: true
  });
}

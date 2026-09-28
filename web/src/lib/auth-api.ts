import { getToken, request, setToken } from "./api";

export interface UserDto {
  id: number;
  username: string;
  nickname: string;
  avatar_url: string | null;
  email: string | null;
  phone: string | null;
  balance: string;
  role: string;
  roles: string[];
  status: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  user: UserDto;
}

export interface UploadResult {
  url: string;
  filename: string;
  content_type: string;
  size: number;
}

export async function login(account: string, password: string) {
  const result = await request<AuthResponse>("/auth/login", {
    method: "POST",
    body: JSON.stringify({ account, password })
  });
  setToken(result.access_token);
  return result;
}

export async function register(input: {
  username: string;
  password: string;
  email?: string;
  phone?: string;
  nickname?: string;
}) {
  return request<UserDto>("/auth/register", {
    method: "POST",
    body: JSON.stringify(input)
  });
}

export async function fetchMe() {
  return request<UserDto>("/auth/me", { auth: true });
}

export async function uploadImage(file: File) {
  const form = new FormData();
  form.append("file", file);
  return request<UploadResult>("/uploads", {
    method: "POST",
    body: form,
    auth: true
  });
}

export function isAuthenticated(): boolean {
  return Boolean(getToken());
}

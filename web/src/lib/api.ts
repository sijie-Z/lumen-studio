export const API_BASE = "/api/v1";

export interface ApiEnvelope<T> {
  code: number;
  message: string;
  data: T;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  has_next: boolean;
}

export class ApiError extends Error {
  status: number;
  code: number;

  constructor(status: number, code: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
  }
}

interface RequestOptions extends RequestInit {
  auth?: boolean;
  raw?: boolean;
}

export async function request<T>(
  path: string,
  options: RequestOptions = {}
): Promise<T> {
  const headers = new Headers(options.headers);
  if (!(options.body instanceof FormData) && options.body && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }
  if (options.auth) {
    const token = getToken();
    if (token) headers.set("Authorization", `Bearer ${token}`);
  }

  const response = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers
  });

  const body = await response.json().catch(() => null);
  if (!response.ok) {
    throw new ApiError(
      response.status,
      body?.code ?? response.status,
      body?.message ?? response.statusText
    );
  }

  return body?.data as T;
}

export function getToken(): string | null {
  return localStorage.getItem("lumina.token");
}

export function setToken(token: string | null): void {
  if (token) localStorage.setItem("lumina.token", token);
  else localStorage.removeItem("lumina.token");
}

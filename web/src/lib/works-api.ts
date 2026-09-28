import { PaginatedResponse, request } from "./api";

export interface WorkDto {
  id: number;
  user_id: number;
  image_url: string;
  title: string | null;
  description: string | null;
  category: string | null;
  creator_name: string;
  created_at: string;
}

export interface ListWorksParams {
  page?: number;
  page_size?: number;
  category?: string;
  creator_id?: number;
  q?: string;
}

export async function listWorksPage(
  params: ListWorksParams = {}
): Promise<PaginatedResponse<WorkDto>> {
  const search = new URLSearchParams();
  if (params.page) search.set("page", String(params.page));
  if (params.page_size) search.set("page_size", String(params.page_size));
  if (params.category?.trim()) search.set("category", params.category.trim());
  if (params.creator_id) search.set("creator_id", String(params.creator_id));
  if (params.q?.trim()) search.set("q", params.q.trim());
  const query = search.toString();
  const suffix = query ? `?${query}` : "";
  return request<PaginatedResponse<WorkDto>>(`/works${suffix}`);
}

export async function getWork(id: number | string): Promise<WorkDto> {
  return request<WorkDto>(`/works/${id}`);
}

export async function listWorks(): Promise<WorkDto[]> {
  const response = await listWorksPage({ page: 1, page_size: 100 });
  return response.items;
}

export async function createWork(input: {
  image_url: string;
  title?: string;
  description?: string;
  category?: string;
}) {
  return request<WorkDto>("/works", {
    method: "POST",
    body: JSON.stringify(input),
    auth: true
  });
}

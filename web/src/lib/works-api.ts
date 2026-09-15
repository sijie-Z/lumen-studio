import { request } from "./api";

export interface WorkDto {
  id: number;
  user_id: number;
  image_url: string;
  title: string | null;
  description: string | null;
  created_at: string;
}

export async function listWorks() {
  return request<WorkDto[]>("/works");
}

export async function createWork(input: {
  image_url: string;
  title?: string;
  description?: string;
}) {
  return request<WorkDto>("/works", {
    method: "POST",
    body: JSON.stringify(input),
    auth: true
  });
}

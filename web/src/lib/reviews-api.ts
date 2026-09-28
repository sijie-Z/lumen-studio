import { PaginatedResponse, request } from "./api";

export interface ReviewDto {
  id: number;
  appointment_id: number;
  creator_id: number;
  rating: string;
  content: string | null;
  images: unknown;
  is_anonymous: boolean;
  photographer_reply: string | null;
  replied_at: string | null;
  created_at: string;
}

export interface CreateReviewInput {
  appointment_id: number;
  rating: number;
  content?: string;
  is_anonymous?: boolean;
}

export async function createReview(input: CreateReviewInput) {
  return request<ReviewDto>("/reviews", {
    method: "POST",
    body: JSON.stringify(input),
    auth: true
  });
}

export async function listCreatorReviews(
  id: number | string,
  params: { page: number; page_size: number }
) {
  const query = new URLSearchParams({
    page: String(params.page),
    page_size: String(params.page_size)
  });
  return request<PaginatedResponse<ReviewDto>>(`/reviews/creator/${id}?${query}`);
}

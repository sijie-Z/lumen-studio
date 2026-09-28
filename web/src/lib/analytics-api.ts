import { request } from "./api";

export interface RevenueByMonthDto {
  month: string;
  amount: string;
}

export interface AppointmentStatusCountDto {
  status: string;
  count: number;
}

export interface RatingDistributionDto {
  rating: number;
  count: number;
}

export interface CreatorAnalyticsDto {
  total_income: string;
  total_appointments: number;
  completed_appointments: number;
  pending_appointments: number;
  completion_rate: string;
  avg_rating: string;
  review_count: number;
  pending_withdrawal_amount: string;
  revenue_by_month: RevenueByMonthDto[];
  appointments_by_status: AppointmentStatusCountDto[];
  rating_distribution: RatingDistributionDto[];
}

export async function getCreatorAnalytics() {
  return request<CreatorAnalyticsDto>("/creator/analytics", { auth: true });
}

import { request } from "./api";

export interface CreatorProfile {
  id: number;
  user_id: number;
  introduction: string | null;
  bio: string | null;
  rating: string;
  certification_level: string;
  service_areas: unknown;
  style_vector_id: string | null;
  portfolio_url: string | null;
  total_services: number;
  total_appointments: number;
  total_income: string;
  avg_rating: string;
  created_at: string;
  updated_at: string;
}

export interface ServiceType {
  id: number;
  name: string;
  description: string | null;
}

export interface Service {
  id: number;
  creator_id: number;
  type_id: number;
  title: string;
  description: string | null;
  price: string;
  duration: number | null;
  cover_image_url: string | null;
  location: string | null;
  tags: string | null;
  options: unknown;
  is_active: boolean;
  is_featured: boolean;
  appointments_count: number;
  created_at: string;
  updated_at: string;
}

export interface Appointment {
  id: number;
  user_id: number;
  creator_id: number;
  service_id: number;
  appointment_date: string;
  start_time: string;
  end_time: string;
  location: string | null;
  status: string;
  total_price: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface UpsertProfileInput {
  introduction?: string;
  bio?: string;
  service_areas?: unknown;
  portfolio_url?: string;
}

export interface CreateServiceInput {
  type_id: number;
  title: string;
  description?: string;
  price: string;
  duration?: number;
  cover_image_url?: string;
  location?: string;
  tags?: string;
}

export interface CreateAppointmentInput {
  service_id: number;
  start_time: string;
  end_time: string;
  location?: string;
  notes?: string;
}

export async function listCreators() {
  return request<CreatorProfile[]>("/creators");
}

export async function getCreator(id: number | string) {
  return request<CreatorProfile>(`/creators/${id}`);
}

export async function upsertProfile(input: UpsertProfileInput) {
  return request<CreatorProfile>("/creators", {
    method: "POST",
    body: JSON.stringify(input),
    auth: true
  });
}

export async function listServiceTypes() {
  return request<ServiceType[]>("/service-types");
}

export async function listServices() {
  return request<Service[]>("/services");
}

export async function getService(id: number | string) {
  return request<Service>(`/services/${id}`);
}

export async function createService(input: CreateServiceInput) {
  return request<Service>("/services", {
    method: "POST",
    body: JSON.stringify(input),
    auth: true
  });
}

export async function listAppointments() {
  return request<Appointment[]>("/appointments", { auth: true });
}

export async function listCreatorAppointments() {
  return request<Appointment[]>("/appointments/creator", { auth: true });
}

export async function createAppointment(input: CreateAppointmentInput) {
  return request<Appointment>("/appointments", {
    method: "POST",
    body: JSON.stringify(input),
    auth: true
  });
}

export async function transitionAppointment(id: number, status: string) {
  return request<Appointment>(`/appointments/${id}`, {
    method: "PATCH",
    body: JSON.stringify({ status }),
    auth: true
  });
}

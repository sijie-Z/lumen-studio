import { A, useNavigate, useParams } from "@solidjs/router";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { ArrowLeft, CalendarDays, Clock, MapPin, Tag } from "lucide-solid";
import { createSignal, Show } from "solid-js";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { Skeleton } from "../components/ui/skeleton";
import SiteHeader from "../components/layout/site-header";
import { createAppointment, getService, type Service } from "../lib/marketplace-api";
import { isAuthenticated } from "../lib/auth-api";

export default function ServiceDetail() {
  const params = useParams();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const service = createQuery(() => ({
    queryKey: ["service", params.id] as const,
    queryFn: () => getService(params.id ?? "")
  }));
  const [startLocal, setStartLocal] = createSignal("");
  const [location, setLocation] = createSignal("");
  const [notes, setNotes] = createSignal("");
  const [submitting, setSubmitting] = createSignal(false);
  const [error, setError] = createSignal("");
  const [success, setSuccess] = createSignal(false);

  async function submit() {
    if (!isAuthenticated()) {
      navigate("/login");
      return;
    }
    if (!startLocal()) {
      setError("请选择预约时间");
      return;
    }
    setSubmitting(true);
    setError("");
    setSuccess(false);
    try {
      const current = service.data as Service;
      const start = new Date(startLocal()).toISOString();
      const durationMinutes = current.duration ?? 60;
      const end = new Date(new Date(start).getTime() + durationMinutes * 60_000).toISOString();
      await createAppointment({
        service_id: current.id,
        start_time: start,
        end_time: end,
        location: location() || undefined,
        notes: notes() || undefined
      });
      setSuccess(true);
      await queryClient.invalidateQueries({ queryKey: ["appointments"] });
    } catch (err) {
      setError(err instanceof Error ? err.message : "预约失败");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div class="min-h-screen bg-background text-foreground">
      <SiteHeader />
      <main class="mx-auto max-w-5xl px-5 pt-24 pb-24 md:px-8 md:pt-28">
        <A href="/services" class="inline-flex items-center gap-2 text-sm text-muted no-underline hover:text-foreground">
          <ArrowLeft size={16} />
          返回服务列表
        </A>

        <Show
          when={service.data}
          fallback={
            service.isLoading ? (
              <div class="mt-8 space-y-4">
                <Skeleton class="aspect-[16/9] w-full rounded-lg" />
                <Skeleton class="h-9 w-1/2" />
              </div>
            ) : (
              <div class="mt-16 text-center text-muted">服务不存在</div>
            )
          }
        >
          <div class="mt-8 grid gap-8 lg:grid-cols-[1.1fr_0.9fr]">
            <div>
              <div class="overflow-hidden rounded-lg border border-line bg-secondary">
                <img
                  src={service.data!.cover_image_url ?? "/demo/work-4.jpg"}
                  alt={service.data!.title}
                  class="aspect-[16/10] w-full object-cover"
                />
              </div>
              <div class="mt-6">
                <div class="flex flex-wrap items-center gap-3">
                  <h1 class="font-display text-3xl font-semibold">{service.data!.title}</h1>
                  <Badge variant="accent">¥{service.data!.price}</Badge>
                </div>
                <div class="mt-4 flex flex-wrap items-center gap-4 text-sm text-muted">
                  {service.data!.duration && (
                    <span class="flex items-center gap-1.5"><Clock size={15} />{service.data!.duration} 分钟</span>
                  )}
                  {service.data!.location && (
                    <span class="flex items-center gap-1.5"><MapPin size={15} />{service.data!.location}</span>
                  )}
                  {service.data!.tags && (
                    <span class="flex items-center gap-1.5"><Tag size={15} />{service.data!.tags}</span>
                  )}
                </div>
                {service.data!.description && (
                  <p class="mt-6 leading-7 text-muted">{service.data!.description}</p>
                )}
              </div>
            </div>

            <Card class="h-fit">
              <CardHeader>
                <CardTitle class="flex items-center gap-2">
                  <CalendarDays size={18} class="text-primary" />
                  预约档期
                </CardTitle>
              </CardHeader>
              <CardContent class="space-y-4">
                <label class="block">
                  <span class="mb-2 block text-sm text-muted">开始时间</span>
                  <Input
                    type="datetime-local"
                    value={startLocal()}
                    onInput={(event) => setStartLocal(event.currentTarget.value)}
                  />
                </label>
                <label class="block">
                  <span class="mb-2 block text-sm text-muted">拍摄地点</span>
                  <Input
                    value={location()}
                    onInput={(event) => setLocation(event.currentTarget.value)}
                    placeholder="可选"
                  />
                </label>
                <label class="block">
                  <span class="mb-2 block text-sm text-muted">备注</span>
                  <Input
                    value={notes()}
                    onInput={(event) => setNotes(event.currentTarget.value)}
                    placeholder="拍摄需求或偏好"
                  />
                </label>

                {error() && (
                  <div class="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                    {error()}
                  </div>
                )}
                {success() && (
                  <div class="rounded-lg border border-accent/30 bg-accent/10 px-4 py-3 text-sm text-accent">
                    预约已提交，创作者确认后即可锁定档期
                  </div>
                )}

                <Button
                  class="w-full"
                  size="lg"
                  disabled={submitting()}
                  onClick={submit}
                >
                  {submitting() ? "提交中" : "立即预约"}
                </Button>
              </CardContent>
            </Card>
          </div>
        </Show>
      </main>
    </div>
  );
}

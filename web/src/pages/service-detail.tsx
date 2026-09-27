import { A, useNavigate, useParams } from "@solidjs/router";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { ArrowLeft, CalendarDays, Clock, MapPin, Tag } from "lucide-solid";
import { createSignal, Show } from "solid-js";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { EmptyState, ErrorState, LoadingState } from "../components/ui/state";
import { StepGuide, type GuideStep } from "../components/onboarding/step-guide";
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
  const [createdAppointmentId, setCreatedAppointmentId] = createSignal<number | null>(null);
  const bookingSteps = (): GuideStep[] => [
    {
      title: "选择预约时间",
      description: "确认开始时间、拍摄地点和具体需求。",
      done: Boolean(startLocal())
    },
    {
      title: "提交预约",
      description: "提交后预约进入待支付状态。",
      done: success()
    },
    {
      title: "到客户中心支付",
      description: "完成支付后，创作者会收到预约并确认档期。",
      done: false
    }
  ];

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
      const appointment = await createAppointment({
        service_id: current.id,
        start_time: start,
        end_time: end,
        location: location() || undefined,
        notes: notes() || undefined
      });
      setCreatedAppointmentId(appointment.id);
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
      <main class="mx-auto max-w-5xl px-5 pt-24 pb-24 md:px-8 md:pt-28">
        <A href="/services" class="inline-flex items-center gap-2 text-sm text-muted no-underline hover:text-foreground">
          <ArrowLeft size={16} />
          返回服务列表
        </A>

        <Show
          when={service.data}
          fallback={
            service.isLoading ? (
              <LoadingState class="mt-8" title="正在加载服务" description="正在获取预约档期和价格信息。" />
            ) : service.isError ? (
              <ErrorState
                class="mt-8"
                title="服务加载失败"
                description="暂时无法获取服务详情，请稍后重试。"
                onRetry={() => service.refetch()}
              />
            ) : (
              <EmptyState
                class="mt-16"
                title="服务不存在或已下线"
                description="返回服务列表看看其他可预约项目。"
                ctaLabel="返回服务列表"
                href="/services"
              />
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

            <div class="space-y-4">
              <StepGuide
                id="service-booking"
                title="预约流程"
                description="按顺序完成时间和支付，避免预约中断。"
                steps={bookingSteps()}
              />
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
                  <div class="space-y-3 rounded-lg border border-accent/30 bg-accent/10 px-4 py-3 text-sm text-accent">
                    <p>
                      预约 #{createdAppointmentId()} 已提交，请在客户中心完成支付后锁定档期。
                    </p>
                    <A
                      href={`/account?appointment=${createdAppointmentId()}&action=pay`}
                      class="inline-flex h-9 items-center justify-center rounded-lg bg-accent px-4 text-sm font-medium text-accent-foreground no-underline"
                    >
                      去支付
                    </A>
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
          </div>
        </Show>
      </main>
    </div>
  );
}

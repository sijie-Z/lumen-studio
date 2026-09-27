import { A, useNavigate } from "@solidjs/router";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import {
  CalendarDays,
  Camera,
  Compass,
  CreditCard,
  LogOut,
  Sparkles,
  Star,
  Store
} from "lucide-solid";
import { createEffect, createSignal, For, Show } from "solid-js";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { NotificationBell } from "../components/layout/site-header";
import { EmptyState } from "../components/onboarding/empty-state";
import { StepGuide, type GuideStep } from "../components/onboarding/step-guide";
import { fetchMe, isAuthenticated } from "../lib/auth-api";
import { listAppointments, transitionAppointment } from "../lib/marketplace-api";
import { payAppointment, recharge } from "../lib/payment-api";
import { createReview } from "../lib/reviews-api";
import { useAuthStore } from "../stores/auth";

const statusLabel: Record<string, string> = {
  pending: "待支付",
  confirmed: "已确认",
  ongoing: "进行中",
  completed: "已完成",
  cancelled: "已取消",
  refunded: "已退款"
};

function friendlyError(error: unknown, fallback: string) {
  const message = error instanceof Error ? error.message : "";
  const translations: Record<string, string> = {
    "insufficient balance": "余额不足，请先充值。",
    "only pending appointments can be paid": "只有待支付的预约可以付款。",
    "appointment has already been paid": "该预约已经支付过了。",
    "appointment does not belong to this user": "这个预约不属于当前账户。",
    "only completed appointments can be reviewed": "只有已完成的预约可以评价。",
    "appointment has already been reviewed": "这个预约已经评价过了。",
    "rating must be between 1 and 5": "评分需要在 1 到 5 星之间。",
    "recharge amount must be greater than zero": "充值金额必须大于 0。"
  };
  return translations[message] ?? (message || fallback);
}

export default function Account() {
  const navigate = useNavigate();
  const auth = useAuthStore();
  const queryClient = useQueryClient();
  const me = createQuery(() => ({
    queryKey: ["me"] as const,
    queryFn: fetchMe,
    enabled: isAuthenticated()
  }));
  const appointments = createQuery(() => ({
    queryKey: ["appointments"] as const,
    queryFn: listAppointments,
    enabled: isAuthenticated()
  }));
  const [rechargeAmount, setRechargeAmount] = createSignal("5000");
  const [recharging, setRecharging] = createSignal(false);
  const [payingId, setPayingId] = createSignal<number | null>(null);
  const [reviewOpenId, setReviewOpenId] = createSignal<number | null>(null);
  const [reviewRating, setReviewRating] = createSignal(5);
  const [reviewContent, setReviewContent] = createSignal("");
  const [submittingReview, setSubmittingReview] = createSignal(false);
  const [reviewedIds, setReviewedIds] = createSignal<number[]>([]);
  const [actionMessage, setActionMessage] = createSignal("");
  const [actionError, setActionError] = createSignal("");
  const hasAppointments = () => (appointments.data?.length ?? 0) > 0;
  const customerSteps = (): GuideStep[] => [
    {
      title: "浏览服务",
      description: "从服务列表选择拍摄类型、价格和创作者。",
      done: hasAppointments()
    },
    {
      title: "提交并支付预约",
      description: "提交需求后，在客户中心完成余额支付。",
      done:
        appointments.data?.some((item) =>
          ["confirmed", "ongoing", "completed"].includes(item.status)
        ) ?? false
    },
    {
      title: "完成后评价",
      description: "服务完成后留下评价，帮助其他客户做选择。",
      done:
        appointments.data?.some(
          (item) => item.status === "completed" && reviewedIds().includes(item.id)
        ) ?? false
    }
  ];

  createEffect(() => {
    if (!isAuthenticated()) navigate("/login");
  });
  createEffect(() => {
    if (me.data) auth.setCurrentUser(me.data);
  });

  async function cancel(id: number) {
    setActionError("");
    setActionMessage("");
    try {
      await transitionAppointment(id, "cancelled");
      await queryClient.invalidateQueries({ queryKey: ["appointments"] });
      setActionMessage("预约已取消。");
    } catch (err) {
      setActionError(friendlyError(err, "取消预约失败，请稍后重试。"));
    }
  }

  async function handleRecharge() {
    const amount = rechargeAmount().trim();
    if (!amount || !Number.isFinite(Number(amount)) || Number(amount) <= 0) {
      setActionError("请输入大于 0 的充值金额。");
      return;
    }

    setRecharging(true);
    setActionError("");
    setActionMessage("");
    try {
      await recharge(amount);
      await queryClient.invalidateQueries({ queryKey: ["payments"] });
      setActionMessage(`充值 ¥${amount} 成功，余额已到账。`);
    } catch (err) {
      setActionError(friendlyError(err, "充值失败，请稍后重试。"));
    } finally {
      setRecharging(false);
    }
  }

  async function handlePay(id: number) {
    setPayingId(id);
    setActionError("");
    setActionMessage("");
    try {
      await payAppointment(id);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["appointments"] }),
        queryClient.invalidateQueries({ queryKey: ["payments"] })
      ]);
      setActionMessage("支付成功，预约已进入已确认状态。");
    } catch (err) {
      setActionError(friendlyError(err, "支付失败，请稍后重试。"));
    } finally {
      setPayingId(null);
    }
  }

  function openReview(id: number) {
    setReviewOpenId(id);
    setReviewRating(5);
    setReviewContent("");
    setActionError("");
    setActionMessage("");
  }

  async function submitReview(id: number) {
    setSubmittingReview(true);
    setActionError("");
    setActionMessage("");
    try {
      await createReview({
        appointment_id: id,
        rating: reviewRating(),
        content: reviewContent().trim() || undefined
      });
      setReviewedIds((ids) => [...ids, id]);
      setReviewOpenId(null);
      setActionMessage("评价已提交，感谢你的反馈。");
    } catch (err) {
      setActionError(friendlyError(err, "评价提交失败，请稍后重试。"));
    } finally {
      setSubmittingReview(false);
    }
  }

  return (
    <div class="min-h-screen bg-background text-foreground">
      <header class="sticky top-0 z-40 border-b border-line bg-background/85 backdrop-blur-xl">
        <div class="mx-auto flex h-16 max-w-6xl items-center justify-between px-5 md:px-8">
          <button type="button" onClick={() => navigate("/")} class="flex items-center gap-2.5">
            <span class="grid size-9 place-items-center rounded-lg bg-primary text-primary-foreground">
              <Camera size={18} />
            </span>
            <span class="font-display text-lg font-semibold">我的账户</span>
          </button>
          <div class="flex items-center gap-3">
            <NotificationBell />
            <button
              type="button"
              onClick={() => {
                auth.signOut();
                navigate("/");
              }}
              class="inline-flex items-center gap-2 rounded-lg border border-line px-3 py-2 text-sm text-muted hover:text-foreground"
            >
              <LogOut size={16} />
              <span class="hidden sm:inline">退出</span>
            </button>
          </div>
        </div>
      </header>

      <main class="mx-auto max-w-6xl px-5 py-8 md:px-8 md:py-10">
        <Show when={actionMessage()}>
          <div class="mb-5 rounded-lg border border-accent/30 bg-accent/10 px-4 py-3 text-sm text-accent" role="status">
            {actionMessage()}
          </div>
        </Show>
        <Show when={actionError()}>
          <div class="mb-5 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive" role="alert">
            {actionError()}
          </div>
        </Show>

        <div class="flex flex-col justify-between gap-5 sm:flex-row sm:items-end">
          <div>
            <p class="text-sm text-muted">欢迎回来</p>
            <h1 class="mt-1 font-display text-3xl font-semibold">{me.data?.nickname ?? "客户"}</h1>
            <p class="mt-1 text-sm text-muted">@{me.data?.username}</p>
          </div>
          <A href="/dashboard" class="inline-flex items-center gap-2 text-sm text-muted no-underline hover:text-foreground">
            <Sparkles size={16} />
            成为创作者
          </A>
        </div>

        <Show when={hasAppointments()}>
          <StepGuide
            id="customer-onboarding"
            class="mt-6"
            title="完成一次预约"
            description="从选服务到评价，按顺序完成即可。"
            steps={customerSteps()}
          />
        </Show>

        <Card class="mt-6 border-primary/30 bg-primary/5">
          <CardContent class="flex flex-col gap-4 pt-5 sm:flex-row sm:items-center sm:justify-between">
            <div class="flex items-center gap-3">
              <span class="grid size-10 place-items-center rounded-lg bg-primary/15 text-primary">
                <CreditCard size={20} />
              </span>
              <div>
                <p class="font-medium">账户充值</p>
                <p class="text-sm text-muted">充值后可直接用于预约支付</p>
              </div>
            </div>
            <div class="flex w-full gap-2 sm:w-auto">
              <Input
                value={rechargeAmount()}
                onInput={(event) => setRechargeAmount(event.currentTarget.value)}
                inputMode="decimal"
                class="w-full sm:w-36"
                aria-label="充值金额"
              />
              <Button onClick={handleRecharge} disabled={recharging()}>
                {recharging() ? "充值中" : "充值"}
              </Button>
            </div>
          </CardContent>
        </Card>

        <div class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <A href="/services" class="no-underline">
            <Card class="transition-colors hover:border-primary/60">
              <CardContent class="flex items-center gap-4 pt-5">
                <Store class="text-primary" size={24} />
                <div>
                  <p class="font-medium">浏览服务</p>
                  <p class="text-sm text-muted">预约创作者服务</p>
                </div>
              </CardContent>
            </Card>
          </A>
          <A href="/explore" class="no-underline">
            <Card class="transition-colors hover:border-primary/60">
              <CardContent class="flex items-center gap-4 pt-5">
                <Compass class="text-accent" size={24} />
                <div>
                  <p class="font-medium">探索作品</p>
                  <p class="text-sm text-muted">寻找灵感</p>
                </div>
              </CardContent>
            </Card>
          </A>
          <Card>
            <CardContent class="flex items-center gap-4 pt-5">
              <CalendarDays class="text-coral" size={24} />
              <div>
                <p class="font-medium">{appointments.data?.length ?? 0} 个预约</p>
                <p class="text-sm text-muted">当前预约数量</p>
              </div>
            </CardContent>
          </Card>
        </div>

        <section class="mt-10">
          <h2 class="font-display text-2xl font-semibold">我的预约</h2>
          <Show
            when={hasAppointments()}
            fallback={
              <EmptyState
                class="mt-5"
                icon={<Compass size={20} />}
                title="还没有预约"
                description="先浏览服务，选择适合你的拍摄类型和创作者。"
                ctaLabel="浏览服务"
                href="/services"
              />
            }
          >
            <div class="mt-5 space-y-3">
              <For each={appointments.data ?? []}>
                {(item) => (
                  <div class="overflow-hidden rounded-lg border border-line bg-secondary">
                    <div class="flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:justify-between">
                      <div>
                        <div class="flex flex-wrap items-center gap-3">
                          <span class="font-medium">
                            {new Date(item.start_time).toLocaleString("zh-CN")}
                          </span>
                          <Badge variant={item.status === "completed" ? "accent" : item.status === "cancelled" || item.status === "refunded" ? "destructive" : "default"}>
                            {statusLabel[item.status] ?? item.status}
                          </Badge>
                        </div>
                        <p class="mt-1 text-sm text-muted">服务 #{item.service_id} · ¥{item.total_price}</p>
                      </div>
                      <div class="flex flex-wrap gap-2">
                        <Show when={item.status === "pending"}>
                          <Button size="sm" onClick={() => handlePay(item.id)} disabled={payingId() === item.id}>
                            {payingId() === item.id ? "支付中" : "支付"}
                          </Button>
                        </Show>
                        <Show when={item.status === "completed" && !reviewedIds().includes(item.id)}>
                          <Button variant="outline" size="sm" onClick={() => openReview(item.id)}>
                            <Star size={14} />
                            评价
                          </Button>
                        </Show>
                        <Show when={["pending", "confirmed", "ongoing"].includes(item.status)}>
                          <Button variant="outline" size="sm" onClick={() => cancel(item.id)}>
                            取消预约
                          </Button>
                        </Show>
                      </div>
                    </div>

                    <Show when={reviewOpenId() === item.id}>
                      <div class="border-t border-line bg-background/40 p-4">
                        <div class="flex flex-wrap items-center justify-between gap-3">
                          <div>
                            <p class="text-sm font-medium">为本次服务评分</p>
                            <p class="mt-1 text-xs text-muted">你的评价会帮助其他客户做出选择</p>
                          </div>
                          <div class="flex items-center gap-1" aria-label="评分">
                            <For each={[1, 2, 3, 4, 5]}>
                              {(value) => (
                                <button
                                  type="button"
                                  aria-label={`${value} 星`}
                                  aria-pressed={value <= reviewRating()}
                                  onClick={() => setReviewRating(value)}
                                  class="rounded-md p-1 transition-colors hover:bg-surface"
                                >
                                  <Star
                                    size={20}
                                    class={value <= reviewRating() ? "fill-current text-primary" : "text-muted"}
                                  />
                                </button>
                              )}
                            </For>
                          </div>
                        </div>
                        <textarea
                          value={reviewContent()}
                          onInput={(event) => setReviewContent(event.currentTarget.value)}
                          rows={3}
                          maxlength={1000}
                          placeholder="说说这次拍摄体验（选填）"
                          class="mt-4 w-full resize-none rounded-lg border border-line bg-background px-3 py-2 text-sm text-foreground outline-none transition-colors placeholder:text-muted/70 focus:border-ring"
                        />
                        <div class="mt-3 flex justify-end gap-2">
                          <Button variant="ghost" size="sm" onClick={() => setReviewOpenId(null)}>
                            取消
                          </Button>
                          <Button size="sm" onClick={() => submitReview(item.id)} disabled={submittingReview()}>
                            {submittingReview() ? "提交中" : "提交评价"}
                          </Button>
                        </div>
                      </div>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </section>
      </main>
    </div>
  );
}

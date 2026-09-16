import { A, useNavigate } from "@solidjs/router";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import {
  CalendarDays,
  Camera,
  Compass,
  LogOut,
  Sparkles,
  Store
} from "lucide-solid";
import { createEffect, For, Show } from "solid-js";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent } from "../components/ui/card";
import { fetchMe, isAuthenticated } from "../lib/auth-api";
import { listAppointments, transitionAppointment } from "../lib/marketplace-api";
import { useAuthStore } from "../stores/auth";

const statusLabel: Record<string, string> = {
  pending: "待确认",
  confirmed: "已确认",
  ongoing: "进行中",
  completed: "已完成",
  cancelled: "已取消",
  refunded: "已退款"
};

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

  createEffect(() => {
    if (!isAuthenticated()) navigate("/login");
  });
  createEffect(() => {
    if (me.data) auth.setCurrentUser(me.data);
  });

  async function cancel(id: number) {
    await transitionAppointment(id, "cancelled");
    await queryClient.invalidateQueries({ queryKey: ["appointments"] });
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
      </header>

      <main class="mx-auto max-w-6xl px-5 py-8 md:px-8 md:py-10">
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
            when={(appointments.data?.length ?? 0) > 0}
            fallback={
              <div class="mt-5 grid min-h-40 place-items-center rounded-lg border border-dashed border-line text-muted">
                还没有预约，去服务列表挑选创作者吧
              </div>
            }
          >
            <div class="mt-5 space-y-3">
              <For each={appointments.data ?? []}>
                {(item) => (
                  <div class="flex flex-col gap-3 rounded-lg border border-line bg-secondary p-4 sm:flex-row sm:items-center sm:justify-between">
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
                    <Show when={["pending", "confirmed", "ongoing"].includes(item.status)}>
                      <Button variant="outline" size="sm" onClick={() => cancel(item.id)}>
                        取消预约
                      </Button>
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

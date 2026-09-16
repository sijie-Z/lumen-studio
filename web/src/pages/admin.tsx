import { useNavigate } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import {
  Camera,
  CalendarDays,
  Image,
  Layers,
  LogOut,
  ShieldCheck,
  Store,
  Users,
  Wallet
} from "lucide-solid";
import { createEffect, For, Show } from "solid-js";
import { Badge } from "../components/ui/badge";
import { Card, CardContent } from "../components/ui/card";
import { Skeleton } from "../components/ui/skeleton";
import { getStats, listUsers } from "../lib/admin-api";
import { isAuthenticated } from "../lib/auth-api";
import { useAuthStore } from "../stores/auth";

export default function Admin() {
  const navigate = useNavigate();
  const auth = useAuthStore();
  const stats = createQuery(() => ({
    queryKey: ["admin-stats"] as const,
    queryFn: getStats,
    enabled: isAuthenticated()
  }));
  const users = createQuery(() => ({
    queryKey: ["admin-users"] as const,
    queryFn: listUsers,
    enabled: isAuthenticated()
  }));

  createEffect(() => {
    if (!isAuthenticated()) navigate("/login");
    if (users.isError) navigate("/login");
  });

  const metrics = () => [
    { icon: Users, label: "用户", value: stats.data?.users ?? 0 },
    { icon: Camera, label: "创作者", value: stats.data?.creators ?? 0 },
    { icon: Store, label: "服务", value: stats.data?.services ?? 0 },
    { icon: CalendarDays, label: "预约", value: stats.data?.appointments ?? 0 },
    { icon: Image, label: "作品", value: stats.data?.works ?? 0 }
  ];

  return (
    <div class="min-h-screen bg-background text-foreground">
      <header class="sticky top-0 z-40 border-b border-line bg-background/85 backdrop-blur-xl">
        <div class="mx-auto flex h-16 max-w-7xl items-center justify-between px-5 md:px-8">
          <div class="flex items-center gap-2.5">
            <span class="grid size-9 place-items-center rounded-lg bg-primary text-primary-foreground">
              <ShieldCheck size={18} />
            </span>
            <div>
              <p class="font-display text-lg font-semibold leading-none">Lumina 管理后台</p>
              <p class="mt-1 text-xs text-muted">Admin Console</p>
            </div>
          </div>
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

      <main class="mx-auto max-w-7xl px-5 py-8 md:px-8 md:py-10">
        <h1 class="font-display text-3xl font-semibold">平台概览</h1>

        <Show when={stats.isLoading}>
          <div class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
            {Array.from({ length: 5 }).map(() => <Skeleton class="h-28 w-full rounded-lg" />)}
          </div>
        </Show>

        <div class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
          <For each={metrics()}>
            {(metric) => (
              <Card>
                <CardContent class="pt-5">
                  <metric.icon size={20} class="text-primary" />
                  <p class="mt-4 text-2xl font-semibold">{metric.value}</p>
                  <p class="mt-1 text-sm text-muted">{metric.label}</p>
                </CardContent>
              </Card>
            )}
          </For>
          <Card class="bg-primary text-primary-foreground">
            <CardContent class="pt-5">
              <Wallet size={20} />
              <p class="mt-4 text-2xl font-semibold">¥{stats.data?.gross_volume ?? "0"}</p>
              <p class="mt-1 text-sm opacity-80">成交额</p>
            </CardContent>
          </Card>
        </div>

        <section class="mt-10">
          <div class="mb-5 flex items-center gap-3">
            <Layers size={20} class="text-primary" />
            <h2 class="font-display text-2xl font-semibold">用户管理</h2>
          </div>
          <Card>
            <CardContent class="overflow-x-auto p-0">
              <table class="w-full min-w-[640px] text-left text-sm">
                <thead class="border-b border-line text-muted">
                  <tr>
                    <th class="px-5 py-3 font-medium">用户</th>
                    <th class="px-5 py-3 font-medium">用户名</th>
                    <th class="px-5 py-3 font-medium">角色</th>
                    <th class="px-5 py-3 font-medium">状态</th>
                    <th class="px-5 py-3 font-medium">注册时间</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={users.data ?? []}>
                    {(user) => (
                      <tr class="border-b border-line last:border-0">
                        <td class="px-5 py-3">
                          <span class="font-medium">{user.nickname}</span>
                          <span class="ml-2 text-xs text-muted">#{user.id}</span>
                        </td>
                        <td class="px-5 py-3 text-muted">{user.username}</td>
                        <td class="px-5 py-3">
                          <Badge variant={user.role === "admin" ? "accent" : "secondary"}>
                            {user.role === "admin" ? "管理员" : user.role === "user" ? "用户" : user.role}
                          </Badge>
                        </td>
                        <td class="px-5 py-3">
                          <Badge variant={user.status === "active" ? "outline" : "destructive"}>
                            {user.status === "active" ? "正常" : user.status}
                          </Badge>
                        </td>
                        <td class="px-5 py-3 text-muted">
                          {new Date(user.created_at).toLocaleDateString("zh-CN")}
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </CardContent>
          </Card>
        </section>
      </main>
    </div>
  );
}

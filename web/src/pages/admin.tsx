import { createQuery, useQueryClient } from "@tanstack/solid-query";
import {
  Camera,
  CalendarDays,
  Check,
  Image,
  Layers,
  Store,
  Users,
  X,
  Wallet
} from "lucide-solid";
import { createSignal, For, Show } from "solid-js";
import { Badge } from "../components/ui/badge";
import { Card, CardContent } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { StepGuide, type GuideStep } from "../components/onboarding/step-guide";
import { EmptyState, ErrorState, LoadingState } from "../components/ui/state";
import { getStats, listUsers } from "../lib/admin-api";
import { isAuthenticated } from "../lib/auth-api";
import { listAllWithdrawals, reviewWithdrawal } from "../lib/withdrawal-api";

export default function Admin() {
  const queryClient = useQueryClient();
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

  const withdrawals = createQuery(() => ({
    queryKey: ["admin-withdrawals"] as const,
    queryFn: listAllWithdrawals,
    enabled: isAuthenticated()
  }));
  const [withdrawalNotes, setWithdrawalNotes] = createSignal<Record<number, string>>({});
  const [reviewingWithdrawalId, setReviewingWithdrawalId] = createSignal<number | null>(null);
  const [withdrawalReviewMsg, setWithdrawalReviewMsg] = createSignal("");
  const [withdrawalReviewError, setWithdrawalReviewError] = createSignal("");

  const metrics = () => [
    { icon: Users, label: "用户", value: stats.data?.users ?? 0 },
    { icon: Camera, label: "创作者", value: stats.data?.creators ?? 0 },
    { icon: Store, label: "服务", value: stats.data?.services ?? 0 },
    { icon: CalendarDays, label: "预约", value: stats.data?.appointments ?? 0 },
    { icon: Image, label: "作品", value: stats.data?.works ?? 0 }
  ];

  const pendingWithdrawals = () =>
    (withdrawals.data ?? []).filter((item) => item.status === "pending");

  const adminSteps = (): GuideStep[] => [
    {
      title: "查看平台数据",
      description: "关注用户、创作者、服务、预约和成交额的变化。",
      done: Boolean(stats.data)
    },
    {
      title: "管理用户与角色",
      description: "在用户列表中核对账号状态和角色信息。",
      done: (users.data?.length ?? 0) > 0
    },
    {
      title: "处理提现审核",
      description: "及时处理待审核申请，保证创作者资金流转。",
      done: Boolean(withdrawals.data) && pendingWithdrawals().length === 0
    }
  ];

  function updateWithdrawalNote(id: number, note: string) {
    setWithdrawalNotes({ ...withdrawalNotes(), [id]: note });
  }

  async function handleWithdrawalReview(id: number, approve: boolean) {
    setReviewingWithdrawalId(id);
    setWithdrawalReviewMsg("");
    setWithdrawalReviewError("");
    try {
      await reviewWithdrawal(id, approve, withdrawalNotes()[id]?.trim() || undefined);
      await queryClient.invalidateQueries({ queryKey: ["admin-withdrawals"] });
      setWithdrawalReviewMsg(approve ? "提现已通过并标记为已打款。" : "提现已拒绝，金额已退回创作者余额。");
    } catch (err) {
      setWithdrawalReviewError(err instanceof Error ? err.message : "提现审核失败，请稍后重试。");
    } finally {
      setReviewingWithdrawalId(null);
    }
  }

  return (
    <div class="min-h-screen bg-background text-foreground">
      <main class="mx-auto max-w-7xl px-5 py-8 md:px-8 md:py-10">
        <div class="flex flex-col justify-between gap-4 sm:flex-row sm:items-end">
          <h1 class="font-display text-3xl font-semibold">平台概览</h1>
          <p class="text-sm text-muted">平台数据、用户管理和资金审核集中在这里。</p>
        </div>

        <StepGuide
          id="admin-onboarding"
          class="mt-6"
          title="管理后台怎么用"
          description="先看数据，再处理用户和提现，日常按这个顺序检查即可。"
          steps={adminSteps()}
        />

        <Show when={stats.isLoading}>
          <LoadingState class="mt-8" title="正在加载平台数据" description="正在汇总用户、服务、预约和成交额。" />
        </Show>

        <Show when={stats.isError}>
          <ErrorState
            class="mt-8"
            title="平台数据加载失败"
            description="暂时无法获取统计数据，请稍后重试。"
            onRetry={() => stats.refetch()}
          />
        </Show>

        <Show when={stats.isSuccess}>
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
        </Show>

        <section id="users" class="mt-10">
          <div class="mb-5 flex items-center gap-3">
            <Layers size={20} class="text-primary" />
            <h2 class="font-display text-2xl font-semibold">用户管理</h2>
          </div>
          <Show when={users.isPending}>
            <LoadingState title="正在加载用户列表" description="正在读取账号与角色信息。" />
          </Show>
          <Show when={users.isError}>
            <ErrorState
              title="用户列表加载失败"
              description="暂时无法获取用户数据，请稍后重试。"
              onRetry={() => users.refetch()}
            />
          </Show>
          <Show when={users.isSuccess && (users.data?.length ?? 0) === 0}>
            <EmptyState title="暂无用户" description="注册用户后会展示在这里。" />
          </Show>
          <Show when={users.isSuccess && (users.data?.length ?? 0) > 0}>
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
          </Show>
        </section>
        <section id="withdrawals" class="mt-10">
          <div class="mb-5 flex items-center gap-3">
            <Wallet size={20} class="text-primary" />
            <div>
              <h2 class="font-display text-2xl font-semibold">提现审核</h2>
              <p class="mt-1 text-sm text-muted">通过后视为已打款；拒绝时金额自动退回创作者余额。</p>
            </div>
          </div>
          <Show when={withdrawalReviewMsg()}>
            <div class="mb-4 rounded-lg border border-teal/30 bg-teal/10 px-4 py-3 text-sm text-teal">
              {withdrawalReviewMsg()}
            </div>
          </Show>
          <Show when={withdrawalReviewError()}>
            <div class="mb-4 rounded-lg border border-coral/30 bg-coral/10 px-4 py-3 text-sm text-coral">
              {withdrawalReviewError()}
            </div>
          </Show>
          <Card>
            <CardContent class="space-y-4 pt-5">
              <Show when={withdrawals.isPending}>
                <LoadingState class="min-h-32" title="正在加载提现申请" description="正在读取待审核资金。" />
              </Show>
              <Show when={withdrawals.isError}>
                <ErrorState
                  class="min-h-32"
                  title="提现申请加载失败"
                  description="暂时无法获取提现数据，请稍后重试。"
                  onRetry={() => withdrawals.refetch()}
                />
              </Show>
              <Show when={withdrawals.data}>
                <Show
                  when={pendingWithdrawals().length > 0}
                  fallback={
                    <EmptyState class="min-h-32" title="当前没有待审核提现" description="新的提现申请会出现在这里。" />
                  }
                >
                  <div class="space-y-4">
                    <For each={pendingWithdrawals()}>
                      {(item) => (
                        <div class="rounded-lg border border-line bg-secondary p-4">
                          <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                            <div class="min-w-0 space-y-2">
                              <div class="flex flex-wrap items-center gap-3">
                                <span class="font-medium">创作者 #{item.creator_id}</span>
                                <Badge variant="accent">待审核</Badge>
                                <span class="font-display text-xl font-semibold text-primary">¥{item.amount}</span>
                              </div>
                              <p class="break-all text-sm text-muted">
                                收款账户：{item.account_info ? JSON.stringify(item.account_info) : "未填写"}
                              </p>
                              <p class="text-xs text-muted">
                                申请时间：{new Date(item.created_at).toLocaleString("zh-CN")}
                              </p>
                            </div>
                            <div class="w-full space-y-3 lg:max-w-md">
                              <Input
                                value={withdrawalNotes()[item.id] ?? ""}
                                onInput={(event) => updateWithdrawalNote(item.id, event.currentTarget.value)}
                                placeholder="审核备注（拒绝原因建议填写）"
                              />
                              <div class="flex justify-end gap-2">
                                <Button
                                  variant="destructive"
                                  size="sm"
                                  onClick={() => handleWithdrawalReview(item.id, false)}
                                  disabled={reviewingWithdrawalId() === item.id}
                                >
                                  <X />
                                  拒绝
                                </Button>
                                <Button
                                  size="sm"
                                  onClick={() => handleWithdrawalReview(item.id, true)}
                                  disabled={reviewingWithdrawalId() === item.id}
                                >
                                  <Check />
                                  通过并打款
                                </Button>
                              </div>
                            </div>
                          </div>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </Show>
            </CardContent>
          </Card>
        </section>
      </main>
    </div>
  );
}

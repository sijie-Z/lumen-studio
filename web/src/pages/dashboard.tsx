import { createQuery, useQueryClient } from "@tanstack/solid-query";
import {
  CalendarDays,
  CloudUpload,
  Image as ImageIcon,
  Sparkles,
  Store,
  CalendarClock,
  Wallet,
  User
} from "lucide-solid";
import { createMemo, createSignal, For, Show } from "solid-js";
import { fetchMe, isAuthenticated, uploadImage } from "../lib/auth-api";
import { ApiError } from "../lib/api";
import { createWork, listWorks } from "../lib/works-api";
import {
  createService,
  listCreatorAppointments,
  listCreators,
  listServices,
  listServiceTypes,
  transitionAppointment,
  upsertProfile
} from "../lib/marketplace-api";
import Assistant from "../components/ai/assistant";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { EmptyState } from "../components/onboarding/empty-state";
import { StepGuide, type GuideStep } from "../components/onboarding/step-guide";
import { applyWithdrawal, listMyWithdrawals } from "../lib/withdrawal-api";

export default function Dashboard() {
  const queryClient = useQueryClient();
  const [selected, setSelected] = createSignal<File | null>(null);
  const [preview, setPreview] = createSignal<string | null>(null);
  const [uploading, setUploading] = createSignal(false);
  const [error, setError] = createSignal("");
  const [intro, setIntro] = createSignal("");
  const [bio, setBio] = createSignal("");
  const [savingProfile, setSavingProfile] = createSignal(false);
  const [profileMsg, setProfileMsg] = createSignal("");
  const [serviceForm, setServiceForm] = createSignal({
    type_id: 1,
    title: "",
    price: "",
    duration: "",
    location: ""
  });
  const [savingService, setSavingService] = createSignal(false);
  const [serviceMsg, setServiceMsg] = createSignal("");
  const [appointmentActionId, setAppointmentActionId] = createSignal<number | null>(null);
  const [appointmentMsg, setAppointmentMsg] = createSignal("");
  const [appointmentError, setAppointmentError] = createSignal("");
  const [withdrawalAmount, setWithdrawalAmount] = createSignal("");
  const [withdrawalAccount, setWithdrawalAccount] = createSignal("");
  const [submittingWithdrawal, setSubmittingWithdrawal] = createSignal(false);
  const [withdrawalMsg, setWithdrawalMsg] = createSignal("");
  const [withdrawalError, setWithdrawalError] = createSignal("");

  const me = createQuery(() => ({
    queryKey: ["me"] as const,
    queryFn: fetchMe,
    enabled: isAuthenticated()
  }));
  const works = createQuery(() => ({
    queryKey: ["works"] as const,
    queryFn: listWorks,
    staleTime: 0
  }));
  const creators = createQuery(() => ({ queryKey: ["creators"] as const, queryFn: listCreators }));
  const serviceTypes = createQuery(() => ({ queryKey: ["service-types"] as const, queryFn: listServiceTypes }));
  const services = createQuery(() => ({ queryKey: ["services"] as const, queryFn: listServices }));
  const creatorAppointments = createQuery(() => ({
    queryKey: ["creator-appointments"] as const,
    queryFn: listCreatorAppointments,
    enabled: isAuthenticated()
  }));

  const withdrawals = createQuery(() => ({
    queryKey: ["withdrawals"] as const,
    queryFn: listMyWithdrawals,
    enabled: isAuthenticated()
  }));
  const myProfile = createMemo(() =>
    creators.data?.find((item) => item.user_id === me.data?.id)
  );
  const myWorks = createMemo(() =>
    (works.data ?? []).filter((item) => item.user_id === me.data?.id)
  );
  const myServices = createMemo(() =>
    (services.data ?? []).filter((item) => item.creator_id === myProfile()?.id)
  );
  const creatorSteps = (): GuideStep[] => [
    {
      title: "完善创作者资料",
      description: "填写简介和个人主页，让客户快速了解你的风格。",
      done: Boolean(myProfile())
    },
    {
      title: "上传第一组作品",
      description: "用真实作品建立信任，作品会展示在主页和探索页。",
      done: myWorks().length > 0
    },
    {
      title: "发布服务",
      description: "设置服务类型、价格和拍摄地点，开始接受预约。",
      done: myServices().length > 0
    },
    {
      title: "完成首单并提现",
      description: "客户付款后确认履约，服务完成后即可申请提现。",
      done:
        creatorAppointments.data?.some((item) => item.status === "completed") ?? false
    }
  ];
  const creatorAppointmentsUnavailable = () => {
    const error = creatorAppointments.error;
    return error instanceof ApiError && [403, 404].includes(error.status);
  };

  function scrollToId(id: string) {
    document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
  }

  function chooseFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    if (!["image/jpeg", "image/png", "image/webp"].includes(file.type)) {
      setError("仅支持 JPG、PNG 或 WebP 图片");
      return;
    }
    setSelected(file);
    setPreview(URL.createObjectURL(file));
  }

  async function handleUpload() {
    const file = selected();
    if (!file || uploading()) return;
    setUploading(true);
    setError("");
    try {
      const result = await uploadImage(file);
      await createWork({
        image_url: result.url,
        title: file.name.replace(/\.[^.]+$/, "")
      });
      await queryClient.invalidateQueries({ queryKey: ["works"] });
      setSelected(null);
      setPreview(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "上传失败");
    } finally {
      setUploading(false);
    }
  }

  async function saveProfile() {
    setSavingProfile(true);
    setProfileMsg("");
    try {
      const profile = await upsertProfile({
        introduction: intro() || undefined,
        bio: bio() || undefined
      });
      await queryClient.invalidateQueries({ queryKey: ["creators"] });
      setProfileMsg(`创作者资料已保存（ID ${profile.id}）`);
    } catch (err) {
      setProfileMsg(err instanceof Error ? err.message : "保存失败");
    } finally {
      setSavingProfile(false);
    }
  }

  function updateService(field: string, value: string) {
    setServiceForm({ ...serviceForm(), [field]: value });
  }

  async function publishService() {
    const form = serviceForm();
    if (!form.title.trim() || !form.price.trim()) {
      setServiceMsg("请填写服务标题和价格");
      return;
    }
    setSavingService(true);
    setServiceMsg("");
    try {
      await createService({
        type_id: Number(form.type_id),
        title: form.title,
        price: form.price,
        duration: form.duration ? Number(form.duration) : undefined,
        location: form.location || undefined
      });
      await queryClient.invalidateQueries({ queryKey: ["services"] });
      await queryClient.invalidateQueries({ queryKey: ["creators"] });
      setServiceForm({ type_id: form.type_id, title: "", price: "", duration: "", location: "" });
      setServiceMsg("服务已发布");
    } catch (err) {
      setServiceMsg(err instanceof Error ? err.message : "发布失败");
    } finally {
      setSavingService(false);
    }
  }

  async function advanceAppointment(id: number, status: "ongoing" | "completed") {
    setAppointmentActionId(id);
    setAppointmentMsg("");
    setAppointmentError("");
    try {
      await transitionAppointment(id, status);
      await queryClient.invalidateQueries({ queryKey: ["creator-appointments"] });
      setAppointmentMsg(status === "ongoing" ? "预约已开始。" : "预约已完成。");
    } catch (err) {
      setAppointmentError(err instanceof Error ? err.message : "预约状态更新失败，请稍后重试。");
    } finally {
      setAppointmentActionId(null);
    }
  }

  async function submitWithdrawal() {
    const amount = withdrawalAmount().trim();
    if (!amount || Number(amount) <= 0) {
      setWithdrawalError("请输入大于 0 的提现金额。");
      return;
    }
    setSubmittingWithdrawal(true);
    setWithdrawalMsg("");
    setWithdrawalError("");
    try {
      const account = withdrawalAccount().trim();
      await applyWithdrawal(amount, account ? { account } : undefined);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["withdrawals"] }),
        queryClient.invalidateQueries({ queryKey: ["me"] })
      ]);
      setWithdrawalAmount("");
      setWithdrawalAccount("");
      setWithdrawalMsg("提现申请已提交，等待管理员审核。");
    } catch (err) {
      setWithdrawalError(err instanceof Error ? err.message : "提现申请失败，请稍后重试。");
    } finally {
      setSubmittingWithdrawal(false);
    }
  }

  return (
    <div class="min-h-screen bg-ink text-paper">
      <main class="mx-auto max-w-7xl px-5 py-8 md:px-8 md:py-10">
        <div class="flex flex-col justify-between gap-5 md:flex-row md:items-end">
          <div class="flex items-center gap-4">
            <div class="grid size-16 place-items-center rounded-lg bg-gradient-to-br from-amber to-coral text-ink">
              <User size={30} stroke-width={1.8} />
            </div>
            <div>
              <p class="text-xs tracking-wide text-muted uppercase">Creator Workspace</p>
              <h1 class="mt-1 font-display text-3xl font-semibold">
                {me.data?.nickname ?? "我的工作台"}
              </h1>
              <p class="mt-1 text-sm text-muted">@{me.data?.username ?? "..."}</p>
            </div>
          </div>
          <div class="flex items-center gap-3">
            <div class="rounded-lg border border-line bg-surface px-4 py-3">
              <p class="text-xs text-muted">账户余额</p>
              <p class="mt-1 text-xl font-medium text-primary">¥{me.data?.balance ?? "0.00"}</p>
            </div>
            <div class="rounded-lg border border-line bg-surface px-4 py-3">
              <p class="text-xs text-muted">已上传作品</p>
              <p class="mt-1 text-xl font-medium text-amber">{myWorks().length}</p>
            </div>
            <div class="rounded-lg border border-line bg-surface px-4 py-3">
              <p class="text-xs text-muted">本月预约</p>
              <p class="mt-1 text-xl font-medium text-teal">0</p>
            </div>
          </div>
        </div>

        <Show when={!creators.isPending && !myProfile()}>
          <EmptyState
            class="mt-8"
            icon={<User size={20} />}
            title="先完善创作者资料"
            description="创建资料后才能发布服务、接收预约和管理作品。"
            ctaLabel="填写创作者资料"
            onClick={() => scrollToId("creator-profile")}
          />
        </Show>

        <Show when={myProfile()}>
          <StepGuide
            id="creator-onboarding"
            class="mt-8"
            title="完成创作者起步流程"
            description="按顺序完成资料、作品、服务和首单。"
            steps={creatorSteps()}
          />
        </Show>

        <div class="mt-10 grid gap-6 lg:grid-cols-[360px_1fr]">
          <aside class="space-y-6">
            <section class="rounded-lg border border-line bg-surface p-5">
              <h2 class="text-base font-medium">账户信息</h2>
              <div class="mt-5 space-y-4 text-sm">
                <div class="flex items-center justify-between gap-4">
                  <span class="text-muted">邮箱</span>
                  <span class="truncate text-paper">{me.data?.email ?? "未设置"}</span>
                </div>
                <div class="flex items-center justify-between gap-4">
                  <span class="text-muted">手机</span>
                  <span class="truncate text-paper">{me.data?.phone ?? "未设置"}</span>
                </div>
                <div class="flex items-center justify-between gap-4">
                  <span class="text-muted">余额</span>
                  <span class="text-paper">¥{me.data?.balance ?? "0.00"}</span>
                </div>
                <div class="flex items-center justify-between gap-4">
                  <span class="text-muted">角色</span>
                  <span class="rounded-full bg-teal/15 px-2.5 py-1 text-xs text-teal">
                    {me.data?.role === "user" ? "创作者" : me.data?.role}
                  </span>
                </div>
              </div>
            </section>

            <section class="rounded-lg border border-line bg-surface p-5">
              <h2 class="text-base font-medium">快捷入口</h2>
              <div class="mt-4 grid grid-cols-2 gap-3">
                <button class="flex flex-col items-start gap-2 rounded-lg border border-line bg-ink/40 p-3.5 text-left transition-colors hover:border-amber/60">
                  <CalendarDays size={19} class="text-amber" />
                  <span class="text-sm text-paper">预约管理</span>
                </button>
                <button class="flex flex-col items-start gap-2 rounded-lg border border-line bg-ink/40 p-3.5 text-left transition-colors hover:border-amber/60">
                  <Sparkles size={19} class="text-teal" />
                  <span class="text-sm text-paper">AI 助手</span>
                </button>
              </div>
            </section>
          </aside>

          <section id="upload-work" class="rounded-lg border border-line bg-surface p-5 md:p-6">
            <div class="flex items-center justify-between gap-4">
              <div>
                <h2 class="text-base font-medium">作品上传</h2>
                <p class="mt-1 text-sm text-muted">JPG、PNG 或 WebP，最大 15MB</p>
              </div>
              <div class="grid size-11 place-items-center rounded-lg bg-amber/15 text-amber">
                <ImageIcon size={20} />
              </div>
            </div>

            <label
              class="mt-6 flex min-h-52 cursor-pointer flex-col items-center justify-center rounded-lg border border-dashed border-line bg-ink/35 px-6 py-10 text-center transition-colors hover:border-amber/70"
            >
              <input type="file" accept="image/jpeg,image/png,image/webp" class="sr-only" onChange={chooseFile} />
              <Show
                when={preview()}
                fallback={
                  <>
                    <span class="grid size-14 place-items-center rounded-full border border-line bg-ink text-muted">
                      <CloudUpload size={24} />
                    </span>
                    <span class="mt-4 text-sm text-paper">点击选择图片</span>
                    <span class="mt-1 text-xs text-muted">图片将保存到你的作品库</span>
                  </>
                }
              >
                <img src={preview()!} alt="上传预览" class="max-h-64 rounded-lg object-contain" />
              </Show>
            </label>

            {error() && (
              <div class="mt-4 rounded-lg border border-coral/30 bg-coral/10 px-4 py-3 text-sm text-coral">
                {error()}
              </div>
            )}

            <div class="mt-4 flex justify-end gap-3">
              <Show when={selected()}>
                <button
                  type="button"
                  onClick={() => {
                    setSelected(null);
                    setPreview(null);
                  }}
                  class="rounded-lg border border-line px-4 py-2.5 text-sm text-muted hover:text-paper"
                >
                  取消
                </button>
                <button
                  type="button"
                  onClick={handleUpload}
                  disabled={uploading()}
                  class="rounded-lg bg-amber px-5 py-2.5 text-sm font-medium text-ink transition-colors hover:bg-[#ffb46f] disabled:cursor-wait disabled:opacity-70"
                >
                  {uploading() ? "上传中" : "上传图片"}
                </button>
              </Show>
            </div>
          </section>
        </div>

        <section class="mt-10">
          <div class="mb-6 flex items-end justify-between">
            <div>
              <h2 class="font-display text-2xl font-semibold">我的作品库</h2>
              <p class="mt-1 text-sm text-muted">上传的图片会出现在这里</p>
            </div>
            <span class="text-sm text-muted">{myWorks().length} 张</span>
          </div>

          <Show
            when={myWorks().length > 0}
            fallback={
              <EmptyState
                icon={<ImageIcon size={20} />}
                title="还没有上传作品"
                description="先上传第一组作品，客户才能看到你的风格。"
                ctaLabel="上传作品"
                onClick={() => scrollToId("upload-work")}
              />
            }
          >
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <For each={myWorks()}>
                {(item) => (
                  <div class="group overflow-hidden rounded-lg border border-line bg-surface">
                    <div class="aspect-square overflow-hidden">
                      <img
                        src={item.image_url}
                        alt={item.title ?? "上传作品"}
                        loading="lazy"
                        class="size-full object-cover transition-transform duration-500 group-hover:scale-105"
                      />
                    </div>
                    <div class="flex items-center justify-between gap-3 p-3">
                      <div class="min-w-0">
                        <p class="truncate text-sm text-paper">{item.title ?? "未命名作品"}</p>
                        <p class="mt-0.5 text-xs text-muted">{new Date(item.created_at).toLocaleDateString("zh-CN")}</p>
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </section>

        <Show when={myProfile() && myServices().length === 0}>
          <EmptyState
            class="mt-10"
            icon={<Store size={20} />}
            title="还没有发布服务"
            description="设置服务类型、价格和地点后，客户就能直接预约。"
            ctaLabel="发布服务"
            onClick={() => scrollToId("publish-service")}
          />
        </Show>

        <section class="mt-10 grid gap-6 lg:grid-cols-2">
          <Card id="creator-profile">
            <CardHeader>
              <CardTitle class="flex items-center gap-2">
                <User size={18} class="text-primary" />
                创作者资料
              </CardTitle>
            </CardHeader>
            <CardContent class="space-y-4">
              <label class="block">
                <span class="mb-2 block text-sm text-muted">简介</span>
                <Input
                  value={intro()}
                  onInput={(event) => setIntro(event.currentTarget.value)}
                  placeholder="一句话介绍你的风格"
                />
              </label>
              <label class="block">
                <span class="mb-2 block text-sm text-muted">个人主页</span>
                <Input
                  value={bio()}
                  onInput={(event) => setBio(event.currentTarget.value)}
                  placeholder="详细介绍你的经历与作品"
                />
              </label>
              {profileMsg() && <p class="text-sm text-muted">{profileMsg()}</p>}
              <Button onClick={saveProfile} disabled={savingProfile()}>
                {savingProfile() ? "保存中" : myProfile() ? "更新资料" : "成为创作者"}
              </Button>
            </CardContent>
          </Card>

          <Card id="publish-service">
            <CardHeader>
              <CardTitle class="flex items-center gap-2">
                <Store size={18} class="text-primary" />
                发布服务
              </CardTitle>
            </CardHeader>
            <CardContent class="space-y-4">
              <div class="grid gap-4 sm:grid-cols-2">
                <label class="block">
                  <span class="mb-2 block text-sm text-muted">服务类型</span>
                  <select
                    value={serviceForm().type_id}
                    onChange={(event) => updateService("type_id", event.currentTarget.value)}
                    class="h-10 w-full rounded-lg border border-line bg-background px-3 text-sm text-foreground outline-none focus:border-ring"
                  >
                    <For each={serviceTypes.data ?? []}>
                      {(type) => <option value={type.id}>{type.name}</option>}
                    </For>
                  </select>
                </label>
                <label class="block">
                  <span class="mb-2 block text-sm text-muted">价格（元）</span>
                  <Input
                    value={serviceForm().price}
                    onInput={(event) => updateService("price", event.currentTarget.value)}
                    placeholder="3999"
                    inputMode="decimal"
                  />
                </label>
              </div>
              <label class="block">
                <span class="mb-2 block text-sm text-muted">服务标题</span>
                <Input
                  value={serviceForm().title}
                  onInput={(event) => updateService("title", event.currentTarget.value)}
                  placeholder="例如：城市人像写真"
                />
              </label>
              <div class="grid gap-4 sm:grid-cols-2">
                <label class="block">
                  <span class="mb-2 block text-sm text-muted">时长（分钟）</span>
                  <Input
                    value={serviceForm().duration}
                    onInput={(event) => updateService("duration", event.currentTarget.value)}
                    placeholder="120"
                    inputMode="numeric"
                  />
                </label>
                <label class="block">
                  <span class="mb-2 block text-sm text-muted">拍摄地点</span>
                  <Input
                    value={serviceForm().location}
                    onInput={(event) => updateService("location", event.currentTarget.value)}
                    placeholder="上海"
                  />
                </label>
              </div>
              {serviceMsg() && <p class="text-sm text-muted">{serviceMsg()}</p>}
              <Button onClick={publishService} disabled={savingService()}>
                {savingService() ? "发布中" : "发布服务"}
              </Button>
            </CardContent>
          </Card>
        </section>

        <section class="mt-10 grid gap-6 lg:grid-cols-[420px_1fr]">
          <Card>
            <CardHeader>
              <CardTitle class="flex items-center gap-2">
                <Wallet size={18} class="text-primary" />
                余额提现
              </CardTitle>
            </CardHeader>
            <CardContent class="space-y-4">
              <div class="rounded-lg border border-line bg-secondary p-4">
                <p class="text-xs text-muted">可提现余额</p>
                <p class="mt-2 font-display text-3xl font-semibold text-primary">
                  ¥{me.data?.balance ?? "0.00"}
                </p>
              </div>
              <label class="block">
                <span class="mb-2 block text-sm text-muted">提现金额</span>
                <Input
                  value={withdrawalAmount()}
                  onInput={(event) => setWithdrawalAmount(event.currentTarget.value)}
                  placeholder="例如：500.00"
                  inputMode="decimal"
                />
              </label>
              <label class="block">
                <span class="mb-2 block text-sm text-muted">收款账户</span>
                <Input
                  value={withdrawalAccount()}
                  onInput={(event) => setWithdrawalAccount(event.currentTarget.value)}
                  placeholder="支付宝账号或银行卡号"
                />
              </label>
              <Show when={withdrawalMsg()}>
                <p class="rounded-lg border border-teal/30 bg-teal/10 px-3 py-2 text-sm text-teal">
                  {withdrawalMsg()}
                </p>
              </Show>
              <Show when={withdrawalError()}>
                <p class="rounded-lg border border-coral/30 bg-coral/10 px-3 py-2 text-sm text-coral">
                  {withdrawalError()}
                </p>
              </Show>
              <Button onClick={submitWithdrawal} disabled={submittingWithdrawal()}>
                {submittingWithdrawal() ? "提交中" : "申请提现"}
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>提现记录</CardTitle>
            </CardHeader>
            <CardContent>
              <Show when={withdrawals.isPending}>
                <div class="grid min-h-32 place-items-center rounded-lg border border-dashed border-line text-sm text-muted">
                  正在加载提现记录...
                </div>
              </Show>
              <Show when={withdrawals.isError}>
                <div class="grid min-h-32 place-items-center rounded-lg border border-dashed border-line px-5 text-center text-sm text-muted">
                  暂无创作者提现记录，成为创作者后即可申请提现。
                </div>
              </Show>
              <Show when={withdrawals.data}>
                <Show
                  when={(withdrawals.data?.length ?? 0) > 0}
                  fallback={
                    <div class="grid min-h-32 place-items-center rounded-lg border border-dashed border-line text-sm text-muted">
                      还没有提现记录
                    </div>
                  }
                >
                  <div class="space-y-3">
                    <For each={withdrawals.data ?? []}>
                      {(item) => (
                        <div class="flex flex-col gap-3 rounded-lg border border-line bg-secondary p-4 sm:flex-row sm:items-center sm:justify-between">
                          <div>
                            <div class="flex flex-wrap items-center gap-3">
                              <span class="font-medium">¥{item.amount}</span>
                              <WithdrawalStatusBadge status={item.status} />
                            </div>
                            <p class="mt-1 text-xs text-muted">
                              {new Date(item.created_at).toLocaleString("zh-CN")}
                              {item.review_note ? ` · ${item.review_note}` : ""}
                            </p>
                          </div>
                          <span class="text-xs text-muted">
                            {item.completed_at
                              ? `完成于 ${new Date(item.completed_at).toLocaleDateString("zh-CN")}`
                              : "等待审核"}
                          </span>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </Show>
            </CardContent>
          </Card>
        </section>
        <section class="mt-10">
          <div class="mb-6 flex items-center gap-3">
            <CalendarClock size={20} class="text-amber" />
            <div>
              <h2 class="font-display text-2xl font-semibold">收到的预约</h2>
              <p class="mt-1 text-sm text-muted">客户付款后确认预约，并推进拍摄与交付状态</p>
            </div>
          </div>

          <Show when={appointmentMsg()}>
            <div class="mb-4 rounded-lg border border-teal/30 bg-teal/10 px-4 py-3 text-sm text-teal" role="status">
              {appointmentMsg()}
            </div>
          </Show>
          <Show when={appointmentError()}>
            <div class="mb-4 rounded-lg border border-coral/30 bg-coral/10 px-4 py-3 text-sm text-coral" role="alert">
              {appointmentError()}
            </div>
          </Show>

          <Show when={creatorAppointments.isPending}>
            <div class="grid min-h-32 place-items-center rounded-lg border border-dashed border-line text-sm text-muted">
              正在加载收到的预约...
            </div>
          </Show>
          <Show when={creatorAppointments.isError}>
            <div class="grid min-h-32 place-items-center rounded-lg border border-dashed border-line px-5 text-center text-sm text-muted">
              {creatorAppointmentsUnavailable()
                ? "成为创作者并完善资料后，这里会显示客户提交的预约。"
                : "暂时无法加载收到的预约，请稍后重试。"}
            </div>
          </Show>
          <Show when={creatorAppointments.data}>
            <Show
              when={(creatorAppointments.data?.length ?? 0) > 0}
              fallback={
                <div class="grid min-h-32 place-items-center rounded-lg border border-dashed border-line px-5 text-center text-sm text-muted">
                  暂时还没有客户预约。完善资料并发布服务后，客户就能在这里下单。
                </div>
              }
            >
              <div class="space-y-3">
                <For each={creatorAppointments.data ?? []}>
                  {(item) => (
                    <div class="flex flex-col gap-3 rounded-lg border border-line bg-surface p-4 sm:flex-row sm:items-center sm:justify-between">
                      <div>
                        <div class="flex flex-wrap items-center gap-3">
                          <span class="font-medium">{new Date(item.start_time).toLocaleString("zh-CN")}</span>
                          <StatusBadge status={item.status} />
                        </div>
                        <p class="mt-1 text-sm text-muted">
                          服务 #{item.service_id} · ¥{item.total_price}
                          {item.location ? ` · ${item.location}` : ""}
                        </p>
                      </div>
                      <div class="flex items-center gap-2">
                        <Show when={item.status === "pending"}>
                          <span class="text-xs text-muted">等待客户支付</span>
                        </Show>
                        <Show when={item.status === "confirmed"}>
                          <Button
                            size="sm"
                            onClick={() => advanceAppointment(item.id, "ongoing")}
                            disabled={appointmentActionId() === item.id}
                          >
                            {appointmentActionId() === item.id ? "处理中" : "开始"}
                          </Button>
                        </Show>
                        <Show when={item.status === "ongoing"}>
                          <Button
                            size="sm"
                            onClick={() => advanceAppointment(item.id, "completed")}
                            disabled={appointmentActionId() === item.id}
                          >
                            {appointmentActionId() === item.id ? "处理中" : "完成"}
                          </Button>
                        </Show>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </Show>
        </section>

      </main>
      <Assistant />
    </div>
  );
}

function StatusBadge(props: { status: string }) {
  const labels: Record<string, string> = {
    pending: "待支付",
    confirmed: "已确认",
    ongoing: "进行中",
    completed: "已完成",
    cancelled: "已取消",
    refunded: "已退款"
  };
  const variant: "default" | "accent" | "destructive" =
    props.status === "cancelled" || props.status === "refunded"
      ? "destructive"
      : props.status === "completed"
        ? "accent"
        : "default";
  return <Badge variant={variant}>{labels[props.status] ?? props.status}</Badge>;
}

function WithdrawalStatusBadge(props: { status: string }) {
  const labels: Record<string, string> = {
    pending: "待审核",
    approved: "已通过",
    completed: "已打款",
    rejected: "已拒绝"
  };
  const variant: "default" | "accent" | "destructive" =
    props.status === "rejected" ? "destructive" : props.status === "completed" ? "accent" : "default";
  return <Badge variant={variant}>{labels[props.status] ?? props.status}</Badge>;
}

import { A, useNavigate, useParams } from "@solidjs/router";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import {
  Award,
  BriefcaseBusiness,
  CalendarCheck,
  Clock,
  Heart,
  Images,
  MapPin,
  MessageSquareQuote,
  Star
} from "lucide-solid";
import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import { Avatar, AvatarFallback, AvatarImage } from "../components/ui/avatar";
import { Badge } from "../components/ui/badge";
import { Card, CardContent } from "../components/ui/card";
import { EmptyState, ErrorState, LoadingState, PaginationState } from "../components/ui/state";
import SiteFooter from "../components/layout/site-footer";
import { isAuthenticated } from "../lib/auth-api";
import { cn } from "../lib/cn";
import { favoriteStatus, toggleFavorite, type FavoriteStateDto } from "../lib/favorites-api";
import { getCreator, listServices } from "../lib/marketplace-api";
import { listCreatorReviews } from "../lib/reviews-api";
import { listWorks } from "../lib/works-api";

type ProfileTab = "services" | "works" | "reviews";

const REVIEW_PAGE_SIZE = 10;

const tabs: { id: ProfileTab; label: string }[] = [
  { id: "services", label: "服务" },
  { id: "works", label: "作品" },
  { id: "reviews", label: "评价" }
];

function formatReviewDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;

  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "short",
    day: "numeric"
  }).format(date);
}

function formatCertification(value: string) {
  const labels: Record<string, string> = {
    standard: "标准认证",
    professional: "专业认证",
    premium: "优选创作者",
    verified: "平台认证"
  };

  return labels[value] ?? value;
}

export default function CreatorProfile() {
  const params = useParams();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = createSignal<ProfileTab>("services");
  const [reviewPage, setReviewPage] = createSignal(1);
  const [pendingFavorite, setPendingFavorite] = createSignal(false);
  const [favoriteState, setFavoriteState] = createSignal<FavoriteStateDto | null>(null);
  const [favoriteError, setFavoriteError] = createSignal("");
  const creator = createQuery(() => ({
    queryKey: ["creator", params.id] as const,
    queryFn: () => getCreator(params.id ?? "")
  }));
  const favorite = createQuery(() => ({
    queryKey: ["favorite-status", "creator", params.id] as const,
    queryFn: () => favoriteStatus("creator", Number(params.id)),
    enabled: isAuthenticated() && Boolean(params.id)
  }));
  const services = createQuery(() => ({
    queryKey: ["services"] as const,
    queryFn: listServices
  }));
  const works = createQuery(() => ({
    queryKey: ["works"] as const,
    queryFn: listWorks
  }));
  const reviews = createQuery(() => ({
    queryKey: ["creator-reviews", params.id, reviewPage()] as const,
    queryFn: () =>
      listCreatorReviews(params.id ?? "", {
        page: reviewPage(),
        page_size: REVIEW_PAGE_SIZE
      })
  }));

  const ownServices = createMemo(() =>
    (services.data ?? []).filter((item) => item.creator_id === Number(params.id))
  );
  const ownWorks = createMemo(() => {
    const profile = creator.data;
    if (!profile) return [];

    return (works.data ?? []).filter((item) => item.user_id === profile.user_id);
  });
  const reviewList = createMemo(() => reviews.data?.items ?? []);
  const favoriteView = createMemo(
    () => favoriteState() ?? favorite.data ?? { favorited: false, count: 0 }
  );
  createEffect(() => {
    params.id;
    setFavoriteState(null);
    setFavoriteError("");
  });
  const reviewTotal = createMemo(() => reviews.data?.total ?? 0);
  const displayName = createMemo(() => {
    const nickname = creator.data?.nickname?.trim();
    if (nickname) return nickname;

    const creatorName = ownWorks().find((item) => item.creator_name)?.creator_name;
    return creatorName?.trim() || `创作者 #${creator.data?.id ?? params.id}`;
  });
  const avatarText = createMemo(() => displayName().replace(/^@/, "").trim().slice(0, 1) || "创");
  const averageRating = createMemo(() => {
    const rating = Number(creator.data?.rating ?? creator.data?.avg_rating ?? 0);
    return rating > 0 ? rating.toFixed(1) : null;
  });
  const serviceAreas = createMemo(() => {
    const areas = creator.data?.service_areas;
    if (!Array.isArray(areas)) return [];

    return areas.filter((item): item is string => typeof item === "string" && item.trim().length > 0);
  });

  async function handleFavoriteCreator() {
    if (!isAuthenticated()) {
      navigate("/login");
      return;
    }
    const profile = creator.data;
    if (!profile) return;

    setPendingFavorite(true);
    setFavoriteError("");
    try {
      const result = await toggleFavorite("creator", profile.id);
      setFavoriteState(result);
      queryClient.setQueryData(["favorite-status", "creator", params.id], result);
      await queryClient.invalidateQueries({ queryKey: ["favorites"] });
    } catch (err) {
      setFavoriteError(err instanceof Error ? err.message : "收藏失败，请稍后重试。");
    } finally {
      setPendingFavorite(false);
    }
  }

  const tabCount = (tab: ProfileTab) => {
    if (tab === "services") return ownServices().length;
    if (tab === "works") return ownWorks().length;
    return reviewTotal();
  };

  return (
    <div class="min-h-screen bg-background text-foreground">
      <main class="mx-auto max-w-6xl px-5 pt-24 pb-24 md:px-8 md:pt-28">
        <Show
          when={creator.data}
          fallback={
            creator.isLoading ? (
              <LoadingState title="正在加载创作者资料" description="正在读取个人主页和风格信息。" />
            ) : creator.isError ? (
              <ErrorState
                title="创作者资料加载失败"
                description="暂时无法获取创作者信息，请稍后重试。"
                onRetry={() => creator.refetch()}
              />
            ) : (
              <EmptyState
                title="创作者不存在"
                description="返回探索页看看其他创作者的影像风格。"
                ctaLabel="返回探索"
                href="/explore"
              />
            )
          }
        >
          {(profile) => (
            <>
              <A
                href="/explore"
                class="inline-flex items-center gap-2 text-sm text-muted no-underline transition-colors hover:text-foreground"
              >
                返回发现
              </A>

              <section class="mt-5 overflow-hidden rounded-lg border border-line bg-surface">
                <div class="grid gap-7 p-6 md:grid-cols-[auto_minmax(0,1fr)_auto] md:items-center md:p-8">
                  <Avatar class="size-24 border border-white/10 shadow-2xl shadow-black/20 md:size-28">
                    <Show when={profile().avatar_url}>
                      {(avatarUrl) => (
                        <AvatarImage src={avatarUrl()} alt={displayName()} />
                      )}
                    </Show>
                    <AvatarFallback class="bg-gradient-to-br from-amber via-coral to-teal text-3xl font-semibold text-ink md:text-4xl">
                      {avatarText()}
                    </AvatarFallback>
                  </Avatar>

                  <div class="min-w-0">
                    <div class="flex flex-wrap items-center gap-3">
                      <h1 class="font-display text-3xl font-semibold tracking-normal md:text-4xl">
                        {displayName()}
                      </h1>
                      <Badge variant="accent">
                        <Award size={13} class="mr-1" />
                        {formatCertification(profile().certification_level)}
                      </Badge>
                    </div>

                    <Show
                      when={profile().introduction || profile().bio}
                      fallback={
                        <p class="mt-3 max-w-3xl text-sm leading-7 text-muted">
                          这位创作者还没有补充个人介绍，你可以先浏览作品与服务。
                        </p>
                      }
                    >
                      <p class="mt-3 max-w-3xl text-sm leading-7 text-muted md:text-base">
                        {profile().introduction ?? profile().bio}
                      </p>
                    </Show>

                    <div class="mt-5 flex flex-wrap items-center gap-x-5 gap-y-3 text-sm">
                      <span class="inline-flex items-center gap-1.5 text-foreground">
                        <Star size={16} class="fill-current text-amber" />
                        <strong class="font-semibold">{averageRating() ?? "暂无评分"}</strong>
                        <Show when={reviewTotal() > 0}>
                          <span class="text-muted">({reviewTotal()} 条评价)</span>
                        </Show>
                      </span>
                      <Show when={serviceAreas().length > 0}>
                        <span class="inline-flex min-w-0 items-center gap-1.5 text-muted">
                          <MapPin size={15} class="shrink-0" />
                          <span class="truncate">{serviceAreas().join(" · ")}</span>
                        </span>
                      </Show>
                    </div>

                    <div class="mt-5 flex flex-wrap items-center gap-3">
                      <button
                        type="button"
                        onClick={handleFavoriteCreator}
                        disabled={pendingFavorite()}
                        aria-pressed={favoriteView().favorited}
                        class={cn(
                          "inline-flex h-10 items-center justify-center gap-2 rounded-lg border px-4 text-sm font-medium transition-colors disabled:opacity-60",
                          favoriteView().favorited
                            ? "border-coral/60 bg-coral/15 text-coral hover:bg-coral/20"
                            : "border-line bg-transparent text-foreground hover:bg-surface"
                        )}
                      >
                        <Heart
                          size={16}
                          class={favoriteView().favorited ? "fill-current" : ""}
                        />
                        {pendingFavorite()
                          ? "处理中"
                          : favoriteView().favorited
                            ? "已收藏创作者"
                            : "收藏创作者"}
                        <Show when={isAuthenticated() && favorite.data}>
                          <span class="text-xs text-muted">{favoriteView().count}</span>
                        </Show>
                      </button>
                      <Show when={favoriteError()}>
                        <span class="text-xs text-destructive" role="alert">
                          {favoriteError()}
                        </span>
                      </Show>
                    </div>
                  </div>

                  <div class="grid grid-cols-2 gap-3 md:w-56">
                    <div class="rounded-lg border border-line bg-ink/45 px-4 py-3">
                      <p class="text-xs text-muted">服务</p>
                      <p class="mt-1 font-display text-2xl font-semibold">
                        {profile().total_services || ownServices().length}
                      </p>
                    </div>
                    <div class="rounded-lg border border-line bg-ink/45 px-4 py-3">
                      <p class="text-xs text-muted">预约</p>
                      <p class="mt-1 font-display text-2xl font-semibold">
                        {profile().total_appointments}
                      </p>
                    </div>
                  </div>
                </div>

                <div class="grid gap-px border-t border-line bg-line sm:grid-cols-3">
                  <div class="flex items-center gap-3 bg-surface px-6 py-4">
                    <BriefcaseBusiness size={17} class="text-amber" />
                    <div>
                      <p class="text-xs text-muted">服务方向</p>
                      <p class="mt-0.5 text-sm">{ownServices().length > 0 ? "可在线预约" : "整理中"}</p>
                    </div>
                  </div>
                  <div class="flex items-center gap-3 bg-surface px-6 py-4">
                    <Images size={17} class="text-teal" />
                    <div>
                      <p class="text-xs text-muted">公开作品</p>
                      <p class="mt-0.5 text-sm">{ownWorks().length} 组作品</p>
                    </div>
                  </div>
                  <div class="flex items-center gap-3 bg-surface px-6 py-4">
                    <CalendarCheck size={17} class="text-coral" />
                    <div>
                      <p class="text-xs text-muted">服务记录</p>
                      <p class="mt-0.5 text-sm">{profile().total_appointments} 次完成或进行中</p>
                    </div>
                  </div>
                </div>
              </section>

              <div class="mt-9 flex gap-1 overflow-x-auto border-b border-line">
                <For each={tabs}>
                  {(tab) => (
                    <button
                      type="button"
                      onClick={() => setActiveTab(tab.id)}
                      class={cn(
                        "relative shrink-0 px-4 py-3 text-sm transition-colors",
                        activeTab() === tab.id
                          ? "font-medium text-foreground"
                          : "text-muted hover:text-foreground"
                      )}
                    >
                      {tab.label}
                      <Show when={tabCount(tab.id) > 0}>
                        <span class="ml-1.5 text-xs text-muted">{tabCount(tab.id)}</span>
                      </Show>
                      <Show when={activeTab() === tab.id}>
                        <span class="absolute inset-x-3 bottom-0 h-0.5 rounded-full bg-amber" />
                      </Show>
                    </button>
                  )}
                </For>
              </div>

              <Show when={activeTab() === "services"}>
                <Show
                  when={!services.isLoading}
                  fallback={
                    <LoadingState
                      class="mt-6 min-h-40"
                      title="正在加载服务"
                      description="正在读取这位创作者的预约服务。"
                    />
                  }
                >
                  <Show
                    when={!services.isError}
                    fallback={
                      <ErrorState
                        class="mt-6 min-h-40"
                        title="服务加载失败"
                        description="暂时无法获取服务列表。"
                        onRetry={() => services.refetch()}
                      />
                    }
                  >
                    <Show
                      when={ownServices().length > 0}
                      fallback={
                        <EmptyState
                          class="mt-6 min-h-40"
                          icon={<BriefcaseBusiness size={20} />}
                          title="暂未发布服务"
                          description="创作者发布服务后，你可以在这里直接查看价格、时长和拍摄地点。"
                        />
                      }
                    >
                      <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        <For each={ownServices()}>
                          {(service) => (
                            <A href={`/services/${service.id}`} class="group no-underline">
                              <Card class="h-full overflow-hidden transition-all duration-300 group-hover:-translate-y-0.5 group-hover:border-amber/60">
                                <div class="aspect-[16/10] overflow-hidden bg-ink">
                                  <img
                                    src={service.cover_image_url ?? "/demo/work-4.jpg"}
                                    alt={service.title}
                                    loading="lazy"
                                    class="size-full object-cover transition-transform duration-500 group-hover:scale-105"
                                  />
                                </div>
                                <CardContent class="p-5">
                                  <div class="flex items-start justify-between gap-3">
                                    <h3 class="min-w-0 line-clamp-2 font-medium leading-6">
                                      {service.title}
                                    </h3>
                                    <span class="shrink-0 font-semibold text-amber">
                                      ¥{service.price}
                                    </span>
                                  </div>
                                  <div class="mt-3 flex flex-wrap gap-x-4 gap-y-2 text-xs text-muted">
                                    <Show when={service.duration}>
                                      <span class="inline-flex items-center gap-1">
                                        <Clock size={13} />
                                        {service.duration} 分钟
                                      </span>
                                    </Show>
                                    <Show when={service.location}>
                                      <span class="inline-flex items-center gap-1">
                                        <MapPin size={13} />
                                        {service.location}
                                      </span>
                                    </Show>
                                  </div>
                                  <p class="mt-4 text-xs text-muted">
                                    {service.appointments_count} 次预约
                                  </p>
                                </CardContent>
                              </Card>
                            </A>
                          )}
                        </For>
                      </div>
                    </Show>
                  </Show>
                </Show>
              </Show>

              <Show when={activeTab() === "works"}>
                <Show
                  when={!works.isLoading}
                  fallback={
                    <LoadingState
                      class="mt-6 min-h-40"
                      title="正在加载作品"
                      description="正在整理这位创作者的公开作品。"
                    />
                  }
                >
                  <Show
                    when={!works.isError}
                    fallback={
                      <ErrorState
                        class="mt-6 min-h-40"
                        title="作品加载失败"
                        description="暂时无法获取公开作品。"
                        onRetry={() => works.refetch()}
                      />
                    }
                  >
                    <Show
                      when={ownWorks().length > 0}
                      fallback={
                        <EmptyState
                          class="mt-6 min-h-40"
                          icon={<Images size={20} />}
                          title="暂无公开作品"
                          description="作品上传后会在这里形成创作者的真实风格档案。"
                        />
                      }
                    >
                      <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        <For each={ownWorks()}>
                          {(work) => (
                            <A
                              href={`/works/${work.id}`}
                              class="group relative overflow-hidden rounded-lg border border-line bg-surface no-underline"
                            >
                              <div class="aspect-[4/5] overflow-hidden bg-ink">
                                <img
                                  src={work.image_url}
                                  alt={work.title ?? "作品"}
                                  loading="lazy"
                                  class="size-full object-cover transition-transform duration-500 group-hover:scale-105"
                                />
                              </div>
                              <div class="pointer-events-none absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/85 via-black/35 to-transparent p-5 pt-16">
                                <p class="line-clamp-1 font-medium text-white">
                                  {work.title ?? "未命名作品"}
                                </p>
                                <p class="mt-1 text-xs text-white/65">
                                  {work.category ?? "摄影作品"}
                                </p>
                              </div>
                            </A>
                          )}
                        </For>
                      </div>
                    </Show>
                  </Show>
                </Show>
              </Show>

              <Show when={activeTab() === "reviews"}>
                <section class="pt-6">
                  <div class="flex flex-wrap items-end justify-between gap-3">
                    <div>
                      <h2 class="font-display text-2xl font-semibold">用户评价</h2>
                      <p class="mt-2 text-sm text-muted">来自已完成预约的真实反馈</p>
                    </div>
                    <Show when={reviewTotal() > 0}>
                      <Badge variant="outline">{reviewTotal()} 条评价</Badge>
                    </Show>
                  </div>

                  <Show
                    when={!reviews.isLoading}
                    fallback={
                      <LoadingState
                        class="mt-6 min-h-40"
                        title="正在加载评价"
                        description="正在读取真实服务反馈。"
                      />
                    }
                  >
                    <Show
                      when={!reviews.isError}
                      fallback={
                        <ErrorState
                          class="mt-6 min-h-40"
                          title="评价加载失败"
                          description="暂时无法获取创作者评价。"
                          onRetry={() => reviews.refetch()}
                        />
                      }
                    >
                      <Show
                        when={reviewList().length > 0}
                        fallback={
                          <EmptyState
                            class="mt-6 min-h-40"
                            icon={<MessageSquareQuote size={20} />}
                            title="暂无用户评价"
                            description="完成服务后，客户评价会展示在这里。"
                          />
                        }
                      >
                        <div class="mt-6 grid gap-4 md:grid-cols-2">
                          <For each={reviewList()}>
                            {(review) => (
                              <Card class="h-full">
                                <CardContent class="p-5">
                                  <div class="flex flex-wrap items-center justify-between gap-3">
                                    <div
                                      class="flex items-center gap-1"
                                      aria-label={`评分 ${review.rating} 分`}
                                    >
                                      <For each={Array.from({ length: 5 }, (_, index) => index)}>
                                        {(index) => (
                                          <Star
                                            size={16}
                                            class={
                                              index < Math.round(Number(review.rating))
                                                ? "fill-current text-amber"
                                                : "text-line"
                                            }
                                          />
                                        )}
                                      </For>
                                      <span class="ml-1 text-sm font-medium">{review.rating}</span>
                                    </div>
                                    <time class="text-xs text-muted" dateTime={review.created_at}>
                                      {formatReviewDate(review.created_at)}
                                    </time>
                                  </div>
                                  <Show
                                    when={review.content}
                                    fallback={
                                      <p class="mt-4 text-sm text-muted">用户未填写文字评价</p>
                                    }
                                  >
                                    <p class="mt-4 whitespace-pre-wrap text-sm leading-6 text-foreground/90">
                                      {review.content}
                                    </p>
                                  </Show>
                                  <div class="mt-4 flex items-center justify-between gap-3">
                                    <Show when={review.is_anonymous}>
                                      <Badge variant="outline">匿名评价</Badge>
                                    </Show>
                                    <Show when={review.photographer_reply}>
                                      <span class="text-xs text-accent">创作者已回复</span>
                                    </Show>
                                  </div>
                                </CardContent>
                              </Card>
                            )}
                          </For>
                        </div>
                        <PaginationState
                          page={reviewPage()}
                          pageCount={Math.ceil(reviewTotal() / REVIEW_PAGE_SIZE)}
                          total={reviewTotal()}
                          disabled={reviews.isFetching}
                          onPageChange={setReviewPage}
                        />
                      </Show>
                    </Show>
                  </Show>
                </section>
              </Show>
            </>
          )}
        </Show>
      </main>
      <SiteFooter />
    </div>
  );
}

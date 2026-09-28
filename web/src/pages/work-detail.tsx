import { A, useNavigate, useParams } from "@solidjs/router";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import {
  ArrowLeft,
  CalendarDays,
  ChevronRight,
  FolderOpen,
  Heart,
  Images,
  UserRound
} from "lucide-solid";
import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import { Avatar, AvatarFallback } from "../components/ui/avatar";
import { Badge } from "../components/ui/badge";
import { EmptyState, ErrorState, LoadingState } from "../components/ui/state";
import Assistant from "../components/ai/assistant";
import { isAuthenticated } from "../lib/auth-api";
import { cn } from "../lib/cn";
import { favoriteStatus, toggleFavorite, type FavoriteStateDto } from "../lib/favorites-api";
import { listCreators } from "../lib/marketplace-api";
import { getWork, listWorks } from "../lib/works-api";

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;

  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "long",
    day: "numeric"
  }).format(date);
}

export default function WorkDetail() {
  const params = useParams();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [pendingFavorite, setPendingFavorite] = createSignal(false);
  const [favoriteState, setFavoriteState] = createSignal<FavoriteStateDto | null>(null);
  const [favoriteError, setFavoriteError] = createSignal("");
  const workDetail = createQuery(() => ({
    queryKey: ["work", params.id] as const,
    queryFn: () => getWork(params.id ?? "")
  }));
  const favorite = createQuery(() => ({
    queryKey: ["favorite-status", "work", params.id] as const,
    queryFn: () => favoriteStatus("work", Number(params.id)),
    enabled: isAuthenticated() && Boolean(params.id)
  }));
  const works = createQuery(() => ({
    queryKey: ["works"] as const,
    queryFn: listWorks
  }));
  const creators = createQuery(() => ({
    queryKey: ["creators"] as const,
    queryFn: listCreators
  }));

  const current = createMemo(() => workDetail.data);
  const favoriteView = createMemo(
    () => favoriteState() ?? favorite.data ?? { favorited: false, count: 0 }
  );
  createEffect(() => {
    // 切换到其他作品时清掉上一件作品留下的本地状态。
    params.id;
    setFavoriteState(null);
    setFavoriteError("");
  });
  const creatorProfile = createMemo(() => {
    const work = current();
    if (!work) return undefined;

    return (creators.data ?? []).find((profile) => profile.user_id === work.user_id);
  });
  const avatarText = createMemo(() => current()?.creator_name?.trim().slice(0, 1) || "创");
  const relatedWorks = createMemo(() => {
    const work = current();
    if (!work) return [];

    return (works.data ?? [])
      .filter((item) => {
        if (item.id === work.id) return false;
        if (item.user_id === work.user_id) return true;
        return Boolean(work.category && item.category === work.category);
      })
      .sort((left, right) => {
        const leftOwned = left.user_id === work.user_id ? 1 : 0;
        const rightOwned = right.user_id === work.user_id ? 1 : 0;
        return rightOwned - leftOwned;
      })
      .slice(0, 3);
  });

  async function handleFavorite() {
    if (!isAuthenticated()) {
      navigate("/login");
      return;
    }
    const work = current();
    if (!work) return;

    setPendingFavorite(true);
    setFavoriteError("");
    try {
      const result = await toggleFavorite("work", work.id);
      setFavoriteState(result);
      queryClient.setQueryData(["favorite-status", "work", params.id], result);
      await queryClient.invalidateQueries({ queryKey: ["favorites"] });
    } catch (err) {
      setFavoriteError(err instanceof Error ? err.message : "收藏失败，请稍后重试。");
    } finally {
      setPendingFavorite(false);
    }
  }

  return (
    <div class="min-h-screen bg-background text-foreground">
      <main class="mx-auto max-w-6xl px-5 pt-24 pb-24 md:px-8 md:pt-28">
        <A
          href="/explore"
          class="inline-flex items-center gap-2 text-sm text-muted no-underline transition-colors hover:text-foreground"
        >
          <ArrowLeft size={16} />
          返回探索
        </A>

        <Show
          when={current()}
          fallback={
            workDetail.isLoading ? (
              <LoadingState class="mt-8" title="正在加载作品" description="正在获取作品详情。" />
            ) : workDetail.isError ? (
              <ErrorState
                class="mt-8"
                title="作品加载失败"
                description="暂时无法获取作品详情，请稍后再试。"
                onRetry={() => workDetail.refetch()}
              />
            ) : (
              <EmptyState
                class="mt-16"
                icon={<Images size={20} />}
                title="作品不存在或已被删除"
                description="返回探索页看看其他创作者的最新作品。"
                ctaLabel="返回探索"
                href="/explore"
              />
            )
          }
        >
          {(work) => (
            <>
              <div class="mt-6 grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_360px]">
                <section class="overflow-hidden rounded-lg border border-line bg-black p-2 md:p-3">
                  <img
                    src={work().image_url}
                    alt={work().title ?? "摄影作品"}
                    class="mx-auto max-h-[78vh] w-full rounded-md object-contain"
                  />
                </section>

                <aside class="lg:sticky lg:top-24">
                  <div class="rounded-lg border border-line bg-surface">
                    <div class="border-b border-line p-6">
                      <div class="flex flex-wrap items-center gap-2">
                        <Badge variant="accent">
                          <FolderOpen size={13} class="mr-1" />
                          {work().category ?? "摄影作品"}
                        </Badge>
                        <span class="inline-flex items-center gap-1.5 text-xs text-muted">
                          <CalendarDays size={13} />
                          {formatDate(work().created_at)}
                        </span>
                      </div>
                      <h1 class="mt-5 font-display text-3xl leading-tight font-semibold">
                        {work().title ?? "未命名作品"}
                      </h1>
                      <Show
                        when={work().description}
                        fallback={
                          <p class="mt-4 text-sm leading-7 text-muted">
                            创作者暂未补充作品说明，你可以从画面、分类和创作者主页继续了解风格。
                          </p>
                        }
                      >
                        <p class="mt-4 whitespace-pre-wrap text-sm leading-7 text-muted">
                          {work().description}
                        </p>
                      </Show>
                    </div>

                    <div class="p-6">
                      <p class="text-xs font-medium tracking-wide text-muted uppercase">创作者</p>
                      <div class="mt-4 flex items-center gap-3">
                        <Avatar class="size-12 border border-white/10">
                          <AvatarFallback class="bg-gradient-to-br from-amber via-coral to-teal text-base font-semibold text-ink">
                            {avatarText()}
                          </AvatarFallback>
                        </Avatar>
                        <div class="min-w-0">
                          <p class="truncate font-medium">{work().creator_name || "匿名创作者"}</p>
                          <p class="mt-1 flex items-center gap-1.5 text-xs text-muted">
                            <UserRound size={13} />
                            平台认证创作者
                          </p>
                        </div>
                      </div>

                      <Show when={creatorProfile()}>
                        {(profile) => (
                          <A
                            href={`/creators/${profile().id}`}
                            class="mt-5 inline-flex h-11 w-full items-center justify-center gap-2 rounded-lg bg-primary px-4 text-sm font-medium text-primary-foreground no-underline transition-colors hover:bg-primary/85"
                          >
                            查看创作者主页
                            <ChevronRight size={16} />
                          </A>
                        )}
                      </Show>

                      <button
                        type="button"
                        onClick={handleFavorite}
                        disabled={pendingFavorite()}
                        aria-pressed={favoriteView().favorited}
                        class={cn(
                          "mt-3 inline-flex h-11 w-full items-center justify-center gap-2 rounded-lg border px-4 text-sm font-medium transition-colors disabled:opacity-60",
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
                            ? "已收藏"
                            : "收藏作品"}
                        <Show when={isAuthenticated() && favorite.data}>
                          <span class="text-xs text-muted">{favoriteView().count}</span>
                        </Show>
                      </button>
                      <Show when={favoriteError()}>
                        <p class="mt-2 text-xs text-destructive" role="alert">
                          {favoriteError()}
                        </p>
                      </Show>

                      <Show when={!creators.isLoading && !creatorProfile()}>
                        <p class="mt-5 rounded-lg border border-line bg-ink/35 px-4 py-3 text-xs leading-5 text-muted">
                          创作者主页暂未公开，你仍可浏览其余作品。
                        </p>
                      </Show>
                    </div>
                  </div>
                </aside>
              </div>

              <Show when={relatedWorks().length > 0}>
                <section class="mt-16 border-t border-line pt-10">
                  <div class="flex flex-wrap items-end justify-between gap-4">
                    <div>
                      <p class="text-xs font-medium tracking-wide text-amber uppercase">More works</p>
                      <h2 class="mt-2 font-display text-2xl font-semibold md:text-3xl">相关作品</h2>
                    </div>
                    <A
                      href="/explore"
                      class="inline-flex items-center gap-1.5 text-sm text-muted no-underline transition-colors hover:text-foreground"
                    >
                      浏览全部
                      <ChevronRight size={15} />
                    </A>
                  </div>

                  <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    <For each={relatedWorks()}>
                      {(item) => (
                        <A
                          href={`/works/${item.id}`}
                          class="group overflow-hidden rounded-lg border border-line bg-surface no-underline"
                        >
                          <div class="aspect-[4/3] overflow-hidden bg-ink">
                            <img
                              src={item.image_url}
                              alt={item.title ?? "相关作品"}
                              loading="lazy"
                              class="size-full object-cover transition-transform duration-500 group-hover:scale-105"
                            />
                          </div>
                          <div class="p-4">
                            <h3 class="line-clamp-1 font-medium">
                              {item.title ?? "未命名作品"}
                            </h3>
                            <div class="mt-2 flex items-center justify-between gap-3 text-xs text-muted">
                              <span class="truncate">@{item.creator_name}</span>
                              <span class="shrink-0">{item.category ?? "摄影"}</span>
                            </div>
                          </div>
                        </A>
                      )}
                    </For>
                  </div>
                </section>
              </Show>
            </>
          )}
        </Show>
      </main>

      <Assistant />
    </div>
  );
}

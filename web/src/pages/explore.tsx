import { A } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import { Search } from "lucide-solid";
import { createMemo, createSignal, For, Show } from "solid-js";
import { Badge } from "../components/ui/badge";
import { Input } from "../components/ui/input";
import { EmptyState, ErrorState, LoadingState } from "../components/ui/state";
import SiteFooter from "../components/layout/site-footer";
import { listWorks } from "../lib/works-api";

export default function Explore() {
  const works = createQuery(() => ({
    queryKey: ["works"] as const,
    queryFn: listWorks
  }));
  const [query, setQuery] = createSignal("");

  const filtered = createMemo(() => {
    const keyword = query().trim().toLowerCase();
    const list = works.data ?? [];
    if (!keyword) return list;
    return list.filter((item) => (item.title ?? "").toLowerCase().includes(keyword));
  });

  return (
    <div class="min-h-screen bg-background text-foreground">
      <main class="mx-auto max-w-7xl px-5 pt-24 pb-24 md:px-8 md:pt-28">
        <div class="flex flex-col gap-6 md:flex-row md:items-end md:justify-between">
          <div>
            <p class="text-sm font-medium tracking-wide text-accent uppercase">Explore</p>
            <h1 class="mt-3 font-display text-4xl font-semibold md:text-5xl">发现作品</h1>
            <p class="mt-3 max-w-xl text-muted">
              浏览平台创作者最新上传的影像，寻找与你审美同频的灵感。
            </p>
          </div>
          <div class="relative w-full md:w-72">
            <Search class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-muted" size={17} />
            <Input
              value={query()}
              onInput={(event) => setQuery(event.currentTarget.value)}
              placeholder="搜索作品标题"
              class="pl-9"
            />
          </div>
        </div>

        <Show when={works.isLoading}>
          <LoadingState class="mt-12" title="正在加载作品" description="正在从创作者作品库整理最新影像。" />
        </Show>

        <Show when={works.isError}>
          <ErrorState
            class="mt-12"
            title="作品加载失败"
            description="暂时无法获取作品列表，请检查网络后重试。"
            onRetry={() => works.refetch()}
          />
        </Show>

        <Show when={works.isSuccess && filtered().length === 0}>
          <EmptyState
            class="mt-12"
            title={query() ? "没有找到匹配的作品" : "还没有作品"}
            description={query() ? "换一个关键词，或清空搜索查看全部作品。" : "创作者上传作品后会展示在这里。"}
            ctaLabel={query() ? "清空搜索" : undefined}
            onClick={query() ? () => setQuery("") : undefined}
          />
        </Show>

        <Show when={works.isSuccess && filtered().length > 0}>
          <div class="mt-12 columns-1 gap-4 sm:columns-2 lg:columns-3 [&>*]:mb-4">
            <For each={filtered()}>
              {(item) => (
                <A
                  href={`/works/${item.id}`}
                  class="group block overflow-hidden rounded-lg border border-line bg-secondary no-underline"
                >
                  <div class="overflow-hidden">
                    <img
                      src={item.image_url}
                      alt={item.title ?? "作品"}
                      loading="lazy"
                      class="w-full object-cover transition-transform duration-500 group-hover:scale-105"
                    />
                  </div>
                  <div class="flex items-start justify-between gap-4 p-4">
                    <div class="min-w-0">
                      <h2 class="truncate font-medium text-foreground">{item.title ?? "未命名作品"}</h2>
                      <p class="mt-1 text-sm text-muted">
                        {new Date(item.created_at).toLocaleDateString("zh-CN")}
                      </p>
                    </div>
                    <Badge variant="outline">{item.category ?? "作品"}</Badge>
                  </div>
                </A>
              )}
            </For>
          </div>
        </Show>
      </main>

      <SiteFooter />
    </div>
  );
}

import { A } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import { Clock, MapPin, Search } from "lucide-solid";
import { createEffect, createMemo, createSignal, For, onCleanup, Show } from "solid-js";
import { Card, CardContent } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { EmptyState, ErrorState, LoadingState, PaginationState } from "../components/ui/state";
import SiteFooter from "../components/layout/site-footer";
import { listServicesPage, listServiceTypes } from "../lib/marketplace-api";

const PAGE_SIZE = 12;

export default function Services() {
  const [query, setQuery] = createSignal("");
  const [location, setLocation] = createSignal("");
  const [debouncedQuery, setDebouncedQuery] = createSignal("");
  const [debouncedLocation, setDebouncedLocation] = createSignal("");
  const [activeType, setActiveType] = createSignal<number | null>(null);
  const [page, setPage] = createSignal(1);

  let filterTimer: ReturnType<typeof setTimeout> | undefined;
  createEffect(() => {
    const keyword = query().trim();
    const place = location().trim();
    clearTimeout(filterTimer);
    filterTimer = setTimeout(() => {
      setDebouncedQuery(keyword);
      setDebouncedLocation(place);
      setPage(1);
    }, 300);
    onCleanup(() => clearTimeout(filterTimer));
  });

  const services = createQuery(() => ({
    queryKey: ["services", page(), activeType(), debouncedQuery(), debouncedLocation()] as const,
    queryFn: () => listServicesPage({
      page: page(),
      page_size: PAGE_SIZE,
      type_id: activeType() ?? undefined,
      q: debouncedQuery() || undefined,
      location: debouncedLocation() || undefined
    })
  }));
  const types = createQuery(() => ({ queryKey: ["service-types"] as const, queryFn: listServiceTypes }));
  const items = createMemo(() => services.data?.items ?? []);
  const pageCount = createMemo(() => Math.ceil((services.data?.total ?? 0) / PAGE_SIZE));

  const clearFilters = () => {
    setQuery("");
    setLocation("");
    setDebouncedQuery("");
    setDebouncedLocation("");
    setActiveType(null);
    setPage(1);
  };

  return (
    <div class="min-h-screen bg-background text-foreground">
      <main class="mx-auto max-w-7xl px-5 pt-24 pb-24 md:px-8 md:pt-28">
        <p class="text-sm font-medium tracking-wide text-accent uppercase">Services</p>
        <h1 class="mt-3 font-display text-4xl font-semibold md:text-5xl">预约创作者服务</h1>
        <p class="mt-3 max-w-2xl text-muted">
          从拍摄、妆造到视觉设计，按类型、关键词和城市找到合适的创作者。
        </p>

        <div class="mt-8 grid gap-3 md:grid-cols-2 md:max-w-3xl">
          <div class="relative">
            <Search class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-muted" size={17} />
            <Input
              value={query()}
              onInput={(event) => setQuery(event.currentTarget.value)}
              placeholder="搜索服务名称"
              class="pl-9"
            />
          </div>
          <div class="relative">
            <MapPin class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-muted" size={17} />
            <Input
              value={location()}
              onInput={(event) => setLocation(event.currentTarget.value)}
              placeholder="按城市筛选"
              class="pl-9"
            />
          </div>
        </div>

        <div class="mt-5 flex gap-2 overflow-x-auto pb-2">
          <button
            type="button"
            onClick={() => {
              setActiveType(null);
              setPage(1);
            }}
            class={activeType() === null
              ? "shrink-0 rounded-full bg-primary px-4 py-2 text-sm text-primary-foreground"
              : "shrink-0 rounded-full border border-line px-4 py-2 text-sm text-muted hover:text-foreground"}
          >
            全部
          </button>
          <For each={types.data ?? []}>
            {(type) => (
              <button
                type="button"
                onClick={() => {
                  setActiveType(type.id);
                  setPage(1);
                }}
                class={activeType() === type.id
                  ? "shrink-0 rounded-full bg-primary px-4 py-2 text-sm text-primary-foreground"
                  : "shrink-0 rounded-full border border-line px-4 py-2 text-sm text-muted hover:text-foreground"}
              >
                {type.name}
              </button>
            )}
          </For>
        </div>

        <Show when={services.isLoading && !services.data}>
          <LoadingState class="mt-10" title="正在加载服务" description="正在整理可预约的创作者服务。" />
        </Show>

        <Show when={services.isError}>
          <ErrorState
            class="mt-10"
            title="服务加载失败"
            description="暂时无法获取服务列表，请稍后重试。"
            onRetry={() => services.refetch()}
          />
        </Show>

        <Show when={services.isSuccess && items().length === 0}>
          <EmptyState
            class="mt-10"
            title="暂无匹配服务"
            description={query() || location() || activeType()
              ? "换一个关键词、城市或服务类型再试。"
              : "创作者发布服务后，会展示在这里供客户预约。"}
            ctaLabel={query() || location() || activeType() ? "清除筛选" : undefined}
            onClick={query() || location() || activeType() ? clearFilters : undefined}
          />
        </Show>

        <Show when={services.isSuccess && items().length > 0}>
          <div class="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <For each={items()}>
              {(service) => (
                <A href={`/services/${service.id}`} class="group no-underline">
                  <Card class="overflow-hidden transition-colors group-hover:border-primary/60">
                    <div class="aspect-[16/10] overflow-hidden bg-secondary">
                      <img
                        src={service.cover_image_url ?? "/demo/work-4.jpg"}
                        alt={service.title}
                        loading="lazy"
                        class="size-full object-cover transition-transform duration-500 group-hover:scale-105"
                      />
                    </div>
                    <CardContent class="pt-5">
                      <div class="flex items-start justify-between gap-3">
                        <div class="min-w-0">
                          <h2 class="truncate font-medium text-foreground">{service.title}</h2>
                          <div class="mt-2 flex flex-wrap items-center gap-3 text-sm text-muted">
                            {service.location && (
                              <span class="flex items-center gap-1"><MapPin size={14} />{service.location}</span>
                            )}
                            {service.duration && (
                              <span class="flex items-center gap-1"><Clock size={14} />{service.duration} 分钟</span>
                            )}
                          </div>
                        </div>
                        <div class="text-right">
                          <p class="text-lg font-semibold text-primary">¥{service.price}</p>
                          <p class="mt-1 text-xs text-muted">{service.appointments_count} 次预约</p>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                </A>
              )}
            </For>
          </div>
        </Show>

        <Show when={services.isSuccess && (services.data?.total ?? 0) > 0}>
          <PaginationState
            page={page()}
            pageCount={pageCount()}
            total={services.data?.total ?? 0}
            disabled={services.isFetching}
            onPageChange={setPage}
          />
        </Show>
      </main>
      <SiteFooter />
    </div>
  );
}

import { A } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import { Clock, MapPin } from "lucide-solid";
import { createMemo, createSignal, For, Show } from "solid-js";
import { Card, CardContent } from "../components/ui/card";
import { Skeleton } from "../components/ui/skeleton";
import SiteFooter from "../components/layout/site-footer";
import SiteHeader from "../components/layout/site-header";
import { listServices, listServiceTypes } from "../lib/marketplace-api";

export default function Services() {
  const services = createQuery(() => ({ queryKey: ["services"] as const, queryFn: listServices }));
  const types = createQuery(() => ({ queryKey: ["service-types"] as const, queryFn: listServiceTypes }));
  const [activeType, setActiveType] = createSignal<number | null>(null);

  const filtered = createMemo(() => {
    const list = services.data ?? [];
    const type = activeType();
    if (type === null) return list;
    return list.filter((item) => item.type_id === type);
  });

  return (
    <div class="min-h-screen bg-background text-foreground">
      <SiteHeader />
      <main class="mx-auto max-w-7xl px-5 pt-24 pb-24 md:px-8 md:pt-28">
        <p class="text-sm font-medium tracking-wide text-accent uppercase">Services</p>
        <h1 class="mt-3 font-display text-4xl font-semibold md:text-5xl">预约创作者服务</h1>

        <div class="mt-8 flex gap-2 overflow-x-auto pb-2">
          <button
            type="button"
            onClick={() => setActiveType(null)}
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
                onClick={() => setActiveType(type.id)}
                class={activeType() === type.id
                  ? "shrink-0 rounded-full bg-primary px-4 py-2 text-sm text-primary-foreground"
                  : "shrink-0 rounded-full border border-line px-4 py-2 text-sm text-muted hover:text-foreground"}
              >
                {type.name}
              </button>
            )}
          </For>
        </div>

        <Show when={services.isLoading}>
          <div class="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {Array.from({ length: 6 }).map(() => <Skeleton class="h-52 w-full rounded-lg" />)}
          </div>
        </Show>

        <Show when={services.isSuccess && filtered().length === 0}>
          <div class="mt-10 grid min-h-48 place-items-center rounded-lg border border-dashed border-line text-muted">
            暂无服务
          </div>
        </Show>

        <div class="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <For each={filtered()}>
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
      </main>
      <SiteFooter />
    </div>
  );
}

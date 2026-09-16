import { A, useParams } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import { Award, Clock, MapPin, Star } from "lucide-solid";
import { createMemo, Show, For } from "solid-js";
import { Avatar, AvatarFallback } from "../components/ui/avatar";
import { Badge } from "../components/ui/badge";
import { Card, CardContent } from "../components/ui/card";
import { Skeleton } from "../components/ui/skeleton";
import SiteHeader from "../components/layout/site-header";
import SiteFooter from "../components/layout/site-footer";
import { getCreator, listServices } from "../lib/marketplace-api";

export default function CreatorProfile() {
  const params = useParams();
  const creator = createQuery(() => ({
    queryKey: ["creator", params.id] as const,
    queryFn: () => getCreator(params.id ?? "")
  }));
  const services = createQuery(() => ({ queryKey: ["services"] as const, queryFn: listServices }));

  const ownServices = createMemo(() =>
    (services.data ?? []).filter((item) => item.creator_id === Number(params.id))
  );

  return (
    <div class="min-h-screen bg-background text-foreground">
      <SiteHeader />
      <main class="mx-auto max-w-6xl px-5 pt-24 pb-24 md:px-8 md:pt-28">
        <Show
          when={creator.data}
          fallback={
            creator.isLoading ? <Skeleton class="h-40 w-full rounded-lg" /> : (
              <div class="text-center text-muted">创作者不存在</div>
            )
          }
        >
          <div class="flex flex-col gap-6 border-b border-line pb-8 md:flex-row md:items-center">
            <Avatar class="size-24 rounded-full border border-line">
              <AvatarFallback class="text-2xl">
                {(creator.data!.bio ?? "创作者").slice(0, 1)}
              </AvatarFallback>
            </Avatar>
            <div class="min-w-0 flex-1">
              <div class="flex flex-wrap items-center gap-3">
                <h1 class="font-display text-3xl font-semibold">创作者 #{creator.data!.id}</h1>
                <Badge variant="accent" class="capitalize">
                  <Award size={13} class="mr-1" />
                  {creator.data!.certification_level}
                </Badge>
              </div>
              {creator.data!.introduction && (
                <p class="mt-2 max-w-2xl leading-7 text-muted">{creator.data!.introduction}</p>
              )}
              <div class="mt-4 flex flex-wrap gap-5 text-sm">
                <span class="flex items-center gap-1.5"><Star size={15} class="text-primary" />{creator.data!.rating}</span>
                <span class="text-muted">{creator.data!.total_services} 个服务</span>
                <span class="text-muted">{creator.data!.total_appointments} 次预约</span>
              </div>
            </div>
          </div>

          <section class="mt-10">
            <h2 class="font-display text-2xl font-semibold">发布的服务</h2>
            <Show
              when={ownServices().length > 0}
              fallback={<div class="mt-6 text-muted">暂无服务</div>}
            >
              <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                <For each={ownServices()}>
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
                            <h3 class="min-w-0 truncate font-medium">{service.title}</h3>
                            <span class="font-semibold text-primary">¥{service.price}</span>
                          </div>
                          <div class="mt-2 flex flex-wrap gap-3 text-xs text-muted">
                            {service.duration && <span class="flex items-center gap-1"><Clock size={13} />{service.duration} 分钟</span>}
                            {service.location && <span class="flex items-center gap-1"><MapPin size={13} />{service.location}</span>}
                          </div>
                        </CardContent>
                      </Card>
                    </A>
                  )}
                </For>
              </div>
            </Show>
          </section>
        </Show>
      </main>
      <SiteFooter />
    </div>
  );
}

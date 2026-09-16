import { A, useParams } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import { ArrowLeft, CalendarDays, Heart, MessageCircle } from "lucide-solid";
import { createSignal, Show } from "solid-js";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Skeleton } from "../components/ui/skeleton";
import SiteHeader from "../components/layout/site-header";
import Assistant from "../components/ai/assistant";
import { listWorks } from "../lib/works-api";

export default function WorkDetail() {
  const params = useParams();
  const works = createQuery(() => ({
    queryKey: ["works"] as const,
    queryFn: listWorks
  }));
  const [liked, setLiked] = createSignal(false);

  const current = () =>
    (works.data ?? []).find((item) => item.id === Number(params.id));

  return (
    <div class="min-h-screen bg-background text-foreground">
      <SiteHeader />

      <main class="mx-auto max-w-5xl px-5 pt-24 pb-24 md:px-8 md:pt-28">
        <A href="/explore" class="inline-flex items-center gap-2 text-sm text-muted no-underline hover:text-foreground">
          <ArrowLeft size={16} />
          返回探索
        </A>

        <Show
          when={current()}
          fallback={
            works.isLoading ? (
              <div class="mt-8 space-y-4">
                <Skeleton class="aspect-[4/3] w-full rounded-lg" />
                <Skeleton class="h-8 w-2/3" />
              </div>
            ) : (
              <div class="mt-16 text-center text-muted">作品不存在或已被删除</div>
            )
          }
        >
          <div class="mt-8 overflow-hidden rounded-lg border border-line bg-secondary">
            <img
              src={current()!.image_url}
              alt={current()!.title ?? "作品"}
              class="max-h-[70vh] w-full object-contain"
            />
          </div>

          <div class="mt-8 flex flex-col gap-6 md:flex-row md:items-start md:justify-between">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-3">
                <h1 class="font-display text-3xl font-semibold">
                  {current()!.title ?? "未命名作品"}
                </h1>
                <Badge variant="accent">用户作品</Badge>
              </div>
              <div class="mt-4 flex flex-wrap items-center gap-4 text-sm text-muted">
                <span class="flex items-center gap-1.5">
                  <CalendarDays size={15} />
                  {new Date(current()!.created_at).toLocaleDateString("zh-CN")}
                </span>
                <span>@新创作者</span>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <Button
                variant={liked() ? "destructive" : "outline"}
                size="lg"
                onClick={() => setLiked(!liked())}
              >
                <Heart size={18} class={liked() ? "fill-current" : ""} />
                {liked() ? "已收藏" : "收藏"}
              </Button>
              <Button size="lg">
                <MessageCircle size={18} />
                联系创作者
              </Button>
            </div>
          </div>
        </Show>
      </main>

      <Assistant />
    </div>
  );
}

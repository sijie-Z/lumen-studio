import { A } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import {
  Aperture,
  ArrowRight,
  Brush,
  Camera,
  Clapperboard,
  Image,
  MessageCircle,
  Scan,
  Sparkles,
  WandSparkles
} from "lucide-solid";
import { createMemo, For, Show } from "solid-js";
import { listWorks } from "../lib/works-api";
import SiteFooter from "../components/layout/site-footer";
import Assistant from "../components/ai/assistant";
import { EmptyState } from "../components/ui/state";
import { Button } from "../components/ui/button";
import { Skeleton } from "../components/ui/skeleton";
import { StepGuide, type GuideStep } from "../components/onboarding/step-guide";
import { isAuthenticated } from "../lib/auth-api";

const categories = [
  { icon: Camera, title: "人像摄影", description: "写真、肖像、家庭记录", tint: "bg-amber/15 text-amber" },
  { icon: Aperture, title: "婚礼纪实", description: "婚礼、活动、长期跟拍", tint: "bg-coral/15 text-coral" },
  { icon: Image, title: "商业拍摄", description: "品牌、产品、空间影像", tint: "bg-teal/15 text-teal" },
  { icon: Clapperboard, title: "短片影像", description: "短片、纪录片、内容制作", tint: "bg-sky-300/15 text-sky-300" }
];

export default function Home() {
  const uploadedWorks = createQuery(() => ({
    queryKey: ["works"] as const,
    queryFn: listWorks
  }));
  const displayWorks = createMemo(() =>
    (uploadedWorks.data ?? []).slice(0, 9).map((item) => ({
      id: item.id,
      src: item.image_url,
      title: item.title ?? "未命名作品",
      creator: item.creator_name,
      tag: item.category ?? "摄影作品",
      href: `/works/${item.id}`
    }))
  );
  const visitorSteps = (): GuideStep[] => [
    {
      title: "注册或登录",
      description: "使用一个账号管理预约、支付和评价。",
      done: isAuthenticated()
    },
    {
      title: "选择服务或创作者",
      description: "从作品风格、价格和拍摄地点找到合适的人。",
      done: false
    },
    {
      title: "提交预约并支付",
      description: "在服务详情选择时间，再到客户中心完成支付。",
      done: false
    }
  ];

  return (
    <div class="min-h-screen overflow-x-hidden bg-ink text-paper">
      <main>
        <section
          id="top"
          class="relative flex min-h-[78svh] items-center overflow-hidden lg:min-h-[82svh]"
          style={{
            "background-image":
              "linear-gradient(180deg, rgba(12,15,20,0.1) 0%, rgba(12,15,20,0.42) 52%, rgba(12,15,20,0.98) 100%), url('/demo/hero.jpg')",
            "background-size": "cover",
            "background-position": "center"
          }}
        >
          <div class="mx-auto w-full max-w-7xl px-5 pt-28 pb-20 md:px-8 md:pt-32 md:pb-24">
            <div class="max-w-3xl">
              <div class="mb-6 inline-flex items-center gap-2 rounded-full border border-white/15 bg-black/25 px-3 py-1.5 text-xs text-paper/90 backdrop-blur">
                <Sparkles size={14} class="text-amber" />
                AI 原生创意服务平台
              </div>
              <h1 class="font-display text-5xl leading-[1.04] font-semibold text-paper sm:text-6xl lg:text-7xl">
                Lumina
              </h1>
              <p class="mt-5 max-w-2xl text-lg leading-8 text-paper/80 md:text-xl">
                发现值得被定格的瞬间，遇见与你同频的影像创作者。
              </p>
              <div class="mt-9 flex flex-col gap-3 sm:flex-row">
                <A
                  href="/explore"
                  class="inline-flex items-center justify-center gap-2 rounded-lg bg-amber px-6 py-3.5 text-sm font-medium text-ink no-underline transition-all hover:-translate-y-0.5 hover:bg-amber/90"
                >
                  探索作品与创作者
                  <ArrowRight size={17} />
                </A>
                <A
                  href="/register"
                  class="inline-flex items-center justify-center gap-2 rounded-lg border border-white/20 bg-black/25 px-6 py-3.5 text-sm text-paper no-underline backdrop-blur transition-colors hover:border-amber hover:bg-black/40"
                >
                  <WandSparkles size={17} />
                  加入创作
                </A>
              </div>
              <div class="mt-10 flex flex-wrap gap-x-6 gap-y-3 text-xs text-paper/65">
                <span>真实创作者与作品</span>
                <span>AI 风格匹配</span>
                <span>预约到交付全流程</span>
              </div>
            </div>
          </div>
        </section>

        <section class="border-t border-white/8 px-5 py-6 md:px-8">
          <div class="mx-auto max-w-7xl">
            <StepGuide
              id="home-onboarding"
              title="新访客从这里开始"
              description="三步完成第一次创意服务预约。"
              steps={visitorSteps()}
            />
          </div>
        </section>

        <section id="services" class="scroll-mt-24 border-t border-white/8 px-5 py-24 md:px-8">
          <div class="mx-auto max-w-7xl">
            <div class="mb-10 flex items-end justify-between gap-6">
              <div>
                <p class="text-xs font-medium tracking-wide text-amber uppercase">Services</p>
                <h2 class="mt-3 font-display text-3xl font-semibold md:text-4xl">选择你的视觉方向</h2>
                <p class="mt-3 max-w-xl text-sm leading-6 text-muted">
                  按服务类型浏览，比较作品、价格与创作者资历。
                </p>
              </div>
              <A
                href="/services"
                class="hidden items-center gap-2 text-sm text-muted no-underline transition-colors hover:text-paper md:inline-flex"
              >
                查看全部
                <ArrowRight size={15} />
              </A>
            </div>
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <For each={categories}>
                {(item) => (
                  <A
                    href={`/services?category=${encodeURIComponent(item.title)}`}
                    class="group flex min-h-44 flex-col justify-between rounded-lg border border-line bg-surface p-6 no-underline transition-all duration-300 hover:-translate-y-0.5 hover:border-amber/60 hover:bg-surface/80"
                  >
                    <div class={`grid size-11 place-items-center rounded-lg ${item.tint}`}>
                      <item.icon size={21} />
                    </div>
                    <div class="mt-7">
                      <div class="flex items-center justify-between gap-3">
                        <h3 class="text-lg font-medium text-paper">{item.title}</h3>
                        <ArrowRight
                          size={16}
                          class="text-muted transition-transform group-hover:translate-x-0.5 group-hover:text-amber"
                        />
                      </div>
                      <p class="mt-2 text-sm leading-6 text-muted">{item.description}</p>
                    </div>
                  </A>
                )}
              </For>
            </div>
          </div>
        </section>

        <section id="ai" class="scroll-mt-24 border-t border-white/8 px-5 py-24 md:px-8">
          <div class="mx-auto grid max-w-7xl items-center gap-12 lg:grid-cols-[0.9fr_1.1fr]">
            <div>
              <div class="inline-flex items-center gap-2 rounded-lg border border-teal/30 bg-teal/10 px-3 py-1.5 text-xs text-teal">
                <Sparkles size={14} />
                AI Studio
              </div>
              <h2 class="mt-5 font-display text-3xl leading-tight font-semibold md:text-5xl">
                把模糊的想法，变成可执行的创意
              </h2>
              <p class="mt-5 max-w-xl text-base leading-7 text-muted">
                用自然语言描述需求，AI 助手会匹配风格、推荐创作者、生成灵感方案并协助完成预约。
              </p>
              <div class="mt-8 grid gap-3 sm:grid-cols-2">
                <For
                  each={[
                    { icon: MessageCircle, text: "多轮创意对话" },
                    { icon: WandSparkles, text: "风格智能匹配" },
                    { icon: Brush, text: "灵感板生成" },
                    { icon: Scan, text: "图像质量分析" }
                  ]}
                >
                  {(item) => (
                    <div class="flex items-center gap-3 rounded-lg border border-line bg-surface px-4 py-3 transition-colors hover:border-teal/40">
                      <item.icon size={18} class="text-amber" />
                      <span class="text-sm text-paper">{item.text}</span>
                    </div>
                  )}
                </For>
              </div>
            </div>
            <div class="relative aspect-[4/3] overflow-hidden rounded-lg border border-white/10 bg-surface">
              <img
                src="/demo/ai-studio.jpg"
                alt="AI 创意工作室"
                class="size-full object-cover transition-transform duration-700 hover:scale-105"
              />
              <div class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/85 via-black/35 to-transparent p-6 pt-20 md:p-7">
                <div class="flex items-end justify-between gap-5">
                  <div>
                    <p class="font-display text-2xl font-semibold">风格 DNA</p>
                    <p class="mt-1 text-sm text-white/70">色调 · 构图 · 情绪</p>
                  </div>
                  <span class="hidden rounded-full border border-white/20 bg-black/25 px-3 py-1.5 text-xs text-white/80 sm:inline-flex">
                    向量检索
                  </span>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section id="creators" class="scroll-mt-24 border-t border-white/8 px-5 py-24 md:px-8">
          <div class="mx-auto max-w-7xl">
            <div class="mb-10 flex items-end justify-between gap-6">
              <div>
                <p class="text-xs font-medium tracking-wide text-coral uppercase">Featured</p>
                <h2 class="mt-3 font-display text-3xl font-semibold md:text-4xl">精选作品</h2>
                <p class="mt-3 text-sm leading-6 text-muted">从真实作品中找到你想表达的气质。</p>
              </div>
              <A
                href="/explore"
                class="hidden items-center gap-2 text-sm text-muted no-underline transition-colors hover:text-paper md:inline-flex"
              >
                浏览全部作品
                <ArrowRight size={15} />
              </A>
            </div>

            <Show when={uploadedWorks.isLoading}>
              <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                <For each={Array.from({ length: 6 })}>
                  {() => (
                    <div class="overflow-hidden rounded-lg border border-line bg-surface">
                      <Skeleton class="aspect-[4/5] rounded-none" />
                      <div class="space-y-3 p-4">
                        <Skeleton class="h-4 w-2/3" />
                        <Skeleton class="h-3 w-1/3" />
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            <Show when={uploadedWorks.isError}>
              <div class="rounded-lg border border-coral/25 bg-coral/8 p-8 text-center">
                <p class="font-medium text-paper">精选作品加载失败</p>
                <p class="mt-2 text-sm text-muted">网络恢复后可以重新获取最新作品。</p>
                <Button class="mt-5" size="sm" variant="outline" onClick={() => uploadedWorks.refetch()}>
                  重新加载
                </Button>
              </div>
            </Show>

            <Show when={!uploadedWorks.isLoading && !uploadedWorks.isError && displayWorks().length === 0}>
              <EmptyState
                icon={<Image size={20} />}
                title="还没有公开展示的作品"
                description="创作者上传作品后，这里会展示真实的风格样本。"
                ctaLabel="浏览服务"
                href="/services"
              />
            </Show>

            <Show when={!uploadedWorks.isLoading && !uploadedWorks.isError && displayWorks().length > 0}>
              <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                <For each={displayWorks()}>
                  {(work) => (
                    <A
                      href={work.href}
                      class="group overflow-hidden rounded-lg border border-line bg-surface no-underline transition-all duration-300 hover:-translate-y-0.5 hover:border-coral/50"
                    >
                      <div class="aspect-[4/5] overflow-hidden bg-ink">
                        <img
                          src={work.src}
                          alt={work.title}
                          loading="lazy"
                          class="size-full object-cover transition-transform duration-500 group-hover:scale-105"
                        />
                      </div>
                      <div class="flex items-start justify-between gap-4 p-4">
                        <div class="min-w-0">
                          <h3 class="truncate font-medium text-paper">{work.title}</h3>
                          <p class="mt-1 truncate text-sm text-muted">@{work.creator}</p>
                        </div>
                        <span class="shrink-0 rounded-full border border-line px-2.5 py-1 text-xs text-muted">
                          {work.tag}
                        </span>
                      </div>
                    </A>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </section>
      </main>

      <SiteFooter />
      <Assistant />
    </div>
  );
}

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
import { createMemo } from "solid-js";
import { listWorks } from "../lib/works-api";
import SiteFooter from "../components/layout/site-footer";
import SiteHeader from "../components/layout/site-header";
import Assistant from "../components/ai/assistant";
import { StepGuide, type GuideStep } from "../components/onboarding/step-guide";
import { isAuthenticated } from "../lib/auth-api";

const categories = [
  { icon: Camera, title: "人像摄影", tint: "bg-amber/15 text-amber" },
  { icon: Aperture, title: "婚礼纪实", tint: "bg-coral/15 text-coral" },
  { icon: Image, title: "商业拍摄", tint: "bg-teal/15 text-teal" },
  { icon: Clapperboard, title: "短片影像", tint: "bg-sky-300/15 text-sky-300" }
];

export default function Home() {
  const uploadedWorks = createQuery(() => ({
    queryKey: ["works"] as const,
    queryFn: listWorks
  }));
  const displayWorks = createMemo(() => {
    return (uploadedWorks.data ?? []).slice(0, 9).map((item) => ({
      src: item.image_url,
      title: item.title ?? "未命名作品",
      creator: item.creator_name,
      tag: item.category ?? "作品",
      href: `/works/${item.id}`
    }));
  });
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
    <div class="min-h-screen bg-ink text-paper">
      <SiteHeader />

      <main>
        <section
          id="top"
          class="relative flex min-h-[92svh] items-center overflow-hidden"
          style={{
            "background-image":
              "linear-gradient(180deg, rgba(12,15,20,0.08) 0%, rgba(12,15,20,0.44) 48%, rgba(12,15,20,0.96) 100%), url('/demo/hero.jpg')",
            "background-size": "cover",
            "background-position": "center"
          }}
        >
          <div class="mx-auto w-full max-w-7xl px-5 pt-24 pb-32 md:px-8 md:pt-28">
            <div class="max-w-3xl">
              <div class="mb-6 inline-flex items-center gap-2 rounded-full border border-white/15 bg-black/20 px-3 py-1.5 text-xs text-paper/90 backdrop-blur">
                <Sparkles size={14} class="text-amber" />
                AI 原生创意服务平台
              </div>
              <h1 class="font-display text-5xl leading-[1.08] font-semibold text-paper sm:text-6xl lg:text-7xl">
                Lumina
              </h1>
              <p class="mt-5 max-w-2xl text-lg leading-8 text-paper/80 md:text-xl">
                发现值得被定格的瞬间，遇见与你同频的影像创作者。
              </p>
              <div class="mt-9 flex flex-col gap-3 sm:flex-row">
                <A
                  href="/register"
                  class="inline-flex items-center justify-center gap-2 rounded-lg bg-amber px-6 py-3.5 text-sm font-medium text-ink no-underline transition-transform hover:-translate-y-0.5"
                >
                  加入创作
                  <ArrowRight size={17} />
                </A>
                <A
                  href="/explore"
                  class="inline-flex items-center justify-center gap-2 rounded-lg border border-white/20 bg-black/20 px-6 py-3.5 text-sm text-paper no-underline backdrop-blur transition-colors hover:border-amber"
                >
                  <WandSparkles size={17} />
                  探索创作者
                </A>
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

        <section id="services" class="border-t border-white/8 px-5 py-20 md:px-8">
          <div class="mx-auto max-w-7xl">
            <div class="mb-10 flex items-end justify-between gap-6">
              <div>
                <p class="text-sm font-medium tracking-wide text-amber uppercase">Services</p>
                <h2 class="mt-3 font-display text-3xl font-semibold md:text-4xl">选择你的视觉方向</h2>
              </div>
              <A
                href="/services"
                class="hidden items-center gap-2 text-sm text-muted no-underline hover:text-paper md:inline-flex"
              >
                查看全部
                <ArrowRight size={15} />
              </A>
            </div>
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              {categories.map((item) => (
                <a
                  href="/services"
                  class="group rounded-lg border border-line bg-surface p-6 no-underline transition-colors hover:border-amber/60"
                >
                  <div class={`grid size-11 place-items-center rounded-lg ${item.tint}`}>
                    <item.icon size={21} />
                  </div>
                  <h3 class="mt-6 text-lg font-medium text-paper">{item.title}</h3>
                  <p class="mt-2 text-sm text-muted">发现创作者</p>
                </a>
              ))}
            </div>
          </div>
        </section>

        <section id="ai" class="px-5 py-20 md:px-8">
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
                用自然语言描述你的需求，AI 助手会匹配风格、推荐创作者、生成灵感方案并协助完成预约。
              </p>
              <div class="mt-8 grid gap-3 sm:grid-cols-2">
                {[
                  { icon: MessageCircle, text: "多轮创意对话" },
                  { icon: WandSparkles, text: "风格智能匹配" },
                  { icon: Brush, text: "灵感板生成" },
                  { icon: Scan, text: "图像质量分析" }
                ].map((item) => (
                  <div class="flex items-center gap-3 rounded-lg border border-line bg-surface px-4 py-3">
                    <item.icon size={18} class="text-amber" />
                    <span class="text-sm text-paper">{item.text}</span>
                  </div>
                ))}
              </div>
            </div>
            <div class="relative aspect-[4/3] overflow-hidden rounded-lg border border-white/10">
              <img
                src="/demo/ai-studio.jpg"
                alt="AI 创意工作室"
                class="size-full object-cover transition-transform duration-700 hover:scale-105"
              />
              <div class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 to-transparent p-6">
                <p class="font-display text-2xl font-semibold">风格 DNA</p>
                <p class="mt-1 text-sm text-white/70">色调 · 构图 · 情绪</p>
              </div>
            </div>
          </div>
        </section>

        <section id="creators" class="px-5 pb-24 pt-8 md:px-8">
          <div class="mx-auto max-w-7xl">
            <div class="mb-10 flex items-end justify-between gap-6">
              <div>
                <p class="text-sm font-medium tracking-wide text-coral uppercase">Featured</p>
                <h2 class="mt-3 font-display text-3xl font-semibold md:text-4xl">本周精选作品</h2>
              </div>
              <A
                href="/dashboard"
                class="hidden items-center gap-2 text-sm text-muted no-underline hover:text-paper md:inline-flex"
              >
                发布你的作品
                <ArrowRight size={15} />
              </A>
            </div>
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {displayWorks().map((work) => (
                <A
                  href={work.href}
                  class="group overflow-hidden rounded-lg border border-line bg-surface no-underline"
                >
                  <div class="aspect-[4/5] overflow-hidden">
                    <img
                      src={work.src}
                      alt={work.title}
                      loading="lazy"
                      class="size-full object-cover transition-transform duration-500 group-hover:scale-105"
                    />
                  </div>
                  <div class="flex items-start justify-between gap-4 p-4">
                    <div>
                      <h3 class="font-medium text-paper">{work.title}</h3>
                      <p class="mt-1 text-sm text-muted">@{work.creator}</p>
                    </div>
                    <span class="rounded-full border border-line px-2.5 py-1 text-xs text-muted">
                      {work.tag}
                    </span>
                  </div>
                </A>
              ))}
            </div>
          </div>
        </section>
      </main>

      <SiteFooter />
      <Assistant />
    </div>
  );
}

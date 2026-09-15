import { A, useLocation } from "@solidjs/router";
import { Camera, Menu, Sparkles, X } from "lucide-solid";
import { createSignal, Show } from "solid-js";
import { useAuthStore } from "../../stores/auth";

export default function SiteHeader() {
  const auth = useAuthStore();
  const location = useLocation();
  const [open, setOpen] = createSignal(false);

  const links = [
    { href: "/#creators", label: "创作者" },
    { href: "/#services", label: "服务" },
    { href: "/#ai", label: "AI 助手" }
  ];

  return (
    <header class="fixed inset-x-0 top-0 z-50 border-b border-white/8 bg-ink/80 backdrop-blur-xl">
      <div class="mx-auto flex h-16 max-w-7xl items-center justify-between px-5 md:px-8">
        <A href="/" class="flex items-center gap-2.5 text-paper no-underline" aria-label="Lumina 首页">
          <span class="grid size-9 place-items-center rounded-lg bg-amber text-ink">
            <Camera size={19} stroke-width={2.2} />
          </span>
          <span class="font-display text-xl font-semibold tracking-normal">Lumina</span>
        </A>

        <nav class="hidden items-center gap-7 md:flex">
          {links.map((link) => (
            <A
              href={link.href}
              class="text-sm text-muted transition-colors hover:text-paper no-underline"
            >
              {link.label}
            </A>
          ))}
        </nav>

        <div class="hidden items-center gap-3 md:flex">
          {auth.token() ? (
            <>
              <A
                href="/dashboard"
                class="grid size-9 place-items-center rounded-full border border-line text-paper no-underline transition-colors hover:border-amber"
                aria-label="工作台"
              >
                <Sparkles size={17} />
              </A>
              <button
                type="button"
                onClick={auth.signOut}
                class="rounded-lg border border-line px-4 py-2 text-sm text-muted transition-colors hover:border-coral hover:text-coral"
              >
                退出
              </button>
            </>
          ) : (
            <Show
              when={location.pathname === "/login" || location.pathname === "/register"}
              fallback={
                <>
                  <A
                    href="/login"
                    class="rounded-lg border border-line px-4 py-2 text-sm text-paper no-underline transition-colors hover:border-amber"
                  >
                    登录
                  </A>
                  <A
                    href="/register"
                    class="rounded-lg bg-amber px-4 py-2 text-sm font-medium text-ink no-underline transition-transform hover:-translate-y-0.5"
                  >
                    开始创作
                  </A>
                </>
              }
            >
              <A
                href={location.pathname === "/login" ? "/register" : "/login"}
                class="rounded-lg border border-line px-4 py-2 text-sm text-paper no-underline transition-colors hover:border-amber"
              >
                {location.pathname === "/login" ? "注册" : "登录"}
              </A>
            </Show>
          )}
        </div>

        <button
          type="button"
          class="grid size-10 place-items-center rounded-lg border border-line text-paper md:hidden"
          onClick={() => setOpen(!open())}
          aria-label="打开菜单"
        >
          {open() ? <X size={19} /> : <Menu size={19} />}
        </button>
      </div>

      {open() && (
        <div class="border-t border-line bg-ink/95 px-5 py-4 backdrop-blur-xl md:hidden">
          <nav class="flex flex-col gap-1">
            {links.map((link) => (
              <A
                href={link.href}
                class="rounded-lg px-3 py-3 text-sm text-muted no-underline hover:bg-surface hover:text-paper"
                onClick={() => setOpen(false)}
              >
                {link.label}
              </A>
            ))}
            <div class="mt-3 flex gap-3">
              {auth.token() ? (
                <A
                  href="/dashboard"
                  class="flex-1 rounded-lg bg-amber px-4 py-3 text-center text-sm font-medium text-ink no-underline"
                >
                  进入工作台
                </A>
              ) : (
                <>
                  <A
                    href="/login"
                    class="flex-1 rounded-lg border border-line px-4 py-3 text-center text-sm text-paper no-underline"
                  >
                    登录
                  </A>
                  <A
                    href="/register"
                    class="flex-1 rounded-lg bg-amber px-4 py-3 text-center text-sm font-medium text-ink no-underline"
                  >
                    注册
                  </A>
                </>
              )}
            </div>
          </nav>
        </div>
      )}
    </header>
  );
}

import { A } from "@solidjs/router";
import {
  Camera,
  ChevronDown,
  Compass,
  LayoutDashboard,
  LogOut,
  Menu,
  ShieldCheck,
  X
} from "lucide-solid";
import { createSignal, For, Show } from "solid-js";
import { useAuthStore } from "../../stores/auth";
import { NotificationBell } from "./notification-bell";

export type ShellMode = "public" | "customer" | "creator" | "admin";

interface AppShellProps {
  mode: ShellMode;
}

interface NavItem {
  href: string;
  label: string;
}

const modeLabels: Record<ShellMode, string> = {
  public: "Lumina",
  customer: "客户中心",
  creator: "创作者工作台",
  admin: "管理后台"
};

export default function AppShell(props: AppShellProps) {
  const auth = useAuthStore();
  const [mobileOpen, setMobileOpen] = createSignal(false);
  const [userOpen, setUserOpen] = createSignal(false);

  const navItems = (): NavItem[] => {
    if (props.mode === "admin") {
      return [
        { href: "/admin", label: "平台概览" },
        { href: "/admin#users", label: "用户管理" },
        { href: "/admin#withdrawals", label: "提现审核" }
      ];
    }
    if (props.mode === "creator") {
      return [
        { href: "/explore", label: "探索" },
        { href: "/services", label: "服务" },
        { href: "/dashboard", label: "工作台" }
      ];
    }
    if (props.mode === "customer") {
      return [
        { href: "/explore", label: "探索" },
        { href: "/services", label: "服务" },
        { href: "/account", label: "我的订单" }
      ];
    }
    return [
      { href: "/explore", label: "探索" },
      { href: "/services", label: "服务" },
      { href: "/#creators", label: "创作者" }
    ];
  };

  function go(path: string) {
    setMobileOpen(false);
    setUserOpen(false);
    window.location.href = path;
  }

  return (
    <header class="sticky top-0 z-50 border-b border-line bg-ink/90 text-paper backdrop-blur-xl">
      <div class="mx-auto flex h-16 max-w-7xl items-center justify-between gap-4 px-5 md:px-8">
        <div class="flex min-w-0 items-center gap-4">
          <A href="/" class="flex shrink-0 items-center gap-2.5 text-paper no-underline" aria-label="Lumina 首页">
            <span class="grid size-9 place-items-center rounded-lg bg-amber text-ink">
              <Camera size={19} stroke-width={2.2} />
            </span>
            <span class="hidden font-display text-xl font-semibold sm:inline">Lumina</span>
          </A>
          <span class="hidden h-5 w-px bg-line sm:block" />
          <span class="hidden text-sm text-muted md:inline">{modeLabels[props.mode]}</span>
        </div>

        <nav class="hidden items-center gap-7 md:flex">
          <For each={navItems()}>
            {(item) => (
              <A
                href={item.href}
                class="text-sm text-muted no-underline transition-colors hover:text-paper"
              >
                {item.label}
              </A>
            )}
          </For>
        </nav>

        <div class="flex shrink-0 items-center gap-2">
          <Show when={auth.token()}>
            <NotificationBell />
          </Show>

          <Show
            when={auth.token()}
            fallback={
              <div class="hidden items-center gap-2 md:flex">
                <A
                  href="/login"
                  class="rounded-lg border border-line px-4 py-2 text-sm text-paper no-underline transition-colors hover:border-amber"
                >
                  登录
                </A>
                <A
                  href="/register"
                  class="rounded-lg bg-amber px-4 py-2 text-sm font-medium text-ink no-underline"
                >
                  注册
                </A>
              </div>
            }
          >
            <div class="relative hidden md:block">
              <button
                type="button"
                class="inline-flex items-center gap-2 rounded-lg border border-line px-3 py-2 text-sm text-paper transition-colors hover:border-amber"
                onClick={() => setUserOpen(!userOpen())}
                aria-expanded={userOpen()}
              >
                <span class="max-w-28 truncate">{auth.user()?.nickname ?? "我的账户"}</span>
                <ChevronDown size={15} />
              </button>

              <Show when={userOpen()}>
                <div class="absolute right-0 top-12 w-56 overflow-hidden rounded-lg border border-line bg-ink py-1 shadow-2xl">
                  <A
                    href="/account"
                    class="flex items-center gap-2 px-4 py-2.5 text-sm text-muted no-underline hover:bg-surface hover:text-paper"
                    onClick={() => setUserOpen(false)}
                  >
                    <Compass size={16} />
                    客户中心
                  </A>
                  <A
                    href="/dashboard"
                    class="flex items-center gap-2 px-4 py-2.5 text-sm text-muted no-underline hover:bg-surface hover:text-paper"
                    onClick={() => setUserOpen(false)}
                  >
                    <LayoutDashboard size={16} />
                    创作者工作台
                  </A>
                  <Show when={auth.user()?.role === "admin"}>
                    <A
                      href="/admin"
                      class="flex items-center gap-2 px-4 py-2.5 text-sm text-muted no-underline hover:bg-surface hover:text-paper"
                      onClick={() => setUserOpen(false)}
                    >
                      <ShieldCheck size={16} />
                      管理后台
                    </A>
                  </Show>
                  <button
                    type="button"
                    onClick={() => {
                      auth.signOut();
                      go("/");
                    }}
                    class="flex w-full items-center gap-2 border-t border-line px-4 py-2.5 text-left text-sm text-muted hover:bg-surface hover:text-coral"
                  >
                    <LogOut size={16} />
                    退出登录
                  </button>
                </div>
              </Show>
            </div>
          </Show>

          <button
            type="button"
            class="grid size-10 place-items-center rounded-lg border border-line text-paper md:hidden"
            onClick={() => setMobileOpen(!mobileOpen())}
            aria-label="打开菜单"
            aria-expanded={mobileOpen()}
          >
            {mobileOpen() ? <X size={19} /> : <Menu size={19} />}
          </button>
        </div>
      </div>

      <Show when={mobileOpen()}>
        <div class="border-t border-line bg-ink px-5 py-4 md:hidden">
          <nav class="flex flex-col gap-1">
            <For each={navItems()}>
              {(item) => (
                <A
                  href={item.href}
                  class="rounded-lg px-3 py-3 text-sm text-muted no-underline hover:bg-surface hover:text-paper"
                  onClick={() => setMobileOpen(false)}
                >
                  {item.label}
                </A>
              )}
            </For>

            <Show
              when={auth.token()}
              fallback={
                <div class="mt-3 flex gap-3">
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
                </div>
              }
            >
              <div class="mt-3 grid gap-2">
                <A
                  href="/account"
                  class="flex items-center gap-2 rounded-lg px-3 py-3 text-sm text-muted no-underline hover:bg-surface hover:text-paper"
                  onClick={() => setMobileOpen(false)}
                >
                  <Compass size={16} />
                  客户中心
                </A>
                <A
                  href="/dashboard"
                  class="flex items-center gap-2 rounded-lg px-3 py-3 text-sm text-muted no-underline hover:bg-surface hover:text-paper"
                  onClick={() => setMobileOpen(false)}
                >
                  <LayoutDashboard size={16} />
                  创作者工作台
                </A>
                <Show when={auth.user()?.role === "admin"}>
                  <A
                    href="/admin"
                    class="flex items-center gap-2 rounded-lg px-3 py-3 text-sm text-muted no-underline hover:bg-surface hover:text-paper"
                    onClick={() => setMobileOpen(false)}
                  >
                    <ShieldCheck size={16} />
                    管理后台
                  </A>
                </Show>
                <button
                  type="button"
                  onClick={() => {
                    auth.signOut();
                    go("/");
                  }}
                  class="flex items-center gap-2 rounded-lg px-3 py-3 text-left text-sm text-muted hover:bg-surface hover:text-coral"
                >
                  <LogOut size={16} />
                  退出登录
                </button>
              </div>
            </Show>
          </nav>
        </div>
      </Show>
    </header>
  );
}

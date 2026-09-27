import { A, useLocation } from "@solidjs/router";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { Bell, Camera, CheckCheck, Menu, Sparkles, X } from "lucide-solid";
import { createSignal, For, Show } from "solid-js";
import {
  listNotifications,
  markAllRead,
  markRead,
  unreadCount
} from "../../lib/notification-api";
import { useAuthStore } from "../../stores/auth";

export function NotificationBell() {
  const auth = useAuthStore();
  const queryClient = useQueryClient();
  const [open, setOpen] = createSignal(false);

  const unread = createQuery(() => ({
    queryKey: ["notification-unread-count"] as const,
    queryFn: unreadCount,
    enabled: Boolean(auth.token()),
    refetchInterval: 30_000
  }));

  const notifications = createQuery(() => ({
    queryKey: ["notifications"] as const,
    queryFn: () => listNotifications(20),
    enabled: Boolean(auth.token()) && open()
  }));

  const count = () => unread.data?.count ?? 0;

  async function refreshNotifications() {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ["notifications"] }),
      queryClient.invalidateQueries({ queryKey: ["notification-unread-count"] })
    ]);
  }

  async function handleMarkAllRead() {
    await markAllRead();
    await refreshNotifications();
  }

  async function handleItemClick(id: number, actionUrl: string | null, event: MouseEvent) {
    if (!actionUrl) event.preventDefault();
    try {
      await markRead(id);
      await refreshNotifications();
    } finally {
      setOpen(false);
    }
  }

  return (
    <div class="relative">
      <button
        type="button"
        class="relative grid size-9 place-items-center rounded-full border border-line text-paper transition-colors hover:border-amber"
        onClick={() => setOpen(!open())}
        aria-label="消息通知"
        aria-expanded={open()}
      >
        <Bell size={17} />
        <Show when={count() > 0}>
          <span class="absolute -right-1.5 -top-1.5 grid min-w-5 place-items-center rounded-full bg-coral px-1 text-[10px] font-semibold leading-5 text-white">
            {count() > 99 ? "99+" : count()}
          </span>
        </Show>
      </button>

      <Show when={open()}>
        <div class="fixed left-4 right-4 top-16 z-50 overflow-hidden rounded-xl border border-line bg-ink shadow-2xl sm:absolute sm:left-auto sm:right-0 sm:top-12 sm:w-96">
          <div class="flex items-center justify-between border-b border-line px-4 py-3">
            <div>
              <p class="text-sm font-medium text-paper">消息通知</p>
              <p class="mt-0.5 text-xs text-muted">{count()} 条未读</p>
            </div>
            <button
              type="button"
              class="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-2 text-xs text-muted transition-colors hover:bg-surface hover:text-paper disabled:cursor-not-allowed disabled:opacity-40"
              onClick={handleMarkAllRead}
              disabled={count() === 0}
            >
              <CheckCheck size={15} />
              全部已读
            </button>
          </div>

          <div class="max-h-96 overflow-y-auto">
            <Show when={notifications.isLoading}>
              <div class="px-4 py-10 text-center text-sm text-muted">加载中...</div>
            </Show>
            <Show when={notifications.isError}>
              <div class="px-4 py-10 text-center text-sm text-coral">通知加载失败</div>
            </Show>
            <Show when={notifications.data?.length === 0}>
              <div class="px-4 py-10 text-center text-sm text-muted">暂无通知</div>
            </Show>
            <For each={notifications.data ?? []}>
              {(item) => (
                <A
                  href={item.action_url ?? "#"}
                  onClick={(event) => void handleItemClick(item.id, item.action_url, event)}
                  class={`block border-b border-line/70 px-4 py-3 no-underline transition-colors last:border-b-0 hover:bg-surface ${item.is_read ? "opacity-65" : ""}`}
                >
                  <div class="flex items-start gap-3">
                    <span class={`mt-1.5 size-2 shrink-0 rounded-full ${item.is_read ? "bg-line" : "bg-coral"}`} />
                    <div class="min-w-0">
                      <p class="truncate text-sm font-medium text-paper">{item.title}</p>
                      <Show when={item.content}>
                        <p class="mt-1 line-clamp-2 text-xs leading-5 text-muted">{item.content}</p>
                      </Show>
                      <p class="mt-1.5 text-[11px] text-muted/70">
                        {new Date(item.created_at).toLocaleString("zh-CN")}
                      </p>
                    </div>
                  </div>
                </A>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
}

export default function SiteHeader() {
  const auth = useAuthStore();
  const location = useLocation();
  const [open, setOpen] = createSignal(false);

  const links = [
    { href: "/explore", label: "探索" },
    { href: "/services", label: "服务" },
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

        <div class="flex items-center gap-3">
          <Show when={auth.token()}>
            <NotificationBell />
          </Show>

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

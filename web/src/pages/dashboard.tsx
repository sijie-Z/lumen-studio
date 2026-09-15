import { createQuery } from "@tanstack/solid-query";
import { useNavigate } from "@solidjs/router";
import {
  Camera,
  CalendarDays,
  CloudUpload,
  Image as ImageIcon,
  LogOut,
  Sparkles,
  Trash2,
  User
} from "lucide-solid";
import { createEffect, createSignal, For, Show } from "solid-js";
import { fetchMe, isAuthenticated, uploadImage, type UploadResult } from "../lib/auth-api";
import { useAuthStore } from "../stores/auth";
import Assistant from "../components/ai/assistant";

const GALLERY_KEY = "lumina.gallery";

function loadGallery(): UploadResult[] {
  try {
    return JSON.parse(localStorage.getItem(GALLERY_KEY) ?? "[]");
  } catch {
    return [];
  }
}

export default function Dashboard() {
  const navigate = useNavigate();
  const auth = useAuthStore();
  const [selected, setSelected] = createSignal<File | null>(null);
  const [preview, setPreview] = createSignal<string | null>(null);
  const [uploading, setUploading] = createSignal(false);
  const [error, setError] = createSignal("");
  const [gallery, setGallery] = createSignal<UploadResult[]>(loadGallery());

  const me = createQuery(() => ({
    queryKey: ["me"] as const,
    queryFn: fetchMe,
    enabled: isAuthenticated()
  }));

  createEffect(() => {
    if (!isAuthenticated()) navigate("/login");
    if (me.isError) navigate("/login");
  });

  createEffect(() => {
    if (me.data) auth.setCurrentUser(me.data);
  });

  function chooseFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    if (!["image/jpeg", "image/png", "image/webp"].includes(file.type)) {
      setError("仅支持 JPG、PNG 或 WebP 图片");
      return;
    }
    setSelected(file);
    setPreview(URL.createObjectURL(file));
  }

  async function handleUpload() {
    const file = selected();
    if (!file || uploading()) return;
    setUploading(true);
    setError("");
    try {
      const result = await uploadImage(file);
      const next = [result, ...gallery()];
      setGallery(next);
      localStorage.setItem(GALLERY_KEY, JSON.stringify(next));
      setSelected(null);
      setPreview(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "上传失败");
    } finally {
      setUploading(false);
    }
  }

  function removeImage(url: string) {
    const next = gallery().filter((item) => item.url !== url);
    setGallery(next);
    localStorage.setItem(GALLERY_KEY, JSON.stringify(next));
  }

  function signOut() {
    auth.signOut();
    navigate("/");
  }

  return (
    <div class="min-h-screen bg-ink text-paper">
      <header class="sticky top-0 z-40 border-b border-white/8 bg-ink/85 backdrop-blur-xl">
        <div class="mx-auto flex h-16 max-w-7xl items-center justify-between px-5 md:px-8">
          <button
            type="button"
            onClick={() => navigate("/")}
            class="flex items-center gap-2.5"
          >
            <span class="grid size-9 place-items-center rounded-lg bg-amber text-ink">
              <Camera size={18} />
            </span>
            <span class="font-display text-lg font-semibold">Lumina Studio</span>
          </button>
          <button
            type="button"
            onClick={signOut}
            class="inline-flex items-center gap-2 rounded-lg border border-line px-3 py-2 text-sm text-muted transition-colors hover:border-coral hover:text-coral"
          >
            <LogOut size={16} />
            <span class="hidden sm:inline">退出登录</span>
          </button>
        </div>
      </header>

      <main class="mx-auto max-w-7xl px-5 py-8 md:px-8 md:py-10">
        <div class="flex flex-col justify-between gap-5 md:flex-row md:items-end">
          <div class="flex items-center gap-4">
            <div class="grid size-16 place-items-center rounded-lg bg-gradient-to-br from-amber to-coral text-ink">
              <User size={30} stroke-width={1.8} />
            </div>
            <div>
              <p class="text-xs tracking-wide text-muted uppercase">Creator Workspace</p>
              <h1 class="mt-1 font-display text-3xl font-semibold">
                {me.data?.nickname ?? "我的工作台"}
              </h1>
              <p class="mt-1 text-sm text-muted">@{me.data?.username ?? "..."}</p>
            </div>
          </div>
          <div class="flex items-center gap-3">
            <div class="rounded-lg border border-line bg-surface px-4 py-3">
              <p class="text-xs text-muted">已上传作品</p>
              <p class="mt-1 text-xl font-medium text-amber">{gallery().length}</p>
            </div>
            <div class="rounded-lg border border-line bg-surface px-4 py-3">
              <p class="text-xs text-muted">本月预约</p>
              <p class="mt-1 text-xl font-medium text-teal">0</p>
            </div>
          </div>
        </div>

        <div class="mt-10 grid gap-6 lg:grid-cols-[360px_1fr]">
          <aside class="space-y-6">
            <section class="rounded-lg border border-line bg-surface p-5">
              <h2 class="text-base font-medium">账户信息</h2>
              <div class="mt-5 space-y-4 text-sm">
                <div class="flex items-center justify-between gap-4">
                  <span class="text-muted">邮箱</span>
                  <span class="truncate text-paper">{me.data?.email ?? "未设置"}</span>
                </div>
                <div class="flex items-center justify-between gap-4">
                  <span class="text-muted">手机</span>
                  <span class="truncate text-paper">{me.data?.phone ?? "未设置"}</span>
                </div>
                <div class="flex items-center justify-between gap-4">
                  <span class="text-muted">角色</span>
                  <span class="rounded-full bg-teal/15 px-2.5 py-1 text-xs text-teal">
                    {me.data?.role === "user" ? "创作者" : me.data?.role}
                  </span>
                </div>
              </div>
            </section>

            <section class="rounded-lg border border-line bg-surface p-5">
              <h2 class="text-base font-medium">快捷入口</h2>
              <div class="mt-4 grid grid-cols-2 gap-3">
                <button class="flex flex-col items-start gap-2 rounded-lg border border-line bg-ink/40 p-3.5 text-left transition-colors hover:border-amber/60">
                  <CalendarDays size={19} class="text-amber" />
                  <span class="text-sm text-paper">预约管理</span>
                </button>
                <button class="flex flex-col items-start gap-2 rounded-lg border border-line bg-ink/40 p-3.5 text-left transition-colors hover:border-amber/60">
                  <Sparkles size={19} class="text-teal" />
                  <span class="text-sm text-paper">AI 助手</span>
                </button>
              </div>
            </section>
          </aside>

          <section class="rounded-lg border border-line bg-surface p-5 md:p-6">
            <div class="flex items-center justify-between gap-4">
              <div>
                <h2 class="text-base font-medium">作品上传</h2>
                <p class="mt-1 text-sm text-muted">JPG、PNG 或 WebP，最大 15MB</p>
              </div>
              <div class="grid size-11 place-items-center rounded-lg bg-amber/15 text-amber">
                <ImageIcon size={20} />
              </div>
            </div>

            <label
              class="mt-6 flex min-h-52 cursor-pointer flex-col items-center justify-center rounded-lg border border-dashed border-line bg-ink/35 px-6 py-10 text-center transition-colors hover:border-amber/70"
            >
              <input type="file" accept="image/jpeg,image/png,image/webp" class="sr-only" onChange={chooseFile} />
              <Show
                when={preview()}
                fallback={
                  <>
                    <span class="grid size-14 place-items-center rounded-full border border-line bg-ink text-muted">
                      <CloudUpload size={24} />
                    </span>
                    <span class="mt-4 text-sm text-paper">点击选择图片</span>
                    <span class="mt-1 text-xs text-muted">图片将保存到你的作品库</span>
                  </>
                }
              >
                <img src={preview()!} alt="上传预览" class="max-h-64 rounded-lg object-contain" />
              </Show>
            </label>

            {error() && (
              <div class="mt-4 rounded-lg border border-coral/30 bg-coral/10 px-4 py-3 text-sm text-coral">
                {error()}
              </div>
            )}

            <div class="mt-4 flex justify-end gap-3">
              <Show when={selected()}>
                <button
                  type="button"
                  onClick={() => {
                    setSelected(null);
                    setPreview(null);
                  }}
                  class="rounded-lg border border-line px-4 py-2.5 text-sm text-muted hover:text-paper"
                >
                  取消
                </button>
                <button
                  type="button"
                  onClick={handleUpload}
                  disabled={uploading()}
                  class="rounded-lg bg-amber px-5 py-2.5 text-sm font-medium text-ink transition-colors hover:bg-[#ffb46f] disabled:cursor-wait disabled:opacity-70"
                >
                  {uploading() ? "上传中" : "上传图片"}
                </button>
              </Show>
            </div>
          </section>
        </div>

        <section class="mt-10">
          <div class="mb-6 flex items-end justify-between">
            <div>
              <h2 class="font-display text-2xl font-semibold">我的作品库</h2>
              <p class="mt-1 text-sm text-muted">上传的图片会出现在这里</p>
            </div>
            <span class="text-sm text-muted">{gallery().length} 张</span>
          </div>

          <Show
            when={gallery().length > 0}
            fallback={
              <div class="grid min-h-48 place-items-center rounded-lg border border-dashed border-line bg-surface/40 text-sm text-muted">
                还没有上传任何作品
              </div>
            }
          >
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <For each={gallery()}>
                {(item) => (
                  <div class="group overflow-hidden rounded-lg border border-line bg-surface">
                    <div class="aspect-square overflow-hidden">
                      <img
                        src={item.url}
                        alt={item.filename}
                        loading="lazy"
                        class="size-full object-cover transition-transform duration-500 group-hover:scale-105"
                      />
                    </div>
                    <div class="flex items-center justify-between gap-3 p-3">
                      <div class="min-w-0">
                        <p class="truncate text-sm text-paper">{item.filename}</p>
                        <p class="mt-0.5 text-xs text-muted">{(item.size / 1024 / 1024).toFixed(2)} MB</p>
                      </div>
                      <button
                        type="button"
                        onClick={() => removeImage(item.url)}
                        class="grid size-8 shrink-0 place-items-center rounded-lg border border-line text-muted transition-colors hover:border-coral hover:text-coral"
                        aria-label="删除图片"
                      >
                        <Trash2 size={15} />
                      </button>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </section>
      </main>
      <Assistant />
    </div>
  );
}

import { A, useNavigate } from "@solidjs/router";
import { AtSign, Camera, LoaderCircle, Lock, Phone, User } from "lucide-solid";
import { createSignal } from "solid-js";
import { register } from "../lib/auth-api";

export default function Register() {
  const navigate = useNavigate();
  const [form, setForm] = createSignal({
    username: "",
    password: "",
    nickname: "",
    email: "",
    phone: ""
  });
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");

  function update(field: keyof ReturnType<typeof form>, value: string) {
    setForm({ ...form(), [field]: value });
  }

  async function handleSubmit(event: Event) {
    event.preventDefault();
    setError("");
    setLoading(true);
    try {
      const input = form();
      const payload = {
        username: input.username,
        password: input.password,
        nickname: input.nickname || undefined,
        email: input.email || undefined,
        phone: input.phone || undefined
      };
      await register(payload);
      navigate("/login");
    } catch (err) {
      setError(err instanceof Error ? err.message : "注册失败");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div class="relative min-h-screen overflow-hidden bg-ink text-paper">
      <img
        src="/demo/login-bg.jpg"
        alt=""
        class="absolute inset-0 size-full object-cover opacity-25"
      />
      <div class="absolute inset-0 bg-gradient-to-b from-ink/80 via-ink/70 to-ink" />

      <div class="relative z-10 mx-auto flex min-h-screen max-w-md flex-col justify-center px-5 py-12">
        <A href="/" class="mb-10 flex items-center gap-2.5 no-underline">
          <span class="grid size-10 place-items-center rounded-lg bg-amber text-ink">
            <Camera size={20} />
          </span>
          <span class="font-display text-2xl font-semibold">Lumina</span>
        </A>

        <div class="rounded-lg border border-white/10 bg-surface/90 p-7 shadow-2xl backdrop-blur-xl md:p-8">
          <h1 class="font-display text-3xl font-semibold">创建账号</h1>
          <p class="mt-2 text-sm text-muted">发布作品，开始你的创作者旅程</p>

          <form class="mt-8 space-y-4" onSubmit={handleSubmit}>
            <div class="grid gap-4 sm:grid-cols-2">
              <label class="block">
                <span class="mb-2 block text-sm text-paper/85">用户名</span>
                <div class="flex items-center gap-3 rounded-lg border border-line bg-ink/50 px-3.5 focus-within:border-amber">
                  <User size={16} class="text-muted" />
                  <input
                    value={form().username}
                    onInput={(e) => update("username", e.currentTarget.value)}
                    class="h-11 w-full min-w-0 bg-transparent text-paper outline-none placeholder:text-muted/60"
                    placeholder="lumina_user"
                    required
                  />
                </div>
              </label>
              <label class="block">
                <span class="mb-2 block text-sm text-paper/85">昵称</span>
                <div class="flex items-center gap-3 rounded-lg border border-line bg-ink/50 px-3.5 focus-within:border-amber">
                  <AtSign size={16} class="text-muted" />
                  <input
                    value={form().nickname}
                    onInput={(e) => update("nickname", e.currentTarget.value)}
                    class="h-11 w-full min-w-0 bg-transparent text-paper outline-none placeholder:text-muted/60"
                    placeholder="你的名字"
                  />
                </div>
              </label>
            </div>

            <label class="block">
              <span class="mb-2 block text-sm text-paper/85">密码</span>
              <div class="flex items-center gap-3 rounded-lg border border-line bg-ink/50 px-3.5 focus-within:border-amber">
                <Lock size={16} class="text-muted" />
                <input
                  type="password"
                  value={form().password}
                  onInput={(e) => update("password", e.currentTarget.value)}
                  class="h-11 w-full bg-transparent text-paper outline-none placeholder:text-muted/60"
                  placeholder="至少 8 位"
                  required
                />
              </div>
            </label>

            <div class="grid gap-4 sm:grid-cols-2">
              <label class="block">
                <span class="mb-2 block text-sm text-paper/85">邮箱</span>
                <div class="flex items-center gap-3 rounded-lg border border-line bg-ink/50 px-3.5 focus-within:border-amber">
                  <AtSign size={16} class="text-muted" />
                  <input
                    type="email"
                    value={form().email}
                    onInput={(e) => update("email", e.currentTarget.value)}
                    class="h-11 w-full min-w-0 bg-transparent text-paper outline-none placeholder:text-muted/60"
                    placeholder="you@example.com"
                  />
                </div>
              </label>
              <label class="block">
                <span class="mb-2 block text-sm text-paper/85">手机号</span>
                <div class="flex items-center gap-3 rounded-lg border border-line bg-ink/50 px-3.5 focus-within:border-amber">
                  <Phone size={16} class="text-muted" />
                  <input
                    value={form().phone}
                    onInput={(e) => update("phone", e.currentTarget.value)}
                    class="h-11 w-full min-w-0 bg-transparent text-paper outline-none placeholder:text-muted/60"
                    placeholder="13800000000"
                  />
                </div>
              </label>
            </div>

            {error() && (
              <div class="rounded-lg border border-coral/30 bg-coral/10 px-4 py-3 text-sm text-coral">
                {error()}
              </div>
            )}

            <button
              type="submit"
              disabled={loading()}
              class="flex h-12 w-full items-center justify-center gap-2 rounded-lg bg-amber font-medium text-ink transition-colors hover:bg-[#ffb46f] disabled:cursor-wait disabled:opacity-70"
            >
              {loading() ? <LoaderCircle size={18} class="animate-spin" /> : null}
              {loading() ? "创建中" : "创建账号"}
            </button>
          </form>

          <p class="mt-7 text-center text-sm text-muted">
            已有账号？
            <A href="/login" class="ml-1 text-amber no-underline hover:text-[#ffb46f]">
              直接登录
            </A>
          </p>
        </div>
      </div>
    </div>
  );
}

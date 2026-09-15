import { A, useNavigate } from "@solidjs/router";
import { Camera, LoaderCircle, Lock, User } from "lucide-solid";
import { createSignal } from "solid-js";
import { login } from "../lib/auth-api";
import { useAuthStore } from "../stores/auth";

export default function Login() {
  const navigate = useNavigate();
  const auth = useAuthStore();
  const [account, setAccount] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");

  async function handleSubmit(event: Event) {
    event.preventDefault();
    setError("");
    setLoading(true);
    try {
      const result = await login(account(), password());
      auth.signIn(result.access_token, result.user);
      navigate("/dashboard");
    } catch (err) {
      setError(err instanceof Error ? err.message : "登录失败");
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
          <h1 class="font-display text-3xl font-semibold">欢迎回来</h1>
          <p class="mt-2 text-sm text-muted">登录后继续你的创作旅程</p>

          <form class="mt-8 space-y-5" onSubmit={handleSubmit}>
            <label class="block">
              <span class="mb-2 block text-sm text-paper/85">用户名或邮箱</span>
              <div class="flex items-center gap-3 rounded-lg border border-line bg-ink/50 px-3.5 focus-within:border-amber">
                <User size={17} class="text-muted" />
                <input
                  type="text"
                  value={account()}
                  onInput={(e) => setAccount(e.currentTarget.value)}
                  autocomplete="username"
                  class="h-12 w-full bg-transparent text-paper outline-none placeholder:text-muted/60"
                  placeholder="name@example.com"
                  required
                />
              </div>
            </label>

            <label class="block">
              <span class="mb-2 block text-sm text-paper/85">密码</span>
              <div class="flex items-center gap-3 rounded-lg border border-line bg-ink/50 px-3.5 focus-within:border-amber">
                <Lock size={17} class="text-muted" />
                <input
                  type="password"
                  value={password()}
                  onInput={(e) => setPassword(e.currentTarget.value)}
                  autocomplete="current-password"
                  class="h-12 w-full bg-transparent text-paper outline-none placeholder:text-muted/60"
                  placeholder="至少 8 位"
                  required
                />
              </div>
            </label>

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
              {loading() ? "登录中" : "登录"}
            </button>
          </form>

          <p class="mt-7 text-center text-sm text-muted">
            还没有账号？
            <A href="/register" class="ml-1 text-amber no-underline hover:text-[#ffb46f]">
              立即注册
            </A>
          </p>
        </div>

        <p class="mt-6 text-center text-xs text-muted/70">
          登录即表示你同意平台服务条款与隐私政策
        </p>
      </div>
    </div>
  );
}

import { Camera } from "lucide-solid";

export default function SiteFooter() {
  return (
    <footer class="border-t border-white/8 bg-ink px-5 py-10 md:px-8">
      <div class="mx-auto flex max-w-7xl flex-col gap-8 md:flex-row md:items-start md:justify-between">
        <div class="max-w-sm">
          <div class="flex items-center gap-2.5">
            <span class="grid size-8 place-items-center rounded-lg bg-amber text-ink">
              <Camera size={16} />
            </span>
            <span class="font-display text-lg font-semibold">Lumen Studio</span>
          </div>
          <p class="mt-4 text-sm leading-6 text-muted">
            摄影、影像与视觉创作者的一站式预约服务平台。
          </p>
        </div>
        <div class="grid grid-cols-2 gap-8 text-sm sm:grid-cols-3">
          <div>
            <p class="font-medium text-paper">平台</p>
            <div class="mt-3 flex flex-col gap-2 text-muted">
              <a href="/#creators" class="hover:text-paper no-underline">发现创作者</a>
              <a href="/#services" class="hover:text-paper no-underline">服务类型</a>
            </div>
          </div>
          <div>
            <p class="font-medium text-paper">创作</p>
            <div class="mt-3 flex flex-col gap-2 text-muted">
              <a href="/dashboard" class="hover:text-paper no-underline">作品管理</a>
              <a href="/dashboard" class="hover:text-paper no-underline">预约管理</a>
            </div>
          </div>
          <div>
            <p class="font-medium text-paper">联系</p>
            <div class="mt-3 flex flex-col gap-2 text-muted">
              <span>hello@lumen.studio</span>
              <span>上海</span>
            </div>
          </div>
        </div>
      </div>
      <div class="mx-auto mt-10 max-w-7xl border-t border-white/8 pt-5 text-xs text-muted">
        © 2026 Lumen Studio
      </div>
    </footer>
  );
}

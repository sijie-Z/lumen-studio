import { Bot, Send, Sparkles, X } from "lucide-solid";
import { createSignal, For, Show } from "solid-js";
import { sendChat, type ChatMessage } from "../../lib/ai-api";

const suggestions = ["推荐摄影师", "查询价格", "预约流程", "找暖色调风格"];

export default function Assistant() {
  const [open, setOpen] = createSignal(false);
  const [input, setInput] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [messages, setMessages] = createSignal<ChatMessage[]>([
    {
      role: "assistant",
      content: "你好，我是 Lumina 创意助手。想拍什么，我帮你找。"
    }
  ]);

  async function submit(text?: string) {
    const content = (text ?? input()).trim();
    if (!content || loading()) return;
    setInput("");
    const history = [...messages(), { role: "user" as const, content }];
    setMessages(history);
    setLoading(true);
    try {
      const response = await sendChat(history);
      setMessages([
        ...history,
        { role: "assistant", content: response.reply }
      ]);
    } catch {
      setMessages([
        ...history,
        { role: "assistant", content: "我这边暂时有点卡，稍后再试一次。" }
      ]);
    } finally {
      setLoading(false);
    }
  }

  return (
    <>
      <Show when={!open()}>
        <button
          type="button"
          onClick={() => setOpen(true)}
          class="fixed right-4 bottom-4 z-[60] grid size-14 place-items-center rounded-full bg-amber text-ink shadow-2xl shadow-black/30 transition-transform hover:-translate-y-1 md:right-6 md:bottom-6"
          aria-label="打开 AI 助手"
        >
          <Sparkles size={22} />
        </button>
      </Show>

      <Show when={open()}>
        <div class="fixed right-0 bottom-0 z-[70] h-[min(680px,88vh)] w-full max-w-[420px] overflow-hidden rounded-t-lg border border-line bg-surface shadow-2xl md:right-6 md:bottom-6 md:rounded-lg">
          <div class="flex h-16 items-center justify-between border-b border-line bg-ink px-5">
            <div class="flex items-center gap-3">
              <span class="grid size-9 place-items-center rounded-lg bg-teal/15 text-teal">
                <Bot size={19} />
              </span>
              <div>
                <p class="text-sm font-medium text-paper">Lumina AI</p>
                <p class="text-xs text-teal">在线</p>
              </div>
            </div>
            <button
              type="button"
              onClick={() => setOpen(false)}
              class="grid size-9 place-items-center rounded-lg text-muted transition-colors hover:bg-surface hover:text-paper"
              aria-label="关闭 AI 助手"
            >
              <X size={18} />
            </button>
          </div>

          <div class="flex h-[calc(100%-8rem)] flex-col gap-4 overflow-y-auto px-4 py-5">
            <For each={messages()}>
              {(message) => (
                <div
                  class={message.role === "user" ? "flex justify-end" : "flex justify-start"}
                >
                  <div
                    class={
                      message.role === "user"
                        ? "max-w-[82%] rounded-lg rounded-tr-sm bg-amber px-4 py-3 text-sm leading-6 text-ink"
                        : "max-w-[88%] rounded-lg rounded-tl-sm border border-line bg-ink/60 px-4 py-3 text-sm leading-6 text-paper"
                    }
                  >
                    {message.content}
                  </div>
                </div>
              )}
            </For>
            <Show when={loading()}>
              <div class="flex items-center gap-1.5 px-1 text-teal">
                <span class="size-1.5 animate-bounce rounded-full bg-teal" />
                <span class="size-1.5 animate-bounce rounded-full bg-teal [animation-delay:120ms]" />
                <span class="size-1.5 animate-bounce rounded-full bg-teal [animation-delay:240ms]" />
              </div>
            </Show>
          </div>

          <div class="absolute inset-x-0 bottom-0 border-t border-line bg-ink px-4 py-4">
            <div class="mb-3 flex gap-2 overflow-x-auto pb-1">
              <For each={suggestions}>
                {(suggestion) => (
                  <button
                    type="button"
                    onClick={() => submit(suggestion)}
                    class="shrink-0 rounded-full border border-line px-3 py-1.5 text-xs text-muted transition-colors hover:border-amber hover:text-amber"
                  >
                    {suggestion}
                  </button>
                )}
              </For>
            </div>
            <div class="flex items-center gap-2">
              <input
                value={input()}
                onInput={(event) => setInput(event.currentTarget.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") submit();
                }}
                class="h-11 flex-1 rounded-lg border border-line bg-surface px-3.5 text-sm text-paper outline-none placeholder:text-muted/60 focus:border-amber"
                placeholder="描述你的创意需求"
              />
              <button
                type="button"
                onClick={() => submit()}
                disabled={!input().trim() || loading()}
                class="grid size-11 shrink-0 place-items-center rounded-lg bg-amber text-ink transition-colors hover:bg-[#ffb46f] disabled:cursor-not-allowed disabled:opacity-40"
                aria-label="发送消息"
              >
                <Send size={18} />
              </button>
            </div>
          </div>
        </div>
      </Show>
    </>
  );
}

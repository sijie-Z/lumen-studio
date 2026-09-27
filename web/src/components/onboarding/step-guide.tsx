import { Check, ChevronDown, ChevronUp, ListChecks, X } from "lucide-solid";
import { createSignal, For, Show, splitProps, type Component } from "solid-js";
import { Card, CardContent } from "../ui/card";
import { cn } from "../../lib/cn";

export interface GuideStep {
  title: string;
  description: string;
  done?: boolean;
}

interface StepGuideProps {
  id: string;
  steps: GuideStep[];
  title?: string;
  description?: string;
  class?: string;
}

function readDismissed(key: string) {
  if (typeof window === "undefined") return false;
  try {
    return window.localStorage.getItem(key) === "dismissed";
  } catch {
    return false;
  }
}

export const StepGuide: Component<StepGuideProps> = (props) => {
  const [local, rest] = splitProps(props, ["id", "steps", "title", "description", "class"]);
  const storageKey = `lumina:onboarding:${local.id}`;
  const [dismissed, setDismissed] = createSignal(readDismissed(storageKey));
  const [collapsed, setCollapsed] = createSignal(false);

  const activeIndex = () => local.steps.findIndex((step) => !step.done);
  const completedCount = () => local.steps.filter((step) => step.done).length;
  const allDone = () => local.steps.length > 0 && activeIndex() === -1;

  function dismiss() {
    setDismissed(true);
    try {
      window.localStorage.setItem(storageKey, "dismissed");
    } catch {
      // Storage can be unavailable in private browsing; the panel still closes for this session.
    }
  }

  return (
    <Show when={!dismissed()}>
      <Card {...rest} class={cn("border-line bg-secondary/80", local.class)}>
        <CardContent class="p-0">
          <div class="flex items-start justify-between gap-4 px-4 py-3.5 sm:px-5">
            <div class="flex min-w-0 items-start gap-3">
              <span class="mt-0.5 grid size-8 shrink-0 place-items-center rounded-lg bg-primary/15 text-primary">
                <ListChecks size={17} />
              </span>
              <div class="min-w-0">
                <p class="text-sm font-medium text-foreground">
                  {local.title ?? "下一步怎么做"}
                </p>
                <Show when={local.description}>
                  <p class="mt-1 text-xs leading-5 text-muted">{local.description}</p>
                </Show>
                <p class="mt-1 text-xs text-muted">
                  进度 {completedCount()}/{local.steps.length}
                </p>
              </div>
            </div>
            <div class="flex shrink-0 items-center gap-1">
              <button
                type="button"
                onClick={() => setCollapsed((value) => !value)}
                class="grid size-8 place-items-center rounded-lg text-muted transition-colors hover:bg-surface hover:text-foreground"
                aria-label={collapsed() ? "展开引导" : "折叠引导"}
                title={collapsed() ? "展开引导" : "折叠引导"}
              >
                {collapsed() ? <ChevronDown size={16} /> : <ChevronUp size={16} />}
              </button>
              <button
                type="button"
                onClick={dismiss}
                class="grid size-8 place-items-center rounded-lg text-muted transition-colors hover:bg-surface hover:text-foreground"
                aria-label="关闭引导"
                title="关闭引导"
              >
                <X size={16} />
              </button>
            </div>
          </div>

          <Show when={!collapsed()}>
            <div class="border-t border-line px-4 py-3 sm:px-5">
              <Show
                when={!allDone()}
                fallback={
                  <div class="flex items-center gap-2 py-1 text-sm text-accent">
                    <span class="grid size-5 place-items-center rounded-full bg-accent/15">
                      <Check size={13} />
                    </span>
                    全部步骤已完成。
                  </div>
                }
              >
                <ol class="space-y-3">
                  <For each={local.steps}>
                    {(step, index) => {
                      const isDone = () => Boolean(step.done);
                      const isActive = () => !isDone() && index() === activeIndex();

                      return (
                        <li class="flex gap-3">
                          <span
                            class={cn(
                              "mt-0.5 grid size-6 shrink-0 place-items-center rounded-full border text-xs",
                              isDone()
                                ? "border-accent/30 bg-accent/15 text-accent"
                                : isActive()
                                  ? "border-primary bg-primary text-primary-foreground"
                                  : "border-line bg-background text-muted"
                            )}
                            aria-current={isActive() ? "step" : undefined}
                          >
                            <Show when={isDone()} fallback={index() + 1}>
                              <Check size={13} />
                            </Show>
                          </span>
                          <div class="min-w-0">
                            <p
                              class={cn(
                                "text-sm",
                                isDone()
                                  ? "text-muted line-through decoration-line"
                                  : isActive()
                                    ? "font-medium text-foreground"
                                    : "text-muted"
                              )}
                            >
                              {step.title}
                            </p>
                            <p class="mt-0.5 text-xs leading-5 text-muted">{step.description}</p>
                          </div>
                        </li>
                      );
                    }}
                  </For>
                </ol>
              </Show>
            </div>
          </Show>
        </CardContent>
      </Card>
    </Show>
  );
};

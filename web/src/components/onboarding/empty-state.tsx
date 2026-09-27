import { A } from "@solidjs/router";
import { Show, splitProps, type Component, type JSX } from "solid-js";
import { Button } from "../ui/button";
import { cn } from "../../lib/cn";

interface EmptyStateProps {
  icon?: JSX.Element;
  title: string;
  description?: string;
  ctaLabel?: string;
  href?: string;
  onClick?: () => void;
  class?: string;
}

export const EmptyState: Component<EmptyStateProps> = (props) => {
  const [local, rest] = splitProps(props, [
    "icon",
    "title",
    "description",
    "ctaLabel",
    "href",
    "onClick",
    "class"
  ]);

  return (
    <div
      {...rest}
      class={cn(
        "flex min-h-44 flex-col items-center justify-center rounded-lg border border-dashed border-line bg-secondary/45 px-6 py-9 text-center",
        local.class
      )}
    >
      <Show when={local.icon}>
        <span class="grid size-11 place-items-center rounded-lg bg-primary/12 text-primary">
          {local.icon}
        </span>
      </Show>
      <h3 class="mt-4 text-sm font-medium text-foreground">{local.title}</h3>
      <Show when={local.description}>
        <p class="mt-2 max-w-md text-sm leading-6 text-muted">{local.description}</p>
      </Show>
      <Show when={local.ctaLabel && (local.href || local.onClick)}>
        <Show
          when={local.href}
          fallback={
            <Button class="mt-5" size="sm" onClick={local.onClick}>
              {local.ctaLabel}
            </Button>
          }
        >
          <A
            href={local.href!}
            class="mt-5 inline-flex h-8 items-center justify-center gap-2 rounded-lg bg-primary px-3 text-xs font-medium text-primary-foreground no-underline transition-colors hover:bg-primary/85"
          >
            {local.ctaLabel}
          </A>
        </Show>
      </Show>
    </div>
  );
};

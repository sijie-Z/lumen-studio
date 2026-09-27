import { AlertCircle, LoaderCircle, RefreshCw } from "lucide-solid";
import type { Component, JSX } from "solid-js";
import { cn } from "../../lib/cn";
import { Button } from "./button";

export { EmptyState } from "../onboarding/empty-state";

interface LoadingStateProps {
  title?: string;
  description?: string;
  class?: string;
}

export const LoadingState: Component<LoadingStateProps> = (props) => (
  <div class={cn("grid min-h-48 place-items-center rounded-lg border border-dashed border-line bg-secondary/30 px-6 py-10 text-center", props.class)}>
    <div>
      <LoaderCircle size={24} class="mx-auto animate-spin text-primary" />
      <p class="mt-4 text-sm font-medium text-foreground">{props.title ?? "正在加载..."}</p>
      <p class="mt-1 text-sm text-muted">{props.description ?? "请稍候，数据马上就好。"}</p>
    </div>
  </div>
);

interface ErrorStateProps {
  title?: string;
  description?: string;
  retryLabel?: string;
  onRetry?: () => void;
  class?: string;
  children?: JSX.Element;
}

export const ErrorState: Component<ErrorStateProps> = (props) => (
  <div class={cn("grid min-h-48 place-items-center rounded-lg border border-destructive/30 bg-destructive/8 px-6 py-10 text-center", props.class)}>
    <div>
      <AlertCircle size={24} class="mx-auto text-destructive" />
      <p class="mt-4 text-sm font-medium text-foreground">{props.title ?? "加载失败"}</p>
      <p class="mt-1 text-sm text-muted">{props.description ?? "暂时无法获取数据，请稍后重试。"}</p>
      {props.children}
      {props.onRetry && (
        <Button class="mt-5" size="sm" variant="outline" onClick={props.onRetry}>
          <RefreshCw size={15} />
          {props.retryLabel ?? "重新加载"}
        </Button>
      )}
    </div>
  </div>
);

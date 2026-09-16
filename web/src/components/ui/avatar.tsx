import { splitProps, type Component, type JSX } from "solid-js";
import { cn } from "../../lib/cn";

export const Avatar: Component<JSX.HTMLAttributes<HTMLSpanElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <span
      {...rest}
      class={cn(
        "relative flex size-9 shrink-0 overflow-hidden rounded-full",
        local.class
      )}
    />
  );
};

export const AvatarImage: Component<JSX.ImgHTMLAttributes<HTMLImageElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return <img {...rest} class={cn("aspect-square size-full object-cover", local.class)} />;
};

export const AvatarFallback: Component<JSX.HTMLAttributes<HTMLSpanElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <span
      {...rest}
      class={cn(
        "flex size-full items-center justify-center rounded-full bg-secondary text-xs font-medium text-muted",
        local.class
      )}
    />
  );
};

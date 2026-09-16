import { splitProps, type Component, type JSX } from "solid-js";
import { cn } from "../../lib/cn";

export const Skeleton: Component<JSX.HTMLAttributes<HTMLDivElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return <div {...rest} class={cn("animate-pulse rounded-md bg-line/60", local.class)} />;
};

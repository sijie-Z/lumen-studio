import { splitProps, type Component, type JSX } from "solid-js";
import { cn } from "../../lib/cn";

export const Separator: Component<JSX.HTMLAttributes<HTMLDivElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <div
      {...rest}
      class={cn("h-px w-full shrink-0 bg-line", local.class)}
      role="separator"
    />
  );
};

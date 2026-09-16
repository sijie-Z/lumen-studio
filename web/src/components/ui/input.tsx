import { splitProps, type Component, type JSX } from "solid-js";
import { cn } from "../../lib/cn";

export const Input: Component<JSX.InputHTMLAttributes<HTMLInputElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <input
      {...rest}
      class={cn(
        "h-10 w-full rounded-lg border border-line bg-background px-3 text-sm text-foreground outline-none transition-colors placeholder:text-muted/70 focus:border-ring",
        local.class
      )}
    />
  );
};

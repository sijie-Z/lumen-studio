import { splitProps, type Component, type JSX } from "solid-js";
import { cn } from "../../lib/cn";

type BadgeVariant = "default" | "secondary" | "outline" | "accent" | "destructive";

const variants: Record<BadgeVariant, string> = {
  default: "border-transparent bg-primary text-primary-foreground",
  secondary: "border-transparent bg-secondary text-secondary-foreground",
  outline: "border-line text-foreground",
  accent: "border-transparent bg-accent/15 text-accent",
  destructive: "border-transparent bg-destructive/15 text-destructive"
};

export const Badge: Component<
  JSX.HTMLAttributes<HTMLSpanElement> & { variant?: BadgeVariant }
> = (props) => {
  const [local, rest] = splitProps(props, ["class", "variant"]);
  return (
    <span
      {...rest}
      class={cn(
        "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-medium",
        variants[local.variant ?? "default"],
        local.class
      )}
    />
  );
};

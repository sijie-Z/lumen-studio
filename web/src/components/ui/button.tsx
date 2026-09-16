import { splitProps, type Component, type JSX } from "solid-js";
import { cn } from "../../lib/cn";

type ButtonVariant =
  | "default"
  | "outline"
  | "secondary"
  | "ghost"
  | "destructive"
  | "link";

type ButtonSize = "default" | "sm" | "lg" | "icon";

const variants: Record<ButtonVariant, string> = {
  default: "bg-primary text-primary-foreground hover:bg-primary/85",
  outline: "border border-line bg-transparent hover:bg-surface hover:text-foreground",
  secondary: "bg-secondary text-secondary-foreground hover:bg-surface",
  ghost: "hover:bg-surface hover:text-foreground",
  destructive: "bg-destructive/15 text-destructive hover:bg-destructive/25",
  link: "text-primary underline-offset-4 hover:underline"
};

const sizes: Record<ButtonSize, string> = {
  default: "h-10 px-4 py-2 text-sm",
  sm: "h-8 px-3 text-xs",
  lg: "h-11 px-6 text-base",
  icon: "size-9"
};

export interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
}

export const Button: Component<ButtonProps> = (props) => {
  const [local, rest] = splitProps(props, ["class", "variant", "size"]);
  return (
    <button
      {...rest}
      class={cn(
        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg font-medium transition-colors outline-none select-none disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-4 [&_svg]:shrink-0",
        variants[local.variant ?? "default"],
        sizes[local.size ?? "default"],
        local.class
      )}
    />
  );
};

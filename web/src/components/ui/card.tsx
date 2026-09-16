import { splitProps, type Component, type JSX } from "solid-js";
import { cn } from "../../lib/cn";

export const Card: Component<JSX.HTMLAttributes<HTMLDivElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <div
      {...rest}
      class={cn("rounded-lg border border-line bg-secondary text-foreground", local.class)}
    />
  );
};

export const CardHeader: Component<JSX.HTMLAttributes<HTMLDivElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return <div {...rest} class={cn("flex flex-col gap-1.5 p-5", local.class)} />;
};

export const CardTitle: Component<JSX.HTMLAttributes<HTMLHeadingElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return (
    <h3 {...rest} class={cn("font-display text-lg font-semibold leading-none", local.class)} />
  );
};

export const CardDescription: Component<JSX.HTMLAttributes<HTMLParagraphElement>> = (
  props
) => {
  const [local, rest] = splitProps(props, ["class"]);
  return <p {...rest} class={cn("text-sm text-muted", local.class)} />;
};

export const CardContent: Component<JSX.HTMLAttributes<HTMLDivElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return <div {...rest} class={cn("p-5 pt-0", local.class)} />;
};

export const CardFooter: Component<JSX.HTMLAttributes<HTMLDivElement>> = (props) => {
  const [local, rest] = splitProps(props, ["class"]);
  return <div {...rest} class={cn("flex items-center p-5 pt-0", local.class)} />;
};

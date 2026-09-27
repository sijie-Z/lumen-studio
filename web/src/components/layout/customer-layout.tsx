import type { RouteSectionProps } from "@solidjs/router";
import AppShell from "./app-shell";
import RequireAuth from "./require-auth";

export default function CustomerLayout(props: RouteSectionProps) {
  return (
    <RequireAuth>
      <AppShell mode="customer" />
      {props.children}
    </RequireAuth>
  );
}

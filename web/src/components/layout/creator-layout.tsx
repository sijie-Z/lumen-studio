import type { RouteSectionProps } from "@solidjs/router";
import AppShell from "./app-shell";
import RequireAuth from "./require-auth";

export default function CreatorLayout(props: RouteSectionProps) {
  return (
    <RequireAuth>
      <AppShell mode="creator" />
      {props.children}
    </RequireAuth>
  );
}

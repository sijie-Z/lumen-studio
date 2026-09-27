import type { RouteSectionProps } from "@solidjs/router";
import AppShell from "./app-shell";
import RequireRole from "./require-role";

export default function AdminLayout(props: RouteSectionProps) {
  return (
    <RequireRole roles={["admin"]}>
      <AppShell mode="admin" />
      {props.children}
    </RequireRole>
  );
}

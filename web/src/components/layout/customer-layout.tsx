import type { RouteSectionProps } from "@solidjs/router";
import AppShell from "./app-shell";
import RequireRole from "./require-role";

export default function CustomerLayout(props: RouteSectionProps) {
  return (
    <RequireRole roles={["customer"]}>
      <AppShell mode="customer" />
      {props.children}
    </RequireRole>
  );
}

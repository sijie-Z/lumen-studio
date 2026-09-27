import { useNavigate } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import { createEffect, Show, type JSX } from "solid-js";
import { fetchMe } from "../../lib/auth-api";
import { useAuthStore } from "../../stores/auth";
import { LoadingState } from "../ui/state";
import RequireAuth from "./require-auth";

interface RequireRoleProps {
  roles: string[];
  redirectTo?: string;
  children: JSX.Element;
}

export default function RequireRole(props: RequireRoleProps) {
  const auth = useAuthStore();
  const navigate = useNavigate();
  const me = createQuery(() => ({
    queryKey: ["me"] as const,
    queryFn: fetchMe,
    enabled: Boolean(auth.token())
  }));

  const currentUser = () => auth.user() ?? me.data ?? null;
  const allowed = () => {
    const user = currentUser();
    return Boolean(user && props.roles.includes(user.role));
  };

  createEffect(() => {
    if (!auth.token()) return;
    const user = currentUser();
    if (user && !props.roles.includes(user.role)) {
      navigate(props.redirectTo ?? "/");
    }
  });

  return (
    <RequireAuth>
      <Show
        when={allowed()}
        fallback={<LoadingState class="m-8" title="正在校验页面权限" description="请稍候。" />}
      >
        {props.children}
      </Show>
    </RequireAuth>
  );
}

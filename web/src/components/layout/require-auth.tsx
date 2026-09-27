import { useNavigate } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import { createEffect, Show, type JSX } from "solid-js";
import { fetchMe } from "../../lib/auth-api";
import { useAuthStore } from "../../stores/auth";
import { LoadingState } from "../ui/state";

interface RequireAuthProps {
  children: JSX.Element;
}

export default function RequireAuth(props: RequireAuthProps) {
  const auth = useAuthStore();
  const navigate = useNavigate();
  const me = createQuery(() => ({
    queryKey: ["me"] as const,
    queryFn: fetchMe,
    enabled: Boolean(auth.token())
  }));

  createEffect(() => {
    if (!auth.token()) {
      navigate("/login");
      return;
    }
    if (me.data) auth.setCurrentUser(me.data);
    if (me.isError) {
      auth.signOut();
      navigate("/login");
    }
  });

  return (
    <Show
      when={auth.token()}
      fallback={
        <div class="min-h-screen bg-background px-5 py-20 text-foreground">
          <LoadingState title="正在确认登录状态" description="完成后会自动进入对应页面。" />
        </div>
      }
    >
      {props.children}
    </Show>
  );
}

import { createSignal } from "solid-js";
import { getToken, setToken } from "../lib/api";
import type { UserDto } from "../lib/auth-api";

const [token, setAuthToken] = createSignal<string | null>(getToken());
const [user, setUser] = createSignal<UserDto | null>(null);

export function useAuthStore() {
  return {
    token,
    user,
    signIn(accessToken: string, currentUser: UserDto) {
      setToken(accessToken);
      setAuthToken(accessToken);
      setUser(currentUser);
    },
    signOut() {
      setToken(null);
      setAuthToken(null);
      setUser(null);
    },
    setCurrentUser(currentUser: UserDto) {
      setUser(currentUser);
    }
  };
}

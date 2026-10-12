import { create } from "zustand";

import type { AuthUser } from "@/api/auth";

interface AuthState {
  user: AuthUser | null;
  setUser: (user: AuthUser | null) => void;
  hasCapability: (capability: string) => boolean;
}

/** Global auth store. Holds the current user and exposes capability checks. */
export const useAuthStore = create<AuthState>((set, get) => ({
  user: null,
  setUser: (user) => {
    set({ user });
  },
  hasCapability: (capability) => get().user?.capabilities.includes(capability) ?? false,
}));

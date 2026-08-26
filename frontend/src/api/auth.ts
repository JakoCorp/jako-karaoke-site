import { api } from "./client";
import type { components } from "./generated";

export const auth = {
  me: () => api.GET("/auth/me", {}),

  register: (body: components["schemas"]["RegisterRequest"]) =>
    api.POST("/auth/register", { body }),

  login: (body: components["schemas"]["LoginRequest"]) => api.POST("/auth/login", { body }),

  logout: () => api.POST("/auth/logout", {}),
};

import type * as React from "react";
import { useState } from "react";

import { auth } from "@/api/auth";
import { useAuthStore } from "@/store/auth";

interface LoginFormProps {
  readonly onSuccess: () => void;
}

export function LoginForm({ onSuccess }: LoginFormProps) {
  const setUser = useAuthStore((state) => state.setUser);
  const [error, setError] = useState<string | null>(null);
  const [isPending, setIsPending] = useState(false);

  async function submitLogin(username: string, password: string): Promise<void> {
    setIsPending(true);
    try {
      const { data } = await auth.login({ username, password });
      if (!data) {
        setError("Invalid username or password.");
        return;
      }
      setUser({ id: data.id, username: data.username, capabilities: data.capabilities });
      onSuccess();
    } finally {
      setIsPending(false);
    }
  }

  const handleSubmit = (event: React.SubmitEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);
    const formData = new FormData(event.currentTarget);
    const username = formData.get("username");
    const password = formData.get("password");
    if (typeof username !== "string" || typeof password !== "string") return;
    void submitLogin(username, password);
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-3">
      {error !== null && <p className="form-error">{error}</p>}
      <div className="form-field">
        <label htmlFor="login-username" className="form-label">
          Username
        </label>
        <input
          id="login-username"
          name="username"
          type="text"
          required
          autoComplete="username"
          className="form-input"
        />
      </div>
      <div className="form-field">
        <label htmlFor="login-password" className="form-label">
          Password
        </label>
        <input
          id="login-password"
          name="password"
          type="password"
          required
          autoComplete="current-password"
          className="form-input"
        />
      </div>
      <button type="submit" disabled={isPending} className="btn btn-primary mt-1 w-full">
        {isPending ? "Signing in..." : "Sign In"}
      </button>
    </form>
  );
}

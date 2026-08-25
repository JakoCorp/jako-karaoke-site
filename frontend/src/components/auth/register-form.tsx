import type * as React from "react";
import { useState } from "react";

import { auth } from "@/api/auth";
import { useAuthStore } from "@/store/auth";

interface RegisterFormProps {
  readonly onSuccess: () => void;
}

export function RegisterForm({ onSuccess }: RegisterFormProps) {
  const setUser = useAuthStore((state) => state.setUser);
  const [error, setError] = useState<string | null>(null);
  const [isPending, setIsPending] = useState(false);

  async function submitRegister(username: string, password: string): Promise<void> {
    setIsPending(true);
    try {
      const { data, response } = await auth.register({ username, password });
      if (!data) {
        const message =
          response.status === 409
            ? "Username already taken."
            : "Registration failed. Please try again.";
        setError(message);
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
    void submitRegister(username, password);
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-3">
      {error !== null && <p className="form-error">{error}</p>}
      <div className="form-field">
        <label htmlFor="register-username" className="form-label">
          Username
        </label>
        <input
          id="register-username"
          name="username"
          type="text"
          required
          autoComplete="username"
          className="form-input"
        />
      </div>
      <div className="form-field">
        <label htmlFor="register-password" className="form-label">
          Password
        </label>
        <input
          id="register-password"
          name="password"
          type="password"
          required
          autoComplete="new-password"
          minLength={8}
          className="form-input"
        />
      </div>
      <button type="submit" disabled={isPending} className="btn btn-primary mt-1 w-full">
        {isPending ? "Creating account..." : "Create Account"}
      </button>
    </form>
  );
}

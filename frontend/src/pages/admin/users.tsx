import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";

import { capabilities as capabilitiesApi } from "@/api/capabilities";
import type { components } from "@/api/generated";
import { users as usersApi } from "@/api/users";

type UserSummary = components["schemas"]["UserSummary"];

export function UsersAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [selectedUser, setSelectedUser] = useState<UserSummary | null>(null);
  const [isGrantingAll, setIsGrantingAll] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(searchInput.trim());
    }, 300);
    return () => {
      clearTimeout(timer);
    };
  }, [searchInput]);

  const queryClient = useQueryClient();

  const { data: allCapabilities } = useQuery({
    queryKey: ["capabilities"],
    queryFn: async () => {
      const { data, error } = await capabilitiesApi.list();
      if (error) throw error;
      return data;
    },
  });

  const { data: userResults, isLoading: usersLoading } = useQuery({
    queryKey: ["admin", "users", debouncedQuery],
    queryFn: async () => {
      const { data, error } = await usersApi.search(debouncedQuery || undefined);
      if (error) throw error;
      return data;
    },
  });

  const { data: userCapabilities, isLoading: capsLoading } = useQuery({
    queryKey: ["admin", "users", selectedUser?.id, "capabilities"],
    queryFn: async () => {
      const { data, error } = await usersApi.listCapabilities(selectedUser!.id);
      if (error) throw error;
      return data ?? [];
    },
    enabled: !!selectedUser,
  });

  const invalidateCaps = () =>
    queryClient.invalidateQueries({
      queryKey: ["admin", "users", selectedUser?.id, "capabilities"],
    });

  const grantMutation = useMutation({
    mutationFn: (capability: string) => usersApi.grantCapability(selectedUser!.id, capability),
    onSuccess: invalidateCaps,
  });

  const revokeMutation = useMutation({
    mutationFn: (capability: string) => usersApi.revokeCapability(selectedUser!.id, capability),
    onSuccess: invalidateCaps,
  });

  function handleToggle(cap: string, has: boolean) {
    if (has) {
      revokeMutation.mutate(cap);
    } else {
      grantMutation.mutate(cap);
    }
  }

  async function handleGrantAll() {
    if (!allCapabilities || !selectedUser) return;
    const missing = allCapabilities.filter((c) => !userCapabilities?.includes(c));
    if (missing.length === 0) return;
    setIsGrantingAll(true);
    try {
      await Promise.all(missing.map((cap) => usersApi.grantCapability(selectedUser.id, cap)));
      await invalidateCaps();
    } finally {
      setIsGrantingAll(false);
    }
  }

  const isMutating = grantMutation.isPending || revokeMutation.isPending || isGrantingAll;

  return (
    <div className="admin-layout">
      <div className="admin-panel">
        <input
          type="search"
          className="form-input"
          placeholder="Search users…"
          value={searchInput}
          onChange={(e) => {
            setSearchInput(e.target.value);
          }}
          aria-label="Search users"
        />
        <ul className="admin-user-list">
          {usersLoading && <li className="admin-empty">Loading…</li>}
          {!usersLoading && userResults?.length === 0 && (
            <li className="admin-empty">No users found.</li>
          )}
          {userResults?.map((u) => (
            <li key={u.id}>
              <button
                className={`admin-user-item${selectedUser?.id === u.id ? " admin-user-item--active" : ""}`}
                onClick={() => {
                  setSelectedUser(u);
                }}
              >
                @{u.username}
              </button>
            </li>
          ))}
        </ul>
      </div>

      <div className="admin-panel">
        {selectedUser ? (
          <>
            <div className="admin-panel-header">
              <h3 className="admin-panel-title">@{selectedUser.username}</h3>
              <button
                className="btn btn-secondary"
                onClick={() => {
                  void handleGrantAll();
                }}
                disabled={isMutating || capsLoading}
              >
                Grant All
              </button>
            </div>
            {capsLoading ? (
              <p className="admin-empty">Loading…</p>
            ) : (
              <ul className="admin-cap-list">
                {allCapabilities?.map((cap) => {
                  const has = userCapabilities?.includes(cap) ?? false;
                  return (
                    <li key={cap} className="admin-cap-item">
                      <label className="admin-cap-label">
                        <input
                          type="checkbox"
                          checked={has}
                          disabled={isMutating}
                          onChange={() => {
                            handleToggle(cap, has);
                          }}
                        />
                        {cap}
                      </label>
                    </li>
                  );
                })}
              </ul>
            )}
          </>
        ) : (
          <p className="admin-empty">Select a user to manage capabilities.</p>
        )}
      </div>
    </div>
  );
}

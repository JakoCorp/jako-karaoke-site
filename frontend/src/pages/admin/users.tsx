import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { usersApi, type UserSummary } from "@/api/users";
import { useCapabilities } from "@/hooks/api/capabilities";
import { useUserCapabilities, userKeys, useUsers } from "@/hooks/api/users";
import { useDebounced } from "@/hooks/use-debounced";

export function UsersAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [selectedUser, setSelectedUser] = useState<UserSummary | null>(null);
  const [isGrantingAll, setIsGrantingAll] = useState(false);

  const debouncedQuery = useDebounced(searchInput.trim());

  const queryClient = useQueryClient();

  const { data: allCapabilities } = useCapabilities();
  const { data: userResults, isLoading: usersLoading } = useUsers(debouncedQuery || undefined);
  const { data: userCapabilities, isLoading: capsLoading } = useUserCapabilities(selectedUser?.id);

  const invalidateCaps = () =>
    queryClient.invalidateQueries({
      queryKey: userKeys.capabilities(selectedUser?.id ?? ""),
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
          onChange={(event) => {
            setSearchInput(event.target.value);
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
                className={
                  selectedUser?.id === u.id
                    ? "admin-user-item admin-user-item--active"
                    : "admin-user-item"
                }
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

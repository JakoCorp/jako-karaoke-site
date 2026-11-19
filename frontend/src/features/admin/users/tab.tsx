import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";

import { ensureOk } from "@/api/client";
import { usersApi, type AdminUserSummary } from "@/api/users";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { useCapabilities } from "@/hooks/api/capabilities";
import {
  useInfiniteUsers,
  useUnsuspendUser,
  useUserCapabilities,
  userKeys,
} from "@/hooks/api/users";
import { useDebounced } from "@/hooks/use-debounced";
import { useInfiniteSentinel } from "@/hooks/use-infinite-sentinel";
import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";
import { useAuthStore } from "@/store/auth";

import { PlaysDialog } from "./plays-dialog";
import { RenameDialog } from "./rename-dialog";
import { SuspendDialog } from "./suspend-dialog";
import { SuspensionHistory } from "./suspension-history";

export function UsersAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [selectedUser, setSelectedUser] = useState<AdminUserSummary | null>(null);
  const [isGrantingAll, setIsGrantingAll] = useState(false);
  const [isRenaming, setIsRenaming] = useState(false);
  const [isViewingPlays, setIsViewingPlays] = useState(false);
  const [isSuspending, setIsSuspending] = useState(false);
  const [isConfirmingUnsuspend, setIsConfirmingUnsuspend] = useState(false);
  const canManagePlays = useAuthStore((state) => state.hasCapability("plays:manage_any"));
  const signedInUserId = useAuthStore((state) => state.user?.id);
  const unsuspend = useUnsuspendUser();

  const debouncedQuery = useDebounced(searchInput.trim());

  const queryClient = useQueryClient();

  const { data: allCapabilities } = useCapabilities();
  const scrollContainerRef = useRef<HTMLUListElement>(null);

  const {
    data: userPages,
    isLoading: usersLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteUsers(debouncedQuery || undefined);
  const sentinelRef = useInfiniteSentinel<HTMLLIElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
    scrollContainerRef,
  });
  const users = userPages?.pages.flatMap((page) => page.items) ?? [];
  const { data: userCapabilities, isLoading: capsLoading } = useUserCapabilities(selectedUser?.id);

  const invalidateCaps = () =>
    queryClient.invalidateQueries({
      queryKey: userKeys.capabilities(selectedUser?.id ?? ""),
    });

  const grantMutation = useMutation({
    mutationFn: (capability: string) =>
      usersApi.grantCapability(selectedUser!.id, capability).then(ensureOk),
    onSuccess: invalidateCaps,
    onError: (error) => {
      toast.error("Could not grant capability", {
        description: errorMessage(error, "Something went wrong."),
      });
    },
  });

  const revokeMutation = useMutation({
    mutationFn: (capability: string) =>
      usersApi.revokeCapability(selectedUser!.id, capability).then(ensureOk),
    onSuccess: invalidateCaps,
    onError: (error) => {
      toast.error("Could not revoke capability", {
        description: errorMessage(error, "Something went wrong."),
      });
    },
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
      await Promise.all(
        missing.map((cap) => usersApi.grantCapability(selectedUser.id, cap).then(ensureOk)),
      );
    } catch (error) {
      toast.error("Could not grant all capabilities", {
        description: errorMessage(error, "Something went wrong."),
      });
    } finally {
      await invalidateCaps();
      setIsGrantingAll(false);
    }
  }

  function handleConfirmUnsuspend() {
    if (!selectedUser) return;
    unsuspend.mutate(selectedUser.id, {
      onSuccess: (user) => {
        setSelectedUser(user);
        setIsConfirmingUnsuspend(false);
      },
    });
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
        <ul ref={scrollContainerRef} className="admin-user-list">
          {usersLoading && <li className="admin-empty">Loading…</li>}
          {!usersLoading && users.length === 0 && <li className="admin-empty">No users found.</li>}
          {users.map((u) => (
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
                {u.suspended_at && <div className="admin-item-sub">Suspended</div>}
              </button>
            </li>
          ))}
          <li ref={sentinelRef} />
          {isFetchingNextPage && <li className="admin-empty">Loading more…</li>}
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
                  setIsRenaming(true);
                }}
              >
                Rename
              </button>
              {canManagePlays && (
                <button
                  className="btn btn-secondary"
                  onClick={() => {
                    setIsViewingPlays(true);
                  }}
                >
                  Plays
                </button>
              )}
              <button
                className="btn btn-secondary"
                onClick={() => {
                  void handleGrantAll();
                }}
                disabled={isMutating || capsLoading}
              >
                Grant All
              </button>
              <button
                className="btn btn-secondary"
                onClick={() => {
                  if (selectedUser.suspended_at) {
                    unsuspend.reset();
                    setIsConfirmingUnsuspend(true);
                  } else {
                    setIsSuspending(true);
                  }
                }}
                disabled={selectedUser.id === signedInUserId}
                title={
                  selectedUser.id === signedInUserId ? "You cannot suspend yourself." : undefined
                }
              >
                {selectedUser.suspended_at ? "Unsuspend" : "Suspend"}
              </button>
            </div>
            <SuspensionHistory userId={selectedUser.id} />
            <SuspendDialog
              user={selectedUser}
              open={isSuspending}
              onOpenChange={setIsSuspending}
              onSuspended={setSelectedUser}
            />
            <ConfirmDialog
              open={isConfirmingUnsuspend}
              onOpenChange={setIsConfirmingUnsuspend}
              onConfirm={handleConfirmUnsuspend}
              title={`Unsuspend @${selectedUser.username}?`}
              description="They will be able to sign in again."
              confirmLabel="Unsuspend"
              pendingLabel="Saving…"
              isPending={unsuspend.isPending}
              error={
                unsuspend.isError
                  ? errorMessage(unsuspend.error, "Could not lift the suspension.")
                  : null
              }
            />
            <RenameDialog
              user={selectedUser}
              open={isRenaming}
              onOpenChange={setIsRenaming}
              onRenamed={setSelectedUser}
            />
            {canManagePlays && (
              <PlaysDialog
                user={selectedUser}
                open={isViewingPlays}
                onOpenChange={setIsViewingPlays}
              />
            )}
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

import type { UserSummary } from "@/api/users";
import { useUserSuspensions } from "@/hooks/api/users";
import { formatDate } from "@/lib/format";

function adminName(admin: UserSummary | null | undefined): string {
  return admin ? `@${admin.username}` : "a deleted account";
}

/** Lists a user's suspensions with who started and lifted each one and the recorded reason. */
export function SuspensionHistory({ userId }: { userId: string }) {
  const { data: suspensions = [] } = useUserSuspensions(userId);

  if (suspensions.length === 0) return null;

  return (
    <div className="admin-detail-section">
      <span className="admin-detail-label">Suspensions</span>
      <ul className="admin-suspension-list">
        {suspensions.map((suspension) => (
          <li key={suspension.id}>
            <div>
              {formatDate(suspension.suspended_at)} by {adminName(suspension.suspended_by)}
            </div>
            {suspension.lifted_at && (
              <div className="admin-item-sub">
                Lifted {formatDate(suspension.lifted_at)} by {adminName(suspension.lifted_by)}
              </div>
            )}
            <div className="admin-item-sub">{suspension.reason}</div>
          </li>
        ))}
      </ul>
    </div>
  );
}

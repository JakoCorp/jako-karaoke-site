import { SuspendedNotice, UsernameClaim } from "@/features/auth";

export function HomePage() {
  return (
    <>
      <SuspendedNotice />
      <UsernameClaim />
      <div className="p-6">Home</div>
    </>
  );
}

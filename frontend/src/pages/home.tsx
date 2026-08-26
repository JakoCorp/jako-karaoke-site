import { UsernameClaim } from "@/components/auth/username-claim";

export function HomePage() {
  return (
    <>
      <UsernameClaim />
      <div className="p-6">Home</div>
    </>
  );
}

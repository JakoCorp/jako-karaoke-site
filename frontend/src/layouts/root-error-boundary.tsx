import { NotFoundPage } from "@/features/not-found/page";

import { RootLayout } from "./root-layout";

export function RootErrorBoundary() {
  return (
    <RootLayout>
      <NotFoundPage />
    </RootLayout>
  );
}

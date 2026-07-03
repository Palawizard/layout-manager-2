import type { PropsWithChildren } from "react";

import { Toaster } from "../components/ui/toaster";
import { UpdateManager } from "../features/updates/update-manager";

export function AppProviders({ children }: PropsWithChildren) {
  return (
    <>
      {children}
      <UpdateManager />
      <Toaster />
    </>
  );
}

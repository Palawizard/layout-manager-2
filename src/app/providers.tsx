import type { PropsWithChildren } from "react";

import { Toaster } from "../components/ui/toaster";

export function AppProviders({ children }: PropsWithChildren) {
  return (
    <>
      {children}
      <Toaster />
    </>
  );
}

import { createHashRouter, Navigate } from "react-router";

import { AppLayout } from "../components/layout/app-layout";
import { LayoutsPage } from "../features/layouts/layouts-page";
import { SettingsPage } from "../features/settings/settings-page";

export const router = createHashRouter([
  {
    element: <AppLayout />,
    children: [
      { index: true, element: <Navigate to="/layouts" replace /> },
      { path: "/layouts", element: <LayoutsPage /> },
      { path: "/settings", element: <SettingsPage /> },
    ],
  },
]);

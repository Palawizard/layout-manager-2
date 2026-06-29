import { createHashRouter, Navigate } from "react-router";

import { AppLayout } from "../components/layout/app-layout";
import { LayoutEditorPage } from "../features/layouts/layout-editor-page";
import { LayoutsPage } from "../features/layouts/layouts-page";
import { ExecutionPage } from "../features/execution/execution-page";
import { SettingsPage } from "../features/settings/settings-page";

export const router = createHashRouter([
  {
    element: <AppLayout />,
    children: [
      { index: true, element: <Navigate to="/layouts" replace /> },
      { path: "/layouts", element: <LayoutsPage /> },
      { path: "/layouts/:layoutId", element: <LayoutEditorPage /> },
      { path: "/execution", element: <ExecutionPage /> },
      { path: "/settings", element: <SettingsPage /> },
    ],
  },
]);

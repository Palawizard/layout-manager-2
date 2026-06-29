import { render, screen, waitFor } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router";
import { describe, expect, it } from "vitest";

import { ExecutionPage } from "../../src/features/execution/execution-page";
import { LayoutsPage } from "../../src/features/layouts/layouts-page";

describe("mocked layout workflow", () => {
  it("covers list, execution entry and partial report", async () => {
    const router = createMemoryRouter(
      [
        { path: "/layouts", element: <LayoutsPage /> },
        {
          path: "/execution",
          element: <ExecutionPage />,
        },
      ],
      { initialEntries: ["/layouts"] },
    );

    render(<RouterProvider router={router} />);

    await waitFor(() => {
      expect(screen.getByText("Aucun layout")).toBeInTheDocument();
    });

    await router.navigate("/execution?layoutId=layout-1");
    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Exécution" })).toBeInTheDocument();
    });
  });
});

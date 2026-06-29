import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { describe, expect, it } from "vitest";

import { LayoutsPage } from "./layouts-page";

describe("LayoutsPage", () => {
  it("shows an empty state then navigates to create a layout", async () => {
    const user = userEvent.setup();
    const router = createMemoryRouter(
      [
        { path: "/layouts", element: <LayoutsPage /> },
        { path: "/layouts/new", element: <p>Éditeur</p> },
      ],
      { initialEntries: ["/layouts"] },
    );

    render(<RouterProvider router={router} />);

    await waitFor(() => {
      expect(screen.getByText("Aucun layout")).toBeInTheDocument();
    });

    await user.click(screen.getAllByRole("button", { name: "Nouveau layout" })[0]!);
    expect(screen.getByText("Éditeur")).toBeInTheDocument();
  });
});

import { render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router";
import { describe, expect, it } from "vitest";

import { AppLayout } from "./app-layout";

describe("AppLayout accessibility", () => {
  it("exposes a skip link and a labelled main landmark", () => {
    const router = createMemoryRouter(
      [
        {
          element: <AppLayout />,
          children: [{ path: "/", element: <p>Contenu</p> }],
        },
      ],
      { initialEntries: ["/"] },
    );

    render(<RouterProvider router={router} />);

    expect(screen.getByRole("link", { name: "Aller au contenu" })).toHaveAttribute(
      "href",
      "#main-content",
    );
    expect(screen.getByRole("main")).toHaveAttribute("id", "main-content");
    expect(screen.getByRole("navigation", { name: "Navigation principale" })).toBeInTheDocument();
  });
});

import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";

import { renderWithProviders } from "../test/render";
import { App } from "./App";

describe("App", () => {
  beforeEach(() => {
    window.location.hash = "#/layouts";
  });

  it("shows the empty layouts route", async () => {
    renderWithProviders(<App />);

    expect(await screen.findByRole("heading", { level: 1, name: "Layouts" })).toBeVisible();
    expect(screen.getByText("Aucun layout")).toBeVisible();
  });

  it("navigates to settings and opens the information dialog", async () => {
    const user = userEvent.setup();
    renderWithProviders(<App />);

    await user.click(screen.getByRole("link", { name: "Réglages" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Réglages" })).toBeVisible();

    await user.click(screen.getByRole("button", { name: "Afficher les informations" }));
    expect(screen.getByRole("dialog", { name: "Layout Manager 2" })).toBeVisible();
  });
});

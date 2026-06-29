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
    expect(await screen.findByText("Aucun layout")).toBeVisible();
  });

  it("navigates to settings and shows preferences", async () => {
    const user = userEvent.setup();
    renderWithProviders(<App />);

    await user.click(screen.getByRole("link", { name: "Réglages" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Réglages" })).toBeVisible();
    expect(screen.getByLabelText("Navigateur préféré")).toBeVisible();
    expect(screen.getByRole("button", { name: "Ouvrir le dossier des données" })).toBeVisible();
  });
});

import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { renderWithProviders } from "../../test/render";
import * as windowsApi from "../../lib/tauri/windows";
import { WindowPicker } from "./window-picker";

describe("WindowPicker", () => {
  it("loads, filters and selects a desktop window", async () => {
    const onSelect = vi.fn();
    vi.spyOn(windowsApi, "listDesktopWindows").mockResolvedValue([
      {
        processId: 42,
        executablePath: "C:\\Windows\\notepad.exe",
        processName: "notepad.exe",
        title: "Notes",
        className: "Notepad",
        bounds: { x: 0, y: 0, width: 800, height: 600 },
        state: "normal",
        monitorId: "DISPLAY1",
      },
    ]);

    const user = userEvent.setup();
    renderWithProviders(<WindowPicker onOpenChange={vi.fn()} onSelect={onSelect} open />);

    await user.type(await screen.findByPlaceholderText("Rechercher une application"), "note");
    await user.click(screen.getByRole("button", { name: /notepad.exe/i }));

    expect(onSelect).toHaveBeenCalledWith(
      expect.objectContaining({ processId: 42 }),
      expect.objectContaining({ processName: "notepad.exe", className: "Notepad" }),
    );
  });
});

import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { UpdateManager } from "./update-manager";

const { checkMock, relaunchMock } = vi.hoisted(() => ({
  checkMock: vi.fn(),
  relaunchMock: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({ check: checkMock }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: relaunchMock }));

describe("UpdateManager", () => {
  beforeEach(() => {
    checkMock.mockReset();
    relaunchMock.mockReset();
  });

  it("stays silent when no update is available", async () => {
    checkMock.mockResolvedValue(null);

    render(<UpdateManager />);

    await waitFor(() => expect(checkMock).toHaveBeenCalledWith({ timeout: 15_000 }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("installs an available update after confirmation", async () => {
    const downloadAndInstall = vi.fn(async (onEvent: (event: object) => void) => {
      onEvent({ event: "Started", data: { contentLength: 100 } });
      onEvent({ event: "Progress", data: { chunkLength: 100 } });
    });
    checkMock.mockResolvedValue({ version: "0.1.0-beta.2", downloadAndInstall });
    relaunchMock.mockResolvedValue(undefined);

    render(<UpdateManager />);

    expect(
      await screen.findByText("La version 0.1.0-beta.2 est disponible.", { exact: false }),
    ).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: "Installer et redémarrer" }));

    await waitFor(() => expect(downloadAndInstall).toHaveBeenCalledOnce());
    expect(relaunchMock).toHaveBeenCalledOnce();
  });

  it("allows postponing an available update", async () => {
    checkMock.mockResolvedValue({ version: "0.1.0-beta.2", downloadAndInstall: vi.fn() });

    render(<UpdateManager />);

    await screen.findByRole("dialog");
    await userEvent.click(screen.getByRole("button", { name: "Plus tard" }));

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});

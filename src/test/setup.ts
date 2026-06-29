import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

afterEach(() => cleanup());

Object.defineProperty(window, "matchMedia", {
  configurable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((command: string) => {
    switch (command) {
      case "get_app_info":
        return Promise.resolve({
          name: "Layout Manager 2",
          version: "0.1.0",
          platform: "windows",
        });
      case "list_layouts":
        return Promise.resolve([]);
      case "get_settings":
        return Promise.resolve({
          preferredBrowser: "edge",
          defaultStartupTimeoutMs: 15_000,
          monitorFallback: "primary",
        });
      default:
        return Promise.resolve(null);
    }
  }),
}));

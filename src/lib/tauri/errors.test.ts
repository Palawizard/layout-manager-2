import { describe, expect, it } from "vitest";

import { readPublicErrorMessage } from "./errors";

describe("readPublicErrorMessage", () => {
  it("reads the backend validation message", () => {
    expect(
      readPublicErrorMessage({
        code: "execution_plan_failed",
        message: "Aucun écran n’est disponible.",
        field: null,
        retryable: false,
      }),
    ).toBe("Aucun écran n’est disponible.");
  });

  it("falls back when the payload is unknown", () => {
    expect(readPublicErrorMessage("boom", "Échec du lancement.")).toBe("Échec du lancement.");
  });
});

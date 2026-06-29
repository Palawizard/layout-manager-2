import { describe, expect, it } from "vitest";

import { layoutDetailsSchema } from "./layout-schema";

describe("layoutDetailsSchema", () => {
  it("requires a layout name", () => {
    const result = layoutDetailsSchema.safeParse({ name: " ", description: "" });
    expect(result.success).toBe(false);
  });

  it("accepts a valid layout name", () => {
    const result = layoutDetailsSchema.safeParse({ name: "Travail", description: "Bureau" });
    expect(result.success).toBe(true);
  });
});

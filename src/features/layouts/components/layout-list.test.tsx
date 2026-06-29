import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { LayoutList } from "./layout-list";

describe("LayoutList", () => {
  it("runs and edits a layout from the card actions", async () => {
    const user = userEvent.setup();
    const onRun = vi.fn();
    const onEdit = vi.fn();

    render(
      <LayoutList
        layouts={[
          {
            id: "layout-1",
            name: "Travail",
            description: "Bureau principal",
            actionCount: 2,
            updatedAt: Date.now(),
          },
        ]}
        onDelete={vi.fn()}
        onDuplicate={vi.fn()}
        onEdit={onEdit}
        onRun={onRun}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Lancer" }));
    await user.click(screen.getByRole("button", { name: "Modifier" }));

    expect(onRun).toHaveBeenCalledWith("layout-1");
    expect(onEdit).toHaveBeenCalledWith("layout-1");
    expect(screen.getByRole("heading", { name: "Travail" })).toBeInTheDocument();
  });
});

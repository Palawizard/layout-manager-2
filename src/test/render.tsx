import { render, type RenderOptions } from "@testing-library/react";
import type { ReactElement } from "react";

import { AppProviders } from "../app/providers";

export function renderWithProviders(element: ReactElement, options?: RenderOptions) {
  return render(element, { wrapper: AppProviders, ...options });
}

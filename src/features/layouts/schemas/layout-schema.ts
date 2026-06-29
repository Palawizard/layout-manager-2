import { z } from "zod";

const normalizedBoundsSchema = z.object({
  x: z.number().min(0).max(1),
  y: z.number().min(0).max(1),
  width: z.number().positive().max(1),
  height: z.number().positive().max(1),
});

const windowMatcherSchema = z.object({
  executablePath: z.string().nullable(),
  processName: z.string().nullable(),
  className: z.string().nullable(),
  titlePattern: z.string().nullable(),
  instanceIndex: z.number().int().nonnegative().nullable(),
});

const windowPlacementSchema = z.object({
  monitorSelector: z.object({
    preferredId: z.string().min(1),
    fallback: z.enum(["primary", "first_available"]),
  }),
  bounds: normalizedBoundsSchema,
  state: z.enum(["normal", "maximized", "minimized"]),
  centerScale: z.number().min(0.1).max(1).nullable().optional(),
});

const layoutActionSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("launch_application"),
    id: z.string().min(1),
    executablePath: z.string().min(1, "Choisissez une application."),
    arguments: z.array(z.string()),
    workingDirectory: z.string().nullable(),
    reuseExistingWindow: z.boolean(),
    windowMatcher: windowMatcherSchema,
    placement: windowPlacementSchema,
    startupTimeoutMs: z.number().int().min(1000).max(120_000),
  }),
  z.object({
    kind: z.literal("place_existing_window"),
    id: z.string().min(1),
    windowMatcher: windowMatcherSchema.refine(
      (matcher) =>
        Boolean(
          matcher.executablePath ||
          matcher.processName ||
          matcher.className ||
          matcher.titlePattern,
        ),
      { message: "Sélectionnez une fenêtre à retrouver." },
    ),
    placement: windowPlacementSchema,
    capturedPlacement: windowPlacementSchema.optional(),
    executablePath: z.string().nullable(),
    reopenIfAbsent: z.boolean(),
    startupTimeoutMs: z.number().int().min(1000).max(120_000),
  }).superRefine((action, context) => {
    if (action.reopenIfAbsent && !action.executablePath) {
      context.addIssue({
        code: z.ZodIssueCode.custom,
        message: "Choisissez une fenêtre pour enregistrer l’application à relancer.",
        path: ["windowMatcher"],
      });
    }
  }),
  z.object({
    kind: z.literal("open_browser_window"),
    id: z.string().min(1),
    browserKind: z.enum(["edge", "chrome", "firefox", "system_default"]),
    executablePath: z.string().nullable(),
    profile: z.string().nullable(),
    urls: z
      .array(z.string().min(1, "Adresse web invalide."))
      .min(1, "Ajoutez au moins une adresse web."),
    placement: windowPlacementSchema,
    startupTimeoutMs: z.number().int().min(1000).max(120_000),
  }),
]);

export const layoutDetailsSchema = z.object({
  name: z
    .string()
    .trim()
    .min(1, "Le nom est requis.")
    .max(80, "Le nom ne peut pas dépasser 80 caractères."),
  description: z
    .string()
    .trim()
    .max(300, "La description ne peut pas dépasser 300 caractères.")
    .optional()
    .or(z.literal("")),
});

export const layoutDraftSchema = layoutDetailsSchema.extend({
  actions: z.array(layoutActionSchema).min(1, "Ajoutez au moins une action."),
  options: z.object({
    minimizeUnmatchedWindows: z.boolean(),
    continueOnError: z.boolean(),
    restorePreviousStateOnCancel: z.boolean(),
  }),
});

export type LayoutDetailsValues = z.infer<typeof layoutDetailsSchema>;
export type LayoutDraftValues = z.infer<typeof layoutDraftSchema>;

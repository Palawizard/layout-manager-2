export interface PublicErrorPayload {
  code?: string;
  message?: string;
  field?: string | null;
  retryable?: boolean;
}

export function readPublicErrorMessage(
  error: unknown,
  fallback = "Une erreur est survenue.",
): string {
  if (error && typeof error === "object" && "message" in error) {
    const message = (error as PublicErrorPayload).message;
    if (typeof message === "string" && message.trim().length > 0) {
      return message;
    }
  }
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message;
  }
  return fallback;
}

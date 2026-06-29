import type { LayoutRunReport } from "../types/execution";

export function getRetryableActionIds(report: LayoutRunReport): string[] {
  return report.results
    .filter((result) => result.status === "failed" && result.retryable)
    .map((result) => result.actionId);
}

export function describeRunStatus(status: LayoutRunReport["status"]): string {
  switch (status) {
    case "success":
      return "Layout appliqué";
    case "partial_failure":
      return "Certaines actions n’ont pas abouti";
    case "failed":
      return "Le layout n’a pas pu être appliqué";
    case "cancelled":
      return "Exécution annulée";
  }
}

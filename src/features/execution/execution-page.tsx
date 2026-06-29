import { ArrowLeft, RotateCcw, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router";
import { toast } from "sonner";

import { Button } from "../../components/ui/button";
import { Card, CardContent } from "../../components/ui/card";
import { Progress } from "../../components/ui/progress";
import { cancelLayoutRun, runLayout } from "../../lib/tauri/execution";
import { readPublicErrorMessage } from "../../lib/tauri/errors";
import { subscribeToEvent } from "../../lib/tauri/events";
import type {
  ActionCompletedEvent,
  ActionRunResult,
  ActionStartedEvent,
  LayoutRunReport,
  RunCompletedEvent,
  RunStartedEvent,
} from "./types/execution";
import { describeRunStatus, getRetryableActionIds } from "./lib/retry";

const ACTION_STATUS_LABELS: Record<ActionRunResult["status"], string> = {
  pending: "En attente",
  running: "En cours",
  succeeded: "Terminée",
  failed: "Échouée",
  skipped: "Ignorée",
  cancelled: "Annulée",
};

interface ExecutionRunnerProps {
  layoutId: string;
  retryIds: string[] | null;
}

function ExecutionRunner({ layoutId, retryIds }: ExecutionRunnerProps) {
  const navigate = useNavigate();
  const [layoutName, setLayoutName] = useState("Layout");
  const [totalActions, setTotalActions] = useState(0);
  const [completedActions, setCompletedActions] = useState(0);
  const [currentLabel, setCurrentLabel] = useState<string | null>(null);
  const [results, setResults] = useState<ActionRunResult[]>([]);
  const [report, setReport] = useState<LayoutRunReport | null>(null);
  const [status, setStatus] = useState<"running" | "done" | "error">("running");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const progressValue = useMemo(() => {
    if (totalActions === 0) return 0;
    if (report) return 100;
    return Math.round((completedActions / totalActions) * 100);
  }, [completedActions, report, totalActions]);

  useEffect(() => {
    let disposed = false;
    const unlisteners: Array<() => void> = [];

    async function start() {
      const listeners = await Promise.all([
        subscribeToEvent<RunStartedEvent>("layout-run://started", (event) => {
          if (disposed) return;
          setLayoutName(event.payload.layoutName);
          setTotalActions(event.payload.totalActions);
        }),
        subscribeToEvent<ActionStartedEvent>("layout-run://action-started", (event) => {
          if (disposed) return;
          setCurrentLabel(event.payload.label);
          setResults((current) => {
            const next = [...current];
            const index = next.findIndex((item) => item.actionId === event.payload.actionId);
            const entry: ActionRunResult = {
              actionId: event.payload.actionId,
              label: event.payload.label,
              status: "running",
              message: null,
              reusedExistingWindow: false,
              retryable: false,
            };
            if (index >= 0) next[index] = entry;
            else next.push(entry);
            return next;
          });
        }),
        subscribeToEvent<ActionCompletedEvent>("layout-run://action-completed", (event) => {
          if (disposed) return;
          setResults((current) => {
            const next = current.filter((item) => item.actionId !== event.payload.result.actionId);
            return [...next, event.payload.result];
          });
          if (event.payload.result.status === "succeeded") {
            setCompletedActions((value) => value + 1);
          }
        }),
        subscribeToEvent<RunCompletedEvent>("layout-run://completed", (event) => {
          if (disposed) return;
          setReport(event.payload.report);
          setCurrentLabel(null);
          setStatus("done");
        }),
      ]);
      unlisteners.push(...listeners);

      if (disposed) {
        return;
      }

      try {
        await runLayout(layoutId, retryIds ?? undefined);
      } catch (error) {
        if (!disposed) {
          const message = readPublicErrorMessage(error, "Impossible de lancer ce layout.");
          setErrorMessage(message);
          setStatus("error");
          toast.error(message);
        }
      }
    }

    void start();

    return () => {
      disposed = true;
      for (const unlisten of unlisteners) unlisten();
      void cancelLayoutRun().catch(() => undefined);
    };
  }, [layoutId, retryIds]);

  async function handleCancel() {
    try {
      await cancelLayoutRun();
    } catch {
      toast.error("Impossible d’annuler l’exécution.");
    }
  }

  function handleRetryFailed() {
    if (!report) return;
    const failedIds = getRetryableActionIds(report);
    if (failedIds.length === 0) return;
    navigate(`/execution?layoutId=${layoutId}&retry=${failedIds.join(",")}`);
  }

  const failedRetryableCount = report ? getRetryableActionIds(report).length : 0;

  if (status === "error") {
    return (
      <Card>
        <CardContent className="p-6 text-center">
          <p>{errorMessage ?? "Impossible de démarrer l’exécution."}</p>
          <Button className="mt-4" onClick={() => navigate("/layouts")} variant="secondary">
            Retour aux layouts
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      <p className="text-sm text-muted-foreground">{layoutName}</p>
      <Card>
        <CardContent className="space-y-4 p-6">
          <div
            aria-atomic="true"
            aria-live="polite"
            className="flex items-center justify-between gap-4"
            role="status"
          >
            <p className="text-sm font-medium">
              {report
                ? describeRunStatus(report.status)
                : currentLabel
                  ? `Traitement de ${currentLabel}`
                  : "Préparation du layout"}
            </p>
            {status === "running" && (
              <Button onClick={() => void handleCancel()} size="small" variant="secondary">
                <X aria-hidden="true" className="size-4" />
                Annuler
              </Button>
            )}
          </div>
          <Progress aria-label="Progression du layout" value={progressValue} />
          <p className="text-xs text-muted-foreground">
            {report
              ? `${report.completedActions} action${report.completedActions > 1 ? "s" : ""} terminée${report.completedActions > 1 ? "s" : ""} sur ${report.totalActions}`
              : totalActions > 0
                ? `${completedActions} action${completedActions > 1 ? "s" : ""} terminée${completedActions > 1 ? "s" : ""} sur ${totalActions}`
                : "Démarrage en cours…"}
          </p>
        </CardContent>
      </Card>

      {results.length > 0 && (
        <ul className="space-y-3">
          {results.map((result) => (
            <li key={result.actionId}>
              <Card>
                <CardContent className="flex items-start justify-between gap-4 p-4">
                  <div>
                    <p className="font-medium">{result.label}</p>
                    {result.message ? (
                      <p className="mt-1 text-sm text-muted-foreground">{result.message}</p>
                    ) : null}
                    {result.reusedExistingWindow ? (
                      <p className="mt-1 text-xs text-muted-foreground">
                        Fenêtre existante réutilisée
                      </p>
                    ) : null}
                  </div>
                  <span className="text-sm text-muted-foreground">
                    {ACTION_STATUS_LABELS[result.status]}
                  </span>
                </CardContent>
              </Card>
            </li>
          ))}
        </ul>
      )}

      {report && report.warnings.length > 0 && (
        <Card>
          <CardContent className="space-y-2 p-4">
            <h2 className="font-medium">Avertissements</h2>
            <ul className="space-y-1 text-sm text-muted-foreground">
              {report.warnings.map((warning) => (
                <li key={`${warning.code}-${warning.message}`}>{warning.message}</li>
              ))}
            </ul>
          </CardContent>
        </Card>
      )}

      {report && failedRetryableCount > 0 && (
        <Button onClick={handleRetryFailed}>
          <RotateCcw aria-hidden="true" className="size-4" />
          Réessayer les actions échouées
        </Button>
      )}
    </div>
  );
}

export function ExecutionPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const layoutId = searchParams.get("layoutId");
  const retryIds = searchParams.get("retry")?.split(",").filter(Boolean) ?? null;
  const runnerKey = `${layoutId ?? "missing"}:${retryIds?.join(",") ?? "all"}`;

  return (
    <section aria-labelledby="execution-title">
      <div className="mb-8 flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight" id="execution-title">
            Exécution
          </h1>
        </div>
        <Button onClick={() => navigate("/layouts")} variant="secondary">
          <ArrowLeft aria-hidden="true" className="size-4" />
          Retour aux layouts
        </Button>
      </div>

      {!layoutId ? (
        <Card>
          <CardContent className="p-6 text-center">
            <p>Layout introuvable.</p>
            <Button className="mt-4" onClick={() => navigate("/layouts")} variant="secondary">
              Retour aux layouts
            </Button>
          </CardContent>
        </Card>
      ) : (
        <ExecutionRunner key={runnerKey} layoutId={layoutId} retryIds={retryIds} />
      )}
    </section>
  );
}

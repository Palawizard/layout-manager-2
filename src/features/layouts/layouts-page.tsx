import { LayoutDashboard } from "lucide-react";

import { Card, CardContent } from "../../components/ui/card";

export function LayoutsPage() {
  return (
    <section aria-labelledby="layouts-title">
      <div className="mb-8">
        <h1 className="text-2xl font-semibold tracking-tight" id="layouts-title">
          Layouts
        </h1>
        <p className="mt-2 text-sm text-muted-foreground">
          Organisez vos applications et vos fenêtres selon vos besoins.
        </p>
      </div>
      <Card>
        <CardContent className="flex min-h-64 flex-col items-center justify-center text-center">
          <span className="mb-4 flex size-12 items-center justify-center rounded-lg bg-muted">
            <LayoutDashboard aria-hidden="true" className="size-6 text-muted-foreground" />
          </span>
          <h2 className="font-medium">Aucun layout</h2>
          <p className="mt-2 max-w-sm text-sm text-muted-foreground">
            Créez un layout pour organiser votre espace de travail.
          </p>
        </CardContent>
      </Card>
    </section>
  );
}

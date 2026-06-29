import type { LayoutSummary } from "../types/layout";
import { Button } from "../../../components/ui/button";
import { Card, CardContent } from "../../../components/ui/card";
import { formatDistanceToNow } from "../../../lib/utils/date";

interface LayoutListProps {
  layouts: LayoutSummary[];
  onEdit: (layoutId: string) => void;
}

export function LayoutList({ layouts, onEdit }: LayoutListProps) {
  return (
    <ul className="grid gap-4">
      {layouts.map((layout) => (
        <li key={layout.id}>
          <Card>
            <CardContent className="flex items-start justify-between gap-4 p-5">
              <div className="min-w-0">
                <h2 className="truncate font-medium">{layout.name}</h2>
                {layout.description ? (
                  <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">
                    {layout.description}
                  </p>
                ) : null}
                <p className="mt-3 text-xs text-muted-foreground">
                  {layout.actionCount} action{layout.actionCount > 1 ? "s" : ""} · Modifié{" "}
                  {formatDistanceToNow(layout.updatedAt)}
                </p>
              </div>
              <Button onClick={() => onEdit(layout.id)} size="small" variant="secondary">
                Modifier
              </Button>
            </CardContent>
          </Card>
        </li>
      ))}
    </ul>
  );
}

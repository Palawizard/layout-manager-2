import type { LayoutSummary } from "../types/layout";
import { Button } from "../../../components/ui/button";
import { Card, CardContent } from "../../../components/ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "../../../components/ui/dropdown-menu";
import { formatDistanceToNow } from "../../../lib/utils/date";
import { MoreHorizontal } from "lucide-react";

interface LayoutListProps {
  layouts: LayoutSummary[];
  onEdit: (layoutId: string) => void;
  onDuplicate: (layoutId: string) => void;
  onDelete: (layoutId: string) => void;
}

export function LayoutList({ layouts, onDelete, onDuplicate, onEdit }: LayoutListProps) {
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
              <div className="flex shrink-0 items-center gap-2">
                <Button onClick={() => onEdit(layout.id)} size="small" variant="secondary">
                  Modifier
                </Button>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button aria-label={`Actions pour ${layout.name}`} size="icon" variant="ghost">
                      <MoreHorizontal aria-hidden="true" className="size-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem onClick={() => onDuplicate(layout.id)}>
                      Dupliquer
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => onDelete(layout.id)}>
                      Supprimer
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </CardContent>
          </Card>
        </li>
      ))}
    </ul>
  );
}

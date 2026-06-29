import * as DialogPrimitive from "@radix-ui/react-dialog";
import type { ComponentProps } from "react";

import { cn } from "../../lib/utils/cn";

export function Dialog(props: ComponentProps<typeof DialogPrimitive.Root>) {
  return <DialogPrimitive.Root {...props} />;
}

export function DialogTrigger(props: ComponentProps<typeof DialogPrimitive.Trigger>) {
  return <DialogPrimitive.Trigger {...props} />;
}

export function DialogClose(props: ComponentProps<typeof DialogPrimitive.Close>) {
  return <DialogPrimitive.Close {...props} />;
}

interface DialogContentProps extends ComponentProps<typeof DialogPrimitive.Content> {
  dismissOnEscape?: boolean;
  dismissOnOutsideClick?: boolean;
}

export function DialogContent({
  className,
  children,
  dismissOnEscape = true,
  dismissOnOutsideClick = true,
  onEscapeKeyDown,
  onInteractOutside,
  onPointerDownOutside,
  ...props
}: DialogContentProps) {
  return (
    <DialogPrimitive.Portal>
      <DialogPrimitive.Overlay className="fixed inset-0 z-50 bg-black/50" />
      <DialogPrimitive.Content
        className={cn(
          "fixed left-1/2 top-1/2 z-50 w-[min(32rem,calc(100%-2rem))] -translate-x-1/2 -translate-y-1/2 rounded-lg border border-border bg-surface p-6 text-surface-foreground shadow-xl",
          className,
        )}
        onEscapeKeyDown={(event) => {
          if (!dismissOnEscape) {
            event.preventDefault();
          }
          onEscapeKeyDown?.(event);
        }}
        onInteractOutside={(event) => {
          if (!dismissOnOutsideClick) {
            event.preventDefault();
          }
          onInteractOutside?.(event);
        }}
        onPointerDownOutside={(event) => {
          if (!dismissOnOutsideClick) {
            event.preventDefault();
          }
          onPointerDownOutside?.(event);
        }}
        {...props}
      >
        {children}
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  );
}

export function DialogTitle({ className, ...props }: ComponentProps<typeof DialogPrimitive.Title>) {
  return <DialogPrimitive.Title className={cn("text-lg font-semibold", className)} {...props} />;
}

export function DialogDescription({
  className,
  ...props
}: ComponentProps<typeof DialogPrimitive.Description>) {
  return (
    <DialogPrimitive.Description
      className={cn("mt-2 text-sm text-muted-foreground", className)}
      {...props}
    />
  );
}

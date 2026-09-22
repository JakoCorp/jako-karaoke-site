import { AlertDialog } from "@base-ui/react";

interface ConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  title?: string;
  description?: string;
  confirmLabel?: string;
  variant?: "primary" | "danger";
  isPending?: boolean;
  error?: string | null;
}

export function ConfirmDialog({
  open,
  onOpenChange,
  onConfirm,
  title = "Are you sure?",
  description,
  confirmLabel = "Yes",
  variant = "primary",
  isPending = false,
  error,
}: ConfirmDialogProps) {
  const confirmClass = variant === "danger" ? "btn btn-danger" : "btn btn-primary";

  return (
    <AlertDialog.Root
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen && !isPending) onOpenChange(false);
        else if (nextOpen) onOpenChange(true);
      }}
    >
      <AlertDialog.Portal>
        <AlertDialog.Backdrop className="dialog-backdrop" />
        <AlertDialog.Popup className="dialog-popup">
          <AlertDialog.Title className="dialog-title">{title}</AlertDialog.Title>
          {description && (
            <AlertDialog.Description className="dialog-description">
              {description}
            </AlertDialog.Description>
          )}
          {error != null && <p className="form-error">{error}</p>}
          <div className="dialog-actions">
            <AlertDialog.Close className="btn btn-secondary" disabled={isPending}>
              Cancel
            </AlertDialog.Close>
            <button type="button" className={confirmClass} onClick={onConfirm} disabled={isPending}>
              {isPending ? "Deleting…" : confirmLabel}
            </button>
          </div>
        </AlertDialog.Popup>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}

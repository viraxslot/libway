import { useEffect, useRef } from "react";
import Button from "@/components/ui/Button/Button";

interface Props {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

/**
 * A small modal asking the user to confirm a destructive action.
 * Uses the native <dialog> element so Escape, focus trapping and the backdrop
 * are handled by the platform.
 */
export default function ConfirmDialog({
  title,
  message,
  confirmLabel = "Delete",
  cancelLabel = "Cancel",
  onConfirm,
  onCancel,
}: Props) {
  const ref = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    ref.current?.showModal();
  }, []);

  return (
    <dialog ref={ref} className="modal" onCancel={onCancel} onClose={onCancel}>
      <h3>{title}</h3>
      <p>{message}</p>
      <div className="modal-actions">
        <Button variant="secondary" type="button" onClick={onCancel}>
          {cancelLabel}
        </Button>
        <Button variant="danger" type="button" onClick={onConfirm}>
          {confirmLabel}
        </Button>
      </div>
    </dialog>
  );
}

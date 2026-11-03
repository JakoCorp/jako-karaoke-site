import { Dialog } from "@base-ui/react";
import { useId, useState } from "react";

const MAX_TITLE_LENGTH = 256;

export type PlaylistFormValues = {
  title: string;
  description: string;
  isPublic: boolean;
};

type FormProps = {
  initialValues: PlaylistFormValues;
  submitLabel: string;
  pendingLabel: string;
  isPending: boolean;
  error: string | null;
  onSubmit: (values: PlaylistFormValues) => void;
  onCancel: () => void;
};

function PlaylistForm({
  initialValues,
  submitLabel,
  pendingLabel,
  isPending,
  error,
  onSubmit,
  onCancel,
}: FormProps) {
  const [title, setTitle] = useState(initialValues.title);
  const [description, setDescription] = useState(initialValues.description);
  const [isPublic, setIsPublic] = useState(initialValues.isPublic);
  const titleId = useId();
  const descriptionId = useId();

  const values: PlaylistFormValues = {
    title: title.trim(),
    description: description.trim(),
    isPublic,
  };
  const isUnchanged =
    values.title === initialValues.title &&
    values.description === initialValues.description &&
    values.isPublic === initialValues.isPublic;

  return (
    <form
      className="dialog-form"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit(values);
      }}
    >
      <div className="form-field">
        <label className="form-label" htmlFor={titleId}>
          Title
        </label>
        <input
          id={titleId}
          className="form-input"
          type="text"
          value={title}
          maxLength={MAX_TITLE_LENGTH}
          onChange={(event) => setTitle(event.target.value)}
          autoComplete="off"
        />
      </div>
      <div className="form-field">
        <label className="form-label" htmlFor={descriptionId}>
          Description
        </label>
        <textarea
          id={descriptionId}
          className="form-input"
          rows={3}
          value={description}
          onChange={(event) => setDescription(event.target.value)}
        />
      </div>
      <label className="form-checkbox">
        <input
          type="checkbox"
          checked={isPublic}
          onChange={(event) => setIsPublic(event.target.checked)}
        />
        Make public
      </label>
      <p className="form-hint">Anyone with the link can view a public playlist.</p>
      {error !== null && <p className="form-error">{error}</p>}
      <div className="dialog-actions">
        <button type="button" className="btn btn-secondary" onClick={onCancel} disabled={isPending}>
          Cancel
        </button>
        <button
          type="submit"
          className="btn btn-primary"
          disabled={!values.title || isUnchanged || isPending}
        >
          {isPending ? pendingLabel : submitLabel}
        </button>
      </div>
    </form>
  );
}

type Props = Omit<FormProps, "onCancel"> & {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  heading: string;
};

/**
 * Dialog with the title, description and visibility fields of a playlist.
 *
 * The fields start from `initialValues` each time the dialog opens.
 */
export function PlaylistFormDialog({ open, onOpenChange, heading, ...form }: Props) {
  return (
    <Dialog.Root
      open={open}
      onOpenChange={(nextOpen) => {
        if (nextOpen || !form.isPending) onOpenChange(nextOpen);
      }}
    >
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup">
          <Dialog.Title className="dialog-title">{heading}</Dialog.Title>
          <PlaylistForm {...form} onCancel={() => onOpenChange(false)} />
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

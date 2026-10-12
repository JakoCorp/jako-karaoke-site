import { Toast } from "@base-ui/react";
import {
  CheckCircleIcon,
  InfoIcon,
  SpinnerGapIcon,
  WarningCircleIcon,
  WarningIcon,
  XIcon,
} from "@phosphor-icons/react";

function ToastIcon({ type }: { type: string | undefined }) {
  switch (type) {
    case "success": {
      return <CheckCircleIcon size={20} weight="fill" />;
    }
    case "error": {
      return <WarningCircleIcon size={20} weight="fill" />;
    }
    case "warning": {
      return <WarningIcon size={20} weight="fill" />;
    }
    case "loading": {
      return <SpinnerGapIcon size={20} className="animate-spin" />;
    }
    default: {
      return <InfoIcon size={20} weight="fill" />;
    }
  }
}

function ToastList() {
  const { toasts } = Toast.useToastManager();

  return (
    <>
      {toasts.map((toast) => (
        <Toast.Root
          key={toast.id}
          toast={toast}
          swipeDirection={["left", "right"]}
          className="toast"
        >
          <Toast.Content className="toast-content">
            <span className="toast-icon">
              <ToastIcon type={toast.type} />
            </span>
            <div className="toast-body">
              <Toast.Title className="toast-title" />
              <Toast.Description className="toast-description" />
            </div>
            <Toast.Close className="toast-close" aria-label="Dismiss notification">
              <XIcon size={14} />
            </Toast.Close>
          </Toast.Content>
        </Toast.Root>
      ))}
    </>
  );
}

/** Renders the toast viewport. Must be inside a `Toast.Provider`. */
export function Toaster() {
  return (
    <Toast.Portal>
      <Toast.Viewport className="toast-viewport">
        <ToastList />
      </Toast.Viewport>
    </Toast.Portal>
  );
}

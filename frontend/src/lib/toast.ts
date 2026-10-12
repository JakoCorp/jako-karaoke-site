import { Toast } from "@base-ui/react";

const ERROR_TIMEOUT_MS = 8000;

/** Global toast manager. Passed to `Toast.Provider` so non-React code can show toasts. */
export const toastManager = Toast.createToastManager();

interface ToastOptions {
  /** Reuse an id to update an existing toast in place instead of stacking another. */
  id?: string;
  description?: string;
  timeout?: number;
}

interface ToastContent {
  title: string;
  description?: string;
}

interface TaskResult extends ToastContent {
  /** Defaults to `success`. */
  type?: "success" | "warning" | "info";
}

interface TaskOptions<Value> {
  loading: string;
  success: (value: Value) => TaskResult;
  error: (error: unknown) => ToastContent;
}

/** Shows toast notifications for action outcomes. */
export const toast = {
  success: (title: string, options: ToastOptions = {}) =>
    toastManager.add({ type: "success", title, ...options }),
  info: (title: string, options: ToastOptions = {}) =>
    toastManager.add({ type: "info", title, ...options }),
  warning: (title: string, options: ToastOptions = {}) =>
    toastManager.add({ type: "warning", title, ...options }),
  error: (title: string, options: ToastOptions = {}) =>
    toastManager.add({
      type: "error",
      title,
      priority: "high",
      timeout: ERROR_TIMEOUT_MS,
      ...options,
    }),

  /**
   * Shows one toast that moves from loading to its outcome as the promise settles.
   *
   * A rejection is reported through the toast and is not rethrown.
   */
  task: async <Value>(promise: Promise<Value>, options: TaskOptions<Value>): Promise<void> => {
    const id = toastManager.add({ type: "loading", title: options.loading });
    try {
      const { type = "success", ...content } = options.success(await promise);
      toastManager.update(id, { ...content, type, timeout: undefined });
    } catch (error) {
      toastManager.update(id, {
        ...options.error(error),
        type: "error",
        priority: "high",
        timeout: ERROR_TIMEOUT_MS,
      });
    }
  },
};

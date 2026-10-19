import { ensureOk } from "@/api/client";

/**
 * Sequentially applies a function to each item, throwing on the first error encountered.
 * Used for deferred upload/delete operations that run as part of a save mutation.
 */
export async function applyAll<T>(
  items: Iterable<T>,
  apply: (item: T) => Promise<{ error?: unknown; response: Response }>,
): Promise<void> {
  for (const item of items) {
    ensureOk(await apply(item));
  }
}

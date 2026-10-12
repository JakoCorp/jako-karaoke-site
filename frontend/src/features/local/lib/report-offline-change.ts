import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";

interface OfflineChangeMessages {
  success: string;
  failure: string;
}

/** Reports the outcome of a quick offline storage change as a toast. */
export async function reportOfflineChange(
  task: Promise<void>,
  messages: OfflineChangeMessages,
): Promise<void> {
  try {
    await task;
    toast.success(messages.success);
  } catch (error) {
    toast.error(messages.failure, {
      description: errorMessage(error, "Something went wrong."),
    });
  }
}

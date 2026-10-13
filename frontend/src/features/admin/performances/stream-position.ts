const MAX_STREAM_NUMBER = 255;
const MAX_PERFORMANCE_NUMBER = 65535;

type StreamPositionResult =
  | { ok: true; stream_number: number | null; performance_number: number | null }
  | { ok: false; message: string };

function isInRange(value: number, max: number): boolean {
  return Number.isInteger(value) && value >= 1 && value <= max;
}

/**
 * Validates the stream number and performance number form inputs.
 *
 * Both blank means the performance did not happen on a stream. Setting only one is invalid.
 */
export function parseStreamPosition(stream: string, performance: string): StreamPositionResult {
  const isStreamBlank = stream.trim() === "";
  const isPerformanceBlank = performance.trim() === "";

  if (isStreamBlank && isPerformanceBlank) {
    return { ok: true, stream_number: null, performance_number: null };
  }
  if (isStreamBlank !== isPerformanceBlank) {
    return {
      ok: false,
      message: "Set both stream number and performance number, or leave both blank.",
    };
  }

  const streamNumber = Number(stream);
  const performanceNumber = Number(performance);
  if (
    !isInRange(streamNumber, MAX_STREAM_NUMBER) ||
    !isInRange(performanceNumber, MAX_PERFORMANCE_NUMBER)
  ) {
    return {
      ok: false,
      message: `Stream number must be a whole number from 1 to ${String(MAX_STREAM_NUMBER)} and performance number from 1 to ${String(MAX_PERFORMANCE_NUMBER)}.`,
    };
  }

  return { ok: true, stream_number: streamNumber, performance_number: performanceNumber };
}

const MAX_STREAM_NUMBER = 255;
const MAX_POSITION_NUMBER = 65535;

type StreamPositionResult =
  | { ok: true; stream_number: number | null; position_number: number | null }
  | { ok: false; message: string };

function isInRange(value: number, max: number): boolean {
  return Number.isInteger(value) && value >= 1 && value <= max;
}

/**
 * Validates the stream number and position number form inputs.
 *
 * Both blank means the item did not happen on a stream. Setting only one is invalid.
 * `positionLabel` names the position number in error messages, for example "performance number".
 */
export function parseStreamPosition(
  stream: string,
  position: string,
  positionLabel: string,
): StreamPositionResult {
  const isStreamBlank = stream.trim() === "";
  const isPositionBlank = position.trim() === "";

  if (isStreamBlank && isPositionBlank) {
    return { ok: true, stream_number: null, position_number: null };
  }
  if (isStreamBlank !== isPositionBlank) {
    return {
      ok: false,
      message: `Set both stream number and ${positionLabel}, or leave both blank.`,
    };
  }

  const streamNumber = Number(stream);
  const positionNumber = Number(position);
  if (
    !isInRange(streamNumber, MAX_STREAM_NUMBER) ||
    !isInRange(positionNumber, MAX_POSITION_NUMBER)
  ) {
    return {
      ok: false,
      message: `Stream number must be a whole number from 1 to ${String(MAX_STREAM_NUMBER)} and ${positionLabel} from 1 to ${String(MAX_POSITION_NUMBER)}.`,
    };
  }

  return { ok: true, stream_number: streamNumber, position_number: positionNumber };
}

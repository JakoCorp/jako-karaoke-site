/**
 * Serializes a value to URL-safe base64 (RFC 4648 §5), substituting `+` and `/` so
 * the result can appear in a query string without percent-encoding.
 */
function encodeRaw(obj: unknown): string {
  const bytes = new TextEncoder().encode(JSON.stringify(obj));
  const binary = Array.from(bytes, (byte) => String.fromCharCode(byte)).join("");
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

/**
 * Deserializes a URL-safe base64 string produced by `encodeRaw`. Returns `{}` on any
 * parse failure so a corrupted or manually edited URL degrades gracefully.
 */
function decodeRaw(encoded: string): unknown {
  if (!encoded) return {};
  try {
    const padded = encoded.replace(/-/g, "+").replace(/_/g, "/");
    const remainder = padded.length % 4;
    const normalized = remainder > 0 ? padded + "=".repeat(4 - remainder) : padded;
    const bytes = Uint8Array.from(atob(normalized), (char) => char.charCodeAt(0));
    return JSON.parse(new TextDecoder().decode(bytes));
  } catch {
    return {};
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/**
 * Creates encode/decode/toParam helpers for a URL query string state object of type T.
 *
 * - `compact` strips fields that match their defaults to keep the encoded blob short.
 * - `validate` checks individual fields so unexpected values degrade to defaults.
 */
export function createStateCodec<T extends object>(opts: {
  compact: (state: T) => Partial<T>;
  validate: (parsed: Record<string, unknown>) => T;
}) {
  return {
    encode: (state: T): string => encodeRaw(state),
    decode: (encoded: string): T => {
      const raw = decodeRaw(encoded);
      return opts.validate(isRecord(raw) ? raw : {});
    },
    toParam: (state: T): string | null => {
      const compacted = opts.compact(state);
      return Object.keys(compacted).length > 0 ? encodeRaw(compacted) : null;
    },
  };
}

type MultipartField = File | string | null | undefined;

/**
 * Creates a `bodySerializer` that sends the given fields as `multipart/form-data`.
 *
 * Fields that are `null`, `undefined` or an empty string are omitted.
 */
export function multipartSerializer(fields: Record<string, MultipartField>): () => FormData {
  return () => {
    const form = new FormData();
    for (const [name, value] of Object.entries(fields)) {
      if (value) form.append(name, value);
    }
    return form;
  };
}

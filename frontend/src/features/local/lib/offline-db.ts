const DATABASE_NAME = "offline-audio";
const STORE_NAME = "audio";

function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DATABASE_NAME, 1);
    request.addEventListener("upgradeneeded", () => {
      request.result.createObjectStore(STORE_NAME);
    });
    request.addEventListener("success", () => {
      resolve(request.result);
    });
    request.addEventListener("error", () => {
      reject(request.error ?? new Error("Failed to open offline database."));
    });
  });
}

async function run<T>(
  mode: IDBTransactionMode,
  operation: (store: IDBObjectStore) => IDBRequest<T>,
): Promise<T> {
  const database = await openDatabase();
  return new Promise<T>((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, mode);
    const request = operation(transaction.objectStore(STORE_NAME));
    transaction.addEventListener("complete", () => {
      database.close();
      resolve(request.result);
    });
    const fail = () => {
      database.close();
      reject(transaction.error ?? new Error("Offline database transaction failed."));
    };
    transaction.addEventListener("error", fail);
    transaction.addEventListener("abort", fail);
  });
}

/** Stores the audio blob for a performance, replacing any existing copy. */
export async function putAudioBlob(performanceId: string, blob: Blob): Promise<void> {
  await run("readwrite", (store) => store.put(blob, performanceId));
}

/** Returns the stored audio blob for a performance, or `null` when none is saved. */
export async function getAudioBlob(performanceId: string): Promise<Blob | null> {
  const blob = await run<Blob | undefined>("readonly", (store) => store.get(performanceId));
  return blob ?? null;
}

/** Removes the stored audio blob for a performance. */
export async function deleteAudioBlob(performanceId: string): Promise<void> {
  await run("readwrite", (store) => store.delete(performanceId));
}

/** Removes every stored audio blob. */
export async function clearAudioBlobs(): Promise<void> {
  await run("readwrite", (store) => store.clear());
}

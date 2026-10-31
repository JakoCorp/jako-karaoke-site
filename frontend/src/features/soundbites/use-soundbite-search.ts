import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router";

import { useDebounced } from "@/hooks/use-debounced";
import { createStateCodec } from "@/lib/url-state";

export const PER_PAGE_OPTIONS = [20, 50, 100] as const;
export type PerPage = (typeof PER_PAGE_OPTIONS)[number];

const DEFAULT_PER_PAGE: PerPage = 20;

interface SoundbiteState {
  q?: string;
  page?: number;
  per_page?: PerPage;
}

const soundbiteStateCodec = createStateCodec<SoundbiteState>({
  compact: (state) => {
    const compact: SoundbiteState = {};
    if (state.q) compact.q = state.q;
    if (state.page && state.page > 1) compact.page = state.page;
    if (state.per_page && state.per_page !== DEFAULT_PER_PAGE) compact.per_page = state.per_page;
    return compact;
  },
  validate: (parsed) => {
    const state: SoundbiteState = {};
    if (typeof parsed.q === "string" && parsed.q) state.q = parsed.q;
    if (typeof parsed.page === "number" && parsed.page >= 1) state.page = Math.floor(parsed.page);
    if (parsed.per_page === 50 || parsed.per_page === 100) state.per_page = parsed.per_page;
    return state;
  },
});

function withState(prev: URLSearchParams, partial: Partial<SoundbiteState>): URLSearchParams {
  const current = soundbiteStateCodec.decode(prev.get("query") ?? "");
  const next = soundbiteStateCodec.toParam({ ...current, ...partial });
  const params = new URLSearchParams(prev);
  if (next) {
    params.set("query", next);
  } else {
    params.delete("query");
  }
  return params;
}

/**
 * Search text and paging of the soundbites page, kept in the encoded `query` search param.
 *
 * `params` can be passed straight to the soundbites query.
 */
export function useSoundbiteSearch() {
  const [searchParams, setSearchParams] = useSearchParams();
  const state = soundbiteStateCodec.decode(searchParams.get("query") ?? "");

  const q = state.q ?? "";
  const page = state.page ?? 1;
  const perPage = state.per_page ?? DEFAULT_PER_PAGE;

  const [inputValue, setInputValue] = useState(q);
  const debouncedInput = useDebounced(inputValue);
  const lastSyncedQ = useRef(q);

  useEffect(() => {
    if (q === lastSyncedQ.current) return;
    lastSyncedQ.current = q;
    setInputValue(q);
  }, [q]);

  const debouncedQ = debouncedInput.trim();

  useEffect(() => {
    if (debouncedQ === lastSyncedQ.current) return;
    lastSyncedQ.current = debouncedQ;
    setSearchParams((prev) => withState(prev, { q: debouncedQ || undefined, page: undefined }), {
      replace: true,
    });
  }, [debouncedQ, setSearchParams]);

  function setPage(nextPage: number) {
    setSearchParams((prev) => withState(prev, { page: nextPage }));
  }

  function setPerPage(nextPerPage: PerPage) {
    setSearchParams((prev) => withState(prev, { per_page: nextPerPage, page: undefined }));
  }

  return {
    inputValue,
    setInputValue,
    q,
    page,
    perPage,
    setPage,
    setPerPage,
    params: { q: q || undefined, page, per_page: perPage },
  };
}

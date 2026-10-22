import { type RefObject, useEffect, useRef } from "react";

interface InfiniteSentinelOptions {
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  fetchNextPage: () => unknown;
  /** Scroll container the sentinel is observed in. Defaults to the viewport. */
  scrollContainerRef?: RefObject<Element | null>;
}

/** Returns a ref for an element that loads the next page whenever it scrolls near the viewport. */
export function useInfiniteSentinel({
  hasNextPage,
  isFetchingNextPage,
  fetchNextPage,
  scrollContainerRef,
}: InfiniteSentinelOptions) {
  const sentinelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting && hasNextPage && !isFetchingNextPage) {
          void fetchNextPage();
        }
      },
      { root: scrollContainerRef?.current, rootMargin: "200px" },
    );
    if (sentinelRef.current) observer.observe(sentinelRef.current);
    return () => {
      observer.disconnect();
    };
  }, [hasNextPage, isFetchingNextPage, fetchNextPage, scrollContainerRef]);

  return sentinelRef;
}

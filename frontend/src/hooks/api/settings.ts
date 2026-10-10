import { queryOptions, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { settingsApi } from "@/api/settings";
import type { UserSettings } from "@/api/settings";

export const settingsKeys = {
  all: () => ["settings"] as const,
};

/** Query options for the authenticated user's settings. */
export function userSettingsQueryOptions() {
  return queryOptions({
    queryKey: settingsKeys.all(),
    queryFn: async (): Promise<UserSettings> => {
      const { data, error } = await settingsApi.get();
      if (error || !data) throw error ?? new Error("Failed to load settings.");
      return data;
    },
  });
}

/** Loads the authenticated user's settings. Pass `enabled = false` when signed out. */
export function useUserSettings(enabled = true) {
  return useQuery({ ...userSettingsQueryOptions(), enabled });
}

/** Replaces the authenticated user's settings and refreshes the cached copy. */
export function useUpdateUserSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (settings: UserSettings) => {
      const { data, error } = await settingsApi.update(settings);
      if (error || !data) throw error ?? new Error("Failed to save settings.");
      return data;
    },
    onSuccess: (data) => {
      queryClient.setQueryData(settingsKeys.all(), data);
    },
  });
}

import { CameraIcon, UserIcon } from "@phosphor-icons/react";
import { useMutation } from "@tanstack/react-query";
import { useRef } from "react";

import { authApi, toAuthUser } from "@/api/auth";
import type { AuthUser } from "@/api/auth";
import { useAuthStore } from "@/store/auth";

const MAX_AVATAR_BYTES = 5 * 1024 * 1024;

interface Props {
  user: AuthUser;
  onError: (message: string | null) => void;
}

export function AvatarPicker({ user, onError }: Props) {
  const setUser = useAuthStore((state) => state.setUser);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const upload = useMutation({
    mutationFn: async (file: File) => {
      const { data, error } = await authApi.uploadAvatar(file);
      if (!data) throw error;
      return data;
    },
    onSuccess: (data) => {
      setUser(toAuthUser(data));
    },
    onError: () => {
      onError("Could not upload that image. Use a PNG, JPEG, WebP, AVIF or GIF under 5 MB.");
    },
  });

  function handleFileChange(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;

    if (!file.type.startsWith("image/")) {
      onError("Please choose an image file.");
      return;
    }
    if (file.size > MAX_AVATAR_BYTES) {
      onError("That image is larger than 5 MB.");
      return;
    }
    onError(null);
    upload.mutate(file);
  }

  return (
    <>
      <button
        type="button"
        className="settings-avatar-btn"
        data-tooltip="Change avatar (image, max 5 MB)"
        aria-label="Change avatar"
        disabled={upload.isPending}
        onClick={() => fileInputRef.current?.click()}
      >
        {user.avatarUrl ? (
          <img src={user.avatarUrl} alt="" className="block avatar-lg avatar" />
        ) : (
          <span className="avatar-placeholder avatar-lg avatar" aria-hidden="true">
            <UserIcon size={40} weight="fill" />
          </span>
        )}
        <span className="settings-avatar-overlay" aria-hidden="true">
          <CameraIcon size={24} weight="fill" />
        </span>
      </button>
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        className="absolute size-0 overflow-hidden opacity-0"
        onChange={handleFileChange}
      />
    </>
  );
}

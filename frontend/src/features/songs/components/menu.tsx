import { InfoIcon } from "@phosphor-icons/react";
import { useNavigate } from "react-router";

import { RowContextMenu } from "@/components/row-context-menu";
import type { RowMenuItemDef } from "@/components/row-context-menu";

interface Props {
  songId: string;
  children: React.ReactNode;
}

export function SongRowMenu({ songId, children }: Props) {
  const navigate = useNavigate();
  const items: RowMenuItemDef[] = [
    {
      icon: <InfoIcon size={14} />,
      label: "View details",
      onClick: () => {
        void navigate(`/song/${songId}`);
      },
    },
  ];

  return (
    <RowContextMenu items={items} triggerClassName="song-row" menuBtnAriaLabel="Song options">
      {children}
    </RowContextMenu>
  );
}

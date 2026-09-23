import { Popover } from "@base-ui/react";

import { PlaylistPickerContent } from "./picker-content";

type Props = {
  performanceId: string;
  children: React.ReactElement;
};

export function PlaylistPickerPopover({ performanceId, children }: Props) {
  return (
    <Popover.Root>
      <Popover.Trigger render={children} />
      <Popover.Portal>
        <Popover.Positioner side="bottom" align="start" sideOffset={6}>
          <Popover.Popup className="playlist-picker-popup">
            <PlaylistPickerContent performanceId={performanceId} />
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  );
}

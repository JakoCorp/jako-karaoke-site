import { ContextMenu, Menu } from "@base-ui/react";
import { CaretRightIcon, DotsThreeVerticalIcon } from "@phosphor-icons/react";
import { useState } from "react";
import type { ReactNode } from "react";

import { ConfirmDialog } from "./confirm-dialog";

type MenuActionItem = {
  type?: "action";
  /** Icon rendered before the label. */
  icon: ReactNode;
  label: string;
  onClick: () => void;
  variant?: "danger";
  /** When set, clicking the item opens a confirm dialog before calling onClick. */
  confirm?: { title: string; description?: string; confirmLabel?: string };
};

type MenuSubmenuItem = {
  type: "submenu";
  /** Icon rendered before the label. */
  icon: ReactNode;
  label: string;
  /** Content rendered inside the submenu popup. */
  content: ReactNode;
};

/** A single item definition for RowContextMenu. */
export type RowMenuItemDef = MenuActionItem | MenuSubmenuItem;

interface Props {
  items: RowMenuItemDef[];
  /** CSS class applied to the ContextMenu.Trigger element (e.g. "perf-row"). */
  triggerClassName: string;
  menuBtnAriaLabel?: string;
  children: ReactNode;
  onTriggerClick?: () => void;
}

/**
 * Wraps children in a dual-surface menu: right-click opens a ContextMenu and
 * an inline triple-dot button opens a Menu. Both surfaces render the same items
 * from a single declarative definition.
 */
export function RowContextMenu({
  items,
  triggerClassName,
  menuBtnAriaLabel = "Options",
  children,
  onTriggerClick,
}: Props) {
  const [confirmItem, setConfirmItem] = useState<MenuActionItem | null>(null);

  function renderItems(surface: "menu" | "context") {
    return items.map((item, idx) => {
      if (item.type === "submenu") {
        return (
          <Menu.SubmenuRoot key={idx}>
            <Menu.SubmenuTrigger className="card-menu-item">
              {item.icon}
              {item.label}
              <CaretRightIcon size={12} style={{ marginLeft: "auto" }} />
            </Menu.SubmenuTrigger>
            <Menu.Portal>
              <Menu.Positioner side="right" align="start" sideOffset={4}>
                <Menu.Popup className="playlist-picker-popup">{item.content}</Menu.Popup>
              </Menu.Positioner>
            </Menu.Portal>
          </Menu.SubmenuRoot>
        );
      }

      const className =
        item.variant === "danger" ? "card-menu-item card-menu-item--danger" : "card-menu-item";
      const handleClick = item.confirm
        ? () => {
            setConfirmItem(item);
          }
        : item.onClick;

      if (surface === "context") {
        return (
          <ContextMenu.Item key={idx} className={className} onClick={handleClick}>
            {item.icon}
            {item.label}
          </ContextMenu.Item>
        );
      }
      return (
        <Menu.Item key={idx} className={className} onClick={handleClick}>
          {item.icon}
          {item.label}
        </Menu.Item>
      );
    });
  }

  return (
    <>
      <ContextMenu.Root>
        <ContextMenu.Trigger className={triggerClassName} onClick={onTriggerClick}>
          {children}
          <Menu.Root>
            <Menu.Trigger
              className="perf-row-menu-btn"
              aria-label={menuBtnAriaLabel}
              onClick={(e) => e.stopPropagation()}
            >
              <DotsThreeVerticalIcon size={16} />
            </Menu.Trigger>
            <Menu.Portal>
              <Menu.Positioner>
                <Menu.Popup className="card-menu-popup">{renderItems("menu")}</Menu.Popup>
              </Menu.Positioner>
            </Menu.Portal>
          </Menu.Root>
        </ContextMenu.Trigger>
        <ContextMenu.Portal>
          <ContextMenu.Positioner>
            <ContextMenu.Popup className="card-menu-popup">
              {renderItems("context")}
            </ContextMenu.Popup>
          </ContextMenu.Positioner>
        </ContextMenu.Portal>
      </ContextMenu.Root>
      <ConfirmDialog
        open={confirmItem !== null}
        onOpenChange={(open) => {
          if (!open) setConfirmItem(null);
        }}
        onConfirm={() => {
          confirmItem?.onClick();
          setConfirmItem(null);
        }}
        title={confirmItem?.confirm?.title ?? "Are you sure?"}
        description={confirmItem?.confirm?.description}
        confirmLabel={confirmItem?.confirm?.confirmLabel}
        variant="danger"
      />
    </>
  );
}

import { useState, type ComponentProps } from "react";

const DEFAULT_ROWS = 6;

type Props = Omit<ComponentProps<"textarea">, "rows"> & {
  /** Number of rows of space shown before the field is focused. Defaults to 6. */
  rows?: number;
};

function fitToContent(element: HTMLTextAreaElement) {
  element.style.height = "auto";
  const borderHeight = element.offsetHeight - element.clientHeight;
  element.style.height = `${String(element.scrollHeight + borderHeight)}px`;
}

/**
 * Textarea that shows a fixed number of rows until it is focused, then grows to fit its content.
 *
 * Once expanded it stays fitted to the content as the user types.
 */
export function ExpandingTextarea({ rows = DEFAULT_ROWS, onFocus, onChange, ...props }: Props) {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <textarea
      {...props}
      rows={rows}
      onFocus={(event) => {
        setIsExpanded(true);
        fitToContent(event.currentTarget);
        onFocus?.(event);
      }}
      onChange={(event) => {
        if (isExpanded) fitToContent(event.currentTarget);
        onChange?.(event);
      }}
    />
  );
}

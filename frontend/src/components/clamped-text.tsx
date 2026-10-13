import { useEffect, useRef, useState } from "react";

interface Props {
  text: string;
  className?: string;
}

/**
 * Multi line text clamped to a few lines.
 *
 * A toggle expands the full text and is only shown when the text is actually cut off.
 */
export function ClampedText({ text, className }: Props) {
  const textRef = useRef<HTMLParagraphElement>(null);
  const [isExpanded, setIsExpanded] = useState(false);
  const [isCutOff, setIsCutOff] = useState(false);

  useEffect(() => {
    const observer = new ResizeObserver((entries) => {
      const element = entries[0]?.target;
      if (element) setIsCutOff(element.scrollHeight > element.clientHeight);
    });
    if (textRef.current && !isExpanded) observer.observe(textRef.current);
    return () => {
      observer.disconnect();
    };
  }, [isExpanded]);

  return (
    <div className={className}>
      <p ref={textRef} className="clamped-text" data-expanded={isExpanded}>
        {text}
      </p>
      {(isCutOff || isExpanded) && (
        <button
          type="button"
          className="clamped-text-toggle"
          aria-expanded={isExpanded}
          onClick={() => {
            setIsExpanded((expanded) => !expanded);
          }}
        >
          {isExpanded ? "Show less" : "Read more"}
        </button>
      )}
    </div>
  );
}

import { useEffect, useEffectEvent } from "react";

const BUBBLE: DocumentKeyDownOptions = { capture: false };

export function useDocumentKeyDown(
  handler: (event: KeyboardEvent) => void,
  { capture }: DocumentKeyDownOptions = BUBBLE,
) {
  const onKeyDown = useEffectEvent(handler);

  useEffect(() => {
    const listener = (event: KeyboardEvent): void => {
      onKeyDown(event);
    };

    document.addEventListener("keydown", listener, { capture });

    return () => {
      document.removeEventListener("keydown", listener, { capture });
    };
  }, [capture]);
}

export interface DocumentKeyDownOptions {
  capture: boolean;
}

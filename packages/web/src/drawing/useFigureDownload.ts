import type { Figure, FileEntry } from "@neherlab/treeknit-wasm";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";

import type { AnalysisClient } from "../analysis/client";
import { useAnalysisClient } from "../analysis/context";
import { useSessionFiles } from "../analysis/queries";
import { downloadFile } from "../download";
import { downloadName } from "../results/fileRows";
import type { FigureButtonProps } from "./DrawingControls";
import type { DrawingFailure } from "./DrawingPanel";
import { figureFile } from "./figure";

const FIGURE_FAILED = "The figure could not be made.";

export interface FigureDownload {
  button: FigureButtonProps;
  failure: DrawingFailure | undefined;
}

export function useFigureDownload(
  sessionId: number,
  figure: Figure,
  render: (client: AnalysisClient, sessionId: number) => Promise<string>,
): FigureDownload {
  const client = useAnalysisClient();
  const files = useSessionFiles(sessionId);
  const file = figureFile(files, figure);

  const mutation = useMutation({
    mutationFn: async (entry: FileEntry) => ({ entry, text: await render(client, sessionId) }),
    onSuccess: ({ entry, text }) => {
      downloadFile({ name: downloadName(entry.path), mediaType: entry.mediaType, content: text });
    },
  });

  const { mutate, reset } = mutation;
  const entry = "entry" in file ? file.entry : undefined;

  const download = useCallback(() => {
    if (entry !== undefined) {
      mutate(entry);
    }
  }, [entry, mutate]);

  const button: FigureButtonProps =
    "entry" in file ? { onDownload: download, isPending: mutation.isPending } : { disabledReason: file.disabledReason };

  const failure =
    mutation.error === null ? undefined : { title: FIGURE_FAILED, message: mutation.error.message, onDismiss: reset };

  return { button, failure };
}

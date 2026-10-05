import type { Figure, FileEntry, FigureDownload as NamedFigure } from "@neherlab/treeknit-wasm";
import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";

import type { AnalysisClient } from "../analysis/client";
import { useAnalysisClient } from "../analysis/context";
import { useSessionFiles } from "../analysis/queries";
import { downloadFile } from "../download";
import type { FigureButtonProps } from "./DrawingControls";
import type { DrawingFailure } from "./DrawingPanel";
import { figureFile, figureMutation } from "./figure";

const FIGURE_FAILED = "The figure could not be made.";

export interface FigureDownload {
  button: FigureButtonProps;
  failure: DrawingFailure | undefined;
}

interface RenderedFigure {
  entry: FileEntry;
  figure: NamedFigure;
}

export function useFigureDownload(
  sessionId: number,
  figure: Figure,
  render: (client: AnalysisClient, sessionId: number) => Promise<NamedFigure>,
): FigureDownload {
  const client = useAnalysisClient();
  const files = useSessionFiles(sessionId);
  const file = figureFile(files, figure);

  const mutation = useMutation({
    mutationFn: async (entry: FileEntry): Promise<RenderedFigure> => ({
      entry,
      figure: await render(client, sessionId),
    }),
  });

  const { mutate, reset } = mutation;
  const entry = "entry" in file ? file.entry : undefined;

  const download = useCallback(() => {
    if (entry !== undefined) {
      mutate(entry, { onSuccess: saveFigure });
    }
  }, [entry, mutate]);

  const { isPending, error } = figureMutation(mutation, figure);

  const button: FigureButtonProps =
    "entry" in file ? { onDownload: download, isPending } : { disabledReason: file.disabledReason };

  const failure = error === null ? undefined : { title: FIGURE_FAILED, message: error.message, onDismiss: reset };

  return { button, failure };
}

function saveFigure({ entry, figure }: RenderedFigure): void {
  downloadFile({ name: figure.fileName, mediaType: entry.mediaType, content: figure.text });
}

import type { AnalysisRequest } from "@neherlab/treeknit-wasm";
import { useMutation } from "@tanstack/react-query";

import { useAnalysisClient } from "../analysis/context";
import { downloadFile } from "../download";

export function useSaveSessionFile() {
  const client = useAnalysisClient();

  return useMutation({
    mutationFn: async (request: AnalysisRequest) => client.stateless(async (api) => api.sessionFile(request)),
    onSuccess: ({ path, mediaType, text }) => {
      downloadFile({ name: path, mediaType, content: text });
    },
  });
}

import { useCallback, useMemo, useState } from "react";
import { type DropItem, isFileDropItem, isTextDropItem } from "react-aria-components";

import { useAnalysisClient } from "../analysis/context";
import type { Example } from "../analysis/example";
import { useWorkspaceStore } from "../workspace/context";
import type { NewTree } from "../workspace/store";
import { addFailure, batchRejection, isSessionFileName, readFailure, sessionFailure } from "./treeFiles";

const PLAIN_TEXT = "text/plain";

export interface TreeInput {
  addFiles(files: readonly File[]): Promise<void>;
  addPasted(newick: string, label: string): Promise<void>;
  addExample(example: Example): Promise<void>;
  addDropped(items: readonly DropItem[]): Promise<void>;
  openSessionFile(file: File): Promise<void>;
  replaceFile(treeId: string, file: File): Promise<void>;
}

export interface TreeInputState {
  input: TreeInput;
  error: string | null;
  dismissError: () => void;
}

export function useTreeInput(): TreeInputState {
  const store = useWorkspaceStore();
  const client = useAnalysisClient();
  const [error, setError] = useState<string | null>(null);

  const dismissError = useCallback(() => {
    setError(null);
  }, []);

  const input = useMemo((): TreeInput => {
    const addTrees = async (trees: readonly NewTree[]): Promise<void> => {
      if (trees.length === 0) {
        return;
      }

      try {
        await store.getState().addTrees(trees);
      } catch (cause) {
        setError(addFailure(cause));
      }
    };

    const readTrees = async (files: readonly File[]): Promise<NewTree[]> => {
      const read = await Promise.all(
        files.map(async (file) => {
          try {
            return { newick: await file.text(), source: { kind: "file", name: file.name } } satisfies NewTree;
          } catch (cause) {
            setError(readFailure(file.name, cause));

            return null;
          }
        }),
      );

      return read.filter((tree) => tree !== null);
    };

    const openSessionFile = async (file: File): Promise<void> => {
      try {
        const request = await client.readRequest(await file.text());

        store.getState().loadRequest(request);
        setError(null);
      } catch (cause) {
        setError(sessionFailure(file.name, cause));
      }
    };

    const addBatch = async (files: readonly File[], texts: readonly string[]): Promise<void> => {
      setError(null);

      const sessions = files.filter((file) => isSessionFileName(file.name));
      const treeFiles = files.filter((file) => !isSessionFileName(file.name));

      const rejection = batchRejection(
        sessions.map(({ name }) => name),
        treeFiles.length + texts.length,
      );

      if (rejection !== null) {
        setError(rejection);

        return;
      }

      const [session] = sessions;

      if (session !== undefined) {
        await openSessionFile(session);

        return;
      }

      const pasted = texts.map((newick): NewTree => ({ newick, source: { kind: "paste" } }));

      await addTrees([...(await readTrees(treeFiles)), ...pasted]);
    };

    const addFiles = async (files: readonly File[]): Promise<void> => addBatch(files, []);

    return {
      addFiles,
      openSessionFile,
      async addPasted(newick, label) {
        const trimmed = label.trim();

        await addTrees([{ newick, source: { kind: "paste" }, ...(trimmed === "" ? undefined : { label: trimmed }) }]);
      },
      async addExample(example) {
        setError(null);

        try {
          const trees = await example.load();

          await addTrees(
            trees.map(({ fileName, newick }) => ({ newick, source: { kind: "example", name: fileName } })),
          );
        } catch (cause) {
          setError(addFailure(cause));
        }
      },
      async addDropped(items) {
        try {
          const files = await Promise.all(items.filter(isFileDropItem).map(async (item) => item.getFile()));

          const texts = await Promise.all(
            items
              .filter(isTextDropItem)
              .filter((item) => item.types.has(PLAIN_TEXT))
              .map(async (item) => item.getText(PLAIN_TEXT)),
          );

          await addBatch(
            files,
            texts.filter((text) => text.trim() !== ""),
          );
        } catch (cause) {
          setError(addFailure(cause));
        }
      },
      async replaceFile(treeId, file) {
        const [tree] = await readTrees([file]);

        if (tree !== undefined) {
          store.getState().replaceTree(treeId, tree.newick, tree.source);
        }
      },
    };
  }, [client, store]);

  return { input, error, dismissError };
}

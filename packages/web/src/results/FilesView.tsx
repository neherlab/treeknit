import type { FileEntry } from "@neherlab/treeknit-wasm";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useCallback, useMemo } from "react";
import DownloadIcon from "~icons/lucide/download";
import ArchiveIcon from "~icons/lucide/file-archive";
import SaveIcon from "~icons/lucide/save";

import { useAnalysisClient } from "../analysis/context";
import { analysisKeys, useCommandLine, useSessionFiles } from "../analysis/queries";
import { downloadFile } from "../download";
import { useDelayedIndicator } from "../indicator/useDelayedIndicator";
import { Button } from "../ui/Button";
import { CodeBlock } from "../ui/CodeBlock";
import { IconButton } from "../ui/IconButton";
import { InfoButton } from "../ui/InfoButton";
import { InlineNotice } from "../ui/InlineNotice";
import { ProgressBar } from "../ui/ProgressBar";
import { Cell, Column, Row, Table, TableBody, TableHeader } from "../ui/Table";
import { useCurrentRequest, useWorkspace } from "../workspace/context";
import { downloadName, type FileRow, fileRow, RESULTS_ARCHIVE_NAME, ZIP_MEDIA_TYPE } from "./fileRows";

export function FilesView() {
  const sessionId = useWorkspace((state) => state.result?.sessionId);

  if (sessionId === undefined) {
    return null;
  }

  return <SessionFiles sessionId={sessionId} />;
}

function SessionFiles({ sessionId }: SessionFilesProps) {
  const client = useAnalysisClient();
  const queryClient = useQueryClient();
  const request = useCurrentRequest();
  const files = useSessionFiles(sessionId);
  const commandLine = useCommandLine(sessionId);
  const loading = useDelayedIndicator(files.isPending);

  const zip = useMutation({
    mutationFn: async () => client.zip(sessionId),
    onSuccess: (bytes) => {
      downloadFile({ name: RESULTS_ARCHIVE_NAME, mediaType: ZIP_MEDIA_TYPE, content: new Uint8Array(bytes) });
    },
  });

  const sessionFile = useMutation({
    mutationFn: async () => client.requestFile(request),
    onSuccess: ({ path, mediaType, text }) => {
      downloadFile({ name: downloadName(path), mediaType, content: text });
    },
  });

  const entry = useMutation({
    mutationFn: async ({ file, name }: FileRow) => ({ file, name, text: await client.fileText(sessionId, file.path) }),
    onSuccess: async ({ file, name, text }) => {
      downloadFile({ name, mediaType: file.mediaType, content: text });

      if (file.size === null) {
        await queryClient.invalidateQueries({ queryKey: analysisKeys.files(sessionId) });
      }
    },
  });

  const downloadAll = useCallback(() => {
    zip.mutate();
  }, [zip]);

  const saveSession = useCallback(() => {
    sessionFile.mutate();
  }, [sessionFile]);

  const failure = zip.error ?? sessionFile.error ?? entry.error;

  return (
    <div className="flex max-w-5xl flex-col gap-8 px-8 py-6">
      <div className="flex flex-wrap items-center gap-2">
        <Button variant="primary" icon={ArchiveIcon} isPending={zip.isPending} onPress={downloadAll}>
          Download all (ZIP)
        </Button>
        <Button icon={SaveIcon} isPending={sessionFile.isPending} onPress={saveSession}>
          Save session file
        </Button>
      </div>
      {failure === null ? null : (
        <InlineNotice tone="danger" title="The download failed.">
          {failure.message}
        </InlineNotice>
      )}
      <section aria-labelledby="files-list" className="flex flex-col gap-3">
        <h2 id="files-list" className="text-base font-semibold">
          Files
        </h2>
        {files.error === null ? null : (
          <InlineNotice tone="danger" title="The files could not be listed.">
            {files.error.message}
          </InlineNotice>
        )}
        {loading ? <ProgressBar label="Listing the files" labelHidden isIndeterminate className="max-w-xs" /> : null}
        {files.data === undefined ? null : <FileTable files={files.data} onDownload={entry.mutate} />}
      </section>
      {commandLine.data === undefined ? null : (
        <section aria-labelledby="files-command" className="flex flex-col gap-3">
          <div className="flex items-center gap-1">
            <h2 id="files-command" className="text-base font-semibold">
              Command line
            </h2>
            <InfoButton topic="the command line">
              <p>
                Extract {RESULTS_ARCHIVE_NAME} and run this command in the directory that holds the extracted
                treeknit_results directory. The command line then writes the same files.
              </p>
            </InfoButton>
          </div>
          <CodeBlock code={commandLine.data} label="Command" />
        </section>
      )}
    </div>
  );
}

interface SessionFilesProps {
  sessionId: number;
}

function FileTable({ files, onDownload }: FileTableProps) {
  const rows = useMemo(() => files.map(fileRow), [files]);
  const rowDependencies = useMemo(() => [onDownload], [onDownload]);

  return (
    <div className="max-w-full overflow-x-auto">
      <Table aria-label="Files">
        <TableHeader>
          <Column isRowHeader>Path</Column>
          <Column align="end">Size</Column>
          <Column align="end">
            <span className="sr-only">Download</span>
          </Column>
        </TableHeader>
        <TableBody items={rows} dependencies={rowDependencies}>
          {(row) => (
            <Row id={row.id}>
              <Cell className="wrap-anywhere">{row.path}</Cell>
              <Cell align="end" className="text-ink-muted whitespace-nowrap">
                {row.size}
              </Cell>
              <Cell align="end" className="w-10">
                <DownloadButton row={row} onDownload={onDownload} />
              </Cell>
            </Row>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

interface FileTableProps {
  files: readonly FileEntry[];
  onDownload: (row: FileRow) => void;
}

function DownloadButton({ row, onDownload }: DownloadButtonProps) {
  const download = useCallback(() => {
    onDownload(row);
  }, [row, onDownload]);

  return <IconButton label={`Download ${row.path}`} icon={DownloadIcon} size="sm" onPress={download} />;
}

interface DownloadButtonProps {
  row: FileRow;
  onDownload: (row: FileRow) => void;
}

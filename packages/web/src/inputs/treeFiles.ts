export const TREE_FILE_TYPES = [".nwk", ".newick", ".tre", ".tree", ".txt"] as const;

export const SESSION_FILE_TYPES = [".json"] as const;

export function isSessionFileName(name: string): boolean {
  return SESSION_FILE_TYPES.some((extension) => name.toLowerCase().endsWith(extension));
}

export interface ReadFailure {
  name: string;
  cause: unknown;
}

export function readFailures(failures: readonly ReadFailure[], added: number): string | null {
  const [only] = failures;

  if (only === undefined) {
    return null;
  }

  const unread =
    failures.length === 1
      ? `${only.name} could not be read: ${causeMessage(only.cause)}. Check the file and add it again.`
      : `${String(failures.length)} files could not be read: ${failures
          .map(({ name, cause }) => `${name} (${causeMessage(cause)})`)
          .join("; ")}. Check the files and add them again.`;

  return added === 0 ? unread : `${unread} ${addedFiles(added)}`;
}

function addedFiles(added: number): string {
  return added === 1 ? "The other file was added." : `The other ${String(added)} files were added.`;
}

export function sessionFailure(name: string, cause: unknown): string {
  return `${name} is not a TreeKnit session file: ${causeMessage(cause)}. Open a treeknit_request.json file.`;
}

export function batchRejection(sessionNames: readonly string[], treeCount: number): string | null {
  const [first] = sessionNames;

  if (first === undefined || (sessionNames.length === 1 && treeCount === 0)) {
    return null;
  }

  if (sessionNames.length > 1) {
    return `${sessionNames.join(", ")} were not opened: open one session file at a time. Nothing was added.`;
  }

  return `${first} was not opened together with trees, because a session file replaces the trees. Nothing was added. Open the session file on its own, or add only the trees.`;
}

export function addFailure(cause: unknown): string {
  return `The trees could not be added: ${causeMessage(cause)}. Try again.`;
}

function causeMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

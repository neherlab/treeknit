import { getErrorMessage } from "react-error-boundary";

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
      ? `${only.name} could not be read: ${getErrorMessage(only.cause) ?? String(only.cause)}. Check the file and add it again.`
      : `${String(failures.length)} files could not be read: ${failures
          .map(({ name, cause }) => `${name} (${getErrorMessage(cause) ?? String(cause)})`)
          .join("; ")}. Check the files and add them again.`;

  return added === 0 ? unread : `${unread} ${addedTrees(added)}`;
}

export function batchProblem(
  failures: readonly ReadFailure[],
  added: number,
  addProblem: string | null,
): string | null {
  const readProblem = readFailures(failures, addProblem === null ? added : 0);

  return [readProblem, addProblem].filter((message) => message !== null).join(" ") || null;
}

function addedTrees(added: number): string {
  return added === 1 ? "1 tree was added." : `${String(added)} trees were added.`;
}

export function sessionFailure(name: string, cause: unknown): string {
  return `${name} is not a TreeKnit session file: ${getErrorMessage(cause) ?? String(cause)}. Open a treeknit_session.json file.`;
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
  return `The trees could not be added: ${getErrorMessage(cause) ?? String(cause)}. Try again.`;
}

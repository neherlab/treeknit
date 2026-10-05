export const TREE_FILE_TYPES = [".nwk", ".newick", ".tre", ".tree", ".txt"] as const;

export const SESSION_FILE_TYPES = [".json"] as const;

export function isSessionFileName(name: string): boolean {
  return SESSION_FILE_TYPES.some((extension) => name.toLowerCase().endsWith(extension));
}

export function readFailure(name: string, cause: unknown): string {
  return `${name} could not be read: ${causeMessage(cause)}. Check the file and add it again.`;
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

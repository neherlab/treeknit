import { ProgressBar } from "./ProgressBar";

export function Loading({ label }: LoadingProps) {
  return (
    <div className="flex h-full items-center justify-center p-6">
      <ProgressBar label={label} isIndeterminate className="w-64" />
    </div>
  );
}

export interface LoadingProps {
  label: string;
}

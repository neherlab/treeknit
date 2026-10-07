import type { ReactNode } from "react";

import { InlineNotice } from "./InlineNotice";

export function ErrorNotice({ title, details, action }: ErrorNoticeProps) {
  return (
    <InlineNotice tone="danger" title={title} action={action}>
      {details.map((detail) => (
        <p key={detail}>{detail}</p>
      ))}
    </InlineNotice>
  );
}

export interface ErrorNoticeProps {
  title: string;
  details: readonly string[];
  action?: ReactNode;
}

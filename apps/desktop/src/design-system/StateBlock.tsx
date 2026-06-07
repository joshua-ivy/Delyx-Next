import type { ReactNode } from "react";

interface StateBlockProps {
  title: string;
  detail: string;
  tone?: "neutral" | "warning" | "danger" | "success";
  action?: ReactNode;
}

export function StateBlock({ action, detail, title, tone = "neutral" }: StateBlockProps) {
  return (
    <div className={`state-block state-${tone}`}>
      <strong>{title}</strong>
      <p>{detail}</p>
      {action}
    </div>
  );
}

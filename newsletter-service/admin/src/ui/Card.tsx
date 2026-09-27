import type { HTMLAttributes } from "react";

export function Card({ pad = true, className = "", ...rest }: HTMLAttributes<HTMLDivElement> & { pad?: boolean }) {
  return <div className={`ui-card ${pad ? "ui-card--pad" : ""} ${className}`.trim()} {...rest} />;
}

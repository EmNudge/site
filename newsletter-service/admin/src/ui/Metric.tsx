import type { ReactNode } from "react";

export function MetricGrid({ children }: { children: ReactNode }) {
  return <div className="ui-metric-grid">{children}</div>;
}

export function Metric({
  value,
  label,
  hint,
  primary,
}: {
  value: ReactNode;
  label: string;
  hint?: ReactNode;
  primary?: boolean;
}) {
  return (
    <div className={`ui-metric ${primary ? "ui-metric--primary" : ""}`.trim()}>
      <div className="ui-metric__value">{value}</div>
      <div className="ui-metric__label">{label}</div>
      {hint != null && <div className="ui-metric__hint">{hint}</div>}
    </div>
  );
}

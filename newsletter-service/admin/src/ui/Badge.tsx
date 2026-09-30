import type { ReactNode } from "react";

type Variant = "ok" | "warn" | "danger" | "neutral";

const STATUS_VARIANT: Record<string, Variant> = {
  confirmed: "ok",
  pending: "warn",
  unsubscribed: "danger",
  bounced: "neutral",
};

export function Badge({ children, variant = "neutral" }: { children: ReactNode; variant?: Variant }) {
  return <span className={`ui-badge ui-badge--${variant}`}>{children}</span>;
}

export function StatusBadge({ status }: { status: string }) {
  return <Badge variant={STATUS_VARIANT[status] ?? "neutral"}>{status}</Badge>;
}

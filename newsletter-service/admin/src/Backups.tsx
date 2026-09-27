import { useCallback, useEffect, useState } from "react";
import { getJSON, postJSON } from "./api";
import type { BackupStatus } from "./types";
import { Button, Card, Section } from "./ui";

const fmtWhen = (iso: string) =>
  new Date(iso).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
const fmtSize = (bytes: number | null) =>
  bytes == null ? "" : bytes < 1024 * 1024 ? `${Math.max(1, Math.round(bytes / 1024))} KB` : `${(bytes / 1024 / 1024).toFixed(1)} MB`;

export function Backups() {
  const [status, setStatus] = useState<BackupStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState("");

  const load = useCallback(() => {
    getJSON<BackupStatus>("/admin/backup/status").then(setStatus).catch(() => {});
  }, []);
  useEffect(load, [load]);

  async function backUpNow() {
    setBusy(true);
    setMsg("Backing up…");
    const { ok, data } = await postJSON("/admin/backup/run", {});
    setMsg(ok ? "Backed up to R2." : `Backup failed: ${data?.error || "unknown error"}`);
    setBusy(false);
    load();
  }

  const failed = status?.last && !status.last.ok ? status.last : null;

  return (
    <Section
      title="Backups"
      desc={status?.r2Configured ? "Backed up to R2 daily." : undefined}
      action={
        <div style={{ display: "flex", gap: "var(--sp-2)" }}>
          {status?.r2Configured && (
            <Button size="sm" onClick={backUpNow} disabled={busy || status.running}>
              {busy ? "Backing up…" : "Back up to R2 now"}
            </Button>
          )}
          <Button size="sm" variant="primary" onClick={() => location.assign("/admin/backup/download")}>
            ⤓ Download backup
          </Button>
        </div>
      }
    >
      <Card>
        {!status ? (
          <span className="muted">Loading…</span>
        ) : !status.r2Configured ? (
          <span className="muted">
            Automatic R2 backups are off. Set <code>R2_ACCOUNT_ID</code>, <code>R2_ACCESS_KEY_ID</code>,{" "}
            <code>R2_SECRET_ACCESS_KEY</code> and <code>R2_BUCKET</code> in the service's <code>.env</code> to turn them on.
          </span>
        ) : status.lastSuccess ? (
          <span>
            Last R2 backup: <b>{fmtWhen(status.lastSuccess.finishedAt)}</b>
            <span className="muted"> · {fmtSize(status.lastSuccess.bytes)} · {status.lastSuccess.objectKey}</span>
          </span>
        ) : (
          <span className="muted">No R2 backup yet — the first runs within the hour.</span>
        )}
        {failed && (
          <div style={{ color: "var(--danger)", marginTop: "var(--sp-2)" }}>
            Last attempt failed ({fmtWhen(failed.finishedAt)}): {failed.error}
          </div>
        )}
        {msg && <div className="muted" style={{ marginTop: "var(--sp-2)" }}>{msg}</div>}
      </Card>
    </Section>
  );
}

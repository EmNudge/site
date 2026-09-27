import { useCallback, useEffect, useRef, useState } from "react";
import { getJSON, postJSON } from "./api";
import type { SentRow, Stats, SubscriberRow } from "./types";
import { Backups } from "./Backups";
import { Button, Card, Column, DataTable, Input, Metric, MetricGrid, Section, Select, StatusBadge } from "./ui";

const PAGE = 50;
const fmtDate = (iso: string | null) =>
  iso ? new Date(iso).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" }) : "—";

interface ListRow {
  list: string;
  confirmed: number;
  pending: number;
  unsubscribed: number;
  bounced: number;
}

const byListCols: Column<ListRow>[] = [
  { key: "list", header: "List" },
  { key: "confirmed", header: "Confirmed", align: "right" },
  { key: "pending", header: "Pending", align: "right" },
  { key: "unsubscribed", header: "Unsub", align: "right" },
  { key: "bounced", header: "Bounced", align: "right" },
];

const sentCols: Column<SentRow>[] = [
  { key: "slug", header: "Post" },
  { key: "list", header: "List" },
  { key: "sent_at", header: "Sent", render: (r) => fmtDate(r.sent_at) },
  { key: "recipients", header: "Recipients", align: "right" },
];

export function Dashboard() {
  const [stats, setStats] = useState<Stats | null>(null);
  const [statsErr, setStatsErr] = useState("");
  const [rows, setRows] = useState<SubscriberRow[]>([]);
  const [total, setTotal] = useState(0);
  const [subsErr, setSubsErr] = useState("");
  const [q, setQ] = useState("");
  const [list, setList] = useState("");
  const [status, setStatus] = useState("");
  const [offset, setOffset] = useState(0);
  const [sent, setSent] = useState<SentRow[]>([]);
  const [syncing, setSyncing] = useState(false);
  const [syncMsg, setSyncMsg] = useState("");

  const loadStats = useCallback(() => {
    getJSON<Stats>("/admin/stats").then((s) => { setStats(s); setStatsErr(""); }).catch((e) => setStatsErr(String(e.message)));
  }, []);
  const loadSent = useCallback(() => {
    getJSON<{ rows: SentRow[] }>("/admin/sent").then((d) => setSent(d.rows)).catch(() => {});
  }, []);
  const loadSubs = useCallback(() => {
    const p = new URLSearchParams();
    if (q) p.set("q", q);
    if (list) p.set("list", list);
    if (status) p.set("status", status);
    p.set("limit", String(PAGE));
    p.set("offset", String(offset));
    getJSON<{ rows: SubscriberRow[]; total: number }>("/admin/subscribers?" + p)
      .then((d) => { setRows(d.rows); setTotal(d.total); setSubsErr(""); })
      .catch((e) => setSubsErr(String(e.message)));
  }, [q, list, status, offset]);

  useEffect(() => { loadStats(); loadSent(); }, [loadStats, loadSent]);

  const t = useRef<number | undefined>(undefined);
  useEffect(() => {
    clearTimeout(t.current);
    t.current = window.setTimeout(loadSubs, 200);
    return () => clearTimeout(t.current);
  }, [loadSubs]);

  const refresh = () => { loadStats(); loadSubs(); loadSent(); };

  const syncSuppressions = async () => {
    setSyncing(true);
    setSyncMsg("");
    const res = await postJSON("/admin/sync-suppressions", {});
    setSyncing(false);
    if (res.ok) {
      const d = res.data ?? {};
      setSyncMsg(`Scanned ${d.scanned ?? 0} · ${d.suppressed ?? 0} newly bounced.`);
      loadStats();
      loadSubs();
    } else {
      setSyncMsg(`Sync failed (${res.status}).`);
    }
  };

  const deleteSubscriber = async (email: string) => {
    if (!confirm(`Delete ${email}?\nThis erases the address from every list, along with its delivery records. It can't be undone.`)) return;
    const res = await postJSON("/admin/delete-subscriber", { email });
    if (res.ok) {
      loadStats();
      loadSubs();
    } else {
      setSubsErr(`Delete failed (${res.data?.error || res.status}).`);
    }
  };

  const subCols: Column<SubscriberRow>[] = [
    { key: "email", header: "Email" },
    { key: "list", header: "List" },
    { key: "status", header: "Status", render: (r) => <StatusBadge status={r.status} /> },
    { key: "created_at", header: "Signed up", render: (r) => fmtDate(r.created_at) },
    { key: "confirmed_at", header: "Confirmed", render: (r) => fmtDate(r.confirmed_at) },
    { key: "actions", header: "", align: "right", render: (r) => <Button variant="ghost" size="sm" onClick={() => deleteSubscriber(r.email)}>Delete</Button> },
  ];

  const tot = stats?.totals;
  const s7 = stats?.signups.last7d ?? 0;
  const s30 = stats?.signups.last30d ?? 0;
  const listRows: ListRow[] = (["blog", "notes"] as const).map((l) => ({ list: l, ...(stats?.byList[l] ?? { confirmed: 0, pending: 0, unsubscribed: 0, bounced: 0 }) }));

  const from = total === 0 ? 0 : offset + 1;
  const to = Math.min(offset + PAGE, total);

  return (
    <>
      <div className="toolbar">
        <Button variant="ghost" size="sm" onClick={refresh}>↻ Refresh</Button>
        <Button variant="ghost" size="sm" onClick={syncSuppressions} disabled={syncing}>
          {syncing ? "Syncing…" : "⤓ Sync suppressions"}
        </Button>
        {syncMsg && <span className="muted" style={{ marginLeft: "var(--sp-2)" }}>{syncMsg}</span>}
      </div>
      {statsErr && <div style={{ color: "var(--danger)", marginBottom: "var(--sp-3)" }}>Failed to load stats ({statsErr})</div>}

      <div className="metric-groups">
        <div>
          <p className="metric-group__label">Audience</p>
          <MetricGrid>
            <Metric primary value={tot?.confirmed ?? "—"} label="Confirmed subscribers" hint={`+${s7} in the last 7 days`} />
            <Metric value={tot?.uniqueEmails ?? "—"} label="Unique emails" />
            <Metric value={tot?.pending ?? "—"} label="Pending" />
            <Metric value={tot?.unsubscribed ?? "—"} label="Unsubscribed" />
            <Metric value={tot?.bounced ?? "—"} label="Bounced" />
            <Metric value={tot?.complaints ?? "—"} label="Complaints" hint="spam reports (subset of bounced)" />
          </MetricGrid>
        </div>
        <div>
          <p className="metric-group__label">Sending &amp; growth</p>
          <MetricGrid>
            <Metric value={stats?.newsletter.emailsDelivered ?? "—"} label="Newsletter emails sent" />
            <Metric value={stats?.newsletter.postsSent ?? "—"} label="Posts sent" />
            <Metric value={s30} label="New in 30 days" hint={`${s7} this week`} />
            <Metric value={tot?.queuedConfirmations ?? "—"} label="Queued confirmations" hint="waiting on a send cap" />
          </MetricGrid>
        </div>
      </div>

      <div className="grid-2">
        <Section title="By list">
          <DataTable columns={byListCols} rows={listRows} getKey={(r) => r.list} />
        </Section>
        <Section title="Why they left">
          <Card pad={false}>
            <div className="reasons-list">
              {stats && stats.unsubReasons.length ? (
                stats.unsubReasons.map((r) => (
                  <div className="row" key={r.reason}><span>{r.reason}</span><span className="count">{r.count}</span></div>
                ))
              ) : (
                <div className="row"><span className="muted">No unsubscribe feedback yet.</span></div>
              )}
            </div>
          </Card>
        </Section>
      </div>

      <Backups />

      <Section title="Subscribers" desc={total ? `${from}–${to} of ${total}` : undefined}>
        <div className="controls">
          <Input className="search" type="search" placeholder="Search by email…" value={q} onChange={(e) => { setOffset(0); setQ(e.target.value); }} />
          <Select value={list} onChange={(e) => { setOffset(0); setList(e.target.value); }} style={{ width: "auto" }}>
            <option value="">All lists</option>
            <option value="blog">Blog</option>
            <option value="notes">Notes</option>
          </Select>
          <Select value={status} onChange={(e) => { setOffset(0); setStatus(e.target.value); }} style={{ width: "auto" }}>
            <option value="">All statuses</option>
            <option value="confirmed">Confirmed</option>
            <option value="pending">Pending</option>
            <option value="unsubscribed">Unsubscribed</option>
            <option value="bounced">Bounced</option>
          </Select>
        </div>
        <DataTable columns={subCols} rows={rows} error={subsErr || undefined} empty="No subscribers match." getKey={(r, i) => r.email + r.list + i} />
        <div className="pager">
          <Button size="sm" disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - PAGE))}>← Prev</Button>
          <span className="muted">{from}–{to} of {total}</span>
          <Button size="sm" disabled={offset + PAGE >= total} onClick={() => setOffset(offset + PAGE)}>Next →</Button>
        </div>
      </Section>

      <Section title="Sent newsletters">
        <DataTable columns={sentCols} rows={sent} empty="Nothing sent yet." getKey={(r, i) => r.slug + i} />
      </Section>
    </>
  );
}

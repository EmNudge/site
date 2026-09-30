import { useCallback, useEffect, useRef, useState } from "react";
import { getJSON, postJSON } from "./api";
import type { AdminConfig, SendJob, TemplateResult } from "./types";
import { Button, Card, Field, Input, Progress, Segmented, Select, Textarea } from "./ui";

type TplType = "post" | "confirmation";
const TEST_TO_KEY = "newsletter-admin:test-to";
const slugify = (s: string) => s.toLowerCase().trim().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");

const DEFAULT_BODY = `The burgeoning technology of the decade has opened lots of new questions. Some of them with answers less obvious proportional to the volume of their questions. That is, some questions are obvious and many are asking them. Some are difficult and few seem to talk of them.

This is a hard question. Hard for me, at least.

And difficult things have cultural value. When I see someone do a difficult thing, I am impressed. When someone exhibits proof of having done a difficult thing, I am impressed. When I do something of note, I want my accomplishment to sit alongside its effort. People should know I tried.

I struggled a lot to learn what I do. Because of that struggle, I know lots of niche things which come up at times during my work. I can win very specific arguments because of the hours and days I toiled over small documents in corners of the internet.

I'm really proud of authoring [watlings](https://github.com/EmNudge/watlings). It's barely important. It wasn't difficult to make and it surely wasn't original. Near the end of the readme you can find the essence of this paragraph.

> Studies have shown that one cannot learn effectively without effort. This applies to practically every domain of knowledge. These projects should be educational, not easy.

I'm a bit embarrassed to see myself write "studies have shown" without pointing to specific studies, but it is backed up and well agreed upon. It is scientific consensus. To learn properly, it must at least be a little difficult.

But what of the pedagogy? Surely there is value in effort. We are seeing those who take the path of easement greatly disenfranchised. If you cheat your way through classes you will land a degree without the knowledge it's meant to represent.

The struggle was important in many ways, but it was not wholly important. Directed struggle is what matters.

On effort, I think there is value. But it is not the whole value.`;

function summarize(j: SendJob) {
  const parts = [`sent ${j.sent}/${j.total}`];
  if (j.skipped) parts.push(`${j.skipped} already sent`);
  if (j.bounced) parts.push(`${j.bounced} bounced`);
  if (j.failed) parts.push(`${j.failed} failed`);
  return parts.join(", ");
}

export function Templates({ config }: { config: AdminConfig }) {
  const [type, setType] = useState<TplType>("post");

  const [list, setList] = useState<"notes" | "blog">("notes");
  const [title, setTitle] = useState("On effort");
  const [slug, setSlug] = useState("on-effort");
  const [slugDirty, setSlugDirty] = useState(false);
  const [author, setAuthor] = useState("");
  const [date, setDate] = useState("Sep 17, 2026");
  const [body, setBody] = useState(DEFAULT_BODY);

  const [cBlog, setCBlog] = useState(false);
  const [cNotes, setCNotes] = useState(true);

  const [view, setView] = useState<"html" | "text" | "headers">("html");
  const [result, setResult] = useState<TemplateResult | null>(null);

  const [count, setCount] = useState<number | null>(null);
  const [sendStatus, setSendStatus] = useState("");
  const [pct, setPct] = useState<number | null>(null);
  const [sending, setSending] = useState(false);
  const [jobId, setJobId] = useState<string | null>(null);
  const [testTo, setTestTo] = useState(() => {
    try { return localStorage.getItem(TEST_TO_KEY) ?? ""; } catch { return ""; }
  });
  const [testing, setTesting] = useState(false);

  useEffect(() => { if (config.authorName) setAuthor((a) => a || config.authorName); }, [config.authorName]);
  useEffect(() => { if (!slugDirty) setSlug(slugify(title)); }, [title, slugDirty]);

  const previewQuery = useCallback(() => {
    const p = new URLSearchParams({ type });
    if (type === "post") {
      p.set("list", list); p.set("title", title); p.set("author", author); p.set("date", date); p.set("body", body);
    } else {
      p.set("lists", [cBlog && "blog", cNotes && "notes"].filter(Boolean).join(",") || "notes");
    }
    return p.toString();
  }, [type, list, title, author, date, body, cBlog, cNotes]);

  const dref = useRef<number | undefined>(undefined);
  useEffect(() => {
    clearTimeout(dref.current);
    dref.current = window.setTimeout(() => {
      getJSON<TemplateResult>("/admin/template?" + previewQuery()).then(setResult).catch(() => setResult(null));
    }, 200);
    return () => clearTimeout(dref.current);
  }, [previewQuery]);

  const refreshCount = useCallback(() => {
    getJSON<{ count: number }>(`/admin/recipient-count?list=${list}`).then((d) => setCount(d.count)).catch(() => setCount(null));
  }, [list]);
  useEffect(() => { if (type === "post") refreshCount(); }, [type, refreshCount]);

  function poll(jobId: string) {
    const tick = async () => {
      try {
        const j = await getJSON<SendJob>(`/admin/send-status?jobId=${jobId}`);
        setPct(j.total ? Math.round((j.processed / j.total) * 100) : 0);
        if (j.status === "running") { setSendStatus(`Sending… ${summarize(j)}`); window.setTimeout(tick, 800); }
        else if (j.status === "done") { setSendStatus(`Done — ${summarize(j)}.`); setSending(false); setJobId(null); refreshCount(); }
        else if (j.status === "cancelled") { setSendStatus(`Cancelled — ${summarize(j)}. Send again to resume with the rest.`); setSending(false); setJobId(null); }
        else { setSendStatus(`Error: ${j.error || "send failed"}`); setSending(false); setJobId(null); }
      } catch { window.setTimeout(tick, 1500); }
    };
    tick();
  }

  async function doSend() {
    if (!title || !slug) { setSendStatus("Title and slug are required."); return; }
    if (!body.trim()) { setSendStatus("Body is empty."); return; }
    if (!confirm(`Send "${title}" to all confirmed ${list} subscribers now?\nThis emails them immediately and can't be undone.`)) return;
    setSending(true); setSendStatus("Starting…"); setPct(0);
    const { ok, data } = await postJSON("/admin/send", { list, slug, title, author, date, url: `${config.siteOrigin}/${list}/${slug}`, body });
    if (ok && data?.skipped) { setSendStatus(`Already sent for "${slug}" — not re-sent.`); setPct(null); setSending(false); return; }
    if (!ok || !data?.jobId) { setSendStatus(`Error: ${data?.error || "failed"}`); setPct(null); setSending(false); return; }
    setJobId(data.jobId);
    poll(data.jobId);
  }

  async function doCancel() {
    if (!jobId) return;
    setSendStatus("Cancelling…");
    await postJSON("/admin/send-cancel", { jobId });
  }

  async function doTestSend() {
    const to = testTo.trim();
    if (!to) { setSendStatus("Enter an address to send the test to."); return; }
    if (!title || !body.trim()) { setSendStatus("Title and body are required."); return; }
    try { localStorage.setItem(TEST_TO_KEY, to); } catch { /* ignore */ }
    setTesting(true); setSendStatus(`Sending test to ${to}…`);
    const { ok, data } = await postJSON("/admin/send-test", { to, list, slug, title, author, date, url: `${config.siteOrigin}/${list}/${slug}`, body });
    setSendStatus(ok ? `Test sent to ${to}.` : `Error: ${data?.error || "test send failed"}`);
    setTesting(false);
  }

  return (
    <div className="layout">
      <div className="panel">
        <nav className="tpl-menu">
          <div className="tpl-group">Emails</div>
          <button className="tpl-item" aria-pressed={type === "post"} onClick={() => setType("post")}>New post</button>
          <button className="tpl-item" aria-pressed={type === "confirmation"} onClick={() => setType("confirmation")}>Confirmation</button>
        </nav>
        <hr className="tpl-sep" />

        {type === "post" && (
          <>
            <Field label="List">
              <Select value={list} onChange={(e) => setList(e.target.value as "notes" | "blog")}>
                <option value="notes">Notes</option>
                <option value="blog">Blog</option>
              </Select>
            </Field>
            <Field label="Title"><Input value={title} onChange={(e) => setTitle(e.target.value)} /></Field>
            <Field label="Slug" hint="(post URL + send id)">
              <Input value={slug} onChange={(e) => { setSlugDirty(true); setSlug(e.target.value); }} />
            </Field>
            <Field label="Author"><Input value={author} onChange={(e) => setAuthor(e.target.value)} /></Field>
            <Field label="Date"><Input value={date} onChange={(e) => setDate(e.target.value)} /></Field>
            <Field label="Body" hint="(markdown)">
              <Textarea style={{ minHeight: 260 }} value={body} onChange={(e) => setBody(e.target.value)} />
            </Field>
          </>
        )}

        {type === "confirmation" && (
          <Field label="Lists being confirmed">
            <label style={{ display: "block", fontSize: 14, marginBottom: 4 }}>
              <input type="checkbox" checked={cBlog} onChange={(e) => setCBlog(e.target.checked)} /> Blog
            </label>
            <label style={{ display: "block", fontSize: 14 }}>
              <input type="checkbox" checked={cNotes} onChange={(e) => setCNotes(e.target.checked)} /> Notes
            </label>
          </Field>
        )}

        <p className="note">Links use placeholder tokens and won't work — this is a visual preview of what recipients receive.</p>
      </div>

      <div>
        {type === "post" && (
          <Card style={{ marginBottom: "var(--sp-3)" }}>
            <div className="send-bar">
              <div className="send-info">
                Send to all confirmed <b>{list}</b> subscribers {count != null && <span className="muted">({count})</span>}
              </div>
              <Button variant="primary" disabled={sending || testing} onClick={doSend}>Send newsletter</Button>
              {sending && <Button disabled={!jobId} onClick={doCancel}>Cancel send</Button>}
              <Input type="email" placeholder="you@example.com" style={{ width: 220 }} value={testTo} onChange={(e) => setTestTo(e.target.value)} />
              <Button disabled={sending || testing} onClick={doTestSend}>Send test</Button>
              <span className="muted">{sendStatus}</span>
              {pct != null && <Progress pct={pct} />}
            </div>
          </Card>
        )}

        <div className="subject">
          <div className="l">Subject</div>
          <div className="v">{result?.subject ?? "—"}</div>
        </div>
        <div className="view-tabs">
          <Segmented
            value={view}
            onChange={setView}
            options={[
              { value: "html", label: "HTML" },
              { value: "text", label: "Plain text" },
              { value: "headers", label: "Headers" },
            ]}
          />
        </div>
        {view === "html" && <iframe title="email preview" sandbox="" srcDoc={result?.html ?? ""} />}
        {view === "text" && <pre>{result?.text ?? ""}</pre>}
        {view === "headers" && (
          <pre>
            {result && Object.keys(result.headers || {}).length
              ? Object.entries(result.headers).map(([k, v]) => `${k}: ${v}`).join("\n")
              : "(no custom headers)"}
          </pre>
        )}
      </div>
    </div>
  );
}

export interface Stats {
  totals: { uniqueEmails: number; confirmed: number; pending: number; unsubscribed: number; bounced: number; complaints: number; queuedConfirmations: number };
  byList: Record<"blog" | "notes", { confirmed: number; pending: number; unsubscribed: number; bounced: number }>;
  newsletter: { postsSent: number; emailsDelivered: number };
  signups: { last7d: number; last30d: number };
  unsubReasons: { reason: string; count: number }[];
}

export interface SubscriberRow {
  email: string;
  list: string;
  status: string;
  created_at: string;
  confirmed_at: string | null;
}

export interface SentRow {
  slug: string;
  list: string;
  sent_at: string;
  recipients: number;
}

export interface AdminConfig {
  siteOrigin: string;
  authorName: string;
}

export interface TemplateResult {
  subject: string;
  html: string;
  text: string;
  headers: Record<string, string>;
}

export interface SendJob {
  id: string;
  slug: string;
  list: string;
  total: number;
  processed: number;
  sent: number;
  skipped: number;
  bounced: number;
  failed: number;
  status: "running" | "done" | "error" | "cancelled";
  error?: string;
}

export interface BackupRow {
  startedAt: string;
  finishedAt: string;
  ok: boolean;
  objectKey: string | null;
  bytes: number | null;
  error: string | null;
}

export interface BackupStatus {
  r2Configured: boolean;
  running: boolean;
  last: BackupRow | null;
  lastSuccess: BackupRow | null;
}

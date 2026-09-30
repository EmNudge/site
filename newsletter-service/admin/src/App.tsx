import { useEffect, useState } from "react";
import { getJSON } from "./api";
import type { AdminConfig } from "./types";
import { Dashboard } from "./Dashboard";
import { Templates } from "./Templates";

type Tab = "dashboard" | "templates";

const initialTab = (): Tab =>
  new URLSearchParams(location.search).get("tab") === "templates" ? "templates" : "dashboard";

export function App() {
  const [tab, setTab] = useState<Tab>(initialTab);
  const [config, setConfig] = useState<AdminConfig>({ siteOrigin: "", authorName: "" });

  useEffect(() => {
    getJSON<AdminConfig>("/admin/config")
      .then(setConfig)
      .catch(() => {});
  }, []);

  // Route via ?tab= so refresh restores the view.
  const go = (t: Tab) => {
    setTab(t);
    const u = new URL(location.href);
    u.searchParams.set("tab", t);
    history.replaceState(null, "", u);
  };

  return (
    <>
      <aside className="sidebar">
        <div className="brand">Newsletter</div>
        <nav className="side-nav">
          <button aria-pressed={tab === "dashboard"} onClick={() => go("dashboard")}>Dashboard</button>
          <button aria-pressed={tab === "templates"} onClick={() => go("templates")}>Templates</button>
        </nav>
        <form className="sign-out" method="post" action="/admin/logout">
          <button type="submit">Sign out</button>
        </form>
      </aside>
      <main className="main">
        {/* Both stay mounted so their state persists across tab switches. */}
        <div className="dashboard" hidden={tab !== "dashboard"}>
          <Dashboard />
        </div>
        <div className="templates" hidden={tab !== "templates"}>
          <Templates config={config} />
        </div>
      </main>
    </>
  );
}

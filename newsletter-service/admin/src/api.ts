// Same-origin fetches to the service API. The session cookie set by the login
// form rides along automatically.

// A 401 means the session expired (or was signed out elsewhere): back to login.
function toLoginOn401(res: Response) {
  if (res.status === 401) location.assign("/admin/login");
}

export async function getJSON<T>(url: string): Promise<T> {
  const res = await fetch(url, { headers: { accept: "application/json" } });
  toLoginOn401(res);
  if (!res.ok) throw new Error(String(res.status));
  return (await res.json()) as T;
}

export async function postJSON(
  url: string,
  body: unknown,
): Promise<{ ok: boolean; status: number; data: any }> {
  const res = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  toLoginOn401(res);
  let data: any = null;
  try {
    data = await res.json();
  } catch {
    /* ignore */
  }
  return { ok: res.ok, status: res.status, data };
}

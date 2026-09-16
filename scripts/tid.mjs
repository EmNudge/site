// standard.site lexicons declare `key: tid`, so record keys must be valid TIDs
// (13 chars, base32-sortable) — not human-readable slugs. TIDs encode a
// microsecond timestamp plus a random clock id, so they can't be derived from a
// slug; the slug -> TID binding has to be minted once and persisted.
const S32 = "234567abcdefghijklmnopqrstuvwxyz";
const TID_RE = /^[234567abcdefghij][234567a-z]{12}$/;

function s32encode(n) {
  let s = "";
  while (n > 0n) {
    s = S32[Number(n % 32n)] + s;
    n = n / 32n;
  }
  return s.padStart(13, "2");
}

let lastMicros = 0;
const clockid = Math.floor(Math.random() * 1024);

export function genTid() {
  let micros = Date.now() * 1000;
  if (micros <= lastMicros) micros = lastMicros + 1;
  lastMicros = micros;
  return s32encode((BigInt(micros) << 10n) | BigInt(clockid));
}

export const isTid = (rkey) => typeof rkey === "string" && TID_RE.test(rkey);

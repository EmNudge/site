#!/usr/bin/env bash
# Build and (re)install the newsletter service on the VPS, then restart it.
# Run as root. Also does the first install.
#
#   /opt/newsletter-src      git checkout of the site repo (sparse: newsletter-service/)
#   /opt/newsletter-service  what systemd runs: binary, admin/dist, data/, .env
#
# Keeping the two apart means a pull never touches the running files and the
# checkout never holds secrets or the database.
#
# Don't run this while a newsletter is sending: the restart ends the send job. It
# is safe to re-send afterwards — anyone already delivered to is skipped.

set -Eeuo pipefail  # -E: the ERR trap also fires inside main()
trap 'echo "update: FAILED at line $LINENO: $BASH_COMMAND" >&2' ERR

# Everything runs from a function, which bash reads in full before running any of
# it: the fetch below may rewrite this very file, and bash reads scripts lazily.
main() {
  SRC="${SRC:-/opt/newsletter-src}"
  APP="${APP:-/opt/newsletter-service}"
  CARGO="${CARGO:-$(command -v cargo || echo /root/.cargo/bin/cargo)}"

  [ "$(id -u)" = 0 ] || { echo "update: run as root" >&2; exit 1; }
  id newsletter >/dev/null 2>&1 ||
    { echo "update: create the user first: useradd --system --no-create-home --shell /usr/sbin/nologin newsletter" >&2; exit 1; }

  if [ "${SKIP_PULL:-0}" != 1 ]; then
    # Reset rather than pull: the branch may be rewritten (squash + force-push), and
    # the checkout is a build mirror with nothing local worth keeping.
    git -C "$SRC" fetch --quiet origin
    git -C "$SRC" reset --quiet --hard "@{upstream}"
  fi
  echo "update: building $(git -C "$SRC" log -1 --format='%h %s')"

  cd "$SRC/newsletter-service"
  (cd admin && npm ci --no-audit --no-fund --loglevel=error && npm run build)
  nice -n 10 "$CARGO" build --release --locked

  install -d -m755 "$APP" "$APP/admin"
  install -d -m700 -o newsletter -g newsletter "$APP/data"
  install -m755 target/release/newsletter-service "$APP/newsletter-service"
  # Swap the UI in whole so the running service never serves a half-copied dir.
  rm -rf "$APP/admin/dist.new" "$APP/admin/dist.old"
  cp -r admin/dist "$APP/admin/dist.new"
  [ -d "$APP/admin/dist" ] && mv "$APP/admin/dist" "$APP/admin/dist.old"
  mv "$APP/admin/dist.new" "$APP/admin/dist"
  rm -rf "$APP/admin/dist.old"

  if [ ! -f "$APP/.env" ]; then
    echo "update: installed; now create $APP/.env (from .env.example, chmod 600) and enable the unit" >&2
    exit 0
  fi

  systemctl restart newsletter
  port="$(grep -E '^PORT=' "$APP/.env" | cut -d= -f2)"
  for _ in $(seq 1 20); do
    if curl -fsS "http://127.0.0.1:${port:-8787}/health" >/dev/null 2>&1; then
      echo "update: newsletter is up"
      exit 0
    fi
    sleep 0.5
  done
  echo "update: service did not come up — journalctl -u newsletter -n 50" >&2
  exit 1
}

main "$@"

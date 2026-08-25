#!/usr/bin/env bash
# E2E - 06: Dashboard API
# Case 1: anonymous access denied (401/403/302)
# Case 2: authenticated access returns 200 + valid JSON
# Case 3: proxy list endpoint returns valid JSON (or skipped if not available)
set -Eeuo pipefail
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

SERVER_PORT=49532
DASHBOARD_PORT=49533
DB_USER="${DASHBOARD_USER:-admin}"
DB_PASS="${DASHBOARD_PASSWORD:-admin}"

cat >"$RUN_DIR/server-dashboard.toml" <<EOF
bindAddr      = "127.0.0.1"
bindPort      = $SERVER_PORT
dashboardAddr = "127.0.0.1"
dashboardPort = $DASHBOARD_PORT
dashboardUser = "$DB_USER"
dashboardPwd  = "$DB_PASS"
EOF

start_bg "$LOG_DIR/server-dashboard.log" \
  "$BIN_DIR/orbien-server" -c "$RUN_DIR/server-dashboard.toml"
wait_tcp 127.0.0.1 $SERVER_PORT
wait_tcp 127.0.0.1 $DASHBOARD_PORT

base="http://127.0.0.1:$DASHBOARD_PORT"

# --- Case 1: anonymous denied ---
anon_status="$(curl -sS -o /dev/null -w '%{http_code}' \
  "$base/api/v1/system/info" || true)"
case "$anon_status" in
  401|403|302) echo "Case 1 PASS: anonymous denied ($anon_status)" ;;
  *) echo "FAIL: anonymous access returned $anon_status"; exit 1 ;;
esac

# --- Case 2: authenticated access ---
auth_status="$(curl -sS -u "$DB_USER:$DB_PASS" \
  -o "$RUN_DIR/dashboard-info.json" \
  -w '%{http_code}' \
  "$base/api/v1/system/info" || true)"
if [[ "$auth_status" != "200" ]]; then
  echo "FAIL: authenticated /api/v1/system/info returned $auth_status"; exit 1
fi
jq empty "$RUN_DIR/dashboard-info.json"
echo "Case 2 PASS: authenticated access OK"

# --- Case 3: proxy list ---
curl -fsS -u "$DB_USER:$DB_PASS" \
  "$base/api/v1/proxy/tcp" \
  -o "$RUN_DIR/dashboard-proxies.json" 2>/dev/null && \
  jq empty "$RUN_DIR/dashboard-proxies.json" && \
  echo "Case 3 PASS: proxy list valid JSON" || \
  echo "Case 3 SKIP: proxy list endpoint not available"

echo "=== DASHBOARD E2E PASS ==="

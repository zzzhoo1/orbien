#!/usr/bin/env bash
# E2E - 05: Token authentication
# Case 1: wrong token  -> rejected
# Case 2: empty token  -> rejected
# Case 3: correct token -> accepted, TCP data flows
set -Eeuo pipefail
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

SERVER_PORT=49531
ECHO_LOCAL=49610
ECHO_REMOTE=49611
TOKEN="e2e-accept-token"

cat >"$RUN_DIR/server-token.toml" <<EOF
bindAddr = "127.0.0.1"
bindPort = $SERVER_PORT

[auth]
token = "$TOKEN"
EOF

cat >"$RUN_DIR/client-token-ok.toml" <<EOF
serverAddr = "127.0.0.1"
serverPort = $SERVER_PORT

[auth]
token = "$TOKEN"

[[proxies]]
name       = "tok-tcp"
type       = "tcp"
localIp    = "127.0.0.1"
localPort  = $ECHO_LOCAL
remotePort = $ECHO_REMOTE
EOF

cat >"$RUN_DIR/client-token-wrong.toml" <<EOF
serverAddr = "127.0.0.1"
serverPort = $SERVER_PORT

[auth]
token = "wrong-token"

[[proxies]]
name       = "tok-wrong"
type       = "tcp"
localIp    = "127.0.0.1"
localPort  = $ECHO_LOCAL
remotePort = 49699
EOF

cat >"$RUN_DIR/client-token-empty.toml" <<EOF
serverAddr = "127.0.0.1"
serverPort = $SERVER_PORT

[[proxies]]
name       = "tok-empty"
type       = "tcp"
localIp    = "127.0.0.1"
localPort  = $ECHO_LOCAL
remotePort = 49698
EOF

start_bg "$LOG_DIR/server-token.log" \
  "$BIN_DIR/orbien-server" -c "$RUN_DIR/server-token.toml"
wait_tcp 127.0.0.1 $SERVER_PORT

# --- Case 1: wrong token ---
set +e
timeout 12 "$BIN_DIR/orbien" \
  -c "$RUN_DIR/client-token-wrong.toml" \
  >>"$LOG_DIR/client-token-wrong.log" 2>&1
wrong_rc=$?
set -e
[[ $wrong_rc -ne 0 ]] || { echo "FAIL: wrong token unexpectedly succeeded"; exit 1; }
echo "Case 1 PASS: wrong token rejected (rc=$wrong_rc)"

# --- Case 2: empty token ---
set +e
timeout 12 "$BIN_DIR/orbien" \
  -c "$RUN_DIR/client-token-empty.toml" \
  >>"$LOG_DIR/client-token-empty.log" 2>&1
empty_rc=$?
set -e
[[ $empty_rc -ne 0 ]] || { echo "FAIL: empty token unexpectedly succeeded"; exit 1; }
echo "Case 2 PASS: empty token rejected (rc=$empty_rc)"

# --- Case 3: correct token ---
start_bg "$LOG_DIR/tok-upstream.log" \
  python3 "$script_dir/tcp_echo.py" 127.0.0.1 $ECHO_LOCAL
wait_tcp 127.0.0.1 $ECHO_LOCAL

start_bg "$LOG_DIR/client-token-ok.log" \
  "$BIN_DIR/orbien" -c "$RUN_DIR/client-token-ok.toml"
wait_tcp 127.0.0.1 $ECHO_REMOTE 20

python3 "$script_dir/tcp_check.py" 127.0.0.1 $ECHO_REMOTE
echo "Case 3 PASS: correct token accepted"

echo "=== TOKEN AUTH E2E PASS ==="

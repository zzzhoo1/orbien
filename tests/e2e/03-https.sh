#!/usr/bin/env bash
# E2E - 03: HTTPS tunnel (SNI passthrough)
# curl --resolve avoids modifying /etc/hosts while still sending correct SNI.
set -Eeuo pipefail
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

SERVER_PORT=49529
VHOST_PORT=49443
UPSTREAM_PORT=49800
SNI_DOMAIN="demo-tls.test.local"

openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout "$RUN_DIR/key.pem" \
  -out    "$RUN_DIR/cert.pem" \
  -days 1 \
  -subj "/CN=$SNI_DOMAIN" \
  -addext "subjectAltName=DNS:$SNI_DOMAIN" \
  2>/dev/null

cat >"$RUN_DIR/server-https.toml" <<EOF
bindAddr       = "127.0.0.1"
bindPort       = $SERVER_PORT
vhostHTTPSPort = $VHOST_PORT
EOF

cat >"$RUN_DIR/client-https.toml" <<EOF
serverAddr = "127.0.0.1"
serverPort = $SERVER_PORT

[[proxies]]
name          = "https-e2e"
type          = "https"
localIp       = "127.0.0.1"
localPort     = $UPSTREAM_PORT
customDomains = ["$SNI_DOMAIN"]
EOF

start_bg "$LOG_DIR/server-https.log" \
  "$BIN_DIR/orbien-server" -c "$RUN_DIR/server-https.toml"
wait_tcp 127.0.0.1 $SERVER_PORT
wait_tcp 127.0.0.1 $VHOST_PORT

start_bg "$LOG_DIR/https-upstream.log" \
  python3 "$script_dir/https_upstream.py" \
    "$RUN_DIR/cert.pem" "$RUN_DIR/key.pem" $UPSTREAM_PORT
wait_tcp 127.0.0.1 $UPSTREAM_PORT

start_bg "$LOG_DIR/client-https.log" \
  "$BIN_DIR/orbien" -c "$RUN_DIR/client-https.toml"

# Poll until SNI route registered – nc -z only proves port is open,
# not that the vhost entry exists; curl validates both.
end=$((SECONDS + 30))
while (( SECONDS < end )); do
  body="$(curl -kfsS \
    --resolve "$SNI_DOMAIN:$VHOST_PORT:127.0.0.1" \
    "https://$SNI_DOMAIN:$VHOST_PORT/" 2>/dev/null)" && break
  sleep 0.25
done

echo "$body" | grep -Fx 'orbien-https-ok'

# 10 concurrent requests
for i in $(seq 1 10); do
  curl -kfsS \
    --resolve "$SNI_DOMAIN:$VHOST_PORT:127.0.0.1" \
    "https://$SNI_DOMAIN:$VHOST_PORT/" | grep -Fx 'orbien-https-ok'
done

echo "=== HTTPS E2E PASS ==="

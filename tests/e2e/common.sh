#!/usr/bin/env bash
# Shared helpers for all E2E acceptance scripts.
set -Eeuo pipefail

ROOT="${GITHUB_WORKSPACE:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
BIN_DIR="${BIN_DIR:-$ROOT/target/release}"
RUN_DIR="${RUN_DIR:-$ROOT/.e2e}"
LOG_DIR="${LOG_DIR:-$RUN_DIR/logs}"

mkdir -p "$RUN_DIR" "$LOG_DIR"

SERVER_LOG="$LOG_DIR/server.log"
CLIENT_LOG="$LOG_DIR/client.log"

pids=()

cleanup() {
  set +e
  for pid in "${pids[@]:-}"; do
    [[ -n "$pid" ]] || continue
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  done
}
trap cleanup EXIT

start_bg() {
  local logfile="$1"; shift
  "$@" >>"$logfile" 2>&1 &
  local pid=$!
  pids+=("$pid")
  echo "$pid"
}

wait_tcp() {
  local host="$1" port="$2" timeout="${3:-30}"
  local end=$((SECONDS + timeout))
  while (( SECONDS < end )); do
    (echo >/dev/tcp/"$host"/"$port") 2>/dev/null && return 0
    sleep 0.25
  done
  echo "[wait_tcp] timeout waiting for $host:$port" >&2
  return 1
}

wait_log() {
  local file="$1" regex="$2" timeout="${3:-30}"
  local end=$((SECONDS + timeout))
  while (( SECONDS < end )); do
    grep -E "$regex" "$file" >/dev/null 2>&1 && return 0
    sleep 0.25
  done
  echo "[wait_log] timeout waiting for '$regex' in $file" >&2
  tail -n 60 "$file" >&2 || true
  return 1
}

assert_eq() {
  local got="$1" expected="$2" label="${3:-assert_eq}"
  [[ "$got" == "$expected" ]] || {
    echo "[$label] FAIL: got '$got', expected '$expected'" >&2
    return 1
  }
}

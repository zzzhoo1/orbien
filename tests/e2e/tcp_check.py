#!/usr/bin/env python3
"""Verify TCP tunnel by sending test payloads and checking echo responses."""
import socket
import sys
import time

host = sys.argv[1]
port = int(sys.argv[2])

payloads = [
    b"orbien-tcp-ok",
    b"a" * 4096,
    bytes(range(256)) * 32,
]

for payload in payloads:
    received = None
    for _ in range(40):
        try:
            with socket.create_connection((host, port), timeout=5) as conn:
                conn.sendall(payload)
                buf = bytearray()
                while len(buf) < len(payload):
                    chunk = conn.recv(len(payload) - len(buf))
                    if not chunk:
                        break
                    buf.extend(chunk)
            received = bytes(buf)
            break
        except (ConnectionRefusedError, TimeoutError, OSError):
            time.sleep(0.25)

    if received != payload:
        got = 0 if received is None else len(received)
        raise SystemExit(f"TCP FAIL: sent={len(payload)} got={got}")
    print(f"TCP PASS size={len(payload)}", flush=True)

print("TCP E2E ALL PASS")

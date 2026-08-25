#!/usr/bin/env python3
"""Verify UDP tunnel with multiple payload sizes."""
import socket
import sys
import time

host = sys.argv[1]
port = int(sys.argv[2])

# 64B, 1200B (MTU-safe), 4096B (needs udpPacketSize=8192 both sides)
for size in (64, 1200, 4096):
    payload = bytes(i % 251 for i in range(size))
    received = None

    for attempt in range(40):
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.settimeout(0.5)
        try:
            sock.sendto(payload, (host, port))
            data, _ = sock.recvfrom(65535)
            if data == payload:
                received = data
                break
            print(
                f"attempt={attempt+1} size={size} "
                f"sent={len(payload)} got={len(data)}",
                flush=True,
            )
        except (TimeoutError, OSError):
            pass
        finally:
            sock.close()
        time.sleep(0.2)

    if received != payload:
        got = 0 if received is None else len(received)
        raise SystemExit(f"UDP FAIL: size={size} sent={len(payload)} got={got}")
    print(f"UDP PASS size={size}", flush=True)

print("UDP E2E ALL PASS")

#!/usr/bin/env python3
"""UDP echo server used by E2E tests."""
import socket
import sys

port = int(sys.argv[1]) if len(sys.argv) > 1 else 39900
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(("127.0.0.1", port))
print(f"UDP echo listening on 127.0.0.1:{port}", flush=True)

while True:
    data, addr = sock.recvfrom(65535)
    sock.sendto(data, addr)

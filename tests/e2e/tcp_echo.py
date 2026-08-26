#!/usr/bin/env python3
"""Simple TCP echo server used by E2E tests."""
import socket
import sys

host = sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1"
port = int(sys.argv[2]) if len(sys.argv) > 2 else 39600

server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind((host, port))
server.listen(64)
print(f"TCP echo listening on {host}:{port}", flush=True)

while True:
    conn, _ = server.accept()
    with conn:
        while True:
            data = conn.recv(65536)
            if not data:
                break
            conn.sendall(data)

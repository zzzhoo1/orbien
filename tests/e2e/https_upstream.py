#!/usr/bin/env python3
"""Minimal HTTPS upstream (TLS terminator) for SNI passthrough E2E tests."""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import ssl
import sys

cert = sys.argv[1]
key  = sys.argv[2]
port = int(sys.argv[3])


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        body = b"orbien-https-ok\n"
        self.send_response(200)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, fmt, *args):
        print(fmt % args, flush=True)


server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ctx.load_cert_chain(cert, key)
server.socket = ctx.wrap_socket(server.socket, server_side=True)
print(f"HTTPS upstream listening on 127.0.0.1:{port}", flush=True)
server.serve_forever()

#!/usr/bin/env python3
from ssl import SSLContext, PROTOCOL_TLS_SERVER
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from sys import argv

def get_arg(name: str, default_val):
    elems = [e for e in argv if e.startswith(f"{name}=")]
    ret = default_val
    if len(elems) != 0:
        ret = elems[-1][len(name)+1:]
    return ret

port = int(get_arg("port", 8000))
ip = get_arg("ip", "0.0.0.0")
certfile = get_arg("cert", "cert.pem")

if __name__ == "__main__":
    httpd = ThreadingHTTPServer((ip, port), SimpleHTTPRequestHandler)
    context = SSLContext(protocol=PROTOCOL_TLS_SERVER)
    context.load_cert_chain(certfile=certfile)
    httpd.socket = context.wrap_socket(httpd.socket, server_side=True)
    print(f"Starting server @ https://{ip}:{port}")
    httpd.serve_forever()
import sys
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler

class HealthCheckHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain')
            self.end_headers()
            self.wfile.write(f"healthy from {self.server_port}\n".encode())
        else:
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain')
            self.end_headers()
            self.wfile.write(f"Response from backend on port {self.server_port}\n".encode())

    @property
    def server_port(self):
        return self.server.server_address[1]

def run(port):
    server_address = ('127.0.0.1', port)
    httpd = ThreadingHTTPServer(server_address, HealthCheckHandler)
    print(f"Starting python backend server on 127.0.0.1:{port}...")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down server...")
        httpd.server_close()

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python3 server.py <port>")
        sys.exit(1)
    port_arg = int(sys.argv[1])
    run(port_arg)


# used while testing, the program, as backend servers. 
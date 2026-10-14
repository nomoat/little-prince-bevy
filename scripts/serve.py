#!/usr/bin/env python3
"""Static server for dist/ with gzip for .wasm/.js (5x smaller transfer).
Usage: scripts/serve.py [port] [dir]"""
import gzip, http.server, os, sys, functools

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
ROOT = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'dist')
_cache = {}

class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map,
                      '.wasm': 'application/wasm', '.js': 'text/javascript', '.ron': 'text/plain',
                      '.glb': 'model/gltf-binary', '.ogg': 'audio/ogg'}

    def do_GET(self):
        path = self.translate_path(self.path)
        if path.endswith(('.wasm', '.js', '.html', '.ron')) and 'gzip' in self.headers.get('Accept-Encoding', '') and os.path.isfile(path):
            st = os.stat(path)
            key = (path, st.st_mtime)
            if key not in _cache:
                with open(path, 'rb') as f:
                    _cache[key] = gzip.compress(f.read(), 6)
            body = _cache[key]
            self.send_response(200)
            self.send_header('Content-Type', self.guess_type(path))
            self.send_header('Content-Encoding', 'gzip')
            self.send_header('Content-Length', str(len(body)))
            # the loader's progress bar wants the raw size
            self.send_header('X-Raw-Length', str(st.st_size))
            self.send_header('Cache-Control', 'no-cache')
            self.end_headers()
            self.wfile.write(body)
            return
        super().do_GET()

if __name__ == '__main__':
    os.chdir(ROOT)
    print(f'serving {os.path.abspath(ROOT)} on http://localhost:{PORT}/  (debug: ?chapter=b612&test=1&lang=en&free=1&map=1)')
    http.server.ThreadingHTTPServer(('0.0.0.0', PORT), Handler).serve_forever()

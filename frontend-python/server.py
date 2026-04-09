import json
import os
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def build_runtime_config() -> dict[str, str]:
    return {
        "BACKEND_API_BASE_URL": os.getenv("BACKEND_API_BASE_URL", "").strip(),
        "DEFAULT_LOCALE": os.getenv("DEFAULT_LOCALE", "zh-TW").strip() or "zh-TW",
    }


class AegisFrontendHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT), **kwargs)

    def do_GET(self):
        if self.path in ("/config.js", "/config.js?"):
            config = json.dumps(build_runtime_config(), ensure_ascii=False)
            payload = f"window.__AEGIS_CONFIG__ = {config};\n".encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/javascript; charset=utf-8")
            self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
            self.send_header("Pragma", "no-cache")
            self.send_header("Expires", "0")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
            return
        return super().do_GET()


def main():
    port = int(os.getenv("PORT", "3000"))
    server = ThreadingHTTPServer(("0.0.0.0", port), AegisFrontendHandler)
    print(f"frontend-python serving at http://0.0.0.0:{port}")
    server.serve_forever()


if __name__ == "__main__":
    main()

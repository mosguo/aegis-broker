import json
import os
import urllib.error
import urllib.request
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urljoin

ROOT = Path(__file__).resolve().parent
PROXY_PREFIXES = (
    "/health/",
    "/auth/google/",
    "/v1/",
)


def build_runtime_config() -> dict[str, str]:
    return {
        "BACKEND_API_BASE_URL": os.getenv("BACKEND_API_BASE_URL", "").strip(),
        "DEFAULT_LOCALE": os.getenv("DEFAULT_LOCALE", "zh-TW").strip() or "zh-TW",
    }


class AegisFrontendHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT), **kwargs)

    def _backend_base_url(self) -> str:
        return os.getenv("BACKEND_API_BASE_URL", "").strip().rstrip("/")

    def _send_json(self, status: int, payload: dict):
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _proxy_to_backend(self):
        backend = self._backend_base_url()
        if not backend:
            self._send_json(
                503,
                {
                    "error_code": "BACKEND_NOT_CONFIGURED",
                    "message": "BACKEND_API_BASE_URL is not configured on the frontend service.",
                    "path": self.path,
                },
            )
            return

        target_url = urljoin(f"{backend}/", self.path.lstrip("/"))
        content_length = int(self.headers.get("Content-Length", "0") or "0")
        payload = self.rfile.read(content_length) if content_length > 0 else None

        proxied_headers = {}
        for name in ("Authorization", "Content-Type", "Accept"):
            value = self.headers.get(name)
            if value:
                proxied_headers[name] = value

        request = urllib.request.Request(
            target_url,
            data=payload,
            headers=proxied_headers,
            method=self.command,
        )

        try:
            with urllib.request.urlopen(request, timeout=15) as response:
                body = response.read()
                self.send_response(response.status)
                content_type = response.headers.get("Content-Type", "application/octet-stream")
                self.send_header("Content-Type", content_type)
                self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
                self.send_header("Content-Length", str(len(body)))
                location = response.headers.get("Location")
                if location:
                    self.send_header("Location", location)
                self.end_headers()
                self.wfile.write(body)
                return
        except urllib.error.HTTPError as error:
            body = error.read()
            self.send_response(error.code)
            self.send_header(
                "Content-Type",
                error.headers.get("Content-Type", "application/octet-stream"),
            )
            self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
            self.send_header("Content-Length", str(len(body)))
            location = error.headers.get("Location")
            if location:
                self.send_header("Location", location)
            self.end_headers()
            self.wfile.write(body)
            return
        except Exception as error:  # pragma: no cover - network/runtime failure path
            self._send_json(
                502,
                {
                    "error_code": "BACKEND_PROXY_FAILED",
                    "message": str(error),
                    "target_url": target_url,
                },
            )
            return

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
        if self.path.startswith(PROXY_PREFIXES):
            return self._proxy_to_backend()
        return super().do_GET()

    def do_PUT(self):
        if self.path.startswith(PROXY_PREFIXES):
            return self._proxy_to_backend()
        self._send_json(
            404,
            {
                "error_code": "ROUTE_NOT_FOUND",
                "message": "This frontend service only proxies known API routes.",
                "path": self.path,
            },
        )


def main():
    port = int(os.getenv("PORT", "3000"))
    server = ThreadingHTTPServer(("0.0.0.0", port), AegisFrontendHandler)
    print(f"frontend-python serving at http://0.0.0.0:{port}")
    server.serve_forever()


if __name__ == "__main__":
    main()

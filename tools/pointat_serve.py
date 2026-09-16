#!/usr/bin/env python3
"""The local runner behind `python3 tools/pointat.py serve`.

It serves the generated weekNN/examples.html pages from this checkout and
compiles and runs what you type into them, with the same toolchain that
produced the committed captures.  Without it the pages run edited code on the
public Rust Playground, which cannot pass program arguments and returns
stdout and stderr separately.

    python3 tools/pointat.py serve            # every week, port 8326
    python3 tools/pointat.py serve week04 --port 9000 --timeout 5

Threat model: this compiles and runs code somebody typed into a browser page,
on the machine that runs it.  It does not sandbox that code, and it is not
meant to: it is your machine and your code, the same as `cargo run`.  What it
does prevent is any OTHER page, user or machine making it run code:

  * it binds 127.0.0.1, so nothing off this machine can reach it;
  * only POST /run compiles anything, and it needs the X-Pointat-Token header
    whose value exists only inside the pages this process served.  A header is
    what makes every cross-origin attempt non-simple: a cross-origin POST of
    text/plain, or a plain <form enctype="text/plain">, reaches a server
    without a preflight, so refusing CORS alone would not stop it.  This
    server answers no OPTIONS and sends no Access-Control-* header, so the
    browser refuses the request before it arrives;
  * the Host header must name loopback and this port, so a hostile hostname
    re-resolved to 127.0.0.1 (DNS rebinding) cannot become same-origin with
    us and read the token out of a page;
  * an Origin, when sent, must be this server; "null" (a file:// page) is
    refused, because such a page should use the Playground.

Each run gets a fresh temporary directory, one run happens at a time, output
is capped while the program is still running, and a program that overruns its
timeout is killed by process group, so nothing it spawned outlives it.
"""

from __future__ import annotations

import argparse
import datetime
import hmac
import html
import json
import os
import re
import secrets
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

TOOL = "pointat/2"
OUTPUT_CAP = 256 * 1024          # bytes of program output kept
SOURCE_CAP = 256 * 1024          # bytes of source accepted
COMPILE_TIMEOUT = 20.0           # seconds
DEFAULT_TIMEOUT = 10.0           # seconds a program may run
STEM_RE = re.compile(r"^[0-9A-Za-z_]{1,64}$")
LOOPBACK = ("127.0.0.1", "localhost", "::1", "[::1]")
MAX_ARGS, MAX_ARG_LEN = 16, 256


def pointat():
    """The generator module, however this process was started."""
    mod = sys.modules.get("pointat")
    if mod is None:
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        import pointat as mod  # noqa: F401  (self-import by design)
    return mod


# ---------------------------------------------------------------------------
# Running a child process without letting it take the machine
# ---------------------------------------------------------------------------

def _kill_group(p: subprocess.Popen) -> None:
    """SIGTERM then SIGKILL the child's whole process group."""
    for sig in (signal.SIGTERM, signal.SIGKILL):
        try:
            os.killpg(os.getpgid(p.pid), sig)
        except (ProcessLookupError, PermissionError, OSError):
            return
        try:
            p.wait(timeout=0.25 if sig == signal.SIGTERM else 2.0)
            return
        except subprocess.TimeoutExpired:
            continue


def spawn_capped(argv: list, cwd, env: dict, timeout: float, cap: int | None = OUTPUT_CAP,
                 executable: str | None = None) -> dict:
    """Run `argv`, merging stdout and stderr, and stop it if it runs too long
    or says too much.

    Output goes to a temporary file rather than a pipe, so the size can be
    checked while the program is still running and a runaway `loop { println!()
    }` cannot fill this process's memory.  The child gets its own session, so
    anything it spawns dies with it.
    """
    t0 = time.monotonic()
    with tempfile.TemporaryFile() as sink:
        p = subprocess.Popen(argv, executable=executable, cwd=str(cwd), env=env,
                             stdin=subprocess.DEVNULL, stdout=sink, stderr=subprocess.STDOUT,
                             start_new_session=True)
        timed_out = truncated = False
        while p.poll() is None:
            if time.monotonic() - t0 > timeout:
                timed_out = True
            elif cap is not None and os.fstat(sink.fileno()).st_size > cap:
                truncated = True
            if timed_out or truncated:
                _kill_group(p)
                break
            time.sleep(0.05)
        sink.seek(0)
        data = sink.read()
    if cap is not None and len(data) > cap:
        data, truncated = data[:cap], True
    code = p.returncode
    sig = -code if code is not None and code < 0 else None
    return {"data": data, "exit": None if timed_out or sig else code, "signal": sig,
            "timed_out": timed_out, "truncated": truncated,
            "ms": int((time.monotonic() - t0) * 1000), "pid": p.pid}


# ---------------------------------------------------------------------------
# Compile and run one edited program
# ---------------------------------------------------------------------------

def stage(tmp: Path, stem: str, source: str) -> Path:
    """Lay the source out the way the crate does, so diagnostics and panic
    lines read src/bin/<stem>.rs exactly as the captures do."""
    rel = Path("src/bin") / f"{stem}.rs"
    (tmp / rel).parent.mkdir(parents=True, exist_ok=True)
    (tmp / "target/debug").mkdir(parents=True, exist_ok=True)
    (tmp / rel).write_text(source, encoding="utf-8")
    return rel


def compile_one(tmp: Path, stem: str, edition: str, env: dict,
                timeout: float = COMPILE_TIMEOUT) -> dict:
    """rustc the staged file.  Returns the capture's diagnostics shape."""
    pa = pointat()
    rel = Path("src/bin") / f"{stem}.rs"
    argv = ["rustc", "--edition", edition, "--crate-name", stem,
            "-C", "debug-assertions=on", "-C", "overflow-checks=on", "-C", "debuginfo=0",
            "--error-format=json", "-o", f"target/debug/{stem}", str(rel)]
    res = spawn_capped(argv, tmp, env, timeout, cap=OUTPUT_CAP)
    rendered, diags, errors = [], [], 0
    for line in res["data"].decode("utf-8", "replace").splitlines(keepends=True):
        if not line.startswith("{"):
            rendered.append(line)
            continue
        try:
            obj = json.loads(line)
        except ValueError:
            rendered.append(line)
            continue
        if obj.get("$message_type", "diagnostic") != "diagnostic":
            continue
        rendered.append(obj.get("rendered") or "")
        if obj.get("level") in ("warning", "error"):
            diags.append(pa.flatten_diag(obj, f"{stem}.rs"))
            errors += obj.get("level") == "error"
    exe = tmp / "target/debug" / stem
    return {"compiled": exe.exists() and errors == 0, "diagnostics": diags,
            "rendered": "".join(rendered), "errors": errors,
            "timed_out": res["timed_out"], "ms": res["ms"]}


def run_once(tmp: Path, stem: str, args: list, env: dict, timeout: float) -> dict:
    """Run the compiled program the way `cargo run` would show it."""
    exe = tmp / "target/debug" / stem
    argv0 = f"target/debug/{stem}"
    res = spawn_capped([argv0, *args], tmp, env, timeout, cap=OUTPUT_CAP, executable=str(exe))
    out = res["data"].decode("utf-8", "replace")
    return {"output": out, "exit": res["exit"], "signal": res["signal"],
            "timed_out": res["timed_out"], "truncated": res["truncated"],
            "replacement_chars": out.count("�"), "ms": res["ms"]}


def build_and_run(week_dir: Path, stem: str, source: str, args: list, edition: str,
                  extra_env: dict, timeout: float) -> dict:
    """compile_one + run_once in a temporary directory that is then removed."""
    pa = pointat()
    env = dict(pa.run_env(), **(extra_env or {}))
    tmp = Path(tempfile.mkdtemp(prefix="pointat-run-"))
    try:
        stage(tmp, stem, source)
        built = compile_one(tmp, stem, edition, env, timeout=COMPILE_TIMEOUT)
        out = {"ok": True, "runner": "local", "file": f"src/bin/{stem}.rs",
               "edition": edition, "compiled": built["compiled"],
               "diagnostics": built["diagnostics"], "rendered": built["rendered"],
               "compile_ms": built["ms"], "compile_timed_out": built["timed_out"]}
        if not built["compiled"]:
            out.update(output="", exit=None, signal=None, timed_out=False, truncated=False,
                       replacement_chars=0, run_ms=0)
            return out
        ran = run_once(tmp, stem, args, env, timeout)
        out.update(output=ran["output"], exit=ran["exit"], signal=ran["signal"],
                   timed_out=ran["timed_out"], truncated=ran["truncated"],
                   replacement_chars=ran["replacement_chars"], run_ms=ran["ms"])
        return out
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ---------------------------------------------------------------------------
# The server
# ---------------------------------------------------------------------------

EDITION_RE = re.compile(r'^\s*edition\s*=\s*"(\d{4})"', re.M)


class ServeState:
    """What the handler needs: which weeks, the token, the page cache, one lock."""

    def __init__(self, weeks: list, port: int, timeout: float):
        pa = pointat()
        self.weeks = weeks
        self.port = port
        self.timeout = timeout
        self.token = secrets.token_hex(16)
        self.rustc = pa.tool_version(["rustc", "--version"])
        self.lock = threading.Lock()
        self.pages: dict = {}          # week -> (fingerprint, html)
        self.editions: dict = {}       # week -> "2021"

    def edition(self, week: str) -> str:
        if week not in self.editions:
            pa = pointat()
            toml = pa.REPO / week / "examples" / "Cargo.toml"
            m = EDITION_RE.search(toml.read_text(encoding="utf-8")) if toml.exists() else None
            self.editions[week] = m.group(1) if m else "2021"
        return self.editions[week]

    def stems(self, week: str) -> list:
        pa = pointat()
        return sorted(p.stem for p in (pa.REPO / week / "examples" / "src" / "bin").glob("*.rs"))

    def fingerprint(self, week: str) -> float:
        pa = pointat()
        ex = pa.REPO / week / "examples"
        paths = [*(ex / "src" / "bin").glob("*.rs"), *(ex / "broken").glob("*.rs"),
                 pa.REPO / week / "README.md", pa.captures_path(week),
                 pa.HERE / "pointat.css", pa.HERE / "pointat.js", pa.HERE / "pointat.edit.js",
                 pa.HERE / "pointat.py"]
        return max((p.stat().st_mtime for p in paths if p.exists()), default=0.0)

    def page(self, week: str) -> str:
        """The week's page, rendered from what is on disk right now, with the
        local runner and this process's token in its config."""
        pa = pointat()
        fp = self.fingerprint(week)
        hit = self.pages.get(week)
        if hit and hit[0] == fp:
            return hit[1]
        html_text = pa.build_page(week, pa.SERVE_BACKLINK, cfg={
            "runner": "local", "endpoint": "/run", "token": self.token,
            "timeout": self.timeout, "rustc": self.rustc,
        })
        self.pages[week] = (fp, html_text)
        return html_text


def _json_bytes(obj) -> bytes:
    return (json.dumps(obj, ensure_ascii=False) + "\n").encode("utf-8")


class Handler(BaseHTTPRequestHandler):
    server_version = "pointat"
    sys_version = ""
    protocol_version = "HTTP/1.1"
    state: ServeState = None          # set by build_server

    # ---- plumbing --------------------------------------------------------
    def log_message(self, fmt, *a):   # no token can appear: it is a header
        sys.stderr.write("  %s %s\n" % (self.address_string(), fmt % a))

    def _send(self, code: int, body: bytes, ctype: str) -> None:
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        if code >= 400:
            # A refusal happens before the body is read, so the rest of it is
            # still on the wire: end the connection rather than mistake those
            # bytes for the next request.
            self.send_header("Connection", "close")
            self.close_connection = True
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Referrer-Policy", "no-referrer")
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)

    def _json(self, code: int, obj) -> None:
        self._send(code, _json_bytes(obj), "application/json; charset=utf-8")

    def _html(self, code: int, text: str) -> None:
        self._send(code, text.encode("utf-8"), "text/html; charset=utf-8")

    def _fail(self, code: int, error: str) -> None:
        self._json(code, {"ok": False, "error": error})

    # ---- the checks that keep other pages out ----------------------------
    def _host_ok(self) -> bool:
        host = (self.headers.get("Host") or "").strip()
        if not host:
            return False
        name, _, port = host.rpartition(":")
        if not name:                                   # "localhost" with no port
            name, port = host, str(self.state.port)
        return name.strip("[]") in ("127.0.0.1", "localhost", "::1") and port == str(self.state.port)

    def _origin_ok(self) -> bool:
        origin = self.headers.get("Origin")
        if origin is None:
            return True                                # a same-origin navigation
        return origin in (f"http://127.0.0.1:{self.state.port}",
                          f"http://localhost:{self.state.port}",
                          f"http://[::1]:{self.state.port}")

    def _token_ok(self) -> bool:
        got = self.headers.get("X-Pointat-Token") or ""
        return hmac.compare_digest(got, self.state.token)

    def _gate(self) -> bool:
        """Every request passes here first."""
        if not self._host_ok():
            self._fail(403, "bad_host")
            return False
        if not self._origin_ok():
            self._fail(403, "bad_origin")
            return False
        return True

    # ---- routes ----------------------------------------------------------
    def do_OPTIONS(self):             # no CORS, on purpose
        if self._gate():
            self._fail(405, "method_not_allowed")

    def do_HEAD(self):
        self.do_GET()

    def do_GET(self):
        if not self._gate():
            return
        path = self.path.split("?", 1)[0].rstrip("/") or "/"
        if path == "/":
            self._html(200, self.index_html())
            return
        if path == "/health":
            self._json(200, {"tool": TOOL, "weeks": self.state.weeks,
                             "rustc": self.state.rustc, "port": self.state.port})
            return
        m = re.fullmatch(r"/(week\d\d)(?:/examples\.html)?", path)
        if m and m.group(1) in self.state.weeks:
            try:
                self._html(200, self.state.page(m.group(1)))
            except Exception as exc:                   # a missing capture, a bad README
                self._html(500, f"<h1>pointat</h1><pre>{html.escape(str(exc))}</pre>")
            return
        self._fail(404, "not_found")

    def do_POST(self):
        if not self._gate():
            return
        if self.path.split("?", 1)[0].rstrip("/") != "/run":
            self._fail(404, "not_found")
            return
        if not self._token_ok():
            self._fail(403, "bad_token")
            return
        ctype = (self.headers.get("Content-Type") or "").split(";")[0].strip().lower()
        if ctype != "application/json":
            self._fail(415, "want_json")
            return
        if self.headers.get("Transfer-Encoding"):
            self._fail(411, "want_content_length")
            return
        try:
            length = int(self.headers.get("Content-Length") or "-1")
        except ValueError:
            length = -1
        if length < 0:
            self._fail(411, "want_content_length")
            return
        if length > SOURCE_CAP:
            self._fail(413, "too_large")
            return
        try:
            req = json.loads(self.rfile.read(length).decode("utf-8"))
        except (ValueError, UnicodeDecodeError):
            self._fail(400, "bad_json")
            return
        if not self.state.lock.acquire(blocking=False):
            self.send_response(409)
            body = _json_bytes({"ok": False, "error": "busy"})
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Retry-After", "1")
            self.end_headers()
            self.wfile.write(body)
            return
        try:
            code, obj = self.run_request(req)
        finally:
            self.state.lock.release()
        self._json(code, obj)

    # ---- the run ---------------------------------------------------------
    def run_request(self, req: dict) -> tuple:
        pa = pointat()
        st = self.state
        week, stem = req.get("week"), req.get("stem")
        if week not in st.weeks:
            return 404, {"ok": False, "error": "unknown_week"}
        if not isinstance(stem, str) or not STEM_RE.match(stem) or stem not in st.stems(week):
            return 404, {"ok": False, "error": "unknown_program"}
        source = req.get("code")
        if not isinstance(source, str) or not source.strip():
            return 400, {"ok": False, "error": "no_code"}
        if len(source.encode("utf-8")) > SOURCE_CAP:
            return 413, {"ok": False, "error": "too_large"}
        path = pa.REPO / week / "examples" / "src" / "bin" / f"{stem}.rs"
        runs = pa.load_program(path).runs
        which = req.get("run") if isinstance(req.get("run"), int) else 1
        cmd, run_args, run_env_extra = runs[which - 1] if 1 <= which <= len(runs) else runs[0]
        args = req.get("args")
        if args is None:
            args = run_args
        if (not isinstance(args, list) or len(args) > MAX_ARGS
                or any(not isinstance(a, str) or len(a) > MAX_ARG_LEN for a in args)):
            return 400, {"ok": False, "error": "bad_args"}
        try:
            timeout = float(req.get("timeout") or st.timeout)
        except (TypeError, ValueError):
            timeout = st.timeout
        timeout = max(1.0, min(timeout, st.timeout))
        out = build_and_run(pa.REPO / week, stem, source, args, st.edition(week),
                            run_env_extra, timeout)
        out.update(cmd=cmd, rustc=st.rustc, week=week, stem=stem, run=which, args=args,
                   ran_at=datetime.datetime.now().astimezone().isoformat(timespec="seconds"))
        return 200, out

    def index_html(self) -> str:
        rows = "".join(
            f'<li><a href="/{w}/examples.html">{w} &middot; code and output</a></li>'
            for w in self.state.weeks)
        return ("<!DOCTYPE html><html lang=en><meta charset=utf-8>"
                "<title>pointat</title>"
                "<style>body{font:16px/1.5 system-ui,sans-serif;margin:3em auto;max-width:40em}"
                "a{color:#00543c}code{background:#f2f4f6;padding:0 .25em}</style>"
                "<h1>pointat</h1><p>Editing and running are live on these pages: what you "
                "type runs here, with <code>" + html.escape(self.state.rustc) + "</code>.</p>"
                f"<ul>{rows}</ul>"
                "<p>Stop the server with <kbd>Ctrl+C</kbd>.</p></html>")


def build_server(weeks: list, port: int = 8326, timeout: float = DEFAULT_TIMEOUT):
    """(httpd, state), bound to loopback only.  port 0 picks a free one."""
    state = ServeState(weeks, port, timeout)
    handler = type("BoundHandler", (Handler,), {"state": state})
    httpd = ThreadingHTTPServer(("127.0.0.1", port), handler)
    httpd.daemon_threads = True
    state.port = httpd.server_address[1]
    return httpd, state


def serve_main(argv: list) -> int:
    pa = pointat()
    ap = argparse.ArgumentParser(prog="pointat serve", description=__doc__.split("\n\n")[0],
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("weeks", nargs="*", help="weeks to serve (default: all of them)")
    ap.add_argument("--port", type=int, default=8326, help="port on 127.0.0.1 (default 8326)")
    ap.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT,
                    help="seconds a program may run (default 10)")
    ap.add_argument("--open", action="store_true", help="open the first week in a browser")
    args = ap.parse_args(argv)
    weeks = [pa.normalize_week(w) for w in args.weeks] if args.weeks else sorted(
        d.name for d in pa.REPO.glob("week[0-9]*") if (d / "examples" / "Cargo.toml").exists())
    missing = [w for w in weeks if not pa.captures_path(w).exists()]
    if missing:
        pa.log(f"  ! no captures for {', '.join(missing)}; run: python3 tools/pointat.py {' '.join(missing)}")
        return 1
    try:
        httpd, state = build_server(weeks, args.port, args.timeout)
    except OSError as exc:
        pa.log(f"  ! cannot listen on 127.0.0.1:{args.port}: {exc}")
        return 1
    base = f"http://127.0.0.1:{state.port}"
    pa.log(f"pointat serve · {state.rustc} · {', '.join(weeks)}")
    for w in weeks:
        pa.log(f"  {base}/{w}/examples.html")
    pa.log("  editing and running are live on these pages; Ctrl+C to stop")
    if args.open:
        import webbrowser
        webbrowser.open(f"{base}/{weeks[0]}/examples.html")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pa.log("stopped")
    finally:
        httpd.server_close()
    return 0


if __name__ == "__main__":
    sys.exit(serve_main(sys.argv[1:]))

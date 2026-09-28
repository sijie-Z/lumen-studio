from __future__ import annotations

import hashlib
import os
import shutil
import socket
import sqlite3
import subprocess
import sys
import time
from contextlib import contextmanager
from pathlib import Path
from urllib.error import URLError
from urllib.request import urlopen


WEB_DIR = Path(__file__).resolve().parents[1]
PROJECT_ROOT = WEB_DIR.parent
BACKEND_DIR = PROJECT_ROOT / "backend"
MAIN_DB = BACKEND_DIR / "photography.db"
TEST_DB = BACKEND_DIR / "photography_test.db"
SMOKE_DIR = WEB_DIR / ".smoke"
LOG_DIR = SMOKE_DIR / "logs"
TEST_UPLOADS = SMOKE_DIR / "uploads"

API_URL = "http://127.0.0.1:8080"
WEB_URL = "http://127.0.0.1:5173"


def _creation_flags() -> int:
    if os.name == "nt":
        return subprocess.CREATE_NO_WINDOW
    return 0


def _pnpm_command() -> str:
    for name in ("pnpm.cmd", "pnpm"):
        command = shutil.which(name)
        if command:
            return command
    raise RuntimeError("未找到 pnpm，请先安装并确保它在 PATH 中。")


def _api_command() -> list[str]:
    configured = os.environ.get("LUMINA_TEST_API_BINARY")
    if configured:
        return [configured]
    return ["cargo", "run", "-p", "api"]


def _port_is_open(port: int) -> bool:
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=0.3):
            return True
    except OSError:
        return False


def _ensure_port_free(port: int, service_name: str) -> None:
    if _port_is_open(port):
        raise RuntimeError(
            f"端口 {port} 已被占用（{service_name}）。"
            "请先停止现有服务，避免隔离测试误连到主数据库。"
        )


def _remove_file(path: Path) -> None:
    if path.exists():
        path.unlink()


def reset_test_database() -> None:
    TEST_DB.parent.mkdir(parents=True, exist_ok=True)
    for suffix in ("", "-wal", "-shm"):
        _remove_file(Path(f"{TEST_DB}{suffix}"))


def _file_fingerprint(path: Path) -> dict[str, object] | None:
    if not path.exists():
        return None
    digest = hashlib.sha256()
    with path.open("rb") as file:
        for chunk in iter(lambda: file.read(1024 * 1024), b""):
            digest.update(chunk)
    return {"size": path.stat().st_size, "sha256": digest.hexdigest()}


def main_database_snapshot() -> dict[str, object] | None:
    if not MAIN_DB.exists():
        return None

    uri = f"{MAIN_DB.as_uri()}?mode=ro"
    with sqlite3.connect(uri, uri=True) as connection:
        tables = {
            row[0]
            for row in connection.execute(
                "SELECT name FROM sqlite_master WHERE type = 'table'"
            )
        }
        customer = connection.execute(
            "SELECT balance FROM users WHERE username = 'customer'"
        ).fetchone() if "users" in tables else None
        snapshot: dict[str, object] = {
            "customer_balance": str(customer[0]) if customer else None
        }
        for table in ("users", "appointments", "payments", "prepaids", "reviews", "works"):
            if table in tables:
                snapshot[table] = connection.execute(
                    f"SELECT COUNT(*) FROM {table}"
                ).fetchone()[0]
        snapshot["files"] = {
            path.name: _file_fingerprint(path)
            for path in (
                MAIN_DB,
                Path(f"{MAIN_DB}-wal"),
                Path(f"{MAIN_DB}-shm"),
            )
        }
        return snapshot


def _tail(path: Path, lines: int = 80) -> str:
    if not path.exists():
        return f"{path} 不存在"

    content = path.read_text(encoding="utf-8", errors="replace").splitlines()
    return "\n".join(content[-lines:])


def _wait_for_url(
    url: str,
    process: subprocess.Popen[bytes],
    service_name: str,
    timeout: float,
    log_path: Path,
) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(
                f"{service_name} 提前退出，退出码 {process.returncode}。\n"
                f"日志：\n{_tail(log_path)}"
            )
        try:
            with urlopen(url, timeout=2) as response:
                if response.status < 500:
                    return
        except (OSError, URLError):
            pass
        time.sleep(0.3)

    raise RuntimeError(
        f"{service_name} 在 {timeout:.0f} 秒内未就绪。\n日志：\n{_tail(log_path)}"
    )


def _terminate_process(process: subprocess.Popen[bytes] | None) -> None:
    if process is None or process.poll() is not None:
        return

    if os.name == "nt":
        subprocess.run(
            ["taskkill", "/PID", str(process.pid), "/T", "/F"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    else:
        process.terminate()

    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=10)


@contextmanager
def managed_test_environment():
    _ensure_port_free(8080, "Rust API")
    _ensure_port_free(5173, "Vite")
    reset_test_database()
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    TEST_UPLOADS.mkdir(parents=True, exist_ok=True)

    backend_log = LOG_DIR / "e2e-backend.log"
    frontend_log = LOG_DIR / "e2e-frontend.log"
    backend_output = backend_log.open("wb")
    frontend_output = frontend_log.open("wb")
    backend_process: subprocess.Popen[bytes] | None = None
    frontend_process: subprocess.Popen[bytes] | None = None

    backend_env = os.environ.copy()
    backend_env.update(
        {
            "DATABASE_URL": "sqlite://photography_test.db?mode=rwc",
            "BIND_ADDR": "127.0.0.1:8080",
            "UPLOAD_DIR": str(TEST_UPLOADS),
            "JWT_SECRET": "test-only-jwt-secret-0123456789abcdef",
            "RUST_LOG": "common=error,api=warn,tower_http=warn",
        }
    )

    try:
        print(f"test database: {TEST_DB}")
        backend_process = subprocess.Popen(
            _api_command(),
            cwd=BACKEND_DIR,
            env=backend_env,
            stdout=backend_output,
            stderr=subprocess.STDOUT,
            creationflags=_creation_flags(),
        )
        _wait_for_url(
            f"{API_URL}/health",
            backend_process,
            "Rust API",
            timeout=180,
            log_path=backend_log,
        )

        frontend_process = subprocess.Popen(
            [
                _pnpm_command(),
                "dev",
                "--host",
                "127.0.0.1",
                "--port",
                "5173",
            ],
            cwd=WEB_DIR,
            stdout=frontend_output,
            stderr=subprocess.STDOUT,
            creationflags=_creation_flags(),
        )
        _wait_for_url(
            WEB_URL,
            frontend_process,
            "Vite",
            timeout=60,
            log_path=frontend_log,
        )
        yield
    finally:
        _terminate_process(frontend_process)
        _terminate_process(backend_process)
        frontend_output.close()
        backend_output.close()


def print_main_database_comparison(
    before: dict[str, object] | None, after: dict[str, object] | None
) -> None:
    print(f"main database before: {before}")
    print(f"main database after:  {after}")
    if before != after:
        print("主数据库在隔离测试期间发生变化。", file=sys.stderr)
        raise AssertionError("isolated test polluted backend/photography.db")

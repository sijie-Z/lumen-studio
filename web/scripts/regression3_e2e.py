import re
import socket
import sys
from pathlib import Path
from urllib.parse import urlparse

from playwright.sync_api import Page, expect, sync_playwright
from test_runtime import (
    main_database_snapshot,
    managed_test_environment,
    print_main_database_comparison,
)

ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / ".smoke"
WEB_BASE = "http://127.0.0.1:5173"
API_BASE = "http://127.0.0.1:8080"
API_PREFIX = f"{API_BASE}/api/v1"

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")


def screenshot(page: Page, name: str) -> Path:
    OUT_DIR.mkdir(exist_ok=True)
    path = OUT_DIR / f"regression3-{name}.png"
    page.screenshot(path=str(path), full_page=False)
    print(f"screenshot: {path}")
    return path


def tracked_page(context, errors: dict[str, list[str]]) -> Page:
    page = context.new_page()
    page.on(
        "console",
        lambda msg: errors["console"].append(msg.text) if msg.type == "error" else None,
    )
    page.on("pageerror", lambda error: errors["page"].append(str(error)))
    page.on(
        "response",
        lambda response: errors["api"].append(
            f"{response.status} {response.request.method} {response.url}"
        )
        if "/api/" in response.url and response.status >= 400
        else None,
    )
    return page


def path_of(page: Page) -> str:
    return urlparse(page.url).path


def wait_for_path(page: Page, path: str) -> None:
    page.wait_for_url(lambda url: urlparse(str(url)).path == path, timeout=20000)


def clear_session(page: Page, path: str = "/") -> None:
    page.goto(f"{WEB_BASE}{path}", wait_until="domcontentloaded")
    page.evaluate("localStorage.clear()")
    page.reload(wait_until="domcontentloaded")


def login(page: Page, account: str, password: str, expected_path: str) -> None:
    clear_session(page, "/login")
    page.wait_for_selector("form", timeout=15000)
    page.locator('input[placeholder="name@example.com"]').fill(account)
    page.locator('input[placeholder="至少 8 位"]').fill(password)
    page.locator("form").get_by_role("button", name="登录", exact=True).click()
    wait_for_path(page, expected_path)


def api_data(page: Page, path: str, authenticated: bool = False):
    headers = {}
    if authenticated:
        token = page.evaluate("localStorage.getItem('lumina.token')")
        assert token, "expected authenticated browser session"
        headers["Authorization"] = f"Bearer {token}"
    response = page.request.get(f"{API_PREFIX}{path}", headers=headers)
    assert response.ok, f"GET {path} failed: {response.status} {response.text()}"
    body = response.json()
    assert body.get("code") == 200, f"GET {path} returned {body}"
    return body.get("data")


def verify_route_guards(page: Page) -> None:
    clear_session(page)
    for protected_path in ("/account", "/dashboard", "/admin"):
        page.goto(f"{WEB_BASE}{protected_path}", wait_until="domcontentloaded")
        wait_for_path(page, "/login")
        page.wait_for_selector("text=欢迎回来", timeout=15000)
    screenshot(page, "01-unauth-guards")

    login(page, "customer", "customer123", "/account")
    page.goto(f"{WEB_BASE}/admin", wait_until="domcontentloaded")
    wait_for_path(page, "/")
    screenshot(page, "02-customer-admin-denied")


def verify_role_redirects(page: Page) -> None:
    login(page, "admin", "admin123", "/admin")
    expect(page.get_by_role("link", name="平台概览", exact=True)).to_be_visible()
    screenshot(page, "03-admin-redirect")

    login(page, "chenyu", "creator123", "/dashboard")
    expect(page.get_by_role("link", name="工作台", exact=True)).to_be_visible()
    screenshot(page, "04-creator-redirect")

    login(page, "customer", "customer123", "/account")
    expect(page.get_by_role("link", name="我的订单", exact=True)).to_be_visible()
    screenshot(page, "05-customer-redirect")


def verify_creator_identity(page: Page) -> None:
    creators = api_data(page, "/creators?page=1&page_size=100")["items"]
    chenyu = next(item for item in creators if item["nickname"] == "陈屿")
    assert chenyu["avatar_url"], f"expected avatar_url for 陈屿: {chenyu}"

    page.goto(f"{WEB_BASE}/creators/{chenyu['id']}", wait_until="domcontentloaded")
    expect(page.get_by_role("heading", name="陈屿", exact=True)).to_be_visible(timeout=20000)
    expect(page.get_by_text(re.compile(r"创作者 #\d+"))).to_have_count(0)
    screenshot(page, "06-creator-real-identity")


def verify_work_detail_endpoint(page: Page) -> None:
    works = api_data(page, "/works?page=1&page_size=100")["items"]
    work = works[0]
    work_path = f"/api/v1/works/{work['id']}"

    with page.expect_response(
        lambda response: response.request.method == "GET"
        and urlparse(response.url).path == work_path
        and response.status == 200,
        timeout=20000,
    ):
        page.goto(f"{WEB_BASE}/works/{work['id']}", wait_until="domcontentloaded")

    expect(page.get_by_role("heading", name=work["title"], exact=True)).to_be_visible(
        timeout=20000
    )
    screenshot(page, "07-work-detail-by-id")


def assert_ports_released() -> None:
    for port, label in ((8080, "Rust API"), (5173, "Vite")):
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.3):
                raise AssertionError(f"{label} port {port} is still open after regression run")
        except OSError:
            pass


def main() -> None:
    OUT_DIR.mkdir(exist_ok=True)
    errors = {"console": [], "page": [], "api": []}

    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=True,
            channel="chrome",
            args=["--disable-gpu", "--disable-dev-shm-usage"],
        )
        context = browser.new_context(viewport={"width": 1440, "height": 1000})
        page = tracked_page(context, errors)
        page.set_default_navigation_timeout(120000)

        try:
            health = page.request.get(f"{API_BASE}/health")
            assert health.ok and health.json().get("data", {}).get("status") == "ok"

            verify_route_guards(page)
            verify_role_redirects(page)
            verify_creator_identity(page)
            verify_work_detail_endpoint(page)
        except Exception:
            for category, messages in errors.items():
                print(f"{category} errors:")
                for message in messages:
                    print(message)
            try:
                screenshot(page, "failure")
                print(f"failure url: {page.url}")
                print(page.locator("body").inner_text(timeout=5000))
            except Exception as capture_error:
                print(f"failed to capture failure: {capture_error}")
            raise
        finally:
            browser.close()

    if errors["page"]:
        raise AssertionError(f"browser page errors: {errors['page']}")
    if errors["api"]:
        raise AssertionError(f"unexpected API errors: {errors['api']}")
    if errors["console"]:
        print("console errors:")
        for message in errors["console"]:
            print(message)

    print("Regression3 E2E passed: roles, guards, creator identity, work detail")


if __name__ == "__main__":
    try:
        before = main_database_snapshot()
        with managed_test_environment():
            main()
        after = main_database_snapshot()
        print_main_database_comparison(before, after)
        assert_ports_released()
        print("ports released: 8080, 5173")
        print(f"main database unchanged: {before == after}")
    except Exception as error:
        print(f"REGRESSION3 E2E FAILED: {error}", file=sys.stderr)
        raise
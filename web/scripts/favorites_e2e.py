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
    path = OUT_DIR / f"favorites-{name}.png"
    page.screenshot(path=str(path), full_page=True)
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


def wait_for_path(page: Page, path: str) -> None:
    page.wait_for_url(lambda url: urlparse(str(url)).path == path, timeout=20000)


def login(page: Page, account: str, password: str, expected_path: str) -> None:
    page.goto(f"{WEB_BASE}/login", wait_until="domcontentloaded")
    page.wait_for_selector("text=欢迎回来", timeout=15000)
    page.locator('input[placeholder="name@example.com"]').fill(account)
    page.locator('input[placeholder="至少 8 位"]').fill(password)
    page.get_by_role("button", name="登录", exact=True).click()
    wait_for_path(page, expected_path)


def api_data(page: Page, path: str):
    response = page.request.get(f"{API_PREFIX}{path}")
    assert response.ok, f"GET {path} failed: {response.status} {response.text()}"
    body = response.json()
    assert body.get("code") == 200, f"GET {path} returned {body}"
    return body.get("data")


def expect_favorite_button(page: Page, label: str):
    button = page.get_by_role("button", name=re.compile(label))
    button.wait_for(timeout=15000)
    return button


def assert_ports_released() -> None:
    for port, label in ((8080, "Rust API"), (5173, "Vite")):
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.3):
                raise AssertionError(f"{label} port {port} is still open after favorites run")
        except OSError:
            pass


def main() -> None:
    OUT_DIR.mkdir(exist_ok=True)
    errors = {"console": [], "page": [], "api": []}

    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=True,
            channel="chrome",
            args=["--disable-gpu", "--disable-dev-shm-usage", "--no-sandbox"],
        )
        context = browser.new_context(viewport={"width": 1440, "height": 1000})
        page = tracked_page(context, errors)
        page.set_default_navigation_timeout(120000)

        try:
            health = page.request.get(f"{API_BASE}/health")
            assert health.ok and health.json().get("data", {}).get("status") == "ok"

            works = api_data(page, "/works?page=1&page_size=100")["items"]
            work = next(item for item in works if item.get("title"))
            services = api_data(page, "/services?page=1&page_size=100")["items"]
            service = next(item for item in services if item.get("title"))
            creators = api_data(page, "/creators?page=1&page_size=100")["items"]
            creator = next(item for item in creators if item.get("nickname"))

            page.goto(WEB_BASE, wait_until="domcontentloaded")
            page.evaluate("localStorage.clear()")
            page.goto(f"{WEB_BASE}/works/{work['id']}", wait_until="domcontentloaded")
            expect(page.get_by_role("heading", name=work["title"], exact=True)).to_be_visible(
                timeout=20000
            )
            expect_favorite_button(page, "收藏作品").click()
            wait_for_path(page, "/login")
            expect(page.get_by_text("欢迎回来", exact=True)).to_be_visible(timeout=15000)
            screenshot(page, "01-unauth-redirect")

            login(page, "customer", "customer123", "/account")
            page.goto(f"{WEB_BASE}/works/{work['id']}", wait_until="domcontentloaded")
            work_button = expect_favorite_button(page, "收藏作品")
            work_button.click()
            work_button = page.get_by_role("button", name=re.compile("已收藏"))
            expect(work_button).to_have_attribute("aria-pressed", "true", timeout=15000)
            expect(work_button.locator("span").last).to_have_text(re.compile(r"[1-9]\d*"))
            screenshot(page, "02-work-favorited")

            work_button.click()
            expect(page.get_by_role("button", name=re.compile("收藏作品"))).to_have_attribute(
                "aria-pressed", "false", timeout=15000
            )
            page.get_by_role("button", name=re.compile("收藏作品")).click()
            expect(page.get_by_role("button", name=re.compile("已收藏"))).to_have_attribute(
                "aria-pressed", "true", timeout=15000
            )

            page.goto(f"{WEB_BASE}/creators/{creator['id']}", wait_until="domcontentloaded")
            expect(page.get_by_role("heading", name=creator["nickname"], exact=True)).to_be_visible(
                timeout=20000
            )
            expect_favorite_button(page, "收藏创作者").click()
            expect(
                page.get_by_role("button", name=re.compile("已收藏创作者"))
            ).to_have_attribute("aria-pressed", "true", timeout=15000)
            screenshot(page, "03-creator-favorited")

            page.goto(f"{WEB_BASE}/services/{service['id']}", wait_until="domcontentloaded")
            expect(page.get_by_role("heading", name=service["title"], exact=True)).to_be_visible(
                timeout=20000
            )
            expect_favorite_button(page, "收藏服务").click()
            expect(page.get_by_role("button", name=re.compile("已收藏服务"))).to_have_attribute(
                "aria-pressed", "true", timeout=15000
            )
            screenshot(page, "04-service-favorited")

            page.goto(f"{WEB_BASE}/account", wait_until="domcontentloaded")
            favorites_section = page.locator("section").filter(has_text="我的收藏").first
            favorites_section.wait_for(timeout=15000)
            expect(
                favorites_section.locator(f'a[href="/works/{work["id"]}"]')
            ).to_be_visible(timeout=15000)
            expect(
                favorites_section.locator(f'a[href="/services/{service["id"]}"]')
            ).to_be_visible(timeout=15000)
            expect(
                favorites_section.locator(f'a[href="/creators/{creator["id"]}"]')
            ).to_be_visible(timeout=15000)
            screenshot(page, "05-account-all")

            favorites_section.get_by_role("button", name="作品", exact=True).click()
            expect(
                favorites_section.locator(f'a[href="/works/{work["id"]}"]')
            ).to_be_visible(timeout=15000)
            expect(
                favorites_section.locator(f'a[href="/services/{service["id"]}"]')
            ).to_have_count(0)

            favorites_section.get_by_role("button", name="服务", exact=True).click()
            expect(
                favorites_section.locator(f'a[href="/services/{service["id"]}"]')
            ).to_be_visible(timeout=15000)

            favorites_section.get_by_role("button", name="创作者", exact=True).click()
            expect(
                favorites_section.locator(f'a[href="/creators/{creator["id"]}"]')
            ).to_be_visible(timeout=15000)
            screenshot(page, "06-account-filter-creators")

            page.goto(f"{WEB_BASE}/services/{service['id']}", wait_until="domcontentloaded")
            service_button = expect_favorite_button(page, "已收藏服务")
            service_button.click()
            expect(page.get_by_role("button", name=re.compile("收藏服务"))).to_have_attribute(
                "aria-pressed", "false", timeout=15000
            )

            page.goto(f"{WEB_BASE}/account", wait_until="domcontentloaded")
            favorites_section = page.locator("section").filter(has_text="我的收藏").first
            favorites_section.wait_for(timeout=15000)
            expect(favorites_section.get_by_text(service["title"], exact=True)).to_have_count(0)
            expect(
                favorites_section.locator(f'a[href="/works/{work["id"]}"]')
            ).to_be_visible(timeout=15000)
            expect(
                favorites_section.locator(f'a[href="/creators/{creator["id"]}"]')
            ).to_be_visible(timeout=15000)
            screenshot(page, "07-account-after-remove")
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

    print("Favorites E2E passed: unauth redirect, toggle, joined account list, filters")


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
        print(f"FAVORITES E2E FAILED: {error}", file=sys.stderr)
        raise

import re
import sys
from pathlib import Path

from playwright.sync_api import Page, sync_playwright
from test_runtime import (
    main_database_snapshot,
    managed_test_environment,
    print_main_database_comparison,
)


ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / ".smoke"
WEB_BASE = "http://127.0.0.1:5173"
API_BASE = "http://127.0.0.1:8080"
APPOINTMENT_DATE = "2026-12-20T10:00"
APPOINTMENT_YEAR = "2026"
SERVICE_TITLE = "城市自然光人像写真"


def screenshot(page: Page, name: str) -> Path:
    OUT_DIR.mkdir(exist_ok=True)
    path = OUT_DIR / f"e2e-{name}.png"
    page.screenshot(path=str(path), full_page=True)
    print(f"screenshot: {path}")
    return path


def goto_and_wait_api(
    page: Page, url: str, method: str, api_fragment: str
) -> None:
    with page.expect_response(
        lambda response: response.request.method == method and api_fragment in response.url,
        timeout=20000,
    ) as response_info:
        page.goto(url, wait_until="domcontentloaded")
    response = response_info.value
    assert response.ok, f"{method} {api_fragment} returned {response.status}"


def wait_for_url(page: Page, suffix: str) -> None:
    page.wait_for_url(lambda url: str(url).rstrip("/").endswith(suffix), timeout=20000)


def login(page: Page, account: str, password: str, expected_path: str) -> None:
    page.goto(f"{WEB_BASE}/login", wait_until="domcontentloaded")
    page.wait_for_selector("text=欢迎回来", timeout=15000)
    page.locator('input[placeholder="name@example.com"]').fill(account)
    page.locator('input[placeholder="至少 8 位"]').fill(password)
    page.get_by_role("button", name="登录", exact=True).click()
    wait_for_url(page, expected_path)


def account_appointment(page: Page, status: str):
    return (
        page.locator("div.overflow-hidden.rounded-lg.border")
        .filter(has_text=status)
        .filter(has_text=APPOINTMENT_YEAR)
        .first
    )


def creator_appointment(page: Page, status: str):
    section = page.locator("section").filter(has_text="收到的预约")
    return (
        section.locator("div.flex.flex-col.gap-3.rounded-lg.border.bg-surface")
        .filter(has_text=status)
        .filter(has_text=APPOINTMENT_YEAR)
        .first
    )


def main() -> None:
    OUT_DIR.mkdir(exist_ok=True)
    console_errors: list[str] = []
    page_errors: list[str] = []
    api_errors: list[str] = []

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(viewport={"width": 1440, "height": 1000})
        page = context.new_page()
        page.on(
            "console",
            lambda msg: console_errors.append(msg.text) if msg.type == "error" else None,
        )
        page.on("pageerror", lambda error: page_errors.append(str(error)))
        page.on(
            "response",
            lambda response: api_errors.append(
                f"{response.status} {response.request.method} {response.url}"
            )
            if "/api/" in response.url and response.status >= 400
            else None,
        )

        try:
            health = page.request.get(f"{API_BASE}/health")
            assert health.ok, f"health status was {health.status}"
            health_body = health.json()
            assert health_body.get("data", {}).get("status") == "ok", health_body

            page.goto(WEB_BASE, wait_until="domcontentloaded")
            page.evaluate("localStorage.clear()")

            login(page, "customer", "customer123", "/account")
            page.wait_for_selector("text=账户充值", timeout=15000)
            screenshot(page, "01-customer-login")

            amount = page.locator('input[aria-label="充值金额"]')
            assert amount.input_value() == "5000", amount.input_value()
            page.get_by_role("button", name="充值", exact=True).click()
            page.wait_for_selector("text=充值 ¥5000 成功，余额已到账。", timeout=15000)
            screenshot(page, "02-recharge")

            page.goto(f"{WEB_BASE}/services", wait_until="domcontentloaded")
            service = page.get_by_text(SERVICE_TITLE, exact=True).first
            service.wait_for(timeout=15000)
            service.click()
            page.wait_for_url(re.compile(r".*/services/\d+$"), timeout=20000)
            page.wait_for_selector("text=预约档期", timeout=15000)
            page.locator('input[type="datetime-local"]').fill(APPOINTMENT_DATE)
            page.get_by_role("button", name="立即预约", exact=True).click()
            page.wait_for_selector("text=预约已提交", timeout=15000)
            screenshot(page, "03-appointment-created")

            goto_and_wait_api(page, f"{WEB_BASE}/account", "GET", "/api/v1/appointments")
            page.wait_for_selector("text=我的预约", timeout=15000)
            pending = account_appointment(page, "待支付")
            pending.wait_for(timeout=15000)
            pending.get_by_role("button", name="支付", exact=True).click()
            page.wait_for_selector("text=支付成功，预约已进入已确认状态。", timeout=15000)
            page.wait_for_selector("text=已确认", timeout=15000)
            screenshot(page, "04-payment-confirmed")

            page.get_by_role("button", name="退出", exact=True).click()
            page.wait_for_url(re.compile(r"^http://127\.0\.0\.1:5173/$"), timeout=20000)
            login(page, "chenyu", "creator123", "/dashboard")
            page.wait_for_selector("text=收到的预约", timeout=15000)
            screenshot(page, "05-creator-dashboard")

            confirmed = creator_appointment(page, "已确认")
            confirmed.wait_for(timeout=15000)
            confirmed.get_by_role("button", name="开始", exact=True).click()
            page.wait_for_selector("text=预约已开始。", timeout=15000)
            page.wait_for_selector("text=进行中", timeout=15000)
            screenshot(page, "06-creator-ongoing")

            ongoing = creator_appointment(page, "进行中")
            ongoing.wait_for(timeout=15000)
            ongoing.get_by_role("button", name="完成", exact=True).click()
            page.wait_for_selector("text=预约已完成。", timeout=15000)
            page.wait_for_selector("text=已完成", timeout=15000)
            screenshot(page, "07-creator-completed")

            page.get_by_role("button", name="退出登录", exact=True).click()
            page.wait_for_url(re.compile(r"^http://127\.0\.0\.1:5173/$"), timeout=20000)
            login(page, "customer", "customer123", "/account")
            page.wait_for_selector("text=我的预约", timeout=15000)

            completed = account_appointment(page, "已完成")
            completed.wait_for(timeout=15000)
            completed.get_by_role("button", name="评价", exact=True).click()
            page.wait_for_selector("text=为本次服务评分", timeout=15000)
            page.get_by_role("button", name="5 星", exact=True).click()
            page.locator('textarea[placeholder="说说这次拍摄体验（选填）"]').fill(
                "端到端测试评价：拍摄过程顺利，交付及时。"
            )
            page.get_by_role("button", name="提交评价", exact=True).click()
            page.wait_for_selector("text=评价已提交，感谢你的反馈。", timeout=15000)
            screenshot(page, "08-review-submitted")

            page.close()
            page = context.new_page()
            page.goto(f"{WEB_BASE}/creators/1", wait_until="domcontentloaded")
            page.wait_for_selector("text=用户评价", timeout=15000)
            page.wait_for_selector(
                "text=端到端测试评价：拍摄过程顺利，交付及时。", timeout=15000
            )

        except Exception:
            print("api errors:")
            for error in api_errors:
                print(error)
            print("console errors:")
            for error in console_errors:
                print(error)
            print("page errors:")
            for error in page_errors:
                print(error)
            try:
                screenshot(page, "failure")
                print(f"failure url: {page.url}")
                print("visible text:")
                print(page.locator("body").inner_text(timeout=5000))
            except Exception as capture_error:
                print(f"failed to capture failure state: {capture_error}")
            raise
        finally:
            browser.close()

    if page_errors:
        print("page errors:")
        for error in page_errors:
            print(error)
        raise AssertionError("browser page errors occurred")
    if api_errors:
        print("api errors:")
        for error in api_errors:
            print(error)
        raise AssertionError("API requests returned error status")
    if console_errors:
        print("console errors:")
        for error in console_errors:
            print(error)

    print("E2E flow passed")
    print(f"screenshots directory: {OUT_DIR}")


if __name__ == "__main__":
    try:
        before = main_database_snapshot()
        with managed_test_environment():
            main()
        after = main_database_snapshot()
        print_main_database_comparison(before, after)
    except Exception as error:
        print(f"E2E FAILED: {error}", file=sys.stderr)
        raise

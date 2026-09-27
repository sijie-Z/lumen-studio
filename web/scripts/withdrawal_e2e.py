import re
import sqlite3
import sys
from decimal import Decimal
from pathlib import Path

from playwright.sync_api import Page, sync_playwright
from test_runtime import (
    TEST_DB,
    main_database_snapshot,
    managed_test_environment,
    print_main_database_comparison,
)


ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / ".smoke"
WEB_BASE = "http://127.0.0.1:5173"
API_BASE = "http://127.0.0.1:8080"
CREATOR_ACCOUNT = "chenyu"
CREATOR_PASSWORD = "creator123"
ADMIN_ACCOUNT = "admin"
ADMIN_PASSWORD = "admin123"


def screenshot(page: Page, name: str) -> Path:
    OUT_DIR.mkdir(exist_ok=True)
    path = OUT_DIR / f"withdrawal-e2e-{name}.png"
    page.screenshot(path=str(path), full_page=True)
    print(f"screenshot: {path}")
    return path


def login(page: Page, account: str, password: str, expected_path: str) -> None:
    page.goto(f"{WEB_BASE}/login", wait_until="domcontentloaded")
    page.wait_for_selector("text=欢迎回来", timeout=15000)
    page.locator('input[placeholder="name@example.com"]').fill(account)
    page.locator('input[placeholder="至少 8 位"]').fill(password)
    page.get_by_role("button", name="登录", exact=True).click()
    page.wait_for_url(
        lambda url: str(url).rstrip("/").endswith(expected_path),
        timeout=20000,
    )


def logout_dashboard(page: Page) -> None:
    page.get_by_role("button", name="退出登录", exact=True).click()
    page.wait_for_url(re.compile(r"^http://127\.0\.0\.1:5173/$"), timeout=20000)


def logout_admin(page: Page) -> None:
    page.get_by_role("button", name="退出", exact=True).click()
    page.wait_for_url(re.compile(r"^http://127\.0\.0\.1:5173/$"), timeout=20000)


def creator_section(page: Page):
    return page.locator("section").filter(has_text="余额提现").first


def ui_balance(page: Page) -> Decimal:
    text = creator_section(page).locator("p.font-display.text-3xl").inner_text()
    amount = re.sub(r"[^\d.]", "", text)
    return Decimal(amount or "0")


def assert_ui_balance(page: Page, expected: Decimal) -> None:
    actual = ui_balance(page)
    assert actual == expected, f"expected balance {expected}, got {actual}"


def set_creator_balance(balance: Decimal) -> None:
    with sqlite3.connect(TEST_DB, timeout=10) as connection:
        connection.execute(
            "UPDATE users SET balance = ? WHERE username = ?",
            (str(balance), CREATOR_ACCOUNT),
        )
        connection.commit()


def database_balance() -> Decimal:
    with sqlite3.connect(TEST_DB, timeout=10) as connection:
        row = connection.execute(
            "SELECT balance FROM users WHERE username = ?",
            (CREATOR_ACCOUNT,),
        ).fetchone()
    assert row is not None, "creator user not found in test database"
    return Decimal(str(row[0]))


def database_withdrawals() -> list[tuple[Decimal, str]]:
    with sqlite3.connect(TEST_DB, timeout=10) as connection:
        rows = connection.execute(
            "SELECT amount, status FROM withdrawals ORDER BY id"
        ).fetchall()
    return [(Decimal(str(amount)), status) for amount, status in rows]


def apply_withdrawal(page: Page, amount: str, account: str) -> None:
    section = creator_section(page)
    section.locator('input[placeholder="例如：500.00"]').fill(amount)
    section.locator('input[placeholder="支付宝账号或银行卡号"]').fill(account)
    section.get_by_role("button", name="申请提现", exact=True).click()
    page.wait_for_selector("text=提现申请已提交，等待管理员审核。", timeout=15000)


def admin_withdrawal_row(page: Page, amount: str):
    section = page.locator("section").filter(has_text="提现审核").first
    return (
        section.locator("div.bg-secondary.p-4")
        .filter(has_text=re.compile(rf"¥{re.escape(amount)}(?:\.00)?"))
        .first
    )


def main() -> None:
    OUT_DIR.mkdir(exist_ok=True)
    console_errors: list[str] = []
    page_errors: list[str] = []
    api_errors: list[str] = []

    set_creator_balance(Decimal("5000.00"))

    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True)
        context = browser.new_context(viewport={"width": 1440, "height": 1000})
        page = context.new_page()
        page.on(
            "console",
            lambda message: console_errors.append(message.text)
            if message.type == "error"
            else None,
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
            assert health.json().get("data", {}).get("status") == "ok"

            page.goto(WEB_BASE, wait_until="domcontentloaded")
            page.evaluate("localStorage.clear()")

            login(page, CREATOR_ACCOUNT, CREATOR_PASSWORD, "/dashboard")
            page.wait_for_selector("text=余额提现", timeout=15000)
            assert_ui_balance(page, Decimal("5000.00"))

            apply_withdrawal(page, "1000", "alipay:chenyu@example.com")
            page.get_by_text("待审核", exact=True).first.wait_for(timeout=15000)
            assert_ui_balance(page, Decimal("4000.00"))
            assert database_balance() == Decimal("4000.00")
            screenshot(page, "01-creator-pending")

            logout_dashboard(page)
            login(page, ADMIN_ACCOUNT, ADMIN_PASSWORD, "/admin")
            page.wait_for_selector("text=提现审核", timeout=15000)
            first_row = admin_withdrawal_row(page, "1000")
            first_row.wait_for(timeout=15000)
            first_row.get_by_role("button", name="通过并打款", exact=True).click()
            page.wait_for_selector("text=提现已通过并标记为已打款。", timeout=15000)
            screenshot(page, "02-admin-approved")

            assert database_balance() == Decimal("4000.00")
            assert database_withdrawals() == [(Decimal("1000.00"), "completed")]

            logout_admin(page)
            login(page, CREATOR_ACCOUNT, CREATOR_PASSWORD, "/dashboard")
            page.get_by_text("已打款", exact=True).first.wait_for(timeout=15000)
            assert_ui_balance(page, Decimal("4000.00"))
            screenshot(page, "03-creator-completed")

            apply_withdrawal(page, "600", "alipay:chenyu-reject@example.com")
            page.get_by_text("待审核", exact=True).first.wait_for(timeout=15000)
            assert_ui_balance(page, Decimal("3400.00"))
            assert database_balance() == Decimal("3400.00")
            screenshot(page, "04-creator-second-pending")

            logout_dashboard(page)
            login(page, ADMIN_ACCOUNT, ADMIN_PASSWORD, "/admin")
            page.wait_for_selector("text=提现审核", timeout=15000)
            second_row = admin_withdrawal_row(page, "600")
            second_row.wait_for(timeout=15000)
            second_row.locator('input[placeholder="审核备注（拒绝原因建议填写）"]').fill(
                "提现 E2E 拒绝测试"
            )
            second_row.get_by_role("button", name="拒绝", exact=True).click()
            page.wait_for_selector(
                "text=提现已拒绝，金额已退回创作者余额。",
                timeout=15000,
            )
            screenshot(page, "05-admin-rejected")

            assert database_balance() == Decimal("4000.00")
            assert database_withdrawals() == [
                (Decimal("1000.00"), "completed"),
                (Decimal("600.00"), "rejected"),
            ]

            logout_admin(page)
            login(page, CREATOR_ACCOUNT, CREATOR_PASSWORD, "/dashboard")
            page.get_by_text("已拒绝", exact=True).first.wait_for(timeout=15000)
            assert_ui_balance(page, Decimal("4000.00"))
            screenshot(page, "06-creator-rejected")
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
                print(page.locator("body").inner_text(timeout=5000))
            except Exception as capture_error:
                print(f"failed to capture failure state: {capture_error}")
            raise
        finally:
            browser.close()

    if page_errors:
        raise AssertionError(f"browser page errors occurred: {page_errors}")
    if api_errors:
        raise AssertionError(f"API requests returned error status: {api_errors}")
    if console_errors:
        print("console errors:")
        for error in console_errors:
            print(error)
        raise AssertionError("browser console reported errors")

    print("Withdrawal E2E passed")
    print(f"screenshots directory: {OUT_DIR}")


if __name__ == "__main__":
    try:
        before = main_database_snapshot()
        with managed_test_environment():
            main()
        after = main_database_snapshot()
        print_main_database_comparison(before, after)
    except Exception as error:
        print(f"WITHDRAWAL E2E FAILED: {error}", file=sys.stderr)
        raise

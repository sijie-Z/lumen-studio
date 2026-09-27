import re
import sys
from pathlib import Path

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
APPOINTMENT_DATE = "2026-12-20T10:00"
APPOINTMENT_YEAR = "2026"
SERVICE_TITLE = "城市自然光人像写真"


def screenshot(page: Page, name: str) -> Path:
    OUT_DIR.mkdir(exist_ok=True)
    path = OUT_DIR / f"onboarding-notify-{name}.png"
    page.screenshot(path=str(path))
    print(f"screenshot: {path}")
    return path


def tracked_page(
    context,
    console_errors: list[str],
    page_errors: list[str],
    api_errors: list[str],
) -> Page:
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
    return page


def wait_for_url(page: Page, suffix: str) -> None:
    page.wait_for_url(lambda url: str(url).rstrip("/").endswith(suffix), timeout=20000)


def login(page: Page, account: str, password: str, expected_path: str) -> None:
    page.goto(f"{WEB_BASE}/login", wait_until="domcontentloaded")
    page.wait_for_selector("text=欢迎回来", timeout=15000)
    page.locator('input[placeholder="name@example.com"]').fill(account)
    page.locator('input[placeholder="至少 8 位"]').fill(password)
    page.get_by_role("button", name="登录", exact=True).click()
    wait_for_url(page, expected_path)


def logout(page: Page, button_name: str) -> None:
    page.get_by_role("button", name=button_name, exact=True).click()
    page.wait_for_url(re.compile(r"^http://127\.0\.0\.1:5173/$"), timeout=20000)


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


def notification_bell(page: Page):
    return page.get_by_role("button", name="消息通知", exact=True)


def assert_unread_count(page: Page, expected: int) -> None:
    badge = notification_bell(page).locator("span")
    if expected == 0:
        expect(badge).to_have_count(0, timeout=20000)
        return
    expect(badge).to_have_text(str(expected), timeout=20000)


def open_notifications(page: Page) -> None:
    notification_bell(page).click()
    page.get_by_text("消息通知", exact=True).wait_for(timeout=10000)


def mark_all_notifications_read(page: Page) -> None:
    page.get_by_role("button", name="全部已读", exact=True).click()
    assert_unread_count(page, 0)


def dismiss_guide_and_assert_persisted(page: Page, title: str) -> None:
    page.get_by_text(title, exact=True).wait_for(timeout=15000)
    page.get_by_role("button", name="关闭引导", exact=True).click()
    expect(page.get_by_text(title, exact=True)).to_have_count(0, timeout=10000)
    page.reload(wait_until="domcontentloaded")
    expect(page.get_by_text(title, exact=True)).to_have_count(0, timeout=15000)


def main() -> None:
    OUT_DIR.mkdir(exist_ok=True)
    console_errors: list[str] = []
    page_errors: list[str] = []
    api_errors: list[str] = []

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(viewport={"width": 1440, "height": 1000})
        page = tracked_page(context, console_errors, page_errors, api_errors)

        try:
            health = page.request.get(f"{API_BASE}/health")
            assert health.ok, f"health status was {health.status}"
            assert health.json().get("data", {}).get("status") == "ok"

            page.goto(WEB_BASE, wait_until="domcontentloaded")
            page.evaluate("localStorage.clear()")

            # A1: customer login, onboarding empty state, recharge.
            login(page, "customer", "customer123", "/account")
            page.wait_for_selector("text=账户充值", timeout=15000)
            page.get_by_text("还没有预约", exact=True).wait_for(timeout=15000)
            screenshot(page, "01-customer-onboarding-empty")

            amount = page.locator('input[aria-label="充值金额"]')
            assert amount.input_value() == "5000", amount.input_value()
            page.get_by_role("button", name="充值", exact=True).click()
            page.wait_for_selector("text=充值 ¥5000 成功，余额已到账。", timeout=15000)
            screenshot(page, "02-customer-recharge")

            # A2: create appointment and verify customer step guide.
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

            page.goto(f"{WEB_BASE}/account", wait_until="domcontentloaded")
            page.wait_for_selector("text=我的预约", timeout=15000)
            page.get_by_text("完成一次预约", exact=True).wait_for(timeout=15000)
            page.get_by_text("进度 1/3", exact=True).wait_for(timeout=15000)
            screenshot(page, "04-customer-step-guide")

            # A3: pay appointment.
            pending = account_appointment(page, "待支付")
            pending.wait_for(timeout=15000)
            pending.get_by_role("button", name="支付", exact=True).click()
            page.wait_for_selector("text=支付成功，预约已进入已确认状态。", timeout=15000)
            page.wait_for_selector("text=已确认", timeout=15000)
            screenshot(page, "05-payment-confirmed")

            logout(page, "退出")
            page.close()
            page = tracked_page(context, console_errors, page_errors, api_errors)

            # A4/B1: creator receives appointment-created and payment notifications.
            login(page, "chenyu", "creator123", "/dashboard")
            page.wait_for_selector("text=收到的预约", timeout=15000)
            page.get_by_text("完成创作者起步流程", exact=True).wait_for(timeout=15000)
            assert_unread_count(page, 2)
            screenshot(page, "06-creator-onboarding-unread")

            open_notifications(page)
            page.get_by_text("新的预约请求", exact=True).wait_for(timeout=10000)
            page.get_by_text("客户已支付", exact=True).wait_for(timeout=10000)
            screenshot(page, "07-creator-notifications-open")
            mark_all_notifications_read(page)
            screenshot(page, "08-creator-notifications-read")

            dismiss_guide_and_assert_persisted(page, "完成创作者起步流程")
            page.wait_for_selector("text=收到的预约", timeout=15000)

            # A5: creator executes the appointment.
            confirmed = creator_appointment(page, "已确认")
            confirmed.wait_for(timeout=15000)
            confirmed.get_by_role("button", name="开始", exact=True).click()
            page.wait_for_selector("text=预约已开始。", timeout=15000)
            page.wait_for_selector("text=进行中", timeout=15000)

            ongoing = creator_appointment(page, "进行中")
            ongoing.wait_for(timeout=15000)
            ongoing.get_by_role("button", name="完成", exact=True).click()
            page.wait_for_selector("text=预约已完成。", timeout=15000)
            page.wait_for_selector("text=已完成", timeout=15000)
            screenshot(page, "09-creator-appointment-completed")

            # B2: creator applies for withdrawal, which must notify admins.
            page.locator('input[placeholder="例如：500.00"]').fill("100.00")
            page.locator('input[placeholder="支付宝账号或银行卡号"]').fill("alipay:onboarding-e2e")
            page.get_by_role("button", name="申请提现", exact=True).click()
            page.wait_for_selector("text=提现申请已提交，等待管理员审核。", timeout=15000)
            screenshot(page, "10-creator-withdrawal-submitted")

            logout(page, "退出登录")
            page.close()
            page = tracked_page(context, console_errors, page_errors, api_errors)

            # B3/C2: admin notification + onboarding dismissal.
            login(page, "admin", "admin123", "/admin")
            page.wait_for_selector("text=平台概览", timeout=15000)
            page.get_by_text("管理后台怎么用", exact=True).wait_for(timeout=15000)
            assert_unread_count(page, 1)
            screenshot(page, "11-admin-onboarding-unread")
            open_notifications(page)
            page.get_by_text("新的提现申请", exact=True).wait_for(timeout=10000)
            screenshot(page, "12-admin-notification-open")
            mark_all_notifications_read(page)
            dismiss_guide_and_assert_persisted(page, "管理后台怎么用")
            page.wait_for_selector("text=平台概览", timeout=15000)

            logout(page, "退出")
            page.close()
            page = tracked_page(context, console_errors, page_errors, api_errors)

            # A6/B4: customer receives completion notification and submits review.
            login(page, "customer", "customer123", "/account")
            page.wait_for_selector("text=我的预约", timeout=15000)
            assert_unread_count(page, 1)
            open_notifications(page)
            page.get_by_text("服务已完成，欢迎评价", exact=True).wait_for(timeout=10000)
            screenshot(page, "13-customer-completion-notification")
            mark_all_notifications_read(page)

            completed = account_appointment(page, "已完成")
            completed.wait_for(timeout=15000)
            completed.get_by_role("button", name="评价", exact=True).click()
            page.wait_for_selector("text=为本次服务评分", timeout=15000)
            page.get_by_role("button", name="5 星", exact=True).click()
            page.locator('textarea[placeholder="说说这次拍摄体验（选填）"]').fill(
                "引导与通知端到端测试：流程顺畅，通知及时。"
            )
            page.get_by_role("button", name="提交评价", exact=True).click()
            page.wait_for_selector("text=评价已提交，感谢你的反馈。", timeout=15000)
            page.get_by_text("全部步骤已完成。", exact=True).wait_for(timeout=15000)
            screenshot(page, "14-customer-flow-complete")

            dismiss_guide_and_assert_persisted(page, "完成一次预约")
            page.wait_for_selector("text=我的预约", timeout=15000)

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

    print("Onboarding + notification E2E passed")
    print(f"screenshots directory: {OUT_DIR}")


if __name__ == "__main__":
    try:
        before = main_database_snapshot()
        with managed_test_environment():
            main()
        after = main_database_snapshot()
        print_main_database_comparison(before, after)
    except Exception as error:
        print(f"ONBOARDING NOTIFICATION E2E FAILED: {error}", file=sys.stderr)
        raise

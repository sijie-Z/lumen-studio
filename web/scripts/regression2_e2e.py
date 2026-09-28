import re
import socket
import sys
from decimal import Decimal
from pathlib import Path
from urllib.parse import parse_qs, urlparse

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
SERVICE_TITLE = "城市自然光人像写真"
APPOINTMENT_DATE = "2026-12-20T10:00"
REFUND_APPOINTMENT_START = "2026-12-21T10:00:00Z"
CREATED_SERVICE_PREFIX = "回归分页服务"

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")


def screenshot(page: Page, name: str) -> Path:
    OUT_DIR.mkdir(exist_ok=True)
    path = OUT_DIR / f"regression2-{name}.png"
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


def auth_headers(page: Page) -> dict[str, str]:
    token = page.evaluate("localStorage.getItem('lumina.token')")
    assert token, "expected an authenticated browser session"
    return {"Authorization": f"Bearer {token}"}


def api_data(response, context: str):
    assert response.ok, f"{context} failed: {response.status} {response.text()}"
    body = response.json()
    assert body.get("code") == 200, f"{context} returned {body}"
    return body.get("data")


def api_get(page: Page, path: str, authenticated: bool = True):
    response = page.request.get(
        f"{API_PREFIX}{path}",
        headers=auth_headers(page) if authenticated else {},
    )
    return api_data(response, f"GET {path}")


def api_post(page: Page, path: str, payload: dict, authenticated: bool = True):
    response = page.request.post(
        f"{API_PREFIX}{path}",
        headers=auth_headers(page) if authenticated else {},
        data=payload,
    )
    return api_data(response, f"POST {path}")


def header(page: Page):
    return page.locator("header").first


def notification_bell(page: Page):
    return header(page).get_by_role("button", name="消息通知", exact=True)


def assert_unread_count(page: Page, expected: int) -> None:
    bell = notification_bell(page)
    expect(bell).to_be_visible(timeout=15000)
    if expected == 0:
        expect(bell.locator("span")).to_have_count(0, timeout=20000)
        return
    expect(bell.locator("span")).to_have_text(str(expected), timeout=20000)


def mark_all_notifications_read(page: Page) -> None:
    notification_bell(page).click()
    page.get_by_text("消息通知", exact=True).wait_for(timeout=10000)
    page.get_by_role("button", name="全部已读", exact=True).click()
    assert_unread_count(page, 0)
    notification_bell(page).click()


def assert_public_shell(page: Page) -> None:
    shell = header(page)
    expect(shell).to_be_visible(timeout=15000)
    expect(shell.get_by_role("link", name="Lumen Studio 首页", exact=True)).to_be_visible()
    for label in ("探索", "服务", "创作者"):
        expect(shell.get_by_role("link", name=label, exact=True)).to_be_visible()


def assert_role_shell(page: Page, nickname: str, labels: tuple[str, ...]) -> None:
    shell = header(page)
    expect(shell).to_be_visible(timeout=15000)
    expect(shell.get_by_role("link", name="Lumen Studio 首页", exact=True)).to_be_visible()
    expect(notification_bell(page)).to_be_visible()
    user_button = shell.get_by_role("button", name=re.compile(nickname))
    expect(user_button).to_be_visible()
    for label in labels:
        expect(shell.get_by_role("link", name=label, exact=True)).to_be_visible()


def dismiss_guide_and_reload(page: Page, title: str) -> None:
    page.get_by_text(title, exact=True).wait_for(timeout=15000)
    page.get_by_role("button", name="关闭引导", exact=True).click()
    expect(page.get_by_text(title, exact=True)).to_have_count(0, timeout=10000)
    page.reload(wait_until="domcontentloaded")
    page.wait_for_selector("main", timeout=15000)
    expect(page.get_by_text(title, exact=True)).to_have_count(0, timeout=15000)


def account_card(page: Page, appointment_id: int):
    card = page.locator(f"#appointment-{appointment_id}")
    card.wait_for(timeout=15000)
    return card


def creator_card(page: Page, status: str):
    section = page.locator("section").filter(has_text="收到的预约")
    return (
        section.locator("div.flex.flex-col.gap-3.rounded-lg.border.bg-surface")
        .filter(has_text=status)
        .filter(has_text="2026")
        .first
    )


def work_cards(page: Page):
    return page.locator('main a[href^="/works/"]')


def service_cards(page: Page):
    return page.locator('main a[href^="/services/"]')


def wait_for_api_response(page: Page, method: str, url_fragment: str):
    return page.expect_response(
        lambda response: response.request.method == method
        and url_fragment in response.url,
        timeout=20000,
    )


def assert_pagination_summary(page: Page, expected: str) -> None:
    expect(page.get_by_text(expected, exact=True)).to_be_visible(timeout=20000)


def create_services_for_pagination(page: Page) -> None:
    service_types = api_get(page, "/service-types", authenticated=False)
    type_id = next(item["id"] for item in service_types if item["name"] == "人像摄影")
    for index in range(1, 6):
        api_post(
            page,
            "/services",
            {
                "type_id": type_id,
                "title": f"{CREATED_SERVICE_PREFIX} {index:02d}",
                "description": "浏览器级分页与筛选回归数据。",
                "price": "199.00",
                "duration": 60,
                "cover_image_url": "/demo/works/portrait-1.jpg",
                "location": "杭州",
                "tags": "人像摄影",
            },
        )


def assert_ports_released() -> None:
    for port, label in ((8080, "Rust API"), (5173, "Vite")):
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.3):
                raise AssertionError(f"{label} port {port} is still open after regression run")
        except OSError:
            pass


def verify_route_guards(page: Page) -> None:
    for protected_path in ("/account", "/dashboard", "/admin"):
        page.goto(f"{WEB_BASE}{protected_path}", wait_until="domcontentloaded")
        wait_for_path(page, "/login")
        page.wait_for_selector("text=欢迎回来", timeout=15000)
    screenshot(page, "01-unauthenticated-guards")


def verify_explore_pagination_and_filters(page: Page) -> None:
    page.goto(f"{WEB_BASE}/explore", wait_until="domcontentloaded")
    assert_public_shell(page)
    assert_pagination_summary(page, "共 20 条 · 第 1 / 2 页")
    expect(work_cards(page)).to_have_count(12, timeout=20000)

    with wait_for_api_response(page, "GET", "/api/v1/works?"):
        page.get_by_role("button", name="下一页", exact=True).click()
    assert_pagination_summary(page, "共 20 条 · 第 2 / 2 页")
    expect(work_cards(page)).to_have_count(8, timeout=20000)
    screenshot(page, "02-explore-page-2")

    with wait_for_api_response(page, "GET", "category="):
        page.get_by_role("button", name="婚礼", exact=True).click()
    assert_pagination_summary(page, "共 2 条 · 第 1 / 1 页")
    expect(work_cards(page)).to_have_count(2, timeout=20000)
    screenshot(page, "03-explore-category-filter")

    page.reload(wait_until="domcontentloaded")
    assert_pagination_summary(page, "共 20 条 · 第 1 / 2 页")
    with wait_for_api_response(page, "GET", "q="):
        page.locator('input[placeholder="搜索作品标题"]').fill("夜未央")
    assert_pagination_summary(page, "共 1 条 · 第 1 / 1 页")
    expect(work_cards(page)).to_have_count(1, timeout=20000)
    expect(page.get_by_text("夜未央", exact=True)).to_be_visible()
    screenshot(page, "04-explore-search")


def verify_service_pagination_and_filters(page: Page) -> None:
    page.goto(f"{WEB_BASE}/services", wait_until="domcontentloaded")
    assert_public_shell(page)
    assert_pagination_summary(page, "共 13 条 · 第 1 / 2 页")
    expect(service_cards(page)).to_have_count(12, timeout=20000)

    with wait_for_api_response(page, "GET", "/api/v1/services?"):
        page.get_by_role("button", name="下一页", exact=True).click()
    assert_pagination_summary(page, "共 13 条 · 第 2 / 2 页")
    expect(service_cards(page)).to_have_count(1, timeout=20000)
    screenshot(page, "05-services-page-2")

    with wait_for_api_response(page, "GET", "type_id="):
        page.get_by_role("button", name="人像摄影", exact=True).click()
    assert_pagination_summary(page, "共 8 条 · 第 1 / 1 页")
    expect(service_cards(page)).to_have_count(8, timeout=20000)
    screenshot(page, "06-services-type-filter")

    with wait_for_api_response(page, "GET", "q="):
        page.locator('input[placeholder="搜索服务名称"]').fill(f"{CREATED_SERVICE_PREFIX} 03")
    assert_pagination_summary(page, "共 1 条 · 第 1 / 1 页")
    expect(service_cards(page)).to_have_count(1, timeout=20000)
    expect(
        page.get_by_text(f"{CREATED_SERVICE_PREFIX} 03", exact=True)
    ).to_be_visible()
    screenshot(page, "07-services-title-search")

    page.reload(wait_until="domcontentloaded")
    assert_pagination_summary(page, "共 13 条 · 第 1 / 2 页")
    with wait_for_api_response(page, "GET", "location="):
        page.locator('input[placeholder="按城市筛选"]').fill("杭州")
    assert_pagination_summary(page, "共 6 条 · 第 1 / 1 页")
    expect(service_cards(page)).to_have_count(6, timeout=20000)
    screenshot(page, "08-services-city-filter")


def verify_full_business_flow(page: Page) -> None:
    login(page, "customer", "customer123", "/account")
    assert_role_shell(page, "林小满", ("探索", "服务", "我的订单"))
    dismiss_guide_and_reload(page, "完成一次预约")

    amount = page.locator('input[aria-label="充值金额"]')
    assert amount.input_value() == "5000", amount.input_value()
    page.get_by_role("button", name="充值", exact=True).click()
    page.wait_for_selector("text=充值 ¥5000 成功，余额已到账。", timeout=15000)

    page.goto(f"{WEB_BASE}/services", wait_until="domcontentloaded")
    with wait_for_api_response(page, "GET", "q="):
        page.locator('input[placeholder="搜索服务名称"]').fill(SERVICE_TITLE)
    service = page.get_by_text(SERVICE_TITLE, exact=True).first
    service.wait_for(timeout=15000)
    service.click()
    page.wait_for_url(re.compile(r".*/services/\d+$"), timeout=20000)
    page.locator('input[type="datetime-local"]').fill(APPOINTMENT_DATE)
    page.get_by_role("button", name="立即预约", exact=True).click()
    page.get_by_text(re.compile(r"预约 #\d+ 已提交")).wait_for(timeout=15000)
    pay_link = page.get_by_role("link", name="去支付", exact=True)
    expect(pay_link).to_be_visible()
    pay_link.click()
    page.wait_for_url(
        lambda url: path_of(page) == "/account"
        and "appointment" in parse_qs(urlparse(str(url)).query),
        timeout=20000,
    )
    appointment_id = int(parse_qs(urlparse(page.url).query)["appointment"][0])
    highlighted = account_card(page, appointment_id)
    highlighted.get_by_role("button", name="支付", exact=True).click()
    page.wait_for_selector("text=支付成功，预约已进入已确认状态。", timeout=15000)
    expect(account_card(page, appointment_id).get_by_text("已确认", exact=True)).to_be_visible()
    screenshot(page, "09-customer-payment-confirmed")

    login(page, "chenyu", "creator123", "/dashboard")
    assert_role_shell(page, "陈屿", ("探索", "服务", "工作台"))
    dismiss_guide_and_reload(page, "完成创作者起步流程")
    assert_unread_count(page, 2)
    mark_all_notifications_read(page)

    confirmed = creator_card(page, "已确认")
    confirmed.wait_for(timeout=15000)
    confirmed.get_by_role("button", name="开始", exact=True).click()
    page.wait_for_selector("text=预约已开始。", timeout=15000)
    creator_card(page, "进行中").get_by_role("button", name="完成", exact=True).click()
    page.wait_for_selector("text=预约已完成。", timeout=15000)
    expect(creator_card(page, "已完成")).to_be_visible()
    screenshot(page, "10-creator-completed")

    api_post(
        page,
        "/withdrawals",
        {"amount": "100.00", "account_info": {"account": "alipay:regression2"}},
    )

    login(page, "admin", "admin123", "/admin")
    assert_role_shell(page, "平台管理员", ("平台概览", "用户管理", "提现审核"))
    dismiss_guide_and_reload(page, "管理后台怎么用")
    assert_unread_count(page, 1)
    mark_all_notifications_read(page)
    screenshot(page, "11-admin-notifications-read")

    login(page, "customer", "customer123", "/account")
    completed = account_card(page, appointment_id)
    completed.get_by_role("button", name="评价", exact=True).click()
    page.get_by_role("button", name="5 星", exact=True).click()
    page.locator('textarea[placeholder="说说这次拍摄体验（选填）"]').fill(
        "第二轮回归：分页、筛选和完整业务流程。"
    )
    page.get_by_role("button", name="提交评价", exact=True).click()
    page.wait_for_selector("text=评价已提交，感谢你的反馈。", timeout=15000)
    screenshot(page, "12-review-submitted")

    page.get_by_role("button", name="充值", exact=True).click()
    page.wait_for_selector("text=充值 ¥5000 成功，余额已到账。", timeout=15000)
    balance_before_refund_payment = Decimal(str(api_get(page, "/auth/me")["balance"]))
    services = api_get(page, "/services?page=1&page_size=100", authenticated=False)
    service = next(item for item in services["items"] if item["title"] == SERVICE_TITLE)
    refund_appointment = api_post(
        page,
        "/appointments",
        {
            "service_id": service["id"],
            "start_time": REFUND_APPOINTMENT_START,
            "end_time": "2026-12-21T12:00:00Z",
            "location": "杭州",
            "notes": "第二轮取消退款回归",
        },
    )
    refund_appointment_id = refund_appointment["id"]
    api_post(
        page,
        f"/payments/appointments/{refund_appointment_id}/pay",
        {"method": "balance"},
    )
    page.reload(wait_until="domcontentloaded")
    refund_card = account_card(page, refund_appointment_id)
    expect(refund_card.get_by_text("已确认", exact=True)).to_be_visible(timeout=15000)
    refund_card.get_by_role("button", name="取消预约", exact=True).click()
    page.wait_for_selector("text=预约已取消。", timeout=15000)
    expect(account_card(page, refund_appointment_id).get_by_text("已退款", exact=True)).to_be_visible()

    balance_after_refund = Decimal(str(api_get(page, "/auth/me")["balance"]))
    assert balance_after_refund == balance_before_refund_payment, (
        f"refund balance mismatch: {balance_after_refund} != {balance_before_refund_payment}"
    )
    payments = api_get(page, "/payments")
    refunds = [
        payment
        for payment in payments
        if payment["appointment_id"] == refund_appointment_id
        and payment["payment_type"] == "refund"
        and payment["status"] == "success"
    ]
    assert len(refunds) == 1, f"expected one refund payment, got {refunds}"
    assert Decimal(str(refunds[0]["amount"])) == Decimal(str(service["price"]))

    clear_session(page)
    login(page, "customer", "customer123", "/account")
    page.goto(f"{WEB_BASE}/admin", wait_until="domcontentloaded")
    wait_for_path(page, "/")
    screenshot(page, "13-customer-admin-denied")

    login(page, "customer", "customer123", "/account")
    screenshot(page, "14-refund-confirmed")


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

        try:
            health = page.request.get(f"{API_BASE}/health")
            assert health.ok and health.json().get("data", {}).get("status") == "ok"

            clear_session(page)
            verify_route_guards(page)

            login(page, "chenyu", "creator123", "/dashboard")
            create_services_for_pagination(page)
            clear_session(page)

            verify_explore_pagination_and_filters(page)
            verify_service_pagination_and_filters(page)
            verify_full_business_flow(page)
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

    print("Regression2 E2E passed: pagination, filters, guards, and full flow")


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
        print(f"REGRESSION2 E2E FAILED: {error}", file=sys.stderr)
        raise

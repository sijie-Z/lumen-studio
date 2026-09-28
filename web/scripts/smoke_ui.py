import time
from pathlib import Path

from playwright.sync_api import sync_playwright
from test_runtime import (
    main_database_snapshot,
    managed_test_environment,
    print_main_database_comparison,
)


ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / ".smoke"


def main() -> None:
    OUT_DIR.mkdir(exist_ok=True)
    suffix = str(int(time.time()))
    username = f"ui_{suffix}"

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page(viewport={"width": 1440, "height": 1000})
        console_errors = []
        page.on("console", lambda msg: console_errors.append(msg.text) if msg.type == "error" else None)

        page.goto("http://127.0.0.1:5173", wait_until="domcontentloaded")
        page.wait_for_selector("text=Lumen Studio", timeout=10000)
        page.screenshot(path=str(OUT_DIR / "home-desktop.png"), full_page=True)
        overflow = page.evaluate(
            "document.documentElement.scrollWidth - window.innerWidth"
        )
        if overflow > 1:
            raise AssertionError(f"Desktop page has {overflow}px horizontal overflow")

        hero_style = page.locator("section#top").evaluate(
            "(el) => getComputedStyle(el).backgroundImage"
        )
        if "hero.jpg" not in hero_style:
            raise AssertionError("Hero background image is missing")

        first_work = page.locator("#creators img").first
        work_loaded = page.evaluate(
            "(el) => el && el.naturalWidth > 100", first_work.element_handle()
        )
        if not work_loaded:
            raise AssertionError("Featured work image did not load")

        page.get_by_role("button", name="打开 AI 助手").click()
        page.get_by_role("button", name="推荐摄影师").click()
        page.wait_for_selector("text=平台已收录", timeout=20000)
        page.get_by_role("button", name="关闭 AI 助手").click()

        page.goto("http://127.0.0.1:5173/register", wait_until="domcontentloaded")
        page.locator('input[placeholder="lumina_user"]').fill(username)
        page.locator('input[placeholder="至少 8 位"]').fill("password123")
        page.locator('input[placeholder="you@example.com"]').fill(f"{username}@example.com")
        page.get_by_role("button", name="创建账号").click()
        page.wait_for_selector("text=欢迎回来", timeout=10000)

        page.locator('input[placeholder="name@example.com"]').fill(username)
        page.locator('input[placeholder="至少 8 位"]').fill("password123")
        page.get_by_role("button", name="登录", exact=True).click()
        page.wait_for_url("**/account", timeout=10000)
        page.goto("http://127.0.0.1:5173/dashboard", wait_until="domcontentloaded")
        page.wait_for_selector("text=我的作品库", timeout=10000)

        page.locator('input[placeholder="一句话介绍你的风格"]').fill("测试摄影师")
        page.get_by_role("button", name="成为创作者").click()
        page.wait_for_selector("text=创作者资料已保存", timeout=10000)

        service_title = f"人像写真_{suffix}"
        page.locator('input[placeholder="例如：城市人像写真"]').fill(service_title)
        page.locator('input[placeholder="3999"]').fill("3999")
        page.locator('input[placeholder="120"]').fill("120")
        page.get_by_role("button", name="发布服务").click()
        page.wait_for_selector("text=服务已发布", timeout=10000)

        page.goto("http://127.0.0.1:5173/services", wait_until="domcontentloaded")
        page.wait_for_selector(f"text={service_title}", timeout=10000)
        page.locator("a[href^='/services/']").filter(has_text=service_title).first.click()
        page.wait_for_selector("text=预约档期", timeout=10000)
        page.locator('input[type="datetime-local"]').fill("2026-12-01T10:00")
        page.get_by_role("button", name="立即预约").click()
        page.wait_for_selector("text=预约已提交", timeout=10000)

        page.goto("http://127.0.0.1:5173/dashboard", wait_until="domcontentloaded")
        page.wait_for_selector("text=待支付", timeout=10000)

        page.locator('input[type="file"]').set_input_files(
            str(ROOT / "public/demo/work-1.jpg")
        )
        page.get_by_role("button", name="上传图片").click()
        page.wait_for_selector('[alt="work-1"]', timeout=15000)
        page.screenshot(path=str(OUT_DIR / "dashboard-desktop.png"), full_page=True)

        page.goto("http://127.0.0.1:5173", wait_until="domcontentloaded")
        page.wait_for_selector('img[src*="/uploads/"]', timeout=10000)

        browser.close()
        browser = p.chromium.launch(headless=True)
        page = browser.new_page(viewport={"width": 1440, "height": 1000})
        page.on("console", lambda msg: console_errors.append(msg.text) if msg.type == "error" else None)

        page.goto("http://127.0.0.1:5173/explore", wait_until="domcontentloaded")
        page.wait_for_selector('a[href^="/works/"]', timeout=20000)
        explore_overflow = page.evaluate(
            "document.documentElement.scrollWidth - window.innerWidth"
        )
        if explore_overflow > 1:
            raise AssertionError(f"Explore page has {explore_overflow}px overflow")
        page.screenshot(path=str(OUT_DIR / "explore-desktop.png"), full_page=True)

        page.locator('a[href^="/works/"]').first.click()
        page.wait_for_selector('img[src*="/uploads/"]', timeout=20000)
        page.screenshot(path=str(OUT_DIR / "work-detail-desktop.png"), full_page=True)

        page.set_viewport_size({"width": 390, "height": 844})
        page.goto("http://127.0.0.1:5173", wait_until="domcontentloaded")
        overflow = page.evaluate(
            "document.documentElement.scrollWidth - window.innerWidth"
        )
        if overflow > 1:
            raise AssertionError(f"Mobile page has {overflow}px horizontal overflow")
        page.screenshot(path=str(OUT_DIR / "home-mobile.png"), full_page=True)

        browser.close()

    if console_errors:
        print("Console errors:")
        for error in console_errors:
            print(error)
        raise AssertionError("Browser console reported errors")

    print(f"Smoke test passed for user {username}")
    print(f"Screenshots written to {OUT_DIR}")


if __name__ == "__main__":
    before = main_database_snapshot()
    with managed_test_environment():
        main()
    after = main_database_snapshot()
    print_main_database_comparison(before, after)

import time
from pathlib import Path

from playwright.sync_api import sync_playwright


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

        page.goto("http://127.0.0.1:5173", wait_until="networkidle")
        page.wait_for_selector("text=Lumina", timeout=10000)
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

        page.goto("http://127.0.0.1:5173/register", wait_until="networkidle")
        page.locator('input[placeholder="lumina_user"]').fill(username)
        page.locator('input[placeholder="至少 8 位"]').fill("password123")
        page.locator('input[placeholder="you@example.com"]').fill(f"{username}@example.com")
        page.get_by_role("button", name="创建账号").click()
        page.wait_for_selector("text=欢迎回来", timeout=10000)

        page.locator('input[placeholder="name@example.com"]').fill(username)
        page.locator('input[placeholder="至少 8 位"]').fill("password123")
        page.get_by_role("button", name="登录", exact=True).click()
        page.wait_for_selector("text=我的作品库", timeout=10000)

        page.locator('input[type="file"]').set_input_files(
            str(ROOT / "public/demo/work-1.jpg")
        )
        page.get_by_role("button", name="上传图片").click()
        page.wait_for_selector("text=work-1.jpg", timeout=15000)
        page.screenshot(path=str(OUT_DIR / "dashboard-desktop.png"), full_page=True)

        page.set_viewport_size({"width": 390, "height": 844})
        page.goto("http://127.0.0.1:5173", wait_until="networkidle")
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
    main()

/**
 * Driver bàn đo Story 2.5b — Task 1.3 (Chromium) và một lượt đối chứng WebKit.
 *
 * ⚠️ Đây là **tạo tác của một lượt đo**, không phải công cụ của dự án. Không có gì ở đây
 * vào `package.json` — cùng khuôn `2-2-ban-do/` · `2-3-ban-do/` · `2-4-ban-do/` · `2-5-ban-do/`.
 *
 * 🔴 PLAYWRIGHT CỐ Ý **KHÔNG** LÀ PHỤ THUỘC CỦA KHO — cửa NFR15. Nó được cài ra một thư
 * mục **ngoài cây nguồn**, và đường dẫn đi vào qua biến môi trường `AURA_PW`. Một
 * `import 'playwright'` trần ở đây sẽ buộc gói phải nằm trong `node_modules/` của kho.
 *
 * 🔴 **NHÁNH WEBKIT Ở ĐÂY KHÔNG TRẢ LỜI TASK 1.2.** Playwright-WebKit ≠ WKWebView của
 * Tauri, và Story 2.3 đã trả giá đúng chỗ này: Playwright-WebKit **có** tạo vùng chọn ở
 * lượt bấm vào văn bản chỉ-đọc, WKWebView **không** — nên một bàn đo Playwright cho lượt
 * XANH trên một sản phẩm mà chuột thật không dùng được (`EditorPanel.vue:474-479`).
 * ⇒ Số của Task 1.2 lấy ở `e2e/specs/grid-empty-cell-typing.e2e.mjs`. Nhánh WebKit ở đây
 * chỉ để **so** hai engine ở vế hình học (`subgrid` gap + auto-sizing).
 *
 * Chạy:
 *   PW=/tmp/pw-aura; mkdir -p $PW && (cd $PW && npm init -y && npm i playwright@1.62.1)
 *   npx --yes playwright@1.62.1 install chromium webkit      (một lần, nếu chưa có)
 *   AURA_PW=$PW/node_modules/playwright/index.mjs \
 *     node _bmad-output/implementation-artifacts/2-5b-ban-do/chup.mjs
 *
 * 🔴 LỖI HẠ TẦNG KHÔNG PHẢI MỘT KẾT QUẢ. Không mở được engine, không đọc được báo cáo,
 *    hay báo cáo thiếu một mệnh đề ⇒ thoát khác 0 kèm câu nói rõ đó là lỗi hạ tầng.
 */
const pwPath = process.env.AURA_PW
if (!pwPath) {
  console.error(
    '[ha tang] thieu bien moi truong AURA_PW (duong dan toi playwright NGOAI cay nguon) — day la loi ha tang, khong phai mot ket qua.',
  )
  process.exit(2)
}
const { chromium, webkit } = await import(pwPath)
import { fileURLToPath, pathToFileURL } from 'node:url'
import { dirname, join } from 'node:path'
import { writeFileSync } from 'node:fs'

const here = dirname(fileURLToPath(import.meta.url))
const bench = pathToFileURL(join(here, '..', '2-5b-ban-do-luoi.html')).href

const ENGINES = [
  ['blink', chromium],
  ['webkit', webkit],
]
const THEMES = ['light', 'dark']

const collected = {}

for (const [engineName, engine] of ENGINES) {
  const browser = await engine.launch()
  try {
    for (const theme of THEMES) {
      const page = await browser.newPage({
        viewport: { width: 1180, height: 900 },
        deviceScaleFactor: 2,
      })
      await page.goto(bench)
      await page.evaluate((t) => {
        document.documentElement.dataset.theme = t
      }, theme)

      await page.waitForFunction(() => typeof window.__benchRun === 'function')
      const report = await page.evaluate(() => window.__benchRun())

      // ── Đường CHUỘT THẬT — mệnh đề ① ở dạng trung thực nhất mà engine này cho phép ──
      const box = await page.evaluate(() => window.__benchEmptyCellRect())
      await page.mouse.click(box.x, box.y)
      const mouse = await page.evaluate(() => window.__benchNoteMouse())
      const mouseType = await page.evaluate(() => window.__benchTypeAfterMouse())

      // ── ②③ bằng BÀN PHÍM THẬT — không qua `execCommand` ─────────────────────────
      //
      // 🔴 Đây là chỗ Playwright mạnh hơn bộ e2e: `keyboard.press` đi vào đường nhập văn
      // bản gốc của engine, còn `browser.keys()` của WebdriverIO thì không.
      await page.evaluate(() => window.__benchArm())
      await page.mouse.click(box.x, box.y)
      await page.keyboard.type('An')
      await page.keyboard.press('Backspace')
      await page.keyboard.press('Backspace')
      // Lượt thứ ba: ô ĐÃ RỖNG, caret ở offset 0 — đây là ca Story 2.9 phải bắt.
      await page.keyboard.press('Backspace')
      const keyboard = await page.evaluate(() => window.__benchDrain())

      // Ca THẬT của Story 2.9 — `Backspace` ở **đầu một ô CÓ CHỮ**, không ở một ô đã rỗng.
      await page.evaluate(() => window.__benchArmFilled())
      await page.keyboard.press('Backspace')
      const atStart = await page.evaluate(() => window.__benchDrain())

      const key = `${engineName}-${theme}`
      collected[key] = {
        ...report,
        mouse,
        mouse_go: mouseType,
        ban_phim_that: keyboard,
        backspace_dau_o_co_chu: atStart,
      }

      // Tự kiểm: báo cáo thiếu một mệnh đề ⇒ bàn đo hỏng, KHÔNG phải một kết quả.
      for (const field of ['hang', 'o_rong', 'go_mot_ky_tu', 'backspace_dau_o']) {
        if (collected[key][field] === undefined) {
          console.error(
            `[ha tang] ${key}: bao cao thieu "${field}" — day la loi ha tang, khong phai mot ket qua.`,
          )
          process.exit(2)
        }
      }

      await page.screenshot({ path: join(here, `2-5b-luoi-${key}.png`), fullPage: true })
      await page.close()
      console.log(`[ok] ${key}`)
    }
  } finally {
    await browser.close()
  }
}

writeFileSync(join(here, 'bao-cao.json'), JSON.stringify(collected, null, 1) + '\n')

console.log('\n=== NAM MENH DE ===')
for (const [key, r] of Object.entries(collected)) {
  console.log(`\n--- ${key} ---`)
  console.log(`  subgrid khai bao duoc: ${r.subgrid_khai_bao_duoc}`)
  console.log(
    `  (5) hang thang: lech top lon nhat ${r.lech_top_lon_nhat_px}px · lech bottom lon nhat ${r.lech_bottom_lon_nhat_px}px`,
  )
  console.log(`      chieu cao tung hang: ${r.hang.map((h) => h.cao_px).join(' / ')}`)
  console.log(
    `  (4) o rong cao ${r.o_rong.cao_o_px}px · mot dong o co chu ${r.o_rong.cao_mot_dong_o_co_chu_px}px · bang mot dong=${r.o_rong_cao_bang_mot_dong} (caret rect API tra ${JSON.stringify(r.caret_trong_o_rong)} — gioi han API, xem chu thich)`,
  )
  const kb = r.ban_phim_that
  const bi = kb.su_kien.filter((e) => e.loai === 'beforeinput')
  console.log(
    `  BAN PHIM THAT: van_ban sau cung="${kb.van_ban}" cao_o=${kb.cao_o_px}px type=${kb.selection_type} · ${bi.length} beforeinput: ${bi.map((e) => `${e.inputType}${e.cancelable ? '(huy duoc)' : '(KHONG huy duoc)'}`).join(' · ')}`,
  )
  console.log(
    `  (1) caret bang chuong trinh=${r.caret_dat_duoc_bang_chuong_trinh} · bang CHUOT THAT: active_la_o_rong=${r.mouse.active_la_o_rong} type=${r.mouse.selection_type} neo_trong_o=${r.mouse.neo_trong_o} caret=${JSON.stringify(r.mouse.caret)}`,
  )
  console.log(
    `  (2) go: exec=${r.go_mot_ky_tu.exec_tra_ve} beforeinput=${JSON.stringify(r.go_mot_ky_tu.beforeinput)} ha_canh=${r.go_mot_ky_tu.chu_ha_canh} · sau CHUOT: exec=${r.mouse_go.exec_tra_ve} ha_canh=${r.mouse_go.chu_ha_canh}`,
  )
  const bs = r.backspace_dau_o_co_chu
  const bsEv = bs.su_kien.filter((e) => e.loai === 'beforeinput')
  console.log(
    `  (3) BACKSPACE o DAU mot o CO CHU (ca that cua Story 2.9): ${bsEv.length} beforeinput: ${bsEv.map((e) => `${e.inputType}${e.cancelable ? '(huy duoc)' : '(KHONG huy duoc)'}`).join(' · ') || '(khong co)'} · van_ban con lai="${bs.van_ban}"`,
  )
}

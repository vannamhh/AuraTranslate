/**
 * Driver bàn đo Story 2.5 — Quyết định #2 (hai vạch lề cùng tồn tại).
 *
 * ⚠️ Đây là **tạo tác của một lượt đo**, không phải công cụ của dự án. Không có gì ở đây
 * vào `package.json` — cùng khuôn `2-2-ban-do/` · `2-3-ban-do/` · `2-4-ban-do/`.
 *
 * 🔴 PLAYWRIGHT CỐ Ý **KHÔNG** LÀ PHỤ THUỘC CỦA KHO — AC16 và cửa NFR15.
 * Nó được cài ra một thư mục **ngoài cây nguồn**, và đường dẫn đi vào qua biến môi
 * trường `AURA_PW`. Một `import 'playwright'` trần ở đây sẽ buộc gói phải nằm trong
 * `node_modules/` của kho, tức đúng thứ AC16 cấm — nên `import` ở đây là **động**.
 *
 * Chạy:
 *   PW=/tmp/pw; mkdir -p $PW && cd $PW && npm init -y && npm i playwright@1.62.1
 *   npx --yes playwright@1.62.1 install chromium webkit      (một lần, nếu chưa có)
 *   AURA_PW=$PW/node_modules/playwright/index.mjs \
 *     node _bmad-output/implementation-artifacts/2-5-ban-do/chup.mjs
 *
 * 🔴 LỖI HẠ TẦNG KHÔNG PHẢI MỘT KẾT QUẢ. Không mở được engine, không đọc được báo cáo,
 *    hay báo cáo thiếu một biến thể ⇒ thoát khác 0 kèm câu nói rõ đó là lỗi hạ tầng.
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
const bench = pathToFileURL(join(here, '..', '2-5-ban-do-hai-vach.html')).href

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
      // ⚠️ `deviceScaleFactor: 3` KHÔNG phải trang trí: thứ phải đọc được trên ảnh là một
      // vạch **2px**, và ở tỉ lệ 1× hai làn cách nhau 3px là ba pixel trên màn hình Ice.
      const page = await browser.newPage({
        viewport: { width: 1180, height: 900 },
        deviceScaleFactor: 3,
      })
      await page.goto(bench)
      await page.evaluate((t) => {
        document.documentElement.dataset.theme = t
      }, theme)
      // Đổi theme không đổi hình học, nhưng chạy lại phép đo cho báo cáo mang đúng nhãn.
      await page.evaluate(() => window.__benchRun())
      await page.waitForFunction(() => typeof window.__benchReport === 'function')

      const report = await page.evaluate(() => window.__benchReport())
      const key = `${engineName}-${theme}`
      collected[key] = report

      // Tự kiểm: hai fixture × bốn biến thể phải cùng có mặt — nếu không, bàn đo hỏng.
      for (const fx of ['tron', 'doi-thoai']) {
        const f = report.fixtures[fx]
        const missing = f
          ? ['hien-trang', 'a-nhieu-lan', 'b-mot-vach', 'c-chia-doc'].filter((k) => !f.variants[k])
          : ['(ca fixture)']
        if (missing.length > 0) {
          console.error(
            `[ha tang] ${key}/${fx}: bao cao thieu ${missing.join(', ')} — day la loi ha tang, khong phai mot ket qua.`,
          )
          process.exit(2)
        }
      }

      await page.screenshot({
        path: join(here, `2-5-hai-vach-${key}.png`),
        fullPage: true,
      })
      await page.close()
      console.log(`[ok] ${key}`)
    }
  } finally {
    await browser.close()
  }
}

writeFileSync(join(here, 'bao-cao.json'), JSON.stringify(collected, null, 1) + '\n')
console.log('\n=== SO DO ===')
for (const [key, r] of Object.entries(collected)) {
  console.log(`\n=== ${key} ===`)
  for (const [fx, f] of Object.entries(r.fixtures)) {
    console.log(`\n  --- fixture ${fx} ---`)
    console.log(
      `  dong: ${f.dong.tong_so_dong_co_vach} dong co vach · ${f.dong.so_dong_mang_TU_HAI_CAU_TRO_LEN} dong mang >=2 cau · dong dong nhat ${f.dong.nhieu_cau_nhat_tren_mot_dong} cau · mot dong cao ${f.dong.cao_mot_dong_px}px`,
    )
    console.log(`        chi tiet: ${JSON.stringify(f.dong.chi_tiet)}`)
    for (const [v, d] of Object.entries(f.variants)) {
      console.log(
        `  ${v.padEnd(13)} ve_ra=${d.ve_ra} phan_biet=${d.vi_tri_phan_biet} bi_che=${d.bi_che} so_lan_can=${d.so_lan_can} buoc=${d.buoc_lan_px} mep_phai=${d.mep_phai_lan_ngoai_px}px tran=${d.tran_khoi_mang} cao_nho_nhat=${d.chieu_cao_nho_nhat_px}px noi_doi=${d.dong_bi_noi_doi}`,
      )
    }
  }
}

/**
 * Story 2.10 — **điều hướng segment** trong WKWebView THẬT. AC3 · AC6 · AC8 · AC9.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VAI CỦA TỆP NÀY — ba mệnh đề mà KHÔNG đường nghiệm thu nào khác chạm tới
 * ═════════════════════════════════════════════════════════════════════════════════
 * `tests/frontend/segmentNavigation.test.ts` đã đóng **phép chọn** *(vị từ thuần, tất định)*, và
 * `tests/frontend/editorNavNotice.test.ts` đã đóng **ô nhớ + người đọc nó**. Chép chúng sang đây
 * là dựng nguồn sự thật thứ hai — chậm hơn, chập chờn hơn, cùng một mệnh đề.
 *
 *   Ⓐ **Hợp âm đi qua được luật vùng gõ** — `keys.ts:510` + một `contenteditable` **thật**. Đây
 *     là vế payoff của Quyết định #1(c): `⌥↓` cũ **bị nuốt** khi caret ở trong ô *(đo, có đối
 *     chứng dương — `2-10-ban-do/README.md` §Task 1.3 vòng 2)*; `⌘⌥↓` phải **không**.
 *   Ⓑ **Caret hạ cánh trong một ô DOM, không rơi về `body`** — AD-34 §2.
 *   Ⓒ **Vùng nhìn cuộn tới hàng** — `happy-dom` **không bố cục**, mọi rect ở đó là 0, nên một ca
 *     vitest sẽ xanh trên cả một bản cài đúng lẫn một bản cài rỗng.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 HAI GIỚI HẠN CỦA BỘ ĐO — ghi ra thay vì để người sau tưởng đã được xét
 * ═════════════════════════════════════════════════════════════════════════════════
 * ① **`browser.keys()` đánh rơi phím bổ trợ ở phím KHÔNG-in-được, và chỉ ở đó.** Đo 2026-08-18
 *    trong chính cửa sổ này *(`2-10-ban-do/probe-hop-am-driver.e2e.mjs`)*:
 *
 *    | Lượt gọi | `code` nhận được | `metaKey` · `altKey` |
 *    |---|---|---|
 *    | `browser.keys(['Meta', '2'])` | `Digit2` | **`true`** · — |
 *    | `browser.keys(['Meta', 'Enter'])` | `Enter` | 🔴 **`false`** · — |
 *    | `browser.keys(['Meta', 'Alt', 'ArrowDown'])` | `ArrowDown` | 🔴 **`false`** · 🔴 **`false`** |
 *    | chuỗi WebDriver keys ghép sẵn | `ArrowDown` | 🔴 **`false`** · 🔴 **`false`** |
 *
 *    ⇒ Hợp âm `Mod+Alt+ArrowDown` **không bao giờ khớp** qua driver: `sameMods` thấy một
 *    `ArrowDown` trần. Đây là **giới hạn của BỘ ĐO** — hàng `Digit2` là đối chứng đi qua **cùng**
 *    đường mã, **cùng** cửa sổ, **cùng** lượt chạy, và nó mang đủ phím bổ trợ.
 *    ⇒ Spec này phát một `KeyboardEvent` **tổng hợp**, đúng tiền lệ đã ghi và đang chạy ở
 *    `editor-confirm-segment.e2e.mjs` cho **cùng** lớp giới hạn ở phím `Enter`.
 *    ⚠️ **Thứ nó KHÔNG đo:** rằng một phím **vật lý** `⌘⌥↓` sinh ra đúng sự kiện đó, và rằng
 *    macOS không nuốt hợp âm ấy trước khi webview thấy. Món cho **Ice**, cùng hạng với vế *"một
 *    phím vật lý sinh ra `beforeinput`"* của Story 2.3.
 *
 * ② 🔴 **`editor.next_segment` / `editor.prev_segment` KHÔNG có đường e2e, và đó là hệ quả trực
 *    tiếp của một chữ ký.** Quyết định #2(c) *(Ice ký 2026-08-18)* cho chúng **không phím mặc
 *    định**, nên không hợp âm nào để phát. Đường còn lại là một `dispatch` gọi từ bàn đo — tức
 *    một cửa `window.__…` **chỉ test dùng**, và kho này đã **từ chối đúng hình dạng đó hai lần**
 *    *(`store_contract.rs`: "thêm một hàm `pub` mà chỉ test gọi")*. ⇒ Không dựng.
 *    Coverage của AC1 · AC2 · AC7 nằm ở `tests/frontend/**` *(hành vi)* cộng `check:commands`
 *    *(hai id có đăng ký, `labelKey` có trong `vi.json`)*. Món nợ đã ghi kèm chủ.
 *
 * ⚠️ Mọi lượt bấm đi qua `realClick()` — ESLint **cấm** `.click()` trong `e2e/**` từ Story 1.22.
 *
 *     npm run test:e2e -- --spec e2e/specs/segment-navigation.e2e.mjs
 */
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

/**
 * 40 câu — đủ để `.grid-scroll` tràn *(bàn đo: ~18 hàng lọt vùng nhìn ở cửa sổ 1280×832)*, và đủ
 * nhỏ để lượt tạo Tác phẩm không ăn hết trần `mochaOpts.timeout` 120 s.
 * ⚠️ `sourceLang: 'zh'` ⇒ bộ tách nhìn `。`, không nhìn dấu chấm tiếng Anh.
 */
const SO_CAU = 40
const CAU = (i) => `第${i}句子内容在这里。`
const VAN_BAN = Array.from({ length: SO_CAU }, (_, i) => CAU(i + 1)).join('')
/** Hàng đích của lượt nhảy xa: đủ sâu để buộc phải cuộn. */
const HANG_DICH = 30

/**
 * Phát hợp âm bằng một `KeyboardEvent` **tổng hợp** — xem §Giới hạn ① ở đầu tệp.
 *
 * ⚠️ `code`, không `key`: `keys.ts` so **`event.code`** *(phím vật lý)*, và đó là điều kiện để
 * một hợp âm giữ nguyên vị trí phím trên mọi bố cục bàn phím.
 * ⚠️ `bubbles: true` bắt buộc — keymap nghe ở **pha capture** trên một tổ tiên, nên một sự kiện
 * không nổi bọt sẽ không rời khỏi phần tử đích.
 * 🔴 Bắn vào `document.activeElement`, **không** một selector tĩnh: `event.target` là đúng thứ
 * luật vùng gõ đọc, nên bắn nhầm chỗ là làm ca Ⓐ đo một câu hỏi khác.
 */
async function banCauChuaDichKeTiep() {
  return browser.execute(() => {
    const target = document.activeElement ?? document.body
    target.dispatchEvent(
      new KeyboardEvent('keydown', {
        code: 'ArrowDown',
        key: 'ArrowDown',
        metaKey: true,
        altKey: true,
        bubbles: true,
        cancelable: true,
      }),
    )
  })
}

/** Đổ bản dịch qua ĐÚNG lệnh flush của sản phẩm (Story 2.3), không một đường ghi riêng cho bàn đo. */
async function doChu(chapterId, edits) {
  return browser.execute(
    async (c, e) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) throw new Error('khong co cau IPC trong webview')
      return internals.invoke('save_segment_targets', { chapterId: c, edits: e })
    },
    chapterId,
    edits,
  )
}

async function docTuDia() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('khong co cau IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

/**
 * Đợi lưới nạp **đúng Chương vừa dựng**, rồi tự kiểm danh tính.
 *
 * 🔴 Khuyết tật bàn đo 2.9 mà Task 1.1 đóng: `waitForExist` **không phân biệt** *"Chương mới đã
 * nạp"* với *"Chương cũ còn đó"*. Vế *"nội dung câu đầu"* là vế duy nhất bắt được ca đó.
 */
async function doiChuong(soHang) {
  await browser.waitUntil(
    async () =>
      await browser.execute(
        (n) => document.querySelectorAll('[data-col="tgt"]').length === n,
        soHang,
      ),
    {
      timeout: 60_000,
      timeoutMsg:
        `Sau 60 giay luoi van khong co dung ${soHang} hang — loi HA TANG cua spec, khong mot\n` +
        'khuyet tat san pham. Dung doc mot ca do phia sau no thanh mot hoi quy.',
    },
  )
  const cauDau = await browser.execute(
    () => document.querySelectorAll('[data-col="src"]')[0]?.textContent ?? null,
  )
  if (cauDau !== CAU(1)) {
    throw new Error(
      `DANH TINH PHIEN KHONG KHOP — cau dau la ${JSON.stringify(cauDau)}, mong doi ` +
        `${JSON.stringify(CAU(1))}. Loi HA TANG; moi so sau dong nay noi ve du lieu sai.`,
    )
  }
}

/** Chỉ số hàng đang mang vạch `primary`. Vạch là một **class** ở cột riêng, không một `data-`. */
async function hangCoVach() {
  return browser.execute(() => {
    const o = document.querySelectorAll('.col-rule .cell-rule')
    for (let i = 0; i < o.length; i += 1) {
      if (o[i].querySelector('.rule-primary') !== null) return i
    }
    return null
  })
}

describe('Story 2.10 — điều hướng segment trong WKWebView thật', () => {
  before(async function () {
    // Lượt tạo Tác phẩm + đổ chữ cho 30 câu vượt trần 120 s trên một máy nguội.
    this.timeout(300_000)
    await openWorkspaceWithWork('E2E 2.10 — điều hướng segment', VAN_BAN)
    await doiChuong(SO_CAU)

    // 🔴 Đổ chữ vào 30 câu ĐẦU để câu chưa dịch kế tiếp nằm **xa dưới** vùng nhìn. Không có
    //    bước này, `⌘⌥↓` từ hàng 0 chỉ nhảy sang hàng 1 và ca cuộn không đo được gì.
    const tuDia = await docTuDia()
    const edits = tuDia.segments
      .slice(0, HANG_DICH)
      .map((s) => ({ id: s.id, target_text: 'Ban dich da co san tren dia.' }))
    await doChu(tuDia.chapter_id, edits)

    // Nạp lại để ảnh chụp của webview theo kịp đĩa — lượt ghi trên đi thẳng qua IPC.
    await browser.execute(() => {
      window.location.reload()
    })
    await doiChuong(SO_CAU)
  })

  /**
   * 🔴 **Ⓐ+Ⓑ — VẾ PAYOFF CỦA QUYẾT ĐỊNH #1(c).** Hợp âm cũ `⌥↓` bị `keys.ts:510` nuốt khi caret
   * trong ô đang gõ *(đo, có đối chứng dương)*, tức ca thường nhất của FR25 chết.
   * `Mod+Alt+ArrowDown` mang phím bổ trợ chính nên nó phải **đi qua** — và caret phải hạ cánh
   * **trong một ô**, không rơi về `body` (AD-34 §2).
   */
  it('Ⓐ+Ⓑ AC3 · AC9 — ⌘⌥↓ chạy khi caret đang TRONG ô, và caret hạ cánh trong ô đích', async () => {
    const o = await $$('[data-col="tgt"]')
    await realClick(o[0])
    await browser.pause(300)
    expect(await hangCoVach()).toBe(0)

    // Tiền đề: tiêu điểm đang ở một vùng gõ thật. Không có vế này thì ca đo một câu hỏi khác.
    const truoc = await browser.execute(() => ({
      laVungGo: document.activeElement ? document.activeElement.isContentEditable === true : null,
      col: document.activeElement ? document.activeElement.getAttribute('data-col') : null,
    }))
    expect(truoc.laVungGo).toBe(true)
    expect(truoc.col).toBe('tgt')

    await banCauChuaDichKeTiep()
    await browser.pause(500)

    // 30 câu đầu đã có chữ ⇒ câu chưa dịch kế tiếp là hàng 30. AC4 hai vế đang làm việc ở đây.
    expect(await hangCoVach()).toBe(HANG_DICH)

    const sau = await browser.execute(() => ({
      ten: document.activeElement ? document.activeElement.nodeName : null,
      col: document.activeElement ? document.activeElement.getAttribute('data-col') : null,
      laVungGo: document.activeElement ? document.activeElement.isContentEditable === true : null,
    }))
    expect(sau.ten).not.toBe('BODY')
    expect(sau.col).toBe('tgt')
    expect(sau.laVungGo).toBe(true)
  })

  /**
   * 🔴 **Ⓒ — AC8, và `happy-dom` KHÔNG thay được ca này.** Lượt nhảy ở ca trên đi từ hàng 0 tới
   * hàng 30, vượt xa vùng nhìn ⇒ hộp cuộn phải đã dịch, và hàng đích phải nằm **trọn** trong nó.
   *
   * ⚠️ **BA phép so, và ba là số tối thiểu — bản đầu chỉ có hai và nó KHÔNG đo mã sản phẩm.**
   * Task 7.3 gỡ lời gọi `cuonToiHang` khỏi `GridPanel.vue` và ca này **vẫn xanh**: `focus()` tự
   * cuộn, nên hai phép so đầu *("đã cuộn" · "nằm trọn")* đúng ở **cả hai** thế giới. Bàn đo vòng
   * 3 chỉ ra chỗ chúng khác nhau: `focus()` trần **căn giữa** *(1569)*, đường đã ký cho ngữ
   * nghĩa **nearest** *(1242)*. ⇒ Phép so thứ ba, ở ca Ⓓ, là phép so **duy nhất** phân biệt
   * được — và nó cũng chính là mệnh đề người dùng cảm thấy.
   */
  it('Ⓒ AC8 — vùng nhìn đã cuộn, hàng đích nằm TRỌN trong hộp, và tổ tiên KHÔNG bị đụng', async () => {
    const r = await browser.execute((idx) => {
      const hop = document.querySelector('.grid-scroll')
      const o = document.querySelectorAll('[data-col="tgt"]')[idx]
      const hb = hop.getBoundingClientRect()
      const ob = o.getBoundingClientRect()
      return {
        scrollTop: hop.scrollTop,
        namTronTrongHop: ob.top >= hb.top - 1 && ob.bottom <= hb.bottom + 1,
        // 🔴 Tác dụng phụ mà đường `scrollIntoView` gây ra và đường đã ký thì không — bàn đo
        //    vòng 2 đo được `SECTION.panel` (`overflow-y: hidden`) bị cuộn 0 → 18. Ca này là
        //    lưới tự động **duy nhất** cho vế đó.
        scrollTopPanel: document.querySelector('.panel')?.scrollTop ?? null,
      }
    }, HANG_DICH)

    expect(r.scrollTop).toBeGreaterThan(0)
    expect(r.namTronTrongHop).toBe(true)
    expect(r.scrollTopPanel).toBe(0)
  })

  /**
   * 🔴 **Ⓔ — NGỮ NGHĨA *NEAREST*, và đây là ca DUY NHẤT phân biệt được hai bản cài.** Một phím
   * điều hướng được bấm **liên tục**; nếu mỗi lượt **căn giữa** lại vùng nhìn thì người dùng
   * thấy cả khung chữ nhảy chứ không thấy một hàng được đưa tới.
   *
   * Bàn đo vòng 3 (`2-10-ban-do/focus-co-tu-cuon-khong.e2e.mjs`) đo được hai đường **đi tới hai
   * chỗ khác nhau** trên cùng một hàng: `focus()` trần ⇒ `scrollTop` **1569** *(căn giữa)*;
   * `focus({preventScroll:true})` + `cuonToiHang` ⇒ **1242** *(nearest)*.
   *
   * ⇒ Phép so đúng là **ĐỘ DỊCH**, không phải *"đứng yên"*. Đi xuống **một** hàng, `nearest` dịch
   * **nhiều nhất một chiều cao hàng** *(và thường đúng bằng, khi hàng đích vừa ló khỏi mép dưới)*;
   * một bản căn giữa dịch **hàng trăm** pixel ở cùng lượt.
   *
   * ⚠️ **Bản đầu của ca này khẳng định `scrollTop` KHÔNG đổi, và nó ĐỎ — tiền đề sai, không phải
   * mã sai.** Đo được: 482 → **520**, đúng **38 px = một chiều cao hàng**. Lý do: sau lượt nhảy
   * xa ở ca trên, hàng 30 nằm sát **mép dưới** vùng nhìn, nên hàng 31 ở **ngoài** — `nearest`
   * phải dịch đúng một hàng. Ca cũ đọc một hành vi đúng thành một khuyết tật.
   * ⇒ Giữ lại con số ấy ở đây thay vì xoá: nó là bằng chứng rằng phép so hiện tại được **đo**
   *   chứ không được đoán.
   */
  it('🔴 Ⓔ AC8 — đi xuống một hàng dịch ĐÚNG MỘT hàng (nearest), không căn giữa lại', async () => {
    const truoc = await browser.execute(() => ({
      scrollTop: document.querySelector('.grid-scroll').scrollTop,
      caoHang: Math.round(
        document.querySelectorAll('[data-col="tgt"]')[0].getBoundingClientRect().height,
      ),
    }))

    // Con trỏ đang ở hàng 30. `next_untranslated` bỏ qua **chính** câu đang đứng, và hàng 31
    // cũng chưa dịch, nên lượt này đi đúng **một** hàng xuống.
    await banCauChuaDichKeTiep()
    await browser.pause(400)

    const sau = await browser.execute(() => document.querySelector('.grid-scroll').scrollTop)
    expect(await hangCoVach()).toBe(HANG_DICH + 1)

    const dich = Math.abs(sau - truoc.scrollTop)
    // 🔴 Cận trên là **một chiều cao hàng cộng 1 px làm tròn**. Một bản căn giữa cho hàng trăm.
    expect(dich).toBeLessThanOrEqual(truoc.caoHang + 1)
  })

  /**
   * 🔴 **Ⓕ — AC6 ra tới MÀN HÌNH.** Đây là mệnh đề mà một `console.info` không bao giờ đạt được,
   * và nó là khoảng im lặng story này tồn tại để đóng.
   */
  it('Ⓕ AC6 — hết câu chưa dịch thì thanh trạng thái HIỆN CHỮ, và con trỏ ở nguyên', async () => {
    // ⚠️ Bắt đầu từ hàng **hiện tại**, không từ một hằng số: ca Ⓓ đã dời con trỏ, và một ca
    //    giả định vị trí của ca trước là một ca sẽ hỏng ở lượt sắp xếp lại kế tiếp.
    const dangO = await hangCoVach()
    for (let i = dangO; i < SO_CAU - 1; i += 1) {
      await banCauChuaDichKeTiep()
      await browser.pause(120)
    }
    expect(await hangCoVach()).toBe(SO_CAU - 1)

    await banCauChuaDichKeTiep()
    await browser.pause(400)

    const cau = await browser.execute(() => {
      const el = document.querySelector('.status .notice')
      return el === null ? null : el.textContent
    })
    expect(cau).toBe('Không còn câu nào chưa dịch ở phía dưới — con trỏ giữ nguyên.')
    // 🔴 KHÔNG quay vòng về đầu Chương — quyết định có chữ ký, ghi lý do ở `segmentNavigation.ts`.
    expect(await hangCoVach()).toBe(SO_CAU - 1)
  })
})

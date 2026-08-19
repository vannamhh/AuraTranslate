/**
 * Dọn state cấp module của các panel giữa hai spec — **AC2 của Story 2.12**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO TỆP NÀY TỒN TẠI
 * ═════════════════════════════════════════════════════════════════════════════════
 * Đo 2026-08-18 (Task 0.3): `e2e/support/**` có **0** hook `before`/`beforeEach`. Bù lại,
 * **CHÍN** lượt `window.location.reload()` nằm rải trong **SÁU** tệp spec — và các spec còn
 * lại không vá gì cả. *(Hồ sơ story ghi "ba spec"; đó là một phần ba bề mặt thật.)*
 *
 * Fixture `openWorkspaceWithWork` tạo Tác phẩm qua IPC `create_work_from_text`, tức nó **đi
 * vòng qua** `modes/libraryImport.ts::finishSubmit` — đường DUY NHẤT trong sản phẩm gọi các
 * hàm `reset*`. Nên state của spec trước sống nguyên sang spec sau.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 ICE KÝ ĐƯỜNG (b) — 2026-08-18, quyết định #5
 * ═════════════════════════════════════════════════════════════════════════════════
 * *"Fixture gọi THẲNG các hàm reset."* Hai đường bị loại, và lý do là số đo:
 *
 * - **(a) chuẩn hoá `reload()` vào fixture** — nó reset bằng cách **giết cả webview state**,
 *   nên nó che luôn những rò rỉ thật mà cổng `check:panel-refs` (AC5) tồn tại để thấy. Một
 *   bộ đo không phân biệt được *"module sạch"* với *"trang vừa dựng lại"* thì không đo AC2.
 * - **(c) một phiên app mới mỗi spec** — lượt trọn bộ thứ chín **đã** mất 18m51s, và ba ca
 *   **đã** đụng trần `mochaOpts.timeout` 120 s.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÀ VÌ SAO ĐƯỜNG NÀY LÀ **0 DÒNG MÃ SẢN PHẨM** — đo, không suy
 * ═════════════════════════════════════════════════════════════════════════════════
 * Chữ ký #5 mang một ràng buộc: (b) cần *"một đường gọi được từ driver"*, và nếu đường đó
 * phải **phơi một thứ lên `window`** thì đó là mã sản phẩm phục vụ bộ đo — đúng hình dạng
 * đã bị loại ở quyết định #4(b), tức **cửa `AD` kích hoạt lại**.
 *
 * Nó không phải như vậy, và đây là phép đo (2026-08-18, Vite dev trên cây thật):
 * mọi đường `/src/…` trong module đã biến đổi của `main.ts` **không mang một query string
 * nào** — `"/src/panels/editorPanelState.ts"`, không `"…?v=abc"`. Đo bằng
 * `curl -s …/src/main.ts | grep -oE '"/src/[^"]*"'`, 19 đường, 0 đường có `?`.
 *
 * ⇒ Một `import('/src/panels/editorPanelState.ts')` chạy trong trang phân giải về **đúng
 * URL app đã nạp**, và registry module của ES phân biệt theo URL đã phân giải ⇒ **cùng một
 * module record, cùng những ô nhớ cấp module**. Không cần một dòng nào trong `src/`.
 *
 * ⚠️ **Và mệnh đề đó phải được KIỂM ở lúc chạy, không tin suông** — xem hàng rào ở
 * [`resetPanelState`]. Một `import()` cho ra một **bản sao** module (URL lệch một ký tự,
 * một lượt HMR chèn `?t=`) sẽ dọn state của bản sao và trả về **thành công**. Đó đúng là
 * hình dạng *"xanh giả"* mà cả story này tồn tại để chống: fixture báo đã dọn, state thật
 * còn nguyên, và spec sau đỏ vì một lý do không ai lần được.
 */

/**
 * Năm module mang state **theo Tác phẩm/Chương**, cùng tên hàm reset của mỗi module.
 *
 * 🔴 **`dictSourcesState` và `lookupTiming` CỐ Ý vắng mặt, và cả hai đều có hàm reset.**
 * Đây không phải một chỗ bỏ sót:
 *   · `resetDictSources()` xoá tập nguồn bị tắt — cấu hình tầng **Global**, chỉ nạp lại một
 *     lần lúc khởi động. Gọi nó ở đây làm mọi nguồn người dùng đã tắt bật lại trong bộ nhớ
 *     trong khi đĩa giữ tập cũ. Xem doc-comment của chính hàm đó.
 *   · `resetLookupTiming()` dọn bàn đo độ trễ — một công cụ chẩn đoán, không state của phiên.
 */
const PANEL_MODULES = [
  ['/src/panels/editorPanelState.ts', 'resetEditorPanel'],
  ['/src/panels/sourcePanelState.ts', 'resetSourcePanel'],
  ['/src/panels/lookupPanelState.ts', 'resetLookupPanel'],
  ['/src/panels/lookupHistoryState.ts', 'resetLookupHistory'],
  ['/src/panels/segmentHistoryState.ts', 'resetSegmentHistory'],
]

/**
 * 🔴 **VỨT STATE CŨ LÀ CHƯA ĐỦ — PHẢI NẠP LẠI. Đây là vế mà bản đầu của tệp này THIẾU, và
 * nó đã trả giá bằng một lượt trọn bộ.**
 *
 * ## Phép đo, 2026-08-18 → 19
 *
 * Bản đầu chỉ gọi năm hàm `reset*` rồi coi là xong. Lượt trọn bộ **thứ mười** cho **5 passed
 * / 6 failed** *(mốc gốc: 8/3)*, tức **xấu đi ba spec**. Chẩn đoán trung thực ở lượt **thứ
 * mười một**: `Lần đọc cuối: 0` ở **mọi** ca đỏ, không một lượt `browser.execute` nào ném.
 * Lưới đọc được **0 hàng ở mọi vòng suốt 30 giây** — nó **không bao giờ nạp**, chứ không nạp
 * chậm.
 *
 * ## Vì sao — mã sản phẩm đã viết sẵn câu trả lời
 *
 * `modes/libraryImport.ts::finishSubmit` *(:173-197)* nói thẳng:
 *
 * > 🔴 **VỨT state cũ là CHƯA ĐỦ — phải NẠP LẠI ngay tại đây.** `resetSourcePanel()` gỡ cờ
 * > `chapterRequested`, nhưng không ai gọi lại hàm nạp. Chỗ DUY NHẤT gọi
 * > `ensureChapterLoaded()` là `GridPanel.vue::onMounted`, mà ba chế độ sống trong
 * > `<KeepAlive>` — *"lần hiện thứ hai trở đi không có `mounted`"*.
 *
 * ⇒ Một lượt reset **không kèm** hai lời gọi nạp để lưới rỗng **vĩnh viễn**. Đúng khuyết tật
 * đã bắt bằng test tay 2026-08-07, và `finishSubmit` là bản vá của nó.
 *
 * ## 🔴 Và vì sao lượt `window.location.reload()` cũ CHE được điều này
 *
 * Hồ sơ Story 2.12 và `deferred-work.md` đều mô tả chín lượt `reload()` là *"vá của BÀN ĐO
 * cho state cấp module rò"*. **Mô tả đó thiếu một nửa.** `reload()` dựng lại toàn bộ webview,
 * nên nó chạy lại `main.ts` → `GridPanel.vue::onMounted` → `ensureChapterLoaded()`. Tức nó
 * mang **hai** vai: dọn state **và** phát một lượt nạp. Gỡ nó mà chỉ thay vai thứ nhất là
 * đúng khuôn *"chữ ký thi hành đúng MỘT NỬA"* mà retro Epic 2 đã gọi tên **năm** lần.
 *
 * ⚠️ Ghi ra vì nó đổi cách đọc quyết định #5: đường (a) *(chuẩn hoá `reload()`)* bị loại vì
 * *"giết cả webview state, che luôn những rò rỉ thật"* — lý do ấy vẫn đúng, nhưng nó **không
 * biết** rằng thứ nó "giết" bao gồm một vế bắt buộc. Đường (b) chỉ đủ khi nó soi **cả hai**
 * nửa của `finishSubmit`.
 */
const LOAD_CALLS = [
  ['/src/panels/sourcePanelState.ts', 'ensureChapterLoaded'],
  ['/src/panels/editorPanelState.ts', 'ensureSegmentsLoaded'],
]

/**
 * Dọn state panel trong trang đang mở, **rồi phát lại lượt nạp** — soi đúng `finishSubmit`.
 *
 * 🔴 Gọi **SAU** lượt tạo Tác phẩm, không trước: `finishSubmit` chạy sau khi
 * `create_work_from_*` trả về, tức sau khi `replace_open_work` phía Rust đã trỏ
 * `OpenWorkState` sang Tác phẩm mới. Nạp trước lượt trỏ đó là nạp Tác phẩm **cũ**.
 *
 * @returns {Promise<{ called: string[], rowsAfterReset: number }>}
 * @throws {Error} khi cầu `import()` không cho đúng module của app, hay khi một tên đã đổi
 */
export async function resetPanelState() {
  const outcome = await browser.execute(
    async (modules, loads) => {
      const called = []
      const grab = async (path, name) => {
        let mod
        try {
          mod = await import(/* @vite-ignore */ path)
        } catch (err) {
          return { err: `import("${path}") ném: ${String(err)}` }
        }
        const fn = mod[name]
        if (typeof fn !== 'function') {
          return {
            err:
              `"${path}" không xuất một hàm tên \`${name}\`. ` +
              `Nó xuất: ${Object.keys(mod).join(', ') || '(không gì cả)'}`,
          }
        }
        return { fn }
      }

      for (const [path, fnName] of modules) {
        const got = await grab(path, fnName)
        if (got.err !== undefined) return { ok: false, detail: got.err }
        got.fn()
        called.push(fnName)
      }

      // 🔴 Đọc NGAY SAU reset và TRƯỚC lượt nạp — đây là cửa sổ duy nhất mà lưới phải rỗng,
      // và nó là bằng chứng quan sát-được-từ-ngoài rằng `import()` chạm đúng module app đọc.
      //
      // ═══════════════════════════════════════════════════════════════════════════════
      // 🔵 CODE REVIEW BA TẦNG 2026-08-19 — CHỜ TRẠNG THÁI, KHÔNG ĐỌC MỘT LẦN
      // ═══════════════════════════════════════════════════════════════════════════════
      // 🔴 Bản đầu đọc `querySelectorAll(...).length` **một lần, đồng bộ, ngay sau** vòng
      // reset. Vue vá DOM **bất đồng bộ** *(bộ lập lịch chạy trên microtask)*, nên lượt đọc
      // ấy chỉ thấy 0 hàng **vì** `editorPanelState` tình cờ đứng **ĐẦU** `PANEL_MODULES` và
      // bốn lượt `await import()` sau nó vô tình cho bộ lập lịch kịp chạy.
      //
      // ⚠️ Đó là một bảo đảm **do thứ tự một mảng dữ liệu**, không do một mệnh đề. Đảo
      // `editorPanelState` xuống cuối bảng là hàng rào **đỏ oan MỌI lượt**, và câu chẩn đoán
      // sẽ nói *"cầu reset đang dọn một BẢN SAO"* cho một cuộc đua hoàn toàn vô hại — đúng
      // lớp *"chẩn đoán sai trên một bàn đo"* mà `gridWait`/`flushWait` viết cả một đoạn dài
      // để tránh, chỉ khác là nó nằm trong chính hàng rào mới.
      //
      // ⇒ Chờ **trạng thái đích** với một trần, thay vì tin một thứ tự ngầm. Trần 2 s là
      // rộng gấp nhiều lần một lượt vá DOM *(vài microtask)*; nó chỉ tồn tại để hàng rào
      // KHÔNG treo vô hạn khi reset thật sự không chạm module app.
      const deadline = Date.now() + 2000
      let rowsAfterReset = document.querySelectorAll('[data-col="src"]').length
      while (rowsAfterReset !== 0 && Date.now() < deadline) {
        await new Promise((r) => requestAnimationFrame(() => r()))
        rowsAfterReset = document.querySelectorAll('[data-col="src"]').length
      }

      for (const [path, fnName] of loads) {
        const got = await grab(path, fnName)
        if (got.err !== undefined) return { ok: false, detail: got.err }
        // 🔵 **CODE REVIEW BA TẦNG 2026-08-19** — bản đầu viết `void got.fn()`. Hai hàm này
        // async và `finishSubmit` cũng không chờ chúng, nên **không chờ** là đúng vai. Nhưng
        // `void` **bỏ luôn promise**, nên một rejection thành một *unhandled rejection trong
        // context trình duyệt*: nó không truyền về Node, không về WebdriverIO, và nguyên nhân
        // thật **biến mất**. Spec sau đó chỉ thấy `waitForGridRows` hết giờ 30/60 s.
        // ⇒ Vẫn không chờ, nhưng **giữ NGUYÊN VĂN** — đúng luật mà `flushWait`/`gridWait`
        // viết rất kỹ ở chỗ khác trong cùng story này.
        got.fn().catch((err) => {
          const text = `${fnName}() ném (bất đồng bộ, SAU lượt reset): ${err instanceof Error ? (err.stack ?? err.message) : String(err)}`
          window.__auraPanelResetLoadError = text
          console.error(`[panelReset] ${text}`)
        })
        called.push(fnName)
      }

      return { ok: true, called, rowsAfterReset }
    },
    PANEL_MODULES,
    LOAD_CALLS,
  )

  if (!outcome.ok) {
    throw new Error(
      `Fixture KHÔNG dọn/nạp lại được state panel: ${outcome.detail}\n\n` +
        'Đây là lỗi HẠ TẦNG của bàn đo, KHÔNG một hồi quy sản phẩm. Đừng đọc một ca đỏ phía\n' +
        'sau nó thành một khuyết tật giao diện — và đừng vá bằng `window.location.reload()`:\n' +
        'đường đó đã bị loại có chữ ký (quyết định #5, Ice 2026-08-18).',
    )
  }

  // ── 🔴 HÀNG RÀO: chứng minh cầu `import()` chạm ĐÚNG module của app ────────────────
  //
  // Không có vế này, một `import()` trả về một **bản sao** module sẽ dọn state của bản sao
  // rồi báo thành công — fixture xanh, rò rỉ nguyên vẹn, spec sau đỏ vì một lý do khác hẳn.
  //
  // Phép kiểm đi qua **DOM**, có chủ ý: lưới đối chiếu render từ `segments` của
  // `editorPanelState`, nên 0 hàng `[data-col="src"]` **ngay sau reset** là bằng chứng
  // quan sát được từ bên ngoài rằng chính ô nhớ app đang đọc đã bị dọn.
  //
  // ⚠️ **GIỚI HẠN THẬT (code review 2026-08-19), ghi ra thay vì để người sau tưởng đã xét:**
  // tín hiệu `[data-col="src"]` do **MỘT** trong năm module chi phối. Ba module còn lại
  // *(`lookupPanelState` · `lookupHistoryState` · `segmentHistoryState`)* **không** có một
  // tín hiệu DOM quan sát-được tương đương ở cửa sổ này, nên hàng rào **không** chứng minh
  // riêng từng lượt `import()` của chúng.
  // ⇒ Nó chứng minh đúng một mệnh đề, và mệnh đề ấy là mệnh đề cần: **cây cầu `import()`
  // chạm bản gốc**. Năm lượt đi qua **cùng** một cơ chế phân giải URL của Vite, nên một lượt
  // trúng bản gốc là bằng chứng cho cơ chế, không chỉ cho một đỉnh. Thứ hàng rào KHÔNG bắt
  // được là một module **đổi chỗ riêng lẻ** — ca đó do lượt `grab()` ở trên bắt, bằng câu
  // *"không xuất một hàm tên …"*.
  if (outcome.rowsAfterReset !== 0) {
    throw new Error(
      `Đã gọi ${PANEL_MODULES.length} hàm reset mà lưới còn ${outcome.rowsAfterReset} hàng nguyên văn.\n\n` +
        '🔴 Nghĩa là `import()` KHÔNG chạm module mà app đang đọc — nhiều khả năng URL đã lệch\n' +
        '(một lượt HMR chèn `?t=…`, hay một tệp đổi chỗ). Cầu reset đang dọn một BẢN SAO.\n' +
        'Đo lại bằng: `curl -s http://localhost:1420/src/main.ts | grep -oE \'"/src/[^"]*"\'`\n' +
        '— mọi đường phải KHÔNG mang query string. Đây là tiền đề của quyết định #5(b).',
    )
  }

  // 🔵 2026-08-19 — vế thứ hai của lượt vá `void got.fn()`: một rejection xảy ra **kịp**
  // trước khi `execute` trả về được nêu ĐÍCH DANH ngay đây, thay vì để spec sau đọc một lượt
  // hết giờ. Một rejection **muộn** hơn thì ở lại `window.__auraPanelResetLoadError` cộng một
  // dòng `console.error` mang tiền tố `[panelReset]` — đủ để lần ra, không im lặng.
  const loadError = await browser.execute(() => window.__auraPanelResetLoadError ?? null)
  if (loadError !== null) {
    await browser.execute(() => {
      window.__auraPanelResetLoadError = undefined
    })
    throw new Error(
      `Lượt NẠP LẠI sau reset đã ném: ${loadError}\n\n` +
        'Đây là lỗi HẠ TẦNG của bàn đo, KHÔNG một hồi quy sản phẩm — và nó được nêu ở ĐÂY có\n' +
        'chủ ý: bản đầu `void` mất nguyên văn này, nên ca đỏ lộ ra sau đó là một lượt hết giờ\n' +
        '30/60 s không ai lần được về dòng nào.',
    )
  }

  return { called: outcome.called, rowsAfterReset: outcome.rowsAfterReset }
}

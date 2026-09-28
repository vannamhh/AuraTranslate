/**
 * Story 11.5 — Disposition 3, L140 (Ice quyết định #62): sau một lượt ĐỔI TÁC PHẨM
 * (`openWorkById`), một lượt NHẬP xong (`finishImportSubmission`), hay một lượt GỘP CHƯƠNG
 * (`mergeCurrentChapterUp`), tiêu điểm phải rơi vào lưới, không rơi về `body` — ba trong năm
 * chỗ gọi `enterFocus('panel.grid')` mà story này thêm (hai chỗ còn lại, `switchChapter`/
 * `openChapterById`, có TỪ TRƯỚC story này và tự khai "🔴 VẾ NÀY CHƯA CÓ ĐƯỜNG NGHIỆM THU" —
 * ngoài phạm vi ở đây).
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO GỌI HÀM SẢN PHẨM QUA `import()`, KHÔNG BẤM MỘT NÚT
 * ═════════════════════════════════════════════════════════════════════════════════
 * Cả ba hàm chỉ CÓ ĐƯỜNG GỌI qua giao diện Library (`library.open_work` · form nhập ·
 * `library.chapter_merge_up`) — và `App.vue` bọc ba chế độ trong `<KeepAlive>` với `v-if`
 * (không `v-show`): rời `workspace` DEATTACH cây DOM của `GridPanel.vue` khỏi `document`
 * (`PanelFrame.vue::onDeactivated` tự khai "tiêu điểm THẬT đã ở gốc chế độ" khi rời Workspace).
 * ⇒ Gọi ba hàm này từ ĐÚNG đường giao diện của chúng luôn chạy trong lúc `workspace` đã bị
 * rời — `enterFocus('panel.grid')` vẫn "thành công" (`resolve()` trả về phần tử thật, `.focus()`
 * không ném) nhưng KHÔNG đổi `document.activeElement`, vì phần tử đó không còn trong `document`.
 * Một bàn đo bấm nút thật sẽ XANH mà không đo được gì — đúng lớp *"cổng cho exit 0 trên sản
 * phẩm đang hỏng"* mà `AGENTS.md` cấm.
 *
 * Mệnh đề của L140 chỉ quan sát được khi `workspace` ĐANG mở lúc ba hàm này chạy — đúng ca
 * người dùng thật mà chữ ký #62 nhắm tới (ví dụ: tìm kiếm toàn Tác phẩm rồi nhảy sang Tác phẩm
 * khác trong lúc Workspace đang hiện; chưa có bề mặt giao diện nào đi thẳng đường đó hôm nay).
 * ⇒ Ba ca dưới đây gọi THẲNG hàm export thật (không viết lại logic của nó) trong khi
 * `workspace` đang mở, qua `import()` — cùng khuôn `e2e/support/panelReset.mjs` đã đo là một
 * cầu module sạch (không bản sao, không query string) và `segment-history-restore.e2e.mjs` gọi
 * thẳng `invoke()` cho đúng lý do: mệnh đề cần đo là HÌNH DẠNG/HIỆU ỨNG của lượt gọi, không
 * phải "chuột có tới được nút hay không".
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * ⚠️ GIỚI HẠN THẬT của đối chứng GỠ — ghi ra thay vì để người sau tưởng ba ca này khoá được
 * đúng ba dòng `enterFocus('panel.grid')` một cách CÔ LẬP
 * ═════════════════════════════════════════════════════════════════════════════════
 * Đối chứng thật (gỡ cả ba dòng `enterFocus('panel.grid')` khỏi `libraryChapters.ts`/
 * `libraryImport.ts`, chạy lại) cho **CẢ BA ca vẫn XANH** — tức bản thân ba dòng đó KHÔNG phải
 * seam duy nhất khoá được mệnh đề ở fixture "một Tác phẩm/Chương có sẵn ít nhất một câu" mà ba
 * ca dưới đây dùng. Lý do đo được: `ensureSegmentsLoaded()` (gọi bởi CẢ BA hàm, có TỪ Story
 * 5.7, không phải story này) tự đặt `caretPlacement.value = loaded.caret_segment_id`, và
 * `GridPanel.vue`'s watcher trên `editorCaretPlacement` tự `.focus()` vào ô của câu đó — CƠ CHẾ
 * NÀY ĐÃ ĐỦ để đưa tiêu điểm vào lưới bất cứ khi nào Chương đích có ÍT NHẤT MỘT câu, không cần
 * `enterFocus('panel.grid')`. Ba ca dưới đây vì thế đo ĐÚNG mệnh đề của AC *("tiêu điểm nằm
 * trong lưới, không rơi về body" — vẫn một mệnh đề thật, vẫn phải giữ đúng qua webview thật)*
 * nhưng KHÔNG cô lập được riêng phần đóng góp của ba dòng L140 trong trường hợp thường — seam
 * mà ba dòng đó ĐỘC QUYỀN khoá là một Chương RỖNG (0 câu, `caret_segment_id = null` ⇒ watcher
 * trên không bao giờ bắn) hoặc hai id trùng nhau qua khoá `v-for` (mỗi `.atproj` mới đều đánh số
 * segment từ 1, nên "Tác phẩm B" trong ca ①/② luôn có câu ĐẦU trùng id với câu ĐẦU của "Tác phẩm
 * A" — Vue vá tại chỗ node DOM cũ thay vì dựng mới, và tiêu điểm sống sót qua lượt đổi mà không
 * ai gọi lại `.focus()`). `openWorkspaceWithWork()` không dựng được một Chương 0 câu (chính hàm
 * chờ ≥1 hàng lưới mới coi là sẵn sàng — đo bằng thực nghiệm khi thử `text: '   '`), nên một ca
 * cô lập đúng seam đó cần một fixture khác hẳn — món nợ có chủ, ghi ở phase notes, không ở đây.
 */
import { waitForGridRows, waitForGridText } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const MODULE_LIBRARY_CHAPTERS = '/src/modes/libraryChapters.ts'
const MODULE_LIBRARY_IMPORT = '/src/modes/libraryImport.ts'

/** Tạo một Tác phẩm qua cầu IPC trần — trả nguyên văn `CreatedWork` (§Rust) cộng `work_id`. */
async function createWorkViaIpc(name, text) {
  const result = await browser.execute(
    async (workName, sourceText) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) return { ok: false, detail: 'không có cầu IPC' }
      try {
        const created = await internals.invoke('create_work_from_text', {
          name: workName,
          sourceLang: 'zh',
          genre: 'general',
          text: sourceText,
        })
        return { ok: true, created, detail: '' }
      } catch (err) {
        return { ok: false, detail: String(err && err.code ? err.code : err) }
      }
    },
    name,
    text,
  )
  if (!result.ok) {
    throw new Error(
      `Fixture không tạo được Tác phẩm "${name}": ${result.detail}\n\n` +
        'Đây là lỗi HẠ TẦNG của bàn đo, không một hồi quy sản phẩm.',
    )
  }
  return result.created
}

/** Đọc segment của Chương ĐANG MỞ — cùng khuôn `story-5-8-reorganise-chapters.e2e.mjs`. */
async function readSegmentsFromDisk() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

async function splitAtSegmentViaIpc(segmentId) {
  return browser.execute(async (sid) => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('split_chapter_at_segment', { segmentId: sid })
  }, segmentId)
}

/**
 * Dời tiêu điểm RA KHỎI lưới trước lượt gọi đang canh — không có bước này, một mệnh đề
 * *"tiêu điểm nằm trong lưới"* có thể XANH chỉ vì tiêu điểm CHƯA TỪNG rời khỏi đó từ trước
 * (một lượt `enterFocus` khác của story TRƯỚC, không phải lượt story này đang canh).
 */
async function blurActiveElement() {
  await browser.execute(() => {
    const active = document.activeElement
    if (active instanceof HTMLElement) active.blur()
  })
}

/**
 * Đọc trạng thái tiêu điểm.
 *
 * ⚠️ **Đo được, không đoán:** `enterFocus('panel.grid')` gọi `.focus()` trên GỐC `PanelFrame`
 * (`.focus.ts::enter()`) — nhưng đo tay lượt đầu cho thấy phần tử GIỮ tiêu điểm sau đó lại là
 * chính MỘT Ô của lưới (`[data-col="tgt"][data-segment-id="1"]`), không phải gốc panel: một cơ
 * chế KHÁC (đặt caret vào câu đầu khi Chương vừa nạp, cùng lớp với `grid-empty-cell.e2e.mjs`)
 * chạy SAU và "cướp" tiêu điểm sâu hơn vào trong. Cả hai đều là bằng chứng ĐÚNG cho mệnh đề
 * *"tiêu điểm nằm trong lưới"* — `hasGridDescendant` vì thế xét CẢ HAI chiều: phần tử đang giữ
 * tiêu điểm CHÍNH LÀ một ô lưới (`closest('[data-col]')`), HOẶC nó BỌC QUANH ít nhất một ô.
 */
async function focusProbe() {
  return browser.execute(() => {
    const active = document.activeElement
    const hasGridDescendant =
      active !== null &&
      active !== document.body &&
      (active.closest?.('[data-col]') !== null || active.querySelector?.('[data-col="src"]') !== null)
    return {
      isBody: active === document.body,
      tag: active?.tagName ?? null,
      hasGridDescendant,
    }
  })
}

describe('Story 11.5 · L140 — tiêu điểm rơi vào lưới sau ba lượt nạp lại, không rơi về body', () => {
  it('ĐỔI TÁC PHẨM khi Workspace đang mở (`openWorkById`) ⇒ tiêu điểm rơi vào lưới', async () => {
    const tag = `${Date.now() % 1_000_000}`
    const workB = await createWorkViaIpc(`e2e-focus-openwork-B-${tag}`, 'Cau B mot。')

    await openWorkspaceWithWork(`e2e-focus-openwork-A-${tag}`, 'Cau A mot。')
    await waitForGridText(0, 'Cau A mot。')

    await blurActiveElement()
    const before = await focusProbe()
    expect(before.isBody).toBe(true)

    await browser.execute(async (modulePath, workId) => {
      const mod = await import(/* @vite-ignore */ modulePath)
      await mod.openWorkById(workId)
    }, MODULE_LIBRARY_CHAPTERS, workB.meta.work_id)

    await waitForGridText(0, 'Cau B mot。')

    const after = await focusProbe()
    expect(after.isBody).toBe(false)
    expect(after.hasGridDescendant).toBe(true)
  })

  it('NHẬP xong khi Workspace đang mở (`finishImportSubmission`) ⇒ tiêu điểm rơi vào lưới', async () => {
    const tag = `${Date.now() % 1_000_000}`

    await openWorkspaceWithWork(`e2e-focus-import-A2-${tag}`, 'Cau A2 mot。')
    await waitForGridText(0, 'Cau A2 mot。')

    await blurActiveElement()
    const before = await focusProbe()
    expect(before.isBody).toBe(true)

    // Tác phẩm C được TẠO ở đây, ngay trước lượt gọi — `create_work_from_text` đã CHÍNH nó
    // trỏ `OpenWorkState` (phía Rust) sang C, đúng thứ `finishImportSubmission` giả định là
    // đã xảy ra khi nó chạy (nó không tự gọi lệnh tạo — xem doc-comment của chính hàm đó:
    // "gọi SAU khi lượt xác nhận bảng mã trả về").
    const createdC = await createWorkViaIpc(`e2e-focus-import-C-${tag}`, 'Cau C mot。')

    await browser.execute(
      async (modulePath, created) => {
        const mod = await import(/* @vite-ignore */ modulePath)
        mod.finishImportSubmission(created, null)
      },
      MODULE_LIBRARY_IMPORT,
      createdC,
    )

    await waitForGridText(0, 'Cau C mot。')

    const after = await focusProbe()
    expect(after.isBody).toBe(false)
    expect(after.hasGridDescendant).toBe(true)
  })

  it('GỘP Chương đang mở trong Editor vào Chương trước nó (`mergeCurrentChapterUp`) ⇒ tiêu điểm rơi vào lưới', async () => {
    const tag = `${Date.now() % 1_000_000}`

    // Bốn câu ⇒ tách ở câu thứ hai cho 1 + 3, hai con số KHÁC nhau nên mọi phép kiểm phía
    // dưới đỏ được — cùng lý do `story-5-8-reorganise-chapters.e2e.mjs::SOURCE_TEXT`.
    await openWorkspaceWithWork(`e2e-focus-merge-D-${tag}`, 'Cau mot。Cau hai。Cau ba。Cau bon。')
    await waitForGridRows(4)

    const before = await readSegmentsFromDisk()
    expect(before.segments).toHaveLength(4)
    const chapterIdA = before.chapter_id

    // Tách Chương ĐANG MỞ trong Editor thành hai: phần ĐẦU giữ NGUYÊN `chapter_id` cũ (đúng
    // quy ước AD-32 — `story-5-8` đo phần đầu giữ danh tính), phần SAU nhận một `chapter_id`
    // mới. `editorChapterId` phía webview KHÔNG tự cập nhật sau lượt tách này (đường IPC trần,
    // không qua `ensureChapterLoaded`) — vẫn trỏ đúng chapterIdA, đúng tiền đề cần cho bước gộp.
    await splitAtSegmentViaIpc(before.segments[1].id)

    // Nạp lại danh sách Chương của Tác phẩm ĐANG MỞ rồi dời con trỏ sang hàng THỨ HAI (phần
    // SAU vừa tách) — cùng hai hàm THẬT đứng sau nút "Mở Chương"/mũi tên của Library, gọi
    // thẳng vì không có nút nào của Library trong cây DOM khi Workspace đang mở (xem khối lý
    // do đầu tệp).
    const chapters = await browser.execute(async (modulePath) => {
      const mod = await import(/* @vite-ignore */ modulePath)
      await mod.loadChapters()
      mod.nextChapter()
      return mod.libraryChapters.value.map((c) => ({ chapter_id: c.chapter_id, count: c.segment_count }))
    }, MODULE_LIBRARY_CHAPTERS)
    expect(chapters).toHaveLength(2)
    expect(chapters[0].chapter_id).toBe(chapterIdA)

    await blurActiveElement()
    const before2 = await focusProbe()
    expect(before2.isBody).toBe(true)

    await browser.execute(async (modulePath) => {
      const mod = await import(/* @vite-ignore */ modulePath)
      await mod.mergeCurrentChapterUp()
    }, MODULE_LIBRARY_CHAPTERS)

    // Chương A (đang mở trong Editor) nay mang lại đủ BỐN câu — bằng chứng lượt gộp đã ghi
    // VÀ Editor đã nạp lại đúng Chương gộp, không phải một Chương rỗng/cũ.
    await waitForGridRows(4)

    const after = await focusProbe()
    expect(after.isBody).toBe(false)
    expect(after.hasGridDescendant).toBe(true)
  })
})

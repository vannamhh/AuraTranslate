/**
 * Bàn đo Story 2.9 · Task 1 — **VÒNG 2**, và nó hỏi một câu khác hẳn vòng 1.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÒNG 1 CHO MỘT BẢNG KHÔNG ĐỌC ĐƯỢC — thước hỏng, không phải engine hỏng
 * ═════════════════════════════════════════════════════════════════════════════════
 * Vòng 1 gửi `Backspace` bằng `browser.keys()` và đo được, ở **cả năm** bước:
 * `isTrusted: false` · `document.hasFocus(): false` · **0** `beforeinput` · **0** `input` ·
 * `textContent` **không đổi một byte**.
 *
 * Đối chứng **Ⓔ** là chỗ bảng tự tố cáo: caret ở **GIỮA** ô, `startOffset: 3` — một lượt xoá
 * lui tầm thường nhất trần đời — cũng **không xoá gì**. ⇒ Con số *"0 `beforeinput` ở offset 0"*
 * của vòng 1 trả lời câu *"một phím KHÔNG TIN CẬY trong một tài liệu KHÔNG CÓ TIÊU ĐIỂM làm
 * được gì"*, **không** trả lời câu *"WebKit có phát `beforeinput` ở offset 0 không"*.
 *
 * 🔵 **Và giới hạn này ĐÃ CÓ CHỦ từ 2026-08-13** — `e2e/specs/editor-typing-flush.e2e.mjs:38-54`
 * ghi nguyên văn: *"`browser.keys()` KHÔNG GÕ ĐƯỢC CHỮ […] nó synthesize `keydown`/`keyup` và
 * không đi vào đường nhập văn bản gốc"*. Vòng 1 **đo lại một thứ đã ghi** thay vì đọc nó.
 *
 * ⚠️ **Đây là một lượt sửa THƯỚC, không một vòng chẩn đoán bị bác** — LUẬT DỪNG đếm những vòng
 * mà một **giả thuyết về sản phẩm** bị phép đo bác. Phân biệt này có tiền lệ ở `2-8-ban-do/`
 * §Vòng 1 và `2-5d-ban-do/` §Debug Log Ⓐ.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÒNG NÀY CHỈ HỎI NHỮNG CÂU ĐO ĐƯỢC — và chúng là những câu quyết định hình dạng mã
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ba câu **không** đo được qua driver này, ghi ra thay vì giả vờ: ① `preventDefault()` có chặn
 * nổi lượt xoá của một phím **thật** không; ② auto-repeat của hệ điều hành; ③ `beforeinput` ở
 * offset 0 với phím **thật**. Cả ba đi vào sổ nợ / món cho Ice.
 *
 * Bốn câu **đo được**, và cả bốn dùng `caretRangeFromPoint` — API mà 2.8 đã đo là **CÓ THẬT**
 * trên WKWebView này *(khác `caretPositionFromPoint`, thứ NÉM `TypeError`)*. Nó cho đúng thứ
 * một cú bấm của người dùng cho, **không cần một sự kiện tin cậy nào**:
 *
 *   Ⓐ Engine biểu diễn *"đầu ô"* bằng `(node, offset)` nào — ô rỗng, ô một dòng, ô nhiều dòng.
 *   Ⓑ Engine biểu diễn *"đầu DÒNG THỨ HAI"* bằng `(node, offset)` nào. 🔴 Cạm bẫy ④ của story
 *     đoán *"cũng cho `startOffset === 0`"*; vòng này đo xem đoán đó đúng không.
 *   Ⓒ **Ứng viên helper** chạy đúng/sai trên **chính** những `Selection` engine vừa dựng.
 *   Ⓓ Đường `execCommand('delete')` — đường mà 2.5b đã dùng để ghi tiền đề ①.
 *
 *     TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-9-ban-do/caret-dau-o-vong2.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'

/**
 * 🔴 **ỨNG VIÊN HELPER, chép nguyên văn vào bàn đo để chạy được trên engine thật.**
 *
 * Bốn hình dạng phải cho cùng một câu trả lời, và một phép kiểm hỏi `startOffset === 0` **hỏng
 * ở hai trong bốn**. Cách duy nhất không phải liệt kê hình dạng: dựng một `Range` từ **đầu ô**
 * tới **caret** rồi hỏi nó dài bao nhiêu **ký tự**.
 *
 * Nó đúng theo định nghĩa: *"không còn ký tự nào phía trước caret trong cả ô"* — chính câu mà
 * cạm bẫy ④ đòi. Nó không hỏi node nào, không đếm `childNodes`, không giả định `pre-line` để
 * lại text node hay `<br>`.
 */
const UNG_VIEN = `
function caretODauO(cell, sel) {
  if (!sel || sel.rangeCount === 0) return false
  const r = sel.getRangeAt(0)
  if (!r.collapsed) return false            // vùng chọn ⇒ để engine xoá vùng chọn
  if (!cell.contains(r.startContainer)) return false
  const truoc = document.createRange()
  truoc.setStart(cell, 0)
  truoc.setEnd(r.startContainer, r.startOffset)
  return truoc.toString().length === 0
}
`

/** Đặt caret bằng **engine**, từ một toạ độ — đúng thứ một cú bấm cho. */
async function caretTuDiem(idxO, dx, dy, nhan) {
  return await browser.execute(
    (idx, ddx, ddy, n, nguon) => {
      try {
        // eslint-disable-next-line no-eval
        eval(nguon)
        const cell = document.querySelectorAll('[data-col="tgt"]')[idx]
        const box = cell.getBoundingClientRect()
        const x = box.left + ddx
        const y = box.top + ddy
        // 🔴 `caretRangeFromPoint`, KHÔNG `caretPositionFromPoint` — cái sau NÉM `TypeError`
        // trên WKWebView này (đo ở 2.8, đã vá trong sản phẩm bằng `caretPointAt`).
        if (typeof document.caretRangeFromPoint !== 'function') {
          return { buoc: n, loi: 'caretRangeFromPoint vang mat' }
        }
        const r = document.caretRangeFromPoint(x, y)
        if (r === null) return { buoc: n, loi: 'caretRangeFromPoint tra null' }
        const sel = window.getSelection()
        sel.removeAllRanges()
        sel.addRange(r)

        const sc = r.startContainer
        const truoc = document.createRange()
        truoc.setStart(cell, 0)
        truoc.setEnd(sc, r.startOffset)
        return {
          buoc: n,
          diem: { x: Math.round(x), y: Math.round(y) },
          // ── Engine biểu diễn vị trí này thế nào ──────────────────────────────
          startContainerType: sc.nodeType, // 3 = TEXT, 1 = ELEMENT
          startContainerTen: sc.nodeName,
          startContainerLen: sc.nodeType === 3 ? sc.textContent.length : null,
          startOffset: r.startOffset,
          neoTrongO: cell.contains(sc),
          // ── Hai phép kiểm, đặt cạnh nhau để đọc ra chỗ chúng LỆCH ────────────
          phepSai_startOffsetBang0: r.startOffset === 0,
          // eslint-disable-next-line no-undef
          ungVien_caretODauO: caretODauO(cell, sel),
          soKyTuPhiaTruoc: truoc.toString().length,
          chuPhiaTruoc: JSON.stringify(truoc.toString()),
          // ── Bối cảnh ô ───────────────────────────────────────────────────────
          soPhanTuCon: cell.childNodes.length,
          tenPhanTuCon: [...cell.childNodes].map((c) => c.nodeName),
          textContent: JSON.stringify(cell.textContent),
        }
      } catch (err) {
        return { buoc: n, loiChup: String(err) }
      }
    },
    idxO,
    dx,
    dy,
    nhan,
    UNG_VIEN,
  )
}

function in_(nhan, v) {
  console.log(`\n[2.9·v2 · ${nhan}] ` + JSON.stringify(v, null, 2))
}

describe('Bàn đo 2.9 vòng 2 — engine biểu diễn "đầu ô" thế nào', () => {
  it('Ⓐ+Ⓑ+Ⓒ bốn hình dạng ô, caret đặt bằng caretRangeFromPoint', async () => {
    await openWorkspaceWithWork('Bàn đo 2.9 v2 — caret đầu ô', '一二三。四五六。七八九。')
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="tgt"]').waitForExist({ timeout: 30_000 })

    in_(
      'danh tính phiên',
      await browser.execute(() => ({
        href: window.location.href,
        coApp: !!document.querySelector('#app'),
        userAgent: navigator.userAgent,
        soHang: document.querySelectorAll('[data-col="tgt"]').length,
        caretRangeFromPoint: typeof document.caretRangeFromPoint,
        caretPositionFromPoint: typeof document.caretPositionFromPoint,
      })),
    )

    // ── Ⓐ① Ô RỖNG (chưa dịch) — ca thường nhất, và là ca "sập hố" cũ ──────────────
    in_('Ⓐ① ô RỖNG · bấm giữa ô', await caretTuDiem(0, 40, 12))

    // ── Ⓐ② Ô MỘT DÒNG — caret ở mép TRÁI (đầu ô thật) ────────────────────────────
    await browser.execute(() => {
      const cell = document.querySelectorAll('[data-col="tgt"]')[1]
      cell.focus()
      const sel = window.getSelection()
      sel.removeAllRanges()
      const r = document.createRange()
      r.selectNodeContents(cell)
      sel.addRange(r)
      document.execCommand('insertText', false, 'bốn năm sáu')
    })
    await browser.pause(300)
    in_('Ⓐ② ô MỘT DÒNG · mép TRÁI (đầu ô)', await caretTuDiem(1, 1, 12))
    in_('Ⓐ③ ô MỘT DÒNG · GIỮA chữ (đối chứng ÂM)', await caretTuDiem(1, 30, 12))

    // ── Ⓑ Ô NHIỀU DÒNG (Story 2.5d) — mép TRÁI dòng 1 và mép TRÁI dòng 2 ─────────
    const hinhDang = await browser.execute(() => {
      try {
        const cell = document.querySelectorAll('[data-col="tgt"]')[2]
        cell.focus()
        const sel = window.getSelection()
        sel.removeAllRanges()
        const r = document.createRange()
        r.selectNodeContents(cell)
        sel.addRange(r)
        document.execCommand('insertText', false, 'AAA')
        document.execCommand('insertLineBreak')
        document.execCommand('insertText', false, 'BBB')
        const box = cell.getBoundingClientRect()
        return {
          soPhanTuCon: cell.childNodes.length,
          tenPhanTuCon: [...cell.childNodes].map((c) => c.nodeName),
          doDaiPhanTuCon: [...cell.childNodes].map((c) => (c.nodeType === 3 ? c.textContent.length : -1)),
          textContent: JSON.stringify(cell.textContent),
          // Chiều cao ô cho biết dòng thứ hai nằm ở đâu — dùng cho toạ độ Ⓑ②.
          caoO: Math.round(box.height),
          whiteSpace: getComputedStyle(cell).whiteSpace,
        }
      } catch (err) {
        return { loi: String(err) }
      }
    })
    await browser.pause(300)
    in_('Ⓑ⓪ hình dạng ô sau insertLineBreak', hinhDang)

    const cao = typeof hinhDang.caoO === 'number' ? hinhDang.caoO : 40
    in_('Ⓑ① ô HAI DÒNG · mép trái DÒNG 1 (đầu ô thật)', await caretTuDiem(2, 1, Math.round(cao * 0.25)))
    // 🔴 Câu hỏi trung tâm của cạm bẫy ④.
    in_('Ⓑ② ô HAI DÒNG · mép trái DÒNG 2 (KHÔNG phải đầu ô)', await caretTuDiem(2, 1, Math.round(cao * 0.75)))
    in_('Ⓑ③ ô HAI DÒNG · giữa chữ dòng 2 (đối chứng ÂM)', await caretTuDiem(2, 22, Math.round(cao * 0.75)))
  })

  it('Ⓒ vùng chọn KHÔNG collapsed ⇒ ứng viên phải trả false', async () => {
    const r = await browser.execute((nguon) => {
      try {
        // eslint-disable-next-line no-eval
        eval(nguon)
        const cell = document.querySelectorAll('[data-col="tgt"]')[1]
        cell.focus()
        const sel = window.getSelection()
        sel.removeAllRanges()
        const range = document.createRange()
        // Bôi đen từ ĐẦU ô — `startOffset === 0` ĐÚNG, nhưng nó KHÔNG collapsed.
        const dau = cell.firstChild
        if (!dau || dau.nodeType !== 3) return { loi: 'o khong co text node' }
        range.setStart(dau, 0)
        range.setEnd(dau, Math.min(4, dau.textContent.length))
        sel.addRange(range)
        const rr = sel.getRangeAt(0)
        return {
          collapsed: rr.collapsed,
          startOffset: rr.startOffset,
          chuDaChon: JSON.stringify(sel.toString()),
          phepSai_startOffsetBang0: rr.startOffset === 0,
          // eslint-disable-next-line no-undef
          ungVien_caretODauO: caretODauO(cell, sel),
        }
      } catch (err) {
        return { loi: String(err) }
      }
    }, UNG_VIEN)
    in_('Ⓒ vùng chọn bắt đầu từ ĐẦU ô nhưng KHÔNG collapsed', r)
  })

  it('Ⓓ execCommand("delete") ở offset 0 — đường mà 2.5b dùng để ghi tiền đề ①', async () => {
    const r = await browser.execute(() => {
      try {
        const cell = document.querySelectorAll('[data-col="tgt"]')[1]
        const so = { beforeinput: [], input: [] }
        const gbi = (e) => so.beforeinput.push(e.inputType)
        const gin = (e) => so.input.push(e.inputType ?? null)
        cell.addEventListener('beforeinput', gbi, true)
        cell.addEventListener('input', gin, true)

        cell.focus()
        const sel = window.getSelection()
        sel.removeAllRanges()
        const range = document.createRange()
        const dau = cell.firstChild
        if (dau && dau.nodeType === 3) range.setStart(dau, 0)
        else range.setStart(cell, 0)
        range.collapse(true)
        sel.addRange(range)

        const truoc = cell.textContent
        const ketQua = document.execCommand('delete')
        const sau = cell.textContent

        // ── Đối chứng DƯƠNG: cùng lệnh, caret ở GIỮA ô. Nếu ca này CŨNG cho 0 sự
        //    kiện thì thước hỏng lần nữa và số ở trên vô nghĩa.
        const so2 = { beforeinput: [], input: [] }
        cell.removeEventListener('beforeinput', gbi, true)
        cell.removeEventListener('input', gin, true)
        const gbi2 = (e) => so2.beforeinput.push(e.inputType)
        const gin2 = (e) => so2.input.push(e.inputType ?? null)
        cell.addEventListener('beforeinput', gbi2, true)
        cell.addEventListener('input', gin2, true)
        const sel2 = window.getSelection()
        sel2.removeAllRanges()
        const r2 = document.createRange()
        const d2 = cell.firstChild
        if (d2 && d2.nodeType === 3) r2.setStart(d2, Math.min(3, d2.textContent.length))
        else r2.setStart(cell, 0)
        r2.collapse(true)
        sel2.addRange(r2)
        const truoc2 = cell.textContent
        const ketQua2 = document.execCommand('delete')
        const sau2 = cell.textContent
        cell.removeEventListener('beforeinput', gbi2, true)
        cell.removeEventListener('input', gin2, true)

        return {
          offset0: { truoc: JSON.stringify(truoc), sau: JSON.stringify(sau), execCommandTraVe: ketQua, ...so },
          doiChungDuong_offset3: {
            truoc: JSON.stringify(truoc2),
            sau: JSON.stringify(sau2),
            execCommandTraVe: ketQua2,
            ...so2,
          },
        }
      } catch (err) {
        return { loi: String(err) }
      }
    })
    in_('Ⓓ execCommand("delete") — offset 0 vs đối chứng dương offset 3', r)
  })
})

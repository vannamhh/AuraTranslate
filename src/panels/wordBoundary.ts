/**
 * 🔴 **RANH GIỚI TỪ CHO VÙNG CHỌN** — Story 1.18b, AC3 · AC4 · Quyết định #1(a).
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 ĐÂY LÀ *"CHỌN GÌ ĐỂ TRA"*, KHÔNG PHẢI *"TRA THẾ NÀO"* — ĐỌC TRƯỚC KHI SỬA
 * ═════════════════════════════════════════════════════════════════════════════════
 * Người rà soát tiếp theo sẽ hỏi đúng một câu: *"đây có phải một `Matcher` thứ hai đặt ở
 * webview không?"* Câu trả lời là **không**, và ranh giới đã được phân xử **bằng chữ**
 * từ trước story này — `reviews/review-ad-44-2026-08-05.md:50`:
 *
 * > *"hàng **cụm từ nhiều chữ** là câu hỏi của **Auto-Lookup (chọn gì để tra)**, không phải
 * > của **đường tra cứu (tra thế nào)**"*
 *
 * ⇒ Tệp này trả lời **"chọn gì"**: *trình duyệt sẽ phủ tới đâu khi người dùng double-click
 * trên màn hình*. Nó **không** khớp dữ liệu, không chấm điểm, không quyết định một mục từ
 * điển nào trúng. Vế **"tra thế nào"** vẫn là độc quyền của **AD-17**
 * *(`ARCHITECTURE-SPINE.md:230-236` — **đúng một** cài đặt khớp ngôn ngữ, `jieba-rs`, ở
 * **Rust**, Story 1.12)*, và story này **không chạm một dòng nào** của `core/matching/**`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO ĐẶT NÓ Ở WEBVIEW KHÔNG VI PHẠM AD-1
 * ═════════════════════════════════════════════════════════════════════════════════
 * `ARCHITECTURE-SPINE.md:75-79` nguyên văn:
 *
 * > *"frontend chỉ render và giữ state UI (**focus, cuộn, vùng chọn, bố cục panel**). Không
 * > cài đặt lại bất kỳ quy tắc nghiệp vụ nào ở TypeScript."*
 *
 * **"Vùng chọn" nằm trong danh sách được giữ, viết thẳng.** Củng cố bằng `EXPERIENCE.md:23`:
 * ba thứ **phải** ở Rust là *"tách **câu**, khớp ngôn ngữ, phân giải scope"* — tách **từ để
 * quyết định ranh giới một vùng chọn** không nằm trong ba thứ đó.
 *
 * ⚠️ Và nhớ vì sao `mockups/tm-fuzzy-match.html:267-269` **từ chối** tách từ cho TM: *"tách
 * từ tiếng Trung… sai ở một tỷ lệ nhất định, và mỗi lần sai sẽ làm điểm khớp lệch theo cách
 * không giải thích được"*. Lý lẽ đó đúng cho **khớp** và **không áp** cho **chọn vùng**: một
 * lượt tách sai ở đây chỉ khiến người dùng kéo chọn lại — không làm lệch một điểm số nào.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * ⚠️ VÌ SAO TỆP NÀY SỐNG Ở `src/panels/`, KHÔNG `src/commands/`
 * ═════════════════════════════════════════════════════════════════════════════════
 * Cùng cửa mà `selectionContract.ts` và `sourcePanelState.ts` đã đi qua: `src/commands/**`
 * phải nạp được bằng **Node thuần** để Kiểm C/D/E của `npm run check:commands` chạy trên
 * chính bộ command của sản phẩm. Tệp này không import DOM, nhưng nó phục vụ **một bề mặt
 * render** và người tiêu thụ duy nhất của nó là `SourceHanViet.vue` — để nó ở `panels/` giữ
 * đúng hướng phụ thuộc đó.
 */

/**
 * 🔴 `Intl.Segmenter` — **0 phụ thuộc npm mới, 0 dòng Rust, 0 điểm ra mạng** (AC3 ·
 * NFR13/NFR15/AD-15). Đây là **cùng một bộ tách từ ICU** mà chính trình duyệt dùng để quyết
 * định double-click phủ tới đâu trên một khối văn bản thuần — tức trên **tab nguyên văn**.
 * Dùng lại nó là cách duy nhất khiến AC2 *(hai tab chọn CÙNG một cụm)* đúng **theo cấu
 * trúc**, không đúng nhờ trùng hợp.
 *
 * ⚠️ Một **instance duy nhất, dựng một lần**: `new Intl.Segmenter()` nạp dữ liệu ICU và đắt
 * hơn hẳn một lượt `segment()`. Một Chương chạy đúng một lượt tách (`buildSegments` nằm
 * trong `computed` theo `sourceText`), nhưng instance vẫn phải ra ngoài lượt đó — NFR1.
 *
 * `undefined` = chưa hỏi engine lần nào · `null` = engine **thiếu** API (đường lui, AC3).
 */
let segmenter: Intl.Segmenter | null | undefined

/** Một dòng `console.error` nêu đích danh, rồi rơi về đường lui — dùng chung cho cả hai ca. */
function fallBack(why: string): null {
  console.error(
    `[wordBoundary] ${why} — tab Hán Việt rơi về MỘT KÝ TỰ MỘT TỪ (hành vi trước Story ` +
      '1.18b): double-click chỉ chọn được một ký tự, không cả cụm. Không phụ thuộc nào thay ' +
      'thế được (NFR15); đây là đường lui, không phải lỗi.',
  )
  segmenter = null
  return null
}

function hanWordSegmenter(): Intl.Segmenter | null {
  if (segmenter !== undefined) return segmenter

  // 🔴 ĐƯỜNG LUI ĐO ĐƯỢC, KHÔNG IM LẶNG VÀ KHÔNG NÉM (AC3). Một engine thiếu
  // `Intl.Segmenter` rơi về **một ký tự một từ** — đúng hành vi trước story này, tức bề mặt
  // vẫn dùng được, chỉ mất double-click theo cụm. Nó phải **lần ra được**: một bề mặt Hán
  // Việt lặng lẽ thôi gom từ trên MỘT nền tảng là đúng lớp lỗi mà không cổng nào bắt.
  if (typeof Intl.Segmenter !== 'function') {
    return fallBack('engine này KHÔNG có `Intl.Segmenter`')
  }

  // 🔴 **VÀ CA THỨ HAI: API CÓ MẶT NHƯNG CONSTRUCTOR NÉM** (bắt ở code review 2026-08-08).
  // Chốt `typeof` ở trên chỉ phủ ca *vắng mặt*. Một bản polyfill hỏng hoặc một engine mang
  // ICU bị cắt có thể ném `RangeError` ngay tại đây — và vì lời gọi này nằm trong
  // `wordStartOffsets` → `buildSegments` → `computed(segments)`, một lượt ném **sập cả bề
  // mặt Hán Việt, cả hai kiểu xem**. Đúng thứ mà cam kết "không ném" của AC3 cấm.
  // ⚠️ `'zh'` và `'word'` đều hợp lệ theo ECMA-402 và thiếu dữ liệu locale thì engine rơi về
  // mặc định chứ không ném ⇒ xác suất thấp. Giá của hàng rào cũng gần bằng không.
  try {
    segmenter = new Intl.Segmenter('zh', { granularity: 'word' })
  } catch (cause) {
    return fallBack(`\`new Intl.Segmenter('zh')\` NÉM ở engine này (${String(cause)})`)
  }
  return segmenter
}

/**
 * Các vị trí trong `text` nơi một **TỪ BẮT ĐẦU** — đơn vị **mã UTF-16**, cùng đơn vị mà
 * `Range.startOffset` của DOM đếm.
 *
 * 🔴 Tách trên **TRỌN văn bản đã chuẩn hoá**, không trên từng cụm Hán rời: đó là đúng chuỗi
 * mà tab nguyên văn đưa cho ICU của trình duyệt, nên ranh giới trả về **bằng đúng** ranh
 * giới double-click của tab đó (AC2). Cắt nhỏ đầu vào trước khi tách là tự tạo ra một
 * ngữ cảnh khác và một kết quả khác.
 *
 * ⚠️ Chỗ gọi phải truyền **văn bản đã chuẩn hoá xuống dòng** (`\r\n?` → `\n`) — nếu không,
 * mọi chỉ số trả về lệch đúng số ký tự `\r` đã bị bỏ.
 */
export function wordStartOffsets(text: string): ReadonlySet<number> {
  const starts = new Set<number>()
  const seg = hanWordSegmenter()

  if (seg === null) {
    // Đường lui: mọi ký tự là một từ. Duyệt theo **điểm mã**, không theo đơn vị mã — một ký
    // tự Hán ở mặt phẳng bổ sung (`U+20000`+, hai trong bảy dải của `isHanChar`) chiếm hai
    // đơn vị mã, và cắt vào giữa cặp thay thế là một ký tự hỏng.
    let at = 0
    for (const ch of text) {
      starts.add(at)
      at += ch.length
    }
    return starts
  }

  for (const part of seg.segment(text)) starts.add(part.index)
  return starts
}

/**
 * 🔴 **KÝ TỰ NỐI TỪ `U+2060` (WORD JOINER) — RỘNG BẰNG 0, VÀ ĐÓ LÀ CẢ VẤN ĐỀ LẪN LỜI GIẢI.**
 *
 * Kiểu xem **chuyển đổi** hiện **âm Hán Việt** (chữ Latin). Hai âm của cùng một từ phải:
 *   ① **dính nhau với ICU** — nếu không, double-click chỉ chọn được một âm *(đo 2026-08-07:
 *      hai âm cách nhau bằng một dấu cách thật ⇒ `"thai"`, một âm)*;
 *   ② **rời nhau với mắt** — `"thailoan"` không đọc được *(Ice báo nguyên văn ở lượt 1.16:
 *      `phảnđốitrungcộngkhoác…`)*.
 *
 * Hai vế đó mâu thuẫn nếu khoảng cách đi bằng **ký tự**. `U+00A0` và `U+2009` **vẫn cắt**
 * (đo 2026-08-07) — chúng là khoảng trắng với ICU. ⇒ **tách hai vai**: `U+2060` giữ *tính
 * liền từ* cho engine, còn *khoảng cách nhìn thấy* do **CSS** vẽ (`.hv-syl + .hv-syl`).
 *
 * 🔴 **Và nó KHÔNG được đi theo lượt copy của người dùng** (AC5): một ký tự vô hình dán ra
 * Word là thứ không ai lần ra được. `SourceHanViet.vue::onCopy` đổi nó về **một dấu cách**
 * — không xoá trắng, vì xoá trắng cho ra `"thailoan"` đúng thứ ① vừa tránh.
 */
export const WORD_JOINER = '\u2060'

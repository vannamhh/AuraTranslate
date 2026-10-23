---
title: 'technical research: Gõ chậm trong lưới subgrid lớn trên WebKit/Blink'
type: 'technical'
topic: 'Gõ chậm trong lưới subgrid lớn trên WebKit/Blink'
decision: 'Chọn hướng sửa đường gõ ở Chương 9.850 câu (bug 2.15) làm đầu vào một AD'
source: 'native run (web, 4 digests) + phép thử ba engine trên trang tổng hợp'
status: complete
preset: 'standard'
validation: 'normal'
created: '2026-10-23'
updated: '2026-10-23'
claims_verified: 2
claims_unverified: 7
---

# technical research: Gõ chậm trong lưới subgrid lớn trên WebKit/Blink

**Decision this research serves:** chọn hướng sửa đường gõ ở Chương 9.850 câu (bug 2.15), làm đầu vào một AD do Winston soạn.

## Executive summary

**Chọn lưới chia khối: mỗi khối khoảng 200 hàng, giữ subgrid xếp theo cột bên trong khối.** Trong ba bố cục đã thử, đây là bố cục duy nhất giữ được ≤ 100 ms mỗi phím trên cả ba engine. Trên trang tổng hợp 9.850 hàng, mỗi phím tốn 10 ms trên WKWebView của hệ điều hành, 15 ms trên WebKit mới, 3 ms trên Chromium [17].

- **Chi phí nằm ở WebKit, không nằm trong app.** Trang tổng hợp không có Vue và không có JS của app, chỉ có lưới subgrid nguyên khối 9.850 hàng. Trên đúng engine app đang dùng (WebKit 20621.3.11.11.3), mỗi phím vẫn tốn 468 ms, khớp 0,5–0,65 s đo trong app. Toàn bộ chi phí nằm ngay trong lệnh gõ: thời gian đồng bộ trùng thời gian tới frame kế. Tức là WebKit tính lại bố cục cả lưới trong chính lệnh gõ. Việc này chạy một lần cho mỗi phím và không gộp được, nên phím xếp hàng [17].
- **Bản sửa `305121@main` không cứu đường gõ.** Bug 304869 và bản sửa của nó có thật [1][2]. Nhưng WebKit mới (Playwright WebKit 627.1.2+, sau bản sửa) còn chậm hơn: 998 ms mỗi phím với subgrid nguyên khối [17]. Chromium cũng mất 172 ms mỗi phím [17].
- **Bố cục theo hàng không ổn định giữa các engine.** WebKit mới 14 ms, Chromium 11 ms, nhưng WKWebView của hệ điều hành mất 789 ms mỗi phím [17]. Phương án này trượt điều kiện bắt buộc "hai engine".

**Rủi ro lớn nhất:** vùng chọn. Khi chia khối, cột bản dịch không còn là một phần tử chứa duy nhất. Một vùng chọn kéo qua ranh giới khối sẽ đi qua các cột nguyên văn của khối kế, vì vùng chọn theo thứ tự DOM [13]. Vùng chọn nhiều câu (Story 4.9) và hai bề mặt vùng chọn của lưới phải được thiết kế lại cho nhiều khối. Câu hỏi này chưa đo.

**Giới hạn của phép đo:** trang tổng hợp, 20 phím mỗi biến thể, mỗi engine một phiên, tải máy 5,6–6,2 [17]. Số tuyệt đối có thể lệch so với app thật. Nhưng khoảng cách giữa các bố cục (10 ms so với 468 ms trên cùng engine) lớn hơn nhiều so với độ lệch đó.

## D1. Cơ chế engine

- **Điểm nóng nằm ở bước tính kích thước track nội tại của subgrid.** Bản sửa đổi độ phức tạp của bước này, không bỏ nó đi; tác giả ghi rằng các engine khác vẫn nhanh hơn và đây chỉ là "điểm bắt đầu" [2]. Bug 232176 (lưới lồng nhau chậm, từ 2021) bị đóng là trùng với 304869 [1].
- **Không dùng `contain` để giới hạn bố cục mà vẫn giữ được subgrid.** Theo css-grid-2, một grid container phải lập formatting context độc lập (ví dụ khi có layout containment) thì `subgrid` tính thành `none` [4]. Việc size containment trên subgrid ảnh hưởng tới track của lưới cha ra sao vẫn đang chờ sửa đặc tả [4]. Độ tin trung bình, vì đặc tả được đọc qua bản tóm tắt.
- **Viền focus không phải nguyên nhân ở đây.** Bug 232673 (gõ chậm theo cỡ tài liệu khi có viền focus) [14] không áp vào app, vì ô bản dịch đã đặt `outline: none` (quan sát mã, không phải bằng chứng web).
- **Blink (WebView2) chưa có dữ liệu.** GridNG của Blink có bộ đệm bố cục [15], nhưng không tìm được số nào cho subgrid ở quy mô lớn. Độ tin thấp.

## D2. Bố cục thay thế giữ hàng thẳng

- **Chưa có số đo công khai** so bố cục theo hàng, lưới chia khối hay `table` với subgrid khi đổi nội dung một ô (không có nguồn).
- **`content-visibility: auto`** bỏ qua bố cục phần ngoài màn hình. Muốn thanh cuộn không nhảy thì cần `contain-intrinsic-size`. Phần đang có focus hoặc vùng chọn không bị bỏ qua, và phần ẩn vẫn tìm được bằng Find [6]. Các điểm này mô tả Blink; WebKit chưa kiểm. Ở 2.14, đặt `content-visibility: auto` lên lưới đã làm chậm thêm khoảng 8 lần (số của dự án, không phải bằng chứng web).
- **WebKit coi `contain: layout` là ranh giới bố cục lại,** nhưng nó chỉ có lợi khi thay đổi nằm gọn trong cây con [7]. Độ tin thấp: chỉ có đoạn trích kết quả tìm kiếm.
- **Bố cục theo hàng đảo thứ tự DOM.** Vùng chọn đi theo thứ tự DOM, nên các thư viện lưới giữ thứ tự DOM trùng thứ tự nhìn để chọn chữ tự nhiên [13]. Bố cục theo hàng sẽ xen chữ nguyên văn và bản dịch trong cùng một vùng chọn.

## D3. Editor tài liệu lớn

- **CodeMirror 6 chỉ dựng phần đang hiện cộng một lề.** Vùng chọn nằm trong state của editor, và undo là một extension [8]. Tức nó tự làm lại những gì app này đang giao cho trình duyệt: vùng chọn, `⌘Z`, tìm.
- **ProseMirror dựng toàn bộ DOM, không ảo hoá, và có lịch sử undo riêng** [9]. Tác giả gọi tài liệu rất lớn là "vấn đề đã biết" (2020) [10].
- **Hai thư viện ảo hoá giấy phép MIT có hỗ trợ Vue 3 và hàng cao thấp khác nhau:**
  - TanStack Virtual, `@tanstack/vue-virtual` 3.13.48 [11].
  - virtua 0.53.6, vẫn ở 0.x [12].
  - Giấy phép của cả hai được đọc từ trang repo, chưa đọc tệp LICENSE đã tải về như NFR15 đòi.
- **Chưa tìm được bài viết nào** về cách CAT tool trên web (MateCat, Phrase, Smartcat, Crowdin, Weblate) dựng file lớn, cũng như bài hậu kiểm nào về việc ảo hoá hàng `contenteditable` (không có nguồn).

## D4. Phép thử trên ba engine (đo trong lượt này)

Trang `grid.html` [17] dựng 9.850 hàng chữ dài ngẫu nhiên, 4 cột (vạch lề 3 px, số, nguyên văn, bản dịch `contenteditable`), rộng 1.200 px. Đặt con trỏ ở cuối ô bản dịch của hàng giữa, rồi gọi 20 lần `execCommand('insertText')`, mỗi lần cách nhau 30 ms. Ghi thời gian đồng bộ của lệnh và thời gian tới `requestAnimationFrame` kế tiếp. Ba biến thể:
- `subgrid`: một lưới, cột subgrid trải mọi hàng, như app hiện tại.
- `chunk`: các lưới 200 hàng nối tiếp, mỗi lưới có cột subgrid riêng.
- `rows`: mỗi hàng là một lưới 4 cột.

| Engine | Bố cục | Bố cục đầu (ms) | Một phím, trung vị (ms) | Một phím, lớn nhất (ms) |
|---|---|---|---|---|
| WKWebView hệ điều hành, WebKit 20621.3.11.11.3 | subgrid | 1.614 | 468 | 581 |
| | chunk | 1.366 | **10** | 26 |
| | rows | 2.940 | 789 | 877 |
| Playwright WebKit 627.1.2+ (headless) | subgrid | 3.841 | 998 | 1.432 |
| | chunk | 2.455 | **15** | 31 |
| | rows | 3.408 | 14 | 61 |
| Playwright Chromium 156.0.8078.4 (headless) | subgrid | 1.053 | 172 | 205 |
| | chunk | 986 | **3** | 19 |
| | rows | 1.095 | 11 | 43 |

Ở mọi ô trong bảng, thời gian đồng bộ trùng thời gian tới frame (chênh ≤ 2 ms ở trung vị). Chi phí vì thế nằm ngay trong lệnh gõ. Điều kiện đo: macOS 15.8.2, máy đang làm việc bình thường, tải 5,6–6,2. Mỗi biến thể một phiên 20 phím. Số thô ở `ket-qua-headless.jsonl` và `ket-qua-wkwebview.jsonl` [17].

## Cross-dimension insights

- **Ảo hoá chỉ "rẻ" khi editor tự giữ vùng chọn và undo** (CodeMirror [8], ProseMirror [9]). App này cố ý dùng `⌘Z` gốc và vùng chọn gốc tới trọn một Chương. Vì vậy ảo hoá không phải là thêm một thư viện, mà là viết lại vùng chọn và undo. Điều này đụng hai điều kiện bắt buộc số 3 và 5.
- **Chậm nằm ở độ dài của subgrid, không ở việc dùng lưới.** Cùng engine, cùng DOM, cắt subgrid từ 9.850 xuống 200 hàng thì chi phí một phím giảm từ 468 xuống 10 ms [17]. Điều này khớp với bug 302529, vốn chỉ hết khi tắt subgrid [3], và với điểm nóng ở bước tính track của subgrid trải nhiều hàng [1]. Bản sửa của WebKit nhắm đúng chỗ đó nhưng chưa đủ cho đường gõ, và macOS 15 không nhận được bản sửa ấy cho app bên thứ ba [16].

## Contrary evidence

Không có lượt phản biện riêng (`red_team: off`). Điểm phản bác mạnh nhất: trang tổng hợp không có CSS, vạch lề hay dải Glossary của app, nên số đo trong app có thể cao hơn [17]. Nhưng ở cùng engine, trang tổng hợp với subgrid nguyên khối cho 468 ms, khớp 0,5–0,65 s đo trong app. Như vậy phần chi phí app tự thêm vào là nhỏ.

## Recommendations

1. **Winston soạn một AD mới: lưới chia khối khoảng 200 hàng, giữ subgrid trong mỗi khối.** Thay quyết định #1 của lưới (2.5b), vốn là một lưới subgrid chủ cột cho cả Chương. AD phải trả lời hai câu: vùng chọn và hai bề mặt vùng chọn chạy thế nào khi có nhiều khối; và mọi thứ đang đánh chỉ số hàng trên toàn lưới (e2e, `gridWait.mjs`, điều hướng) đổi ra sao. Gắn vào: bug 2.15 (thay cho cổng `blocked`), bug 2.14 (dời con trỏ cũng nghi do bố cục lại cả lưới), và spine. Độ tin: cao về hiệu năng (một phép đo, ba engine, khoảng cách lớn [17]); vùng chọn chưa đo.
2. **Trước khi duyệt AD, làm một prototype nhỏ trong app,** đo đúng hai thứ: một phím gõ, và một lần dời con trỏ ở 9.850 câu trên WKWebView của hệ điều hành. Cùng lúc thử vùng chọn kéo qua ranh giới khối, kèm một ca e2e. Độ tin trung bình, vì đây là bước kiểm, chưa có số.
3. **Loại bố cục theo hàng:** 789 ms mỗi phím trên engine của hệ điều hành [17]. Độ tin cao.
4. **Loại `contain` và `content-visibility` khi vẫn giữ subgrid,** vì theo đặc tả layout containment tắt subgrid [4]. Độ tin trung bình.
5. **Không cần ảo hoá hàng,** vì lưới chia khối đã đạt mà vẫn giữ đủ DOM. Như vậy tránh được việc viết lại vùng chọn và `⌘Z` [8][9]. Độ tin cao về hiệu năng [17].
6. **Không chờ bản sửa của WebKit:** WebKit mới vẫn chậm với subgrid nguyên khối [17]. Độ tin cao.

## Open questions

- 🔵 2026-10-23 — đã trả lời bằng D4: bản sửa `305121@main` không làm nhanh việc sửa nội dung một ô, vì WebKit 627 vẫn mất 998 ms mỗi phím với subgrid nguyên khối.
- WebKit có ép bố cục đồng bộ ở mỗi lệnh gõ của `contenteditable` không? Cần đọc `editing/TypingCommand.cpp` hoặc profile bằng Web Inspector.
- Bản macOS hay Safari nào đầu tiên mang `305121@main` cho WKWebView của app bên thứ ba? Ghi chú phát hành không dẫn số bug (không có nguồn).
- Khi chia lưới thành khối, vùng chọn kéo qua ranh giới hai khối có giữ đúng cột không? Chưa có nguồn; cần một ca e2e.
- Blink và WebView2 có chậm theo cùng kiểu không? Chưa có số.

## Source appendix

| # | Claim/finding | Publisher | Pub date | Accessed | Confidence |
|---|---|---|---|---|---|
| [1] | Bug 304869: subgrid trải nhiều hàng chậm ở `accumulateIntrinsicSizesForTrack`, RESOLVED FIXED bằng `305121@main`; 232176 trùng | [WebKit Bugzilla](https://bugs.webkit.org/show_bug.cgi?id=304869) | 2026-01-02 | 2026-10-23 | high |
| [2] | Commit `425f9293f8d4`: đệ quy theo hàng thay bằng một lượt lặp; benchmark 12 subgrid × 20.000 hàng, ~0,2 → ~50 lượt/giây; chỉ đo đổi bề rộng | [WebKit (GitHub)](https://github.com/WebKit/WebKit/commit/425f9293f8d4f3cd2423eabd9dd099fe0dce82b2) | 2026-01-05 | 2026-10-23 | high |
| [3] | Bug 302529: subgrid lồng nhau làm Safari treo, Chrome/Firefox ổn; cách lách là tắt subgrid; vẫn NEW | [WebKit Bugzilla](https://bugs.webkit.org/show_bug.cgi?id=302529) | 2025-11-14 | 2026-10-23 | high |
| [4] | Layout containment làm `subgrid` thành `none`; size containment trên subgrid chưa sửa xong đặc tả | [W3C CSSWG](https://github.com/w3c/csswg-drafts/issues/7091) | 2022-02-25 | 2026-10-23 | medium |
| [5] | WebKit 20621.3.11.11.3 = Safari 18.6, macOS 15.6 | [Mr. Macintosh](https://mrmacintosh.com/macos-sequoia-15-6-update-everything-you-need-to-know/) | 2025-07 | 2026-10-23 | medium |
| [6] | `content-visibility: auto`: cần `contain-intrinsic-size`; không bỏ qua phần có focus/vùng chọn; Find vẫn thấy | [web.dev](https://web.dev/articles/content-visibility) | 2025-09-23 | 2026-10-23 | high |
| [7] | `contain: layout` là ranh giới bố cục lại trong WebKit | [WebKit Bugzilla](https://bugs.webkit.org/show_bug.cgi?id=227948) | 2013–2025 | 2026-10-23 | low |
| [8] | CodeMirror 6 chỉ dựng phần đang hiện; vùng chọn trong state; undo là extension | [codemirror.net](https://codemirror.net/docs/guide/) | undated | 2026-10-23 | high |
| [9] | ProseMirror dựng toàn bộ DOM; lịch sử undo là plugin | [prosemirror.net](https://prosemirror.net/docs/guide/) | undated | 2026-10-23 | medium |
| [10] | "Very big documents making things slow is a known issue" | [ProseMirror forum](https://discuss.prosemirror.net/t/performance-issues-with-prosemirror-and-chrome/2498) | 2020-01-31 | 2026-10-23 | medium |
| [11] | TanStack Virtual: MIT, adapter Vue, đo cỡ động; 3.13.48 | [GitHub TanStack](https://github.com/TanStack/virtual) | n/a | 2026-10-23 | medium |
| [12] | virtua: MIT, `virtua/vue`, cỡ động; 0.53.6 | [GitHub inokawa](https://github.com/inokawa/virtua) | n/a | 2026-10-23 | medium |
| [13] | Vùng chọn theo thứ tự DOM; AG Grid `ensureDomOrder` | [AG Grid docs](https://ag-grid.com/react-grid/selection-overview/) | undated | 2026-10-23 | low |
| [14] | Bug 232673: gõ `contenteditable` chậm theo cỡ tài liệu khi có viền focus | [WebKit Bugzilla](https://bugs.webkit.org/show_bug.cgi?id=232673) | 2021-11-03 | 2026-10-23 | medium |
| [15] | GridNG của Blink có bộ đệm bố cục tốt hơn | [Microsoft Edge blog](https://blogs.windows.com/msedgedev/?p=24831) | 2021-08-10 | 2026-10-23 | low |
| [17] | Phép thử ba engine, trang tổng hợp 9.850 hàng, 20 phím mỗi biến thể | [dự án, đo trong lượt này](../../implementation-artifacts/2-15-ban-do/thu-engine/) | 2026-10-23 | 2026-10-23 | high (một phiên mỗi biến thể) |
| [16] | Safari 26.6.1 vá WebKit cho macOS Sonoma và Sequoia | [9to5Mac](https://9to5mac.com/2026/08/18/safari-26-6-1-patches-multiple-webkit-security-flaws-on-macos-sonoma-and-sequoia/) | 2026-08-18 | 2026-10-23 | medium |

## Staleness map

Tính bằng `recon_kit.py staleness`, cửa sổ: trạng thái bug và phiên bản 1 tháng, phiên bản thư viện 6 tháng, đặc tả và mẫu thiết kế 24 tháng.

| Claim | Class | Pub | Re-check | Stale |
|---|---|---|---|---|
| [1] 304869 đã sửa bằng `305121@main` | bug-status | 2026-01-05 | 2026-02-05 | yes |
| [3] 302529 vẫn NEW | bug-status | 2026-01-13 | 2026-02-13 | yes |
| [5] 20621.3.11.11.3 = Safari 18.6 | version | 2025-07 | 2025-08-01 | yes |
| [4] containment làm subgrid thành `none` | spec | 2022-02-25 | 2024-02-25 | yes |
| [11] TanStack Virtual 3.13.48, MIT | library-version | 2026-10-23 | 2027-04-23 | no |
| [12] virtua 0.53.6, MIT | library-version | 2026-10-23 | 2027-04-23 | no |
| [8] CodeMirror 6 chỉ dựng phần đang hiện | pattern | 2026-10-23 | 2028-10-23 | no |

Mốc kiểm lại sớm nhất là 2024-02-25 ([4]). Ba claim về trạng thái bug và phiên bản bị tính là cũ theo ngày phát hành, dù trang bug được đọc ngày 2026-10-23. Đặc tả [4] phải đọc lại tận văn bản css-grid-2 trước khi AD dựa vào nó.

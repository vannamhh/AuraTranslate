---
title: 'technical research: Gõ chậm trong lưới subgrid lớn trên WebKit/Blink'
type: 'technical'
topic: 'Gõ chậm trong lưới subgrid lớn trên WebKit/Blink'
decision: 'Chọn hướng sửa đường gõ ở Chương 9.850 câu (bug 2.15) làm đầu vào một AD'
source: 'native run (web, 4 digests) + phép thử ba engine trên trang tổng hợp; Deepen D5 2026-10-24 (web, 4 digests + 1 lượt kiểm chéo, đọc phiên bản engine trên máy đo)'
status: complete
preset: 'standard'
validation: 'normal'
created: '2026-10-23'
updated: '2026-10-24'
claims_verified: 7
claims_unverified: 11
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

**Deepen 2026-10-24 (D5), không đổi kết luận trên.** Ở lưới đã chia khối, mỗi phím gõ trong WebKit vẫn chạy ít nhất hai lượt tính bố cục đồng bộ bên trong lệnh gõ chữ của engine, và lượt đầu đắt hơn nếu handler trước đó đã làm DOM bẩn [22][23]. App chạy engine 20621 của hệ điều hành, nên không có Event Timing và chưa có bản sửa hiệu năng mới nhất của Highlight API [18][33][38]. Nhịp 40 phím/giây nhanh gấp khoảng năm lần so với 8 phím/giây trung bình của người gõ nhanh [36]. Việc cần làm cho plan giảm chi phí mỗi phím: khuyến nghị 7–11.

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

## D5. Chi phí mỗi phím trong `contenteditable` và Vue trên WebKit (Deepen 2026-10-24)

Chiều này phục vụ plan giảm chi phí mỗi phím gõ ở lưới chia khối (NFR18: phím dồn hàng ở 40 phím/giây, Chương 122 câu). Mã nguồn WebKit dưới đây đọc từ nhánh `main` ngày 2026-10-24, không phải bản 20621 mà app nạp; hành vi có thể khác ở bản cũ.

**Engine mà app thật sự chạy.**
- Trên máy đo, `WebKit.framework` của hệ điều hành là 20621.3.11.11.3, trong khi Safari đã lên 27.2 với WebKit riêng 20625.2.8.18.2 [38]. [16] đã cho thấy macOS 15 không nhận bản sửa engine qua Safari cho app bên thứ ba; nay có số đo trên máy [38]: cập nhật Safari không đổi engine mà WKWebView của app nạp.
- Hệ quả: tính năng chỉ có từ Safari 26.2 trở đi không có trong app. Event Timing API là một ví dụ: Safari 26.2 mới thêm nó [18].

**WebKit làm gì đồng bộ ở mỗi phím.**
- Mỗi lệnh sửa nội dung (gồm `TypingCommand`, lệnh gõ chữ) đi qua `CompositeEditCommand::apply()`, và hàm này gọi `updateLayoutIgnorePendingStylesheets()` ở phần mở đầu, kèm chú thích "Changes to the document may have been made since the last editing operation that require a layout" [22]. Sau khi sửa DOM, `Editor::appliedEditing` gọi `updateLayout()` thêm một lần trước khi đọc vùng chọn cuối [23]. Tức là một ký tự gõ chạy ít nhất hai lượt tính bố cục đồng bộ. Lượt đầu đắt hơn khi handler trước đó đã làm DOM bẩn, tức đã đổi DOM mà chưa tính lại bố cục. Độ tin trung bình–cao: đọc qua bản tóm tắt của công cụ tải trang, trên nhánh `main`.
- `DOMSelection::setBaseAndExtent`, `collapse`, `removeAllRanges` không tự gọi tính bố cục [24]. `FrameSelection::setSelection` bỏ qua bước cập nhật hiển thị và cuộn tới vùng chọn khi style hoặc bố cục đang chờ [21]. Suy luận từ [21][24]: chi phí bố cục có thể dồn sang lệnh gõ chữ kế tiếp, không nằm ở lời gọi vùng chọn. `Node.normalize()` chưa đọc.
- `selectionchange` của document được xếp thành một task (một lượt xử lý của vòng sự kiện trình duyệt) với cờ gộp: nhiều lần đổi vùng chọn trong cùng một task cho một sự kiện; gõ qua nhiều task cho một sự kiện mỗi task [21]. Một báo cáo năm 2024 nói Chromium và WebKit bắn nhiều lần còn Gecko một lần [25]. Hai nguồn này chưa khớp nhau; có thể báo cáo [25] đếm sự kiện qua nhiều task.
- Hai bug liên quan chưa có bản sửa và chưa đủ bằng chứng để dựa vào: 232673, gõ chậm theo cỡ tài liệu khi có viền focus [14]; 263303, kiểm chính tả làm chậm vùng sửa rất dài, chỉ đọc qua đoạn trích [39].

**Đo độ trễ phím.**
- Event Timing, API trình duyệt báo thời gian xử lý một sự kiện (`startTime` là lúc nhận sự kiện, `processingStart`/`processingEnd` bao quanh listener, `duration` tới lần vẽ kế) [19], chỉ có từ Safari 26.2 [18], nên không dùng được trong engine 20621 của app [38].
- `event.timeStamp` của WebKit lấy từ sự kiện của hệ điều hành từ năm 2013, tức lúc hệ điều hành nhận phím chứ không phải lúc handler chạy [20]. Độ tin trung bình: nguồn gốc nhưng cũ, chưa đối chiếu mã hiện tại.

**Vue 3.5.**
- 3.5 viết lại phần phản ứng; benchmark công bố chỉ đo effect thuần, chưa có số ở mức component [26].
- `shouldUpdateComponent` chỉ chặn cập nhật do cha render lại. Component con có render đọc một biến phản ứng (ref) thì chạy lại khi biến đó kích hoạt, bất kể prop, qua hàng đợi render của chính nó [27]. Scheduler gộp mọi kích hoạt đồng bộ vào một lượt flush, bỏ việc trùng và chạy từ cha xuống con [27]. Độ tin trung bình: hai tệp cùng một kho, đọc qua bản tóm tắt.
- `v-memo` bỏ qua cả việc dựng lại nút ảo (vnode) khi mảng phụ thuộc không đổi; tài liệu gọi nó là tối ưu vi mô, hữu ích nhất cho `v-for` trên 1.000 phần tử [29]. Tài liệu không nói lúc nào mảng được tính hay tốn bao nhiêu. Không tìm được benchmark nào cho `v-memo` hay cho một ref kích hoạt nhiều component.
- Tài liệu chính thức khuyên giữ prop ổn định, ví dụ truyền `:active="item.id === activeId"` thay vì `activeId`, để chỉ phần tử đổi trạng thái mới cập nhật [28].
- Bản ổn định mới nhất là 3.5.44; 3.6 còn ở bản thử nên không dựa vào [30].

**Vẽ gạch chân không bọc span.**
- CSS Custom Highlight API có từ Safari 17.2 [31], nên có trong engine 20621 (nhánh Safari 18.6 [5]). Highlight không đổi DOM. Range sống (do trình duyệt tự cập nhật khi DOM đổi) có chi phí cập nhật, còn `StaticRange` thì không tự cập nhật; vẽ lại là bất đồng bộ [32].
- Chi phí vẽ của WebKit tăng theo số range đã đăng ký: 421 range trong 36 nhóm làm một khung hình đi từ 16,9 lên 42,3 ms, vì mỗi hộp chữ lặp qua mọi highlight. Bản sửa `324157@main` được đưa vào ngày 2026-10-19 [33], nên chưa có trong engine của app [38]. Bug 323923 (vài trăm highlight) vẫn NEW [33]. Chưa có phép so nào giữa Highlight API và bọc span.

**Editor lớn giữ việc mỗi phím nhỏ thế nào.**
- CodeMirror 6 tách bước đọc bố cục khỏi bước ghi DOM (`requestMeasure`), bỏ các yêu cầu trùng khoá, và cấm đọc bố cục trong `PluginValue.update` [34].
- ProseMirror, chỉ để tham khảo: với tập trang trí lớn, chỉ vẽ lại vùng đổi; tác giả gợi ý tô quanh vùng chọn trước, phần còn lại sau một khoảng rảnh [35]. Độ tin trung bình: chỉ đoạn trích diễn đàn.

**Nhịp gõ thật.**
- Nghiên cứu 136 triệu phím (CHI 2018): khoảng cách trung bình giữa hai phím là 238,7 ms; nhóm gõ nhanh trung bình 121,7 ms, tức khoảng 8 phím/giây. Đó là trung bình theo người, không phải khoảng ngắn nhất [36].
- Cài đặt macOS nhanh nhất cho khoảng 30 ms mỗi lần lặp phím, tức khoảng 33 phím/giây; nguồn cộng đồng, Apple không công bố số [37].
- Máy đo đặt `KeyRepeat=5`, `InitialKeyRepeat=30` [38]. Theo đơn vị 15 ms mà [37] suy ra, đó là khoảng 75 ms mỗi lần lặp, tức khoảng 13 phím/giây; đơn vị chưa kiểm.
- ⇒ 40 phím/giây (25 ms mỗi phím) nhanh gấp khoảng năm lần so với 8 phím/giây trung bình của người gõ nhanh [36], và nhanh hơn cả mức lặp phím nhanh nhất trong cài đặt [37].

## Cross-dimension insights

- **Ảo hoá chỉ "rẻ" khi editor tự giữ vùng chọn và undo** (CodeMirror [8], ProseMirror [9]). App này cố ý dùng `⌘Z` gốc và vùng chọn gốc tới trọn một Chương. Vì vậy ảo hoá không phải là thêm một thư viện, mà là viết lại vùng chọn và undo. Điều này đụng hai điều kiện bắt buộc số 3 và 5.
- **Chậm nằm ở độ dài của subgrid, không ở việc dùng lưới.** Cùng engine, cùng DOM, cắt subgrid từ 9.850 xuống 200 hàng thì chi phí một phím giảm từ 468 xuống 10 ms [17]. Điều này khớp với bug 302529, vốn chỉ hết khi tắt subgrid [3], và với điểm nóng ở bước tính track của subgrid trải nhiều hàng [1]. Bản sửa của WebKit nhắm đúng chỗ đó nhưng chưa đủ cho đường gõ, và macOS 15 không nhận được bản sửa ấy cho app bên thứ ba [16].

- **(D5) Chia khối làm nhỏ vùng phải bố cục lại, nhưng không bỏ được lượt bố cục mỗi phím.** Lệnh gõ chữ của WebKit vẫn tính lại bố cục ít nhất hai lần cho mỗi phím [22][23]. Số đo ở [17] cho thấy chi phí không còn nằm ở độ dài subgrid. Giả thuyết, chưa đo: phần còn lại có thể nằm ở lượng DOM mà handler làm bẩn trước lệnh gõ chữ, và ở việc Vue render lại sau đó [27].

## Contrary evidence

Không có lượt phản biện riêng (`red_team: off`). Điểm phản bác mạnh nhất: trang tổng hợp không có CSS, vạch lề hay dải Glossary của app, nên số đo trong app có thể cao hơn [17]. Nhưng ở cùng engine, trang tổng hợp với subgrid nguyên khối cho 468 ms, khớp 0,5–0,65 s đo trong app. Như vậy phần chi phí app tự thêm vào là nhỏ.

## Recommendations

1. **Winston soạn một AD mới: lưới chia khối khoảng 200 hàng, giữ subgrid trong mỗi khối.** Thay quyết định #1 của lưới (2.5b), vốn là một lưới subgrid chủ cột cho cả Chương. AD phải trả lời hai câu: vùng chọn và hai bề mặt vùng chọn chạy thế nào khi có nhiều khối; và mọi thứ đang đánh chỉ số hàng trên toàn lưới (e2e, `gridWait.mjs`, điều hướng) đổi ra sao. Gắn vào: bug 2.15 (thay cho cổng `blocked`), bug 2.14 (dời con trỏ cũng nghi do bố cục lại cả lưới), và spine. Độ tin: cao về hiệu năng (một phép đo, ba engine, khoảng cách lớn [17]); vùng chọn chưa đo.
2. **Trước khi duyệt AD, làm một prototype nhỏ trong app,** đo đúng hai thứ: một phím gõ, và một lần dời con trỏ ở 9.850 câu trên WKWebView của hệ điều hành. Cùng lúc thử vùng chọn kéo qua ranh giới khối, kèm một ca e2e. Độ tin trung bình, vì đây là bước kiểm, chưa có số.
3. **Loại bố cục theo hàng:** 789 ms mỗi phím trên engine của hệ điều hành [17]. Độ tin cao.
4. **Loại `contain` và `content-visibility` khi vẫn giữ subgrid,** vì theo đặc tả layout containment tắt subgrid [4]. Độ tin trung bình.
5. **Không cần ảo hoá hàng,** vì lưới chia khối đã đạt mà vẫn giữ đủ DOM. Như vậy tránh được việc viết lại vùng chọn và `⌘Z` [8][9]. Độ tin cao về hiệu năng [17].
6. **Không chờ bản sửa của WebKit:** WebKit mới vẫn chậm với subgrid nguyên khối [17]. Độ tin cao.

7. **Coi 40 phím/giây là nhịp ép tải, không phải nhịp người; đặt nhịp nghiệm thu cho NFR18 bằng một quyết định của John.** Số nhịp gõ và lặp phím ở D5, mục Nhịp gõ thật [36][37][38]; khoảng ngắn nhất giữa hai phím còn là câu hỏi mở. Gắn vào: PRD (NFR18), Story 2.4. Độ tin: trung bình (một nghiên cứu lớn, số lặp phím từ nguồn cộng đồng).
8. **Đo chi phí từng phím trong engine 20621 bằng `performance.now()` quanh handler và Web Inspector, không dựa vào Event Timing** (API trình duyệt báo thời gian xử lý sự kiện; chỉ có từ Safari 26.2 [18], không có trong engine của app [38]). Gắn vào: plan giảm chi phí mỗi phím gõ. Độ tin: cao về việc thiếu API; cách đo là đề xuất, chưa có nguồn đo.
9. **Giữ các handler chạy trước lệnh gõ chữ (`keydown`, `beforeinput`) không ghi DOM, hoặc ghi ít nhất có thể.** Lệnh gõ chữ của WebKit tính lại bố cục ở phần mở đầu nếu DOM bẩn, rồi tính lần nữa sau khi sửa [22][23], nên mọi lần ghi DOM ở handler trước đó làm lượt bố cục đầu đắt hơn. CodeMirror 6 cũng tách bước đọc bố cục khỏi bước ghi DOM [34]. Mã của app đang làm việc này ở `research-grid-text-input-path` §4. Gắn vào: plan giảm chi phí mỗi phím gõ. Độ tin: trung bình (mã WebKit `main`, chưa đo trên 20621).
10. **Cân nhắc CSS Custom Highlight API thay bọc span cho gạch chân soát lỗi, nhưng đo trước.** Nó không đổi DOM [31][32], nhưng chi phí vẽ trong engine của app tăng theo số range, vì bản sửa `324157@main` chưa tới engine đó [33][38]. Gắn vào: plan giảm chi phí mỗi phím gõ nếu đo thấy có lợi; nếu không, một story riêng về gạch chân. Độ tin: thấp–trung bình.
11. **Đừng để một biến phản ứng dùng chung (ref) bị đọc ở mọi khối trong lúc gõ; truyền giá trị nguyên thủy, ví dụ một cờ đúng/sai, cho từng hàng hoặc từng khối.** Một component đọc ref trong render thì chạy lại khi ref kích hoạt, bất kể prop [27]; tài liệu Vue khuyên giữ prop ổn định để chỉ phần tử đổi mới cập nhật [28]. Ref theo từng hàng là giả thuyết chưa có nguồn. Gắn vào: plan giảm chi phí mỗi phím gõ. Độ tin: trung bình.

## Open questions

- 🔵 2026-10-23 — đã trả lời bằng D4: bản sửa `305121@main` không làm nhanh việc sửa nội dung một ô, vì WebKit 627 vẫn mất 998 ms mỗi phím với subgrid nguyên khối.
- 🔵 2026-10-24 — đã trả lời bằng D5: WebKit có tính bố cục đồng bộ ở mỗi lệnh gõ của `contenteditable`, ít nhất hai lượt [22][23]; bản 20621 của app chưa đo.
- Bản macOS hay Safari nào đầu tiên mang `305121@main` cho WKWebView của app bên thứ ba? Ghi chú phát hành không dẫn số bug (không có nguồn).
- Khi chia lưới thành khối, vùng chọn kéo qua ranh giới hai khối có giữ đúng cột không? Chưa có nguồn; cần một ca e2e.
- Blink và WebView2 có chậm theo cùng kiểu không? Chưa có số.

- (D5) Trong engine 20621, `Node.normalize()` và việc gỡ, chèn nút chữ trong handler có ép tính bố cục đồng bộ không? Chưa đọc mã `ContainerNode::normalize`; cần đo bằng Web Inspector trên app thật.
- (D5) Đọc issue của w3c/selection-api năm 2024 để chốt `selectionchange` gộp theo task [21] hay bắn nhiều lần [25].
- (D5) Trong engine 20621, Highlight API so với bọc span cho gạch chân thì bên nào rẻ hơn mỗi phím?
- (D5) Người gõ nhanh có khoảng ngắn nhất giữa hai phím là bao nhiêu (phân vị 5 và 1, không phải trung bình)? Bộ dữ liệu của [36] có sẵn để tính.

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
| [18] | Safari 26.2 thêm Event Timing API | [WebKit blog](https://webkit.org/blog/17640/webkit-features-for-safari-26-2/) | 2025-12-12 | 2026-10-24 | high (một nhà xuất bản) |
| [19] | Event Timing: `startTime`, `processingStart`, `processingEnd`, `duration` tới lần vẽ kế | [W3C](https://w3c.github.io/event-timing/) | 2026-02-23 | 2026-10-24 | medium (đọc qua MDN) |
| [20] | `event.timeStamp` của WebKit lấy từ sự kiện hệ điều hành | [webkit-dev](https://lists.webkit.org/pipermail/webkit-dev/2013-June/025023.html) | 2013-06 | 2026-10-24 | medium |
| [21] | `selectionchange` gộp theo task; `setSelection` bỏ hiện vùng chọn khi bố cục đang chờ | [WebKit `main`, FrameSelection.cpp](https://raw.githubusercontent.com/WebKit/WebKit/main/Source/WebCore/editing/FrameSelection.cpp) | main, đọc 2026-10-24 | 2026-10-24 | high (cơ chế) / medium (dòng) |
| [22] | `CompositeEditCommand::apply()` gọi `updateLayoutIgnorePendingStylesheets()` ở đầu | [WebKit `main`, CompositeEditCommand.cpp](https://raw.githubusercontent.com/WebKit/WebKit/main/Source/WebCore/editing/CompositeEditCommand.cpp) | main, đọc 2026-10-24 | 2026-10-24 | high |
| [23] | `Editor::appliedEditing` gọi `updateLayout()` trước khi đọc vùng chọn cuối | [WebKit `main`, Editor.cpp](https://raw.githubusercontent.com/WebKit/WebKit/main/Source/WebCore/editing/Editor.cpp) | main, đọc 2026-10-24 | 2026-10-24 | medium-high |
| [24] | `DOMSelection::setBaseAndExtent` và họ hàng không tự gọi bố cục | [WebKit `main`, DOMSelection.cpp](https://raw.githubusercontent.com/WebKit/WebKit/main/Source/WebCore/page/DOMSelection.cpp) | main, đọc 2026-10-24 | 2026-10-24 | medium-high |
| [25] | Chromium và WebKit bắn `selectionchange` nhiều lần, Gecko một lần | [W3C public-webapps-github](https://lists.w3.org/Archives/Public/public-webapps-github/2024Aug/0034.html) | 2024-08-02 | 2026-10-24 | medium |
| [26] | Vue 3.5: đếm phiên bản, đồ thị phụ thuộc mới; micro-benchmark effect | [vuejs/core PR #10397](https://github.com/vuejs/core/pull/10397) | 2024-02-25 | 2026-10-24 | high |
| [27] | `shouldUpdateComponent` chỉ chặn cập nhật do cha; render effect tự `queueJob`; scheduler gộp một flush | [vuejs/core renderer.ts](https://github.com/vuejs/core/blob/main/packages/runtime-core/src/renderer.ts) | main | 2026-10-24 | medium (tóm tắt, cùng một kho) |
| [28] | Giữ prop ổn định, truyền giá trị nguyên thủy; `shallowRef` là lối thoát | [vuejs.org](https://vuejs.org/guide/best-practices/performance.html) | undated | 2026-10-24 | high |
| [29] | `v-memo`: bỏ qua cả tạo vnode khi mảng không đổi; tối ưu vi mô | [vuejs.org](https://vuejs.org/api/built-in-directives.html#v-memo) | undated | 2026-10-24 | high |
| [30] | Vue 3.5.44 ổn định; 3.6 RC; Vapor chưa ổn định | [vuejs/core releases](https://github.com/vuejs/core/releases) | 2026-10 | 2026-10-24 | medium |
| [31] | CSS Custom Highlight API có từ Safari 17.2; Baseline 2025 | [MDN](https://developer.mozilla.org/en-US/docs/Web/API/CSS_Custom_Highlight_API); [web.dev](https://web.dev/blog/web-platform-06-2025) | 2025-06 | 2026-10-24 | high |
| [32] | Highlight không đổi DOM; range sống được cập nhật, có chi phí; vẽ lại bất đồng bộ | [W3C CSSWG](https://drafts.csswg.org/css-highlight-api-1/) | draft | 2026-10-24 | high |
| [33] | Bug 323171: chi phí vẽ highlight tăng theo số range; sửa `324157@main` 2026-10-19; 323923 vẫn NEW | [WebKit Bugzilla](https://bugs.webkit.org/show_bug.cgi?id=323171); [commit e174317cd283](https://github.com/WebKit/WebKit/commit/e174317cd283) | 2026-09-01 / 2026-10-19 | 2026-10-24 | high |
| [34] | CodeMirror 6 `requestMeasure`: tách pha đọc và ghi, khử trùng theo khoá | [codemirror.net](https://codemirror.net/docs/ref/#view.EditorView.requestMeasure) | undated | 2026-10-24 | high |
| [35] | ProseMirror: vẽ lại vùng đổi, phần còn lại sau khoảng rảnh | [ProseMirror forum](https://discuss.prosemirror.net/t/responsivness-improvements-for-rendering-of-a-large-set-of-decorations/8834) | 2022–2025 | 2026-10-24 | medium (đoạn trích) |
| [36] | 136 triệu phím: IKI trung bình 238,7 ms; người gõ nhanh 121,7 ms | [Dhakal et al., CHI 2018](https://www.pokristensson.com/pubs/DhakalEtAlCHI2018.pdf) | 2018 | 2026-10-24 | medium (một nghiên cứu; lượt kiểm chéo không đọc được PDF) |
| [37] | Lặp phím nhanh nhất trong cài đặt macOS khoảng 30 ms | [til.vegardstikbakke.com](https://til.vegardstikbakke.com/posts/macos-keyboard-rate); [Apple](https://support.apple.com/en-gb/guide/mac-help/mchl0311bdb4/15.0/mac/15.0) | undated | 2026-10-24 | low-medium |
| [38] | Máy đo: `WebKit.framework` 20621.3.11.11.3, Safari 27.2 (20625.2.8.18.2), macOS 15.8.2, `KeyRepeat=5`, `InitialKeyRepeat=30` | dự án, đọc trên máy đo trong lượt này | 2026-10-24 | 2026-10-24 | high (một máy) |
| [39] | Bug 263303: kiểm chính tả làm chậm vùng sửa rất dài | [WebKit Bugzilla](https://bugs.webkit.org/show_bug.cgi?id=263303) | 2023 | 2026-10-24 | low (đoạn trích) |

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
| [18] Event Timing từ Safari 26.2 | version | 2025-12-12 | 2026-01-12 | yes |
| [33] bản sửa `324157@main` chưa tới engine của app | bug-status | 2026-10-19 | 2026-11-19 | no |
| [30] Vue 3.5.44 ổn định, 3.6 RC | library-version | 2026-10 | 2026-11 | no |
| [21][22][23] cơ chế sửa của WebKit `main` | engine-behavior | 2026-10-24 | 2028-10-24 | no |

Mốc kiểm lại sớm nhất là 2024-02-25 ([4]). Ba claim về trạng thái bug và phiên bản bị tính là cũ theo ngày phát hành, dù trang bug được đọc ngày 2026-10-23. Đặc tả [4] phải đọc lại tận văn bản css-grid-2 trước khi AD dựa vào nó.

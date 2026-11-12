---
title: 'technical research: Gõ chậm trong lưới subgrid lớn trên WebKit/Blink'
type: 'technical'
topic: 'Gõ chậm trong lưới subgrid lớn trên WebKit/Blink'
decision: 'Chọn hướng sửa đường gõ ở Chương 9.850 câu (bug 2.15) làm đầu vào một AD'
source: 'native run (web, 4 digests) + phép thử ba engine trên trang tổng hợp; Deepen D5 2026-10-24 (web, 4 digests + 1 lượt kiểm chéo, đọc phiên bản engine trên máy đo); Deepen D6–D11 2026-11-06 (web, 5 digests + 1 lượt đọc tracker bằng trình duyệt dựng JS + 1 phép đo đối chứng trên Blink 156)'
status: complete
preset: 'standard'
validation: 'normal'
created: '2026-10-23'
updated: '2026-11-06'
claims_verified: 18
claims_unverified: 14
claims_disputed: 1
claims_overturned: 1
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

**Deepen 2026-11-06 (D6–D11), không đổi kết luận trên, và không tìm ra nguyên nhân của triệu chứng mới.** Triệu chứng: gõ trễ 1–2 giây mỗi ký tự chỉ trên Windows (WebView2 156.0.4314.58, MacBook Pro 2015 trên Boot Camp) và chỉ ở kiểu Song song Hán Việt. Năm chiều mới (D6–D10) cộng một phép đo tại máy (D11) **không chống đỡ được** lời giải thích nào dựa trên lượng. Phép đo là chắc nhất: ruby và cỡ DOM không mua thêm chi phí nào, 94.057 nút cho 1,70 ms mỗi phím, bằng 14.057 nút chữ thường [53]. Ba đường còn lại chỉ **thiếu bằng chứng đỡ**, chứ không bị bác: tài liệu trợ năng không nói chi phí mỗi phím tăng theo cỡ DOM, nhưng cũng không nói ngược lại [46]; `software_rendering_list.json` không có mục nào gọi tên GPU của máy đó, nhưng tệp ấy khớp bằng mã hex nên điều đó không chứng minh không mục nào khớp [50]; bộ đệm glyph của Skia khoá theo strike chứ không theo cỡ tệp phông, nhưng đó là mặc định lúc dịch và nguồn không nói Chromium có giữ nó [52]. Tiền lệ mạnh nhất, issue 40688351, ghi đúng sự bất đối xứng Windows so với macOS trong cùng một Blink và đúng hình dạng DOM của lưới này, nhưng trạng thái hiện tại là Won't Fix (Obsolete) và không có CL sửa nào [40]. ⇒ Nguyên nhân nằm ở một chi phí của nền tảng Windows hoặc của riêng máy đó, và không nguồn công khai nào đo được nó: việc còn lại là một vệt DevTools qua cổng 9222 trên chính bản MSI đã cài, khuyến nghị 12–14. Khuyến nghị 15: không sửa CSS hay DOM của lưới dựa trên lượt này.

**Spike 2.17 (2026-11-12, D12) trả lời phần Windows.** Vệt đo trên chính máy Windows cho thấy mỗi lần ô bản dịch đổi chữ, Blink bố cục lại toàn bộ cây bố cục của lưới (37 đối tượng bẩn, 8.562 đối tượng được bố cục, 67 ms trung vị ở Song song, 7 ms ở Nguyên văn). EVKey biến mỗi chữ có dấu thành phím xoá cộng gõ lại, nên có ít nhất hai lần bố cục, và hàng phím chờ dài dần. Trợ năng, vẽ và JS của app bị loại bằng số. Lượt tiếp theo là bug 2.19.

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

## D6. Blink làm gì ở mỗi phím trong `contenteditable` (Deepen 2026-11-06)

Chiều này phục vụ một triệu chứng mới: gõ một ký tự vào ô bản dịch trễ khoảng 1–2 giây **chỉ trên Windows** (WebView2 Evergreen 156.0.4314.58, MacBook Pro 2015 chạy Windows 10 qua Boot Camp) và **chỉ ở kiểu Song song Hán Việt**; trên macOS mọi kiểu đều mượt.

- **Blink có gọi tính style và bố cục ở mức tài liệu trong đường gõ, nhưng đó chưa phải bằng chứng cho một lượt bố cục cả tài liệu.** `CompositeEditCommand::Apply` gọi `GetDocument().UpdateStyleAndLayout(DocumentUpdateReason::kEditing)` trước `DoApply` [41], và `typing_command.cc` có bốn lời gọi cùng loại, ở `TypingCommand::InsertText`, `AdjustSelectionAfterIncrementalInsertion`, `DeleteKeyPressed` và `ForwardDeleteKeyPressed` [42]. Các lời gọi này chỉ tốn việc thật khi style hoặc bố cục đang bẩn. Độ tin trung bình: đọc qua bản tóm tắt của mã base64, không phải nguyên văn.
- 🔴 **Có tiền lệ được ghi nhận cho đúng sự bất đối xứng mà Ice báo, và nó nằm trong cùng một engine.** Chromium issue 40688351 (vốn là `crbug.com/1076718`), "Continues typing in contenteditable is slow in certain cases", thành phần `Blink>Editing` và `Blink>Layout`, trường OS ghi **Mac và Windows**. Bình luận #12 (2021-09-02): *"In mac-chrome there is no much Layout happening. But in win10-chrome Forced Layout is happening on every keystroke."* Bình luận #9 (2020-06-01) khai đúng hình dạng DOM của lưới này — `outerDiv > innerDiv > contentEditableDiv` lặp nhiều lần — và nói nó chậm hẳn khi có nhiều phần tử. Hai cách chữa người báo đo được: `overflow-y: hidden` trên ô sửa, hoặc cho ô sửa ra khỏi dòng chảy bằng `position: absolute`; cả hai cắt cùng một thứ, là việc chiều cao ô sửa dội lên bố cục của tổ tiên [40].
- ⚠️ **Nhưng bug đó đã đóng, và đóng theo cách không chứng minh được gì.** Trạng thái hiện tại là Won't Fix (Obsolete), đóng 2025-08-15 với bình luận #19 *"I'm not able to reproduce this anymore"*; hai trường `Code Changes` và `Fixed By Code Changes` đều trống [40]. ⇒ Nó đóng vì một người không tái lập được, không vì có bản sửa, nên đường mã nó mô tả chưa có bằng chứng là đã mất. Theo luật tươi của pack, không được dẫn nó như nguyên nhân cho bản 156; nó chỉ là tiền lệ cho **cơ chế**.
- **Ca đã đo khớp nhất không nằm ở bố cục mà ở tính lại style.** Một app WebView2 trên Windows có trung vị từ phím tới khung 160,1 ms, xuống 5,0 ms sau khi bỏ các bộ chọn `:has()`; chi phí tăng theo độ dài hội thoại (23,7 ms lên 100,3 ms) và headless Chromium tái lập được, nên là tầng Blink chứ không riêng WebView2 [43]. Độ tin trung bình: một nguồn, tác giả tự báo, 2026-11-04. ⚠️ Nguồn đếm **số lượt hội thoại** (30 lượt), không đếm số nút DOM, nên *theo cỡ DOM* là lời diễn giải, không phải số của nguồn. ⚠️ Cũng không áp thẳng vào app này: 18 chỗ `:has()` trong `src/` đều nằm ở lớp phủ, `controls.css` và chủ đề dockview, mỗi chỗ neo trong cây con của một `input`; không chỗ nào trong lưới (quan sát mã, không phải bằng chứng web).
- **Một đầu mối chưa đọc được, ghi lại để không mất:** issue 41237496 được dẫn là nói về việc tạo `PlainTextRange` lặp lại và một lượt bố cục sạch cho mỗi ký tự [54]. Độ tin thấp: chỉ thấy đoạn trích của công cụ tìm, trang chưa đọc, trạng thái chưa biết.
- **Không tìm được** hồi quy editing hay forced-layout nào trong dải Blink 150–157, ghi chú phát hành WebView2 nào về việc này, hay tài liệu thiết kế Chromium nào về chi phí gõ theo cỡ tài liệu. Cũng không tìm được issue WebView2Feedback nào chỉ ra một chi phí WebView2 thêm vào trên Blink.

## D7. Ruby trong Blink ở quy mô lớn (Deepen 2026-11-06)

- **Lượt viết lại ruby đọc được nguyên văn là intent "Line-breakable ruby"** (Kent Tamura, 2024-05-31), ship ở Chrome 128 trên desktop, Android và WebView, sau cờ Finch `RubyLineBreakable`. Ruby xuất hiện ở khoảng 0,14% lượt xem trang, và luồng không công bố số hiệu năng nào [49]. Tài liệu thiết kế, bug theo dõi `324111880` và mục chromestatus mà intent dẫn tới đều không mở được, nên chỉ là đầu mối.
- **Không tìm được bug Chromium nào về ruby chậm hay giật** — nhiều phần tử, `ruby-position: under`, hay ruby trong nội dung sửa được. ⚠️ WebSearch không tìm được trong tracker, nên vắng bằng chứng ở đây **không** chứng minh là không có bug.
- **Không tìm được chỗ nào nối ruby với CSS Grid hay subgrid.** Các bug Grid nổi lên đều về kích thước nội tại và bộ đệm căn baseline nói chung.
- ⇒ Chi phí ruby trên Windows so với macOS: không xác lập được từ nguồn công khai. D11 trả lời bằng phép đo tại máy.

## D8. Hai chi phí chỉ Windows có: cây trợ năng UIA và nhập liệu TSF (Deepen 2026-11-06)

- **Cây trợ năng tắt theo mặc định và chỉ bật theo yêu cầu** [44]. Trên Windows, `DiscoverAssistiveTech` dò Sticky Keys, `RunningState` của Narrator và Magnifier, và các module AT đã nạp như JAWS hay NVDA; một screen reader được phát hiện thì đặt `kAXModeComplete | kScreenReader`. `uiautomation.dll` **không** bị coi là screen reader [45]. Độ tin trung bình: đọc qua bản tóm tắt.
- 🔵 **Sửa 2026-11-06 sau lượt kiểm ngữ nghĩa: nguồn [46] không nói điều bản viết đầu đã khai.** Bản đầu ghi *chi phí cập nhật là theo nút bẩn, không theo cả cây* và dùng câu đó để bác giả thuyết. Trang của Chrome nói khác: một nút bẩn được tuần tự lại **kèm mọi thuộc tính, cả những thuộc tính không đổi** — đó là nói về sự lãng phí bên trong một nút, không nói về phạm vi của lượt cập nhật, và trang không nói gì về việc gõ hay `contenteditable` [46]. Con số dùng được duy nhất: Chrome tự báo 5–10% người dùng có trợ năng bật, thường qua trình quản lý mật khẩu và phần mềm diệt virus [46]. ⇒ Giả thuyết *cây trợ năng tốn việc theo cỡ DOM mỗi phím* **không bị bác và cũng không được đỡ**; nó vẫn mở, và chỉ phép thử ở khuyến nghị 14 trả lời được.
- **Chrome 138 trở lên bật UIA bản địa theo mặc định** [48], nhưng đó là việc khác với việc cây được dựng.
- **`--disable-renderer-accessibility` còn trong tài liệu WebView2 hiện hành** (cập nhật 2026-10-13) và đè `--force-renderer-accessibility`; tài liệu cảnh báo cờ có thể bị bỏ bất cứ lúc nào [47]. Chưa ai báo nó chữa được độ trễ gõ, và cũng chưa ai báo nó hỏng.
- **Không tìm được**: bằng chứng nào cho việc bàn phím cảm ứng, TSF hay gợi ý chữ gắn một client trợ năng vô điều kiện; phép đo nào cho chi phí TSF mỗi phím trong Chromium, hay một bug Chromium nào về nó (bằng chứng TSF duy nhất là một bug Firefox); chi phí trợ năng riêng nào cho `<ruby>`/`<rt>`.

## D9. Chặn GPU và raster bằng phần mềm trên driver Boot Camp (Deepen 2026-11-06)

- **`software_rendering_list.json` ở `main` không có mục nào đặt `gpu_compositing`, và không mục nào gọi TÊN Iris, HD 4000–6000 hay R9 M370X** [50]. Các mục Intel và AMD trên Windows có thể chạm máy đó đều hẹp: id 122 chặn GPU rasterization theo feature level D3D11 dưới 11.0; id 145 và 163 về WebGL và canvas 2D trên driver cũ; id 181 chỉ HD 4400 và chỉ cho Graphite. ⚠️ Giới hạn của bằng chứng này: tệp đó khớp thiết bị bằng **mã hex của vendor và device**, không bằng tên thương mại, nên *không gọi tên* là đúng nguyên văn mà **không** chứng minh được là không mục nào khớp GPU của máy đó. ⇒ Đường chặn `gpu_compositing` theo danh sách thiếu bằng chứng đỡ; chỉ hàng Problems Detected trong `edge://gpu` trên máy đó chốt được.
- **`gpu_switches.cc` ở HEAD còn định nghĩa `--ignore-gpu-blocklist`, `--enable-gpu-rasterization` và `--disable-gpu-rasterization`; `--disable-gpu` không nằm trong tệp đó** [51]. Tài liệu WebView2 vẫn liệt cả `--disable-gpu` và `--ignore-gpu-blocklist` qua `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, và nói rõ chỉ để chẩn đoán [47].
- **Không tìm được** số đo công khai nào so độ trễ giữa hợp thành bằng GPU và bằng phần mềm; tài liệu Chromium nào về chi phí của tỉ lệ hiển thị 200% (con số bốn lần số điểm ảnh là phép tính, không phải bằng chứng); báo cáo nào về một MacBook Pro 2015 trên Boot Camp hiện "Software only".
- ⇒ Không xác lập. Một lượt khởi tạo D3D11 trượt hoặc tiến trình GPU chết với driver Boot Camp đã ngưng cập nhật vẫn còn khả năng, và chỉ `edge://gpu` trên máy đó mới nói được: bốn hàng cần đọc là Compositing, Rasterization, Problems Detected và Active GPU.

## D10. Phông CJK 24,5 MB qua giao thức asset (Deepen 2026-11-06)

- **Bộ đệm glyph của Skia mặc định 2048 mục và 2 MiB, và khoá theo *strike* chứ không theo cỡ tệp phông** [52], nên một tệp 24,5 MB tự nó không làm tràn. ⚠️ Hai giới hạn: đó là mặc định **lúc dịch**, và nguồn không nói Chromium có giữ nó, cũng không nói trần 2048 đếm strike hay đếm gì khác.
- **Không tìm được**: bug hay tài liệu nào về `ShapeCache` bị đuổi hoặc thrash với CJK; DirectWrite chậm với phông lớn hay với CFF; việc nạp lại, trượt bộ đệm hay giải mã lại phông qua một giao thức tuỳ biến; bất kỳ bằng chứng nào cho việc cỡ phông ảnh hưởng bố cục hay vẽ **sau** khi đã nạp.
- ⇒ Không xác lập. ⚠️ Các URL mã Blink mà nhánh này thử trên googlesource và raw.githubusercontent đều trả 404 hoặc 403, nên `ShapeCache`, `FontCache` và `FontDataCache` chưa được đọc trực tiếp; kết luận này là vắng bằng chứng, không phải bằng chứng vắng.

## D11. Phép đo đối chứng trên Blink 156 (đo trong lượt này, 2026-11-06)

Cùng trang tổng hợp của D4 mở rộng: 2.000 câu, khối 200 hàng, bốn cột subgrid, ô bản dịch `contenteditable` kèm `role="textbox"` và `aria-label` từng ô. Cột nguyên văn chạy hai kiểu: chữ thường, và kiểu Song song với một `<ruby><rt>` mỗi từ. Chèn `kha` bằng `execCommand('insertText')` 40 lượt, mỗi lượt sau một `requestAnimationFrame`, rồi đọc `scrollHeight` để ép bố cục. Engine: Chrome for Testing 156.0.8078.4 (cùng dòng Blink với WebView2 156.0.4314.58 trên máy Ice), headless, 1280×860, macOS.

| Cột nguyên văn | CSS ô bản dịch | Số nút | Trung vị (ms) | p90 (ms) | Lớn nhất (ms) |
|---|---|---|---|---|---|
| chữ thường | gốc, như app | 14.057 | 1,90 | 2,10 | 5,00 |
| chữ thường | `overflow-y: hidden` | 14.057 | 1,60 | 1,90 | 4,50 |
| chữ thường | `contain: size` + cao cố định | 14.057 | 1,80 | 1,90 | 2,10 |
| **Song song (ruby)** | gốc, như app | **94.057** | **1,70** | 2,00 | 4,30 |
| Song song (ruby) | `overflow-y: hidden` | 94.057 | 1,80 | 2,20 | 4,80 |
| Song song (ruby) | `contain: size` + cao cố định | 94.057 | 1,70 | 1,90 | 2,00 |

Ba kết luận [53]:

- 🔴 **Chi phí mỗi phím không tăng theo cỡ DOM.** 94.057 nút với hàng nghìn `<ruby>` cho 1,70 ms, còn 14.057 nút chữ thường cho 1,90 ms — gấp 6,7 lần số nút, chi phí **không** tăng. Kích thước nội tại của ruby được đệm.
- **Hai cách chữa của issue 40688351 không mua được gì ở kiểu Song song, tức ở đúng kiểu đang lỗi** [40][53]. Ở hàng ruby, `overflow-y: hidden` cho 1,80 ms so với 1,70 ms, không cải thiện. ⚠️ Ở hàng chữ thường thì nó hạ trung vị 1,90 xuống 1,60 ms, khoảng 16%, nên câu *không mua được gì* chỉ đúng cho kiểu Song song, và phần cải thiện ở cột thường vẫn là câu hỏi mở. `contain: size` cộng chiều cao cố định hạ **đỉnh** rõ ở cả hai hàng (4,30 xuống 2,00 ms với ruby; 5,00 xuống 2,10 ms với chữ thường) nhưng trung vị gần như đứng (1,70 so với 1,70; 1,80 so với 1,90), tức nó cắt các lượt đổi chiều cao hàng, mà những lượt đó vốn đã rẻ.
- ⇒ Trên Blink 156, cấu trúc DOM và CSS của lưới **không** giải thích được độ trễ 1–2 giây. Nguyên nhân nằm ở thứ chỉ Windows có, hoặc ở chính máy đó.

## D12. Vệt Performance trên máy Windows (spike 2.17, 2026-11-12)

Đo trên máy ảo VMware của Ice: Windows, MSI v1.0.2, WebView2 nối cổng 9222 qua `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, vẽ hoàn toàn bằng phần mềm (`gpu: VMware SVGA 3D`, `ANGLE_D3D11_WARP`, xem `windows-trace/gpu-vmware-2026-11-12.txt`). Ice gõ bằng EVKey. Máy ảo này còn nhanh hơn máy Boot Camp nơi người dùng gặp lỗi. Ba vệt đầu ghi khi app đã chậm. Ba vệt sau ghi lại cùng ba kiểu, với `--disable-renderer-accessibility`. Bảy vệt nặng khoảng 250 MB, nằm ngoài repo; số dưới đây tính bằng hai script trong `windows-trace/`.

| Vệt | Kiểu xem, cách gõ | Đối tượng bố cục | Lần bố cục | Một lần bố cục, trung vị / p90 | Tác vụ > 50 ms, dài nhất |
|---|---|---|---|---|---|
| 145309 | Song song, có dấu | 8.562 | 192 cho 145 chữ | 67 / 101 ms | 99, 865 ms |
| 145536 | Song song, không dấu | 8.562 | 102 cho 99 chữ | 66 / 96 ms | 104, 134 ms |
| 145629 | kiểu thứ ba, có dấu | 8.876 | 179 | 20 / 25 ms | 15, 204 ms |
| 145718 | Nguyên văn, có dấu | 827 | 193 | 7 / 9 ms | 4, 71 ms |
| 150334 | Song song, có dấu, tắt trợ năng | 8.501 | 84 | 63 / 100 ms | 37, 471 ms |
| 150419 | Song song, không dấu, tắt trợ năng | 8.501 | 59 | 61 / 93 ms | 60, 120 ms |
| 150507 | kiểu thứ ba, có dấu, tắt trợ năng | 8.816 | 88 | 19 / 24 ms | 10, 189 ms |

- 🔴 **Mỗi lần sửa chữ là một lần bố cục lại toàn bộ cây**, dù chỉ 37 đối tượng bẩn. Cả 7 vệt đều ghi `dirtyObjects` = 37, `partialLayout` = false, một gốc bố cục. Chi phí tăng gần tuyến tính theo số đối tượng: 827 đối tượng 7 ms, 8.562 đối tượng 67 ms. 827 ứng với khoảng 200 hàng × 4 ô, nên cây được bố cục lại cỡ một khối 200 hàng hoặc cả Chương; vệt không cho biết Chương đó dài bao nhiêu câu. Lượt bố cục nằm trong `textInput` (chữ gõ vào) hoặc `keydown` (phím xoá EVKey gửi). Không lượt nào có stack JS, tức không do mã của app ép.
- **EVKey nhân số lần bố cục.** Một chữ có dấu là phím xoá rồi gõ lại: vệt 145309 có 193 `keydown` cho 145 chữ và 192 lần bố cục. Bố cục chiếm 12,9 s trong 27,8 s ghi. Gõ không dấu là một lần bố cục mỗi phím, và máy theo kịp. ⇒ "Chậm sau một lúc gõ" khớp với một hàng phím chờ dài dần khi nhịp gõ có dấu vượt sức máy. Chưa kiểm chi phí mỗi lần bố cục có tăng dần trong một vệt hay không.
- **Đã loại bằng số:** JS của app (`FunctionCall` 0,10–0,45 s mỗi vệt); cây trợ năng (tắt bằng cờ thì 63 so với 67 ms, 61 so với 66, 19 so với 20, nên D8 đóng); vẽ (`Paint` dưới 1,4 s mỗi vệt dù vẽ bằng CPU, nên D9 không phải nguyên nhân chính).
- ⇒ Câu hỏi còn lại cho bug 2.19: vì sao Blink bố cục lại toàn bộ cây (không chỉ ô đang gõ) ở mỗi lần sửa một ô. D11 đo 1,7 ms ở trang tổng hợp trên Chrome macOS, nên điều kiện khiến app thật bố cục lại toàn bộ chưa được tái hiện ngoài app.

## Cross-dimension insights

- **Ảo hoá chỉ "rẻ" khi editor tự giữ vùng chọn và undo** (CodeMirror [8], ProseMirror [9]). App này cố ý dùng `⌘Z` gốc và vùng chọn gốc tới trọn một Chương. Vì vậy ảo hoá không phải là thêm một thư viện, mà là viết lại vùng chọn và undo. Điều này đụng hai điều kiện bắt buộc số 3 và 5.
- **Chậm nằm ở độ dài của subgrid, không ở việc dùng lưới.** Cùng engine, cùng DOM, cắt subgrid từ 9.850 xuống 200 hàng thì chi phí một phím giảm từ 468 xuống 10 ms [17]. Điều này khớp với bug 302529, vốn chỉ hết khi tắt subgrid [3], và với điểm nóng ở bước tính track của subgrid trải nhiều hàng [1]. Bản sửa của WebKit nhắm đúng chỗ đó nhưng chưa đủ cho đường gõ, và macOS 15 không nhận được bản sửa ấy cho app bên thứ ba [16].

- **(D5) Chia khối làm nhỏ vùng phải bố cục lại, nhưng không bỏ được lượt bố cục mỗi phím.** Lệnh gõ chữ của WebKit vẫn tính lại bố cục ít nhất hai lần cho mỗi phím [22][23]. Số đo ở [17] cho thấy chi phí không còn nằm ở độ dài subgrid. Giả thuyết, chưa đo: phần còn lại có thể nằm ở lượng DOM mà handler làm bẩn trước lệnh gõ chữ, và ở việc Vue render lại sau đó [27].


- **(Deepen 2026-11-06) Năm chiều mới không tìm ra nguyên nhân, nhưng chúng thu hẹp nó rất mạnh, và đó là kết quả.** Mỗi lời giải thích dựa trên *lượng* đã bị loại bằng chứng: ruby không thêm chi phí bố cục và cỡ DOM không mua thêm chi phí nào ở 94.057 nút [53]; còn ba đường kia chỉ thiếu bằng chứng đỡ chứ không bị bác, vì nguồn trợ năng không nói về chi phí mỗi phím [46], danh sách chặn GPU khớp bằng mã hex nên không đọc ra được máy nào [50], và mặc định bộ đệm glyph là mặc định lúc dịch [52]; engine trên máy Ice thì là 156, cùng dòng với engine đã đo [53]. Đường gõ trong mã app thì đã được lập bản đồ ở `research-grid-text-input-path` §1–§4 và là O(1) mỗi phím. ⇒ Thứ còn lại không phải một đại lượng nào trong trang, mà là một chi phí của nền tảng hoặc của riêng máy đó, và không nguồn công khai nào đo được nó.
- **(Deepen 2026-11-06) Tiền lệ mạnh nhất lại là một bug đã đóng vì không tái lập được.** Issue 40688351 ghi đúng sự bất đối xứng Windows so với macOS trong cùng một Blink, với đúng hình dạng DOM của lưới này, nhưng trạng thái hiện tại là Obsolete và không có CL sửa nào [40]. ⇒ Nó đủ để nói cơ chế ấy **từng** tồn tại trong Blink, và không đủ để nói nó đang gây ra việc này. Phân biệt hai điều đó cần một vệt đo trên máy, không cần thêm một vòng tìm kiếm.

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


12. **(Deepen 2026-11-06) Lấy một vệt DevTools từ bản MSI đã cài, trước khi sửa bất cứ dòng nào.** Không cần dựng lại: `--remote-debugging-port=9222 --remote-allow-origins=*` qua `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, và giá trị của biến môi trường được **nối** vào giá trị app truyền chứ không bị thay [55] — việc này cần thiết vì wry luôn đặt `AdditionalBrowserArguments` với một mặc định khác rỗng [56]. Feature `devtools` của Tauri không liên quan: nó chỉ bật cửa sổ DevTools và menu chuột phải, còn cổng CDP là cờ của tiến trình browser. Đường dự phòng là khoá registry `HKCU\Software\Policies\Microsoft\Edge\WebView2\AdditionalBrowserArguments` [55]. Vệt cần trả lời đúng một câu: phần lớn thời gian nằm ở `Recalculate Style`, `Layout`, `Pre-Paint`, `Paint`, `Composite`, hay một hàm JS có tên. Độ tin: cao về cơ chế lấy vệt [55][56]; chưa chạy.
13. **(Deepen 2026-11-06) Đọc `edge://gpu` trên máy đó và A/B ba cờ, vì danh sách chặn đã bị loại nhưng một lượt khởi tạo trượt thì chưa.** Bốn hàng cần đọc: Compositing, Rasterization, Problems Detected, Active GPU. Ba cờ cần so: `--ignore-gpu-blocklist`, `--disable-gpu`, và `force-device-scale-factor=1` để tách phần chi phí do tỉ lệ hiển thị. Nếu `--disable-gpu` **không** làm chậm thêm thì máy vốn đã vẽ bằng CPU, và hướng sửa là giảm việc vẽ chứ không giảm bố cục. Độ tin: trung bình — `software_rendering_list.json` loại đường danh sách chặn [50], nhưng không có số đo công khai nào cho hình phạt của raster phần mềm.
14. **(Deepen 2026-11-06) Thử `--disable-renderer-accessibility` một lượt, và coi kết quả âm là kết quả.** Tài liệu nói chi phí trợ năng là theo nút bẩn [46] và `uiautomation.dll` không bị coi là screen reader [45], nên giả thuyết này **đã** yếu; phép thử tốn một lần khởi động app và đóng được nó hẳn. Kèm: đọc `edge://accessibility` để biết chế độ AX đang bật là gì. Độ tin: trung bình về cờ còn hiệu lực [47]; không có báo cáo nào về việc nó chữa độ trễ gõ.
15. **(Deepen 2026-11-06) Không sửa CSS hay DOM của lưới dựa trên lượt này.** `overflow-y: hidden` và `contain: size` đều được đo và đều không hạ trung vị [53]; `:has()` không có chỗ nào trong lưới. Một bản vá dựa trên tiền lệ của issue 40688351 mà chưa có vệt đo sẽ là đổi mã theo giả thuyết. Độ tin: cao, vì đây là một kết quả âm đo được, không phải một suy luận.

## Open questions

- 🔵 2026-10-23 — đã trả lời bằng D4: bản sửa `305121@main` không làm nhanh việc sửa nội dung một ô, vì WebKit 627 vẫn mất 998 ms mỗi phím với subgrid nguyên khối.
- 🔵 2026-10-24 — đã trả lời bằng D5: WebKit có tính bố cục đồng bộ ở mỗi lệnh gõ của `contenteditable`, ít nhất hai lượt [22][23]; bản 20621 của app chưa đo.
- Bản macOS hay Safari nào đầu tiên mang `305121@main` cho WKWebView của app bên thứ ba? Ghi chú phát hành không dẫn số bug (không có nguồn).
- Khi chia lưới thành khối, vùng chọn kéo qua ranh giới hai khối có giữ đúng cột không? Chưa có nguồn; cần một ca e2e.
- 🔵 2026-11-06 — đã trả lời một nửa bằng D11: trên Blink 156, cấu trúc lưới **không** chậm theo cỡ DOM; 94.057 nút với ruby cho 1,70 ms mỗi phím, bằng 14.057 nút chữ thường [53]. Nửa còn lại chưa trả lời: chi phí chỉ Windows có thì chưa đo được từ xa.

- (D5) Trong engine 20621, `Node.normalize()` và việc gỡ, chèn nút chữ trong handler có ép tính bố cục đồng bộ không? Chưa đọc mã `ContainerNode::normalize`; cần đo bằng Web Inspector trên app thật.
- (D5) Đọc issue của w3c/selection-api năm 2024 để chốt `selectionchange` gộp theo task [21] hay bắn nhiều lần [25].
- (D5) Trong engine 20621, Highlight API so với bọc span cho gạch chân thì bên nào rẻ hơn mỗi phím?
- (D5) Người gõ nhanh có khoảng ngắn nhất giữa hai phím là bao nhiêu (phân vị 5 và 1, không phải trung bình)? Bộ dữ liệu của [36] có sẵn để tính.


- (D6–D11) Trên máy Windows đó, phần lớn thời gian mỗi phím nằm ở bước nào? Chỉ một vệt DevTools qua cổng 9222 trả lời được; mọi nguồn công khai đã cạn. Khuyến nghị 12.
- (D9) Máy đó đang hợp thành bằng GPU hay bằng CPU, và tỉ lệ hiển thị 200% đóng góp bao nhiêu? `edge://gpu` cộng A/B ba cờ. Khuyến nghị 13.
- (D7) Có bug Chromium nào về ruby chậm ở quy mô lớn không? Chưa biết: WebSearch không tìm được trong tracker, và thành phần `Blink>Layout>Ruby` cần một trình duyệt dựng JS để đọc. Đọc bug `324111880` và tài liệu thiết kế của intent [49].
- (D10) `ShapeCache`, `FontCache` và `FontDataCache` của Blink hành xử thế nào với một CFF 24,5 MB? Mã chưa đọc được vì URL trả 404 và 403; cần đọc trên source.chromium.org bằng trình duyệt dựng JS.
- (D6) Issue 40688351 đóng vì không tái lập được, không vì có bản sửa [40]. Đường mã mà bình luận #12 mô tả còn trong Blink 156 không? Cần đọc mã, hoặc tái lập trên một máy Windows khác để tách máy ra khỏi nền tảng.

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
| [40] | Issue 40688351 (cũ `crbug.com/1076718`): gõ liên tục trong `contenteditable` chậm; OS Mac và Windows; #12 Forced Layout mỗi phím trên win10-chrome, #9 hình dạng `outerDiv > innerDiv > contentEditableDiv`; chữa bằng `overflow-y: hidden` hoặc `position: absolute`; trạng thái HIỆN TẠI Won't Fix (Obsolete) 2025-08-15, không CL sửa | [Chromium Issue Tracker](https://issues.chromium.org/issues/40688351) | 2020-04-30 / đóng 2025-08-15 | 2026-11-06 | high (đọc nguyên trang bằng trình duyệt dựng JS) |
| [41] | `CompositeEditCommand::Apply` gọi `UpdateStyleAndLayout(DocumentUpdateReason::kEditing)` trước `DoApply` | [chromium.googlesource.com](https://chromium.googlesource.com/chromium/src/+/main/third_party/blink/renderer/core/editing/commands/composite_edit_command.cc?format=TEXT) | main | 2026-11-06 | medium (bản tóm tắt của mã base64) |
| [42] | `typing_command.cc` có bốn lời gọi `UpdateStyleAndLayout(kEditing)` | [chromium.googlesource.com](https://chromium.googlesource.com/chromium/src/+/main/third_party/blink/renderer/core/editing/commands/typing_command.cc?format=TEXT) | main | 2026-11-06 | medium (cùng lý do) |
| [43] | App WebView2 trên Windows: trung vị phím tới khung 160,1 ms xuống 5,0 ms sau khi bỏ `:has()`; tăng theo cỡ DOM 23,7 lên 100,3 ms; headless Chromium tái lập | [unslothai/unsloth PR 16656](https://github.com/unslothai/unsloth/pull/16656) | 2026-11-04 | 2026-11-06 | medium (một nguồn, tác giả tự báo) |
| [44] | Cây trợ năng tắt theo mặc định, bật theo yêu cầu | [Chromium accessibility overview](https://chromium.googlesource.com/chromium/src/+/main/docs/accessibility/overview.md) | main | 2026-11-06 | high |
| [45] | `DiscoverAssistiveTech` dò Sticky Keys, Narrator, Magnifier, module AT đã nạp; `uiautomation.dll` không coi là screen reader | [browser_accessibility_state_impl_win.cc](https://chromium.googlesource.com/chromium/src/+/main/content/browser/accessibility/browser_accessibility_state_impl_win.cc) | main | 2026-11-06 | medium (bản tóm tắt) |
| [46] | Một nút đổi chỉ tuần tự lại thuộc tính của nút đó, không cả cây; 5–10% người dùng có trợ năng bật | [developer.chrome.com](https://developer.chrome.com/blog/chromium-accessibility-performance) | undated | 2026-11-06 | low (không nói về gõ hay `contenteditable`; nguồn chỉ nói nút bẩn được tuần tự lại với MỌI thuộc tính kể cả thuộc tính không đổi, không nói rõ "không cả cây") |
| [47] | Cờ WebView2: `force-renderer-accessibility`, `disable-renderer-accessibility`, `disable-gpu`, `ignore-gpu-blocklist` qua `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`; chỉ để chẩn đoán; cờ có thể bị bỏ | [Microsoft Learn](https://learn.microsoft.com/microsoft-edge/webview2/concepts/webview-features-flags) | 2026-10-13 | 2026-11-06 | high |
| [48] | Chrome 138 trở lên bật UIA bản địa theo mặc định | [developer.chrome.com](https://developer.chrome.com/blog/windows-uia-support-update) | 2025-08-14 | 2026-11-06 | high |
| [49] | Intent "Line-breakable ruby": ship Chrome 128, cờ Finch `RubyLineBreakable`, ruby ở khoảng 0,14% lượt xem trang, không số hiệu năng | [blink-dev](https://groups.google.com/a/chromium.org/g/blink-dev/c/vkWGs7_yWZM) | 2024-05-31 | 2026-11-06 | high (đọc nguyên luồng hiện) |
| [50] | `software_rendering_list.json` không có mục nào đặt `gpu_compositing`, không mục nào gọi tên Iris, HD 4000–6000 hay R9 M370X | [chromium.googlesource.com](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/gpu/config/software_rendering_list.json?format=TEXT) | main | 2026-11-06 | medium (bản tóm tắt) |
| [51] | `gpu_switches.cc` định nghĩa `ignore-gpu-blocklist`, `enable-gpu-rasterization`, `disable-gpu-rasterization`; `disable-gpu` không trong tệp đó | [chromium.googlesource.com](https://chromium.googlesource.com/chromium/src.git/+/HEAD/gpu/config/gpu_switches.cc) | HEAD | 2026-11-06 | medium |
| [52] | Bộ đệm glyph của Skia mặc định 2048 mục và 2 MiB, khoá theo strike không theo cỡ tệp phông | [SkStrikeCache.h](https://skia.googlesource.com/skia/+/main/src/core/SkStrikeCache.h) | main | 2026-11-06 | medium |
| [53] | Phép đo lượt này: Chrome for Testing 156.0.8078.4 headless, 2.000 câu, khối 200 hàng — ruby 94.057 nút 1,70 ms mỗi phím so với chữ thường 14.057 nút 1,90 ms; `overflow-y: hidden` 1,80 ms; `contain: size` hạ đỉnh 4,30 xuống 2,00 ms nhưng không hạ trung vị | dự án, đo trên máy Ice trong lượt này | 2026-11-06 | 2026-11-06 | high (một máy, 40 phím mỗi biến thể) |
| [54] | Issue 41237496: tạo `PlainTextRange` lặp lại và một lượt bố cục sạch mỗi ký tự | [Chromium Issue Tracker](https://issues.chromium.org/issues/41237496) | undated | 2026-11-06 | low (chỉ đoạn trích, trạng thái chưa đọc) |
| [55] | Giá trị `AdditionalBrowserArguments` từ biến môi trường hoặc registry được NỐI vào giá trị app truyền; đường registry `Software\Policies\Microsoft\Edge\WebView2`; HKCU bị bỏ qua với tiến trình nâng quyền | [Microsoft Learn Win32 Globals](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/webview2-idl?view=webview2-1.0.3912.50); [Develop secure WebView2 apps](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/security) | undated | 2026-11-06 | high (hai trang tài liệu gốc) |
| [56] | wry luôn gọi `set_additional_browser_arguments` với mặc định khác rỗng, nên biến môi trường phải NỐI mới có hiệu lực | `~/.cargo/registry/src/…/wry-0.55.1/src/webview2/mod.rs:294-327` | 0.55.1 | 2026-11-06 | high (đọc nguồn đã tải về) |

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

🔵 Deepen 2026-11-06 — các khẳng định mới, cùng cửa sổ với bảng trên, cộng `performance-number` 6 tháng và `source-code` 6 tháng. Tính bằng `recon_kit.py staleness`.

| Claim | Class | Pub | Re-check | Stale |
|---|---|---|---|---|
| [40] Issue 40688351 Won't Fix (Obsolete), khong CL sua | bug-status | 2026-11 | 2026-12-06 | no |
| [41] CompositeEditCommand::Apply goi UpdateStyleAndLayout(kEditing) | source-code | 2026-11 | 2027-05-06 | no |
| [42] typing_command.cc co bon loi goi UpdateStyleAndLayout(kEditing) | source-code | 2026-11 | 2027-05-06 | no |
| [43] WebView2 Windows 160,1 ms xuong 5,0 ms sau khi bo :has() | performance-number | 2026-11 | 2027-05-04 | no |
| [44] Cay tro nang tat theo mac dinh | source-code | 2026-11 | 2027-05-06 | no |
| [45] DiscoverAssistiveTech khong coi uiautomation.dll la screen reader | source-code | 2026-11 | 2027-05-06 | no |
| [46] Chi phi tro nang theo nut ban, khong ca cay | pattern | 2026-11 | 2028-11-06 | no |
| [47] Co WebView2 con hieu luc | version | 2026-10 | 2026-11-13 | no |
| [48] Chrome 138+ bat UIA ban dia theo mac dinh | version | 2025-08 | 2025-09-14 | yes |
| [49] Line-breakable ruby ship o Chrome 128 | pattern | 2024-05 | 2026-05-31 | yes |
| [50] software_rendering_list.json khong chan gpu_compositing cho Iris/R9 M370X | source-code | 2026-11 | 2027-05-06 | no |
| [51] gpu_switches.cc dinh nghia ignore-gpu-blocklist | source-code | 2026-11 | 2027-05-06 | no |
| [52] Bo dem glyph Skia 2048 muc / 2 MiB, khoa theo strike | source-code | 2026-11 | 2027-05-06 | no |
| [53] Blink 156: ruby 94.057 nut 1,70 ms = chu thuong 14.057 nut 1,90 ms | performance-number | 2026-11 | 2027-05-06 | no |
| [54] Issue 41237496 PlainTextRange moi ky tu, trang thai chua doc | bug-status | 2026-11 | 2026-12-06 | no |
| [55] AdditionalBrowserArguments tu bien moi truong duoc NOI vao | spec | 2026-11 | 2028-11-06 | no |
| [56] wry 0.55.1 luon dat additional_browser_args | library-version | 2026-11 | 2027-05-06 | no |

Mốc sớm nhất phải kiểm lại: **2025-09-14** (2 mục đã cũ hoá).

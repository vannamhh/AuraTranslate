---
baseline_commit: dfa9c95b7488f422d61567857f281fa2bdbec5cd
---

# Story 2.8: Gộp và tách segment tường minh

Status: done

**Covers:** FR78 · AD-5 · AD-37 · AD-46 · AD-47 ④ · UX-DR32 *(vế phím tường minh)*

## Story

As a người dịch,
I want sửa lại chỗ máy tách câu sai,
So that một dấu chấm trong chữ viết tắt không phá cấu trúc cả Chương.

## Acceptance Criteria

*(chép nguyên văn `epics.md:2495-2528`. Tám AC — đừng sửa một chữ để khớp mã đã viết; `project-context.md:456-458`.)*

1. **Given** hai segment liền nhau · **When** người dùng gộp bằng `⌘M` · **Then** cả hai đánh dấu **về hưu** và một segment mới được tạo
2. **Given** một segment · **When** người dùng tách bằng `⌘/` · **Then** segment cũ về hưu và các mảnh mới được tạo
3. **Given** segment mới sinh ra từ gộp hoặc tách · **When** tạo · **Then** bắt đầu ở trạng thái **chưa xác nhận với lịch sử rỗng**
4. **Given** segment đã về hưu · **When** tra · **Then** lịch sử phiên bản của nó vẫn tra lại được
5. **Given** cặp TM đã ghi từ segment cũ · **When** gộp hoặc tách xảy ra · **Then** ở lại nguyên, không bị xoá
6. **Given** gộp một nhóm segment · **When** tính cờ kết đoạn · **Then** cờ theo **câu cuối** của nhóm
7. **Given** tách một segment thành nhiều mảnh · **When** tính cờ kết đoạn · **Then** cờ theo **mảnh cuối**, mọi mảnh trước nhận cờ **tắt**
8. **Given** `⌘M` và `⌘/` · **When** gọi · **Then** là command đã đăng ký, **không phải hệ quả phụ của việc gõ**

---

## Điều kiện khởi hành

🔴 **Cây làm việc BẨN lúc story này được viết, và thứ bẩn KHÔNG phải tạo tác của story này.** Sáu tệp sửa chưa commit là **các bản vá code review của Story 2.7** — `src-tauri/src/commands/segment.rs` + `src-tauri/tests/segment_contract.rs` *(chữ ký thứ mười của Ice: `trim()` hai vế của phép so mốc)*, cộng ba tệp `_bmad-output` và một `.DS_Store`.

⇒ Theo `project-context.md:425-426` *("Cây bẩn TRƯỚC khi bắt đầu một story ⇒ commit riêng, trước, và HỎI Ice trước khi commit. Diff của một story phải đọc được một mình")*: **hỏi Ice, commit riêng, rồi mới chạm dòng mã đầu tiên.** Đừng cuốn chúng vào diff của 2.8.

⚠️ **Số baseline dưới đây đo TRÊN cây đã có các bản vá đó** — nếu Ice chọn không commit chúng, đo lại trước Task 1.

| Đường | Baseline khởi hành | Nguồn |
|---|---|---|
| `cargo test --locked` | **383 passed / 0 failed / 5 ignored** | `2-7-...md:763-766` |
| `segment_contract` riêng | **103 / 103** | ngay trên |
| `npm run test` (vitest) | **133 / 133** *(12 tệp)* | đo lại 2026-08-17 bằng `npx vitest run` |
| 11 cổng npm | 11 xanh *(9 đọc-tệp + `check:scope` + `check:scope:bundled` chạy tay)* | `2-7-...md:645-655` |
| `COMMAND_FLOOR` | sàn **37** · cổng in **44** | `check-commands.mjs:252`; chạy thật `npm run check:commands` |
| e2e | **8 / 8 spec** | `2-7-...md:645-655` |
| Bước di trú kế tiếp | **12** *(dịch ở 11 qua 10 bước `[1,2,3,5,6,7,8,9,10,11]`)* | `PROJECT_MIGRATIONS`, `core/store/schema.rs:849-911` |

🔴 **Đo lại cả bảy dòng từ NGUỒN ở Task 0.3, đừng chép từ đây.** Neo số học của kho này đã sai **ba lần liên tiếp** (2.5c · 2.5d · 2.6) và không cổng nào canh chúng.

⚠️ **Bộ e2e đang chập chờn vì một khuyết tật của BÀN ĐO, không của sản phẩm** — `wdio.conf.mjs::devServerIsUp()` (`:191-198`) chỉ hỏi `res.ok`, nên một Vite hấp hối vẫn trả `200` và cả bộ đỏ oan (`deferred-work.md:3345-3354`). Cộng một biến thứ hai đã đo ở 2.7: `FLUSH_WAIT_MS` không chịu được máy đang biên dịch Rust (`deferred-work.md:3902-3906`). ⇒ **Chạy e2e trên máy rảnh, cổng 1420 và 4445 trống, và đừng `tail` output.**

---

## 🔴 Quyết định mở — Ice chốt TRƯỚC dòng mã đầu tiên

> *"Ice là người chốt các quyết định mở. Gặp một chỗ hai phương án đều hợp lệ: nêu cả hai kèm **số đo**, đừng tự chọn rồi đi tiếp — và cũng đừng loại một phương án chỉ vì nó đắt"* (`project-context.md:464-466`).
>
> Dev agent **dừng ở đây** và trình chín quyết định. Không tự chọn. **Task 0 chặn mọi task khác.**

### 🔴 #1 — Gộp HAI segment hay gộp một NHÓM? Hai AC của cùng story nói hai điều khác nhau

**Số đo.** AC1 viết *"hai segment **liền nhau**"*; AC6 viết *"gộp một **nhóm** segment ⇒ cờ theo câu cuối **của nhóm**"*. Và năng lực chọn nhiều hàng **không tồn tại**: `editorPanelState.ts:57` khai `const caretSegmentId = ref<number | null>(null)` — một số, không mảng, không `Set`; `setEditorCaret(id: number | null)` (`:143`) nhận đúng một id; grep `selectedIds|multiSelect|rowSelection|Set<number>` trên `src/**` = **0 kết quả** *(đo lại 2026-08-17)*.

⚠️ Đây là **lượt lặp lại thứ hai của đúng một câu hỏi**: Quyết định #1 của Story 2.5c hỏi y hệt cho *"một dải câu"* của FR133, và Ice ký đường **(b) một câu**, ghi nợ vế dải câu với chủ ghi thẳng là *"ứng viên tự nhiên là **2.8**"* (`deferred-work.md:3397-3424`). Cùng lý do được ghi lúc đó và vẫn đứng hôm nay: **không tài liệu nào của dự án mô tả CƠ CHẾ chọn** — không Shift+click, không kéo chọn, không Shift+mũi tên ở PRD, epics, EXPERIENCE.md hay DESIGN.md.

⚠️ **Bẫy đọc, ghi ra vì hai khái niệm trùng tên:** `selectionContract.ts` phục vụ **chọn văn bản trong một cột** cho Auto-Lookup. Nó **không phải** cơ chế chọn nhiều **hàng**. Cắm vào đó vừa sai khái niệm vừa làm cổng đỏ — `check-commands.mjs` Kiểm F đếm **tĩnh** số lời gọi `useSelectionSurface(...)` literal, bảng theo tệp khai `GridPanel.vue: 2` và hôm nay đúng 2 (`GridPanel.vue:309-310`).

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Gộp đúng **hai** — câu đang có caret và câu **liền trên** nó. AC6 đọc lại thành ca `n = 2` của cùng bảng | AC6 đóng **một nửa** theo chữ *"nhóm"*; ghi nợ có chủ. Không năng lực UI mới |
| **(b)** | Dựng chọn nhiều hàng ngay trong story này | **Tự thiết kế một tương tác chưa ai đặc tả**, đúng thứ 2.5c đã từ chối một lần. Kéo theo cả bàn phím lẫn chuột, cả hai engine |
| **(c)** | *"Nhóm"* = gọi `⌘M` **nhiều lần liên tiếp** | Sai ngữ nghĩa AD-5 một cách đo được: gộp 3 câu thành **hai** lượt về hưu+tạo mới ⇒ **5 hàng về hưu** thay vì 3, và một segment trung gian có `id` vĩnh viễn không ai từng thấy |

🔴 `paragraph.rs::merged(group: &[ParagraphFlags])` **đã nhận một lát cắt** chứ không nhận hai giá trị (`paragraph.rs:99-101`) — tức tầng thuần đã sẵn sàng cho `n` bất kỳ. Chữ ký này chỉ quyết **bề mặt UI**, không quyết tầng Rust.

### 🔴 #2 — Tách ở đâu? Cột nguyên văn KHÔNG gõ được, nên không có con trỏ để cắt

**Số đo, và đây là điểm chặn thật của story.** Đặc tả nói tách **bắt buộc** làm ở cột nguyên văn: *"không có phép chiếu nào từ vị trí con trỏ bên tiếng Việt sang chỗ cắt bên tiếng Trung. Cùng lý do Trados và memoQ đều bắt tách ở cột nguồn"* (`epics.md:2552`), và `EXPERIENCE.md:267` viết *"Tách câu, thực hiện ở **CỘT NGUYÊN VĂN**"*.

Nhưng đo cây nguồn 2026-08-17: `contenteditable="true"` **viết cứng, chỉ trên ô bản dịch** — `GridPanel.vue:1122`, trong khối `data-col="tgt"`. Cột nguyên văn (`GridPanel.vue:1041-1049`) **không** `contenteditable`, **không** `tabindex`; nó đăng ký làm bề mặt vùng chọn vai `'source'` (`:309`). ⇒ **Không có caret trong cột nguyên văn.** Chưa AC nào của story nêu điều này, và không có nó thì AC2 không có đầu vào.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Lấy chỗ cắt từ **vùng chọn** đang có ở cột nguyên văn — `selectionContract.ts` đã cho một `Range` thật, dùng offset của nó | Tái dùng bề mặt đã có, 0 khái niệm mới. Nhưng nó đổi nghĩa một cử chỉ đang phục vụ Auto-Lookup: bôi đen trong cột nguồn hôm nay **tra từ điển** |
| **(b)** | Cho cột nguyên văn `tabindex="0"` + caret chỉ-đọc *(`contenteditable="false"` vẫn đặt được `Selection`)* | Một bề mặt tiêu điểm **thứ tư** trong lưới ⇒ chạm `focus.ts` và AD-34 §2. Phải đo trên WKWebView: caret trong vùng không sửa được là chỗ hai engine bất đồng |
| **(c)** | Tách tại **ranh giới câu do bộ tách đề xuất** — chạy `split_source_text` trên `source_text` rồi cho người dùng chọn một trong các chỗ cắt ứng viên | Không cần caret nào. Nhưng nó **không giải được ca AC tồn tại để giải**: dấu chấm trong chữ viết tắt là chỗ bộ tách **đã sai**, nên đề xuất của nó cũng sai đúng chỗ đó |
| **(d)** | Một hộp thoại nhập chỉ số ký tự | Bị bác trước khi hỏi — `EXPERIENCE.md:171` khai *"không chặn, không hỏi lại"* cho họ thao tác này. Ghi ra để không ai đi lại |

🔴 **Task 1 (bản đồ WKWebView) CHẶN task cài đặt của #2**: đường (a) và (b) đều là mệnh đề về **engine thật**, và `happy-dom` không phải WebKit (`project-context.md:265-267`). Chữ ký cho #2 **ký SAU phép đo**, đúng khuôn Quyết định #4 của Story 2.5d.

### 🔴 #3 — `target_text` của segment mới: nối bằng gì, và tách thì bản dịch đi đâu

**Số đo.** AC1-AC3 nói về **về hưu, tạo mới, trạng thái, lịch sử** — và **không một chữ** về `target_text`. Nhưng AD-47 ③ liệt kê gộp/tách vào **danh mục đóng các lượt ghi `target_text` không-phải-người-dùng** (`ARCHITECTURE-SPINE.md:712`), tức kiến trúc đã giả định segment mới **có** mang bản dịch.

⚠️ Gộp và tách **không đối xứng**, và đó là gốc của câu hỏi: nối hai `source_text` là xác định; nối hai `target_text` cũng xác định (`epics.md:2551` khai cả hai vế cho Story 2.9). Nhưng **tách** thì `source_text` cắt được tại một điểm còn `target_text` **không có phép chiếu nào** — đúng câu `epics.md:2552` đã viết.

| Đường | Gộp | Tách | Cái giá |
|---|---|---|---|
| **(a)** | nối `source_text` **và** nối `target_text`, phân cách bằng **một khoảng trắng** | mảnh **đầu** giữ toàn bộ `target_text`, các mảnh sau rỗng | Không mất chữ của người dùng. Nhưng "một khoảng trắng" là một lựa chọn của dev cho một chỗ đặc tả im lặng — và với tiếng Trung ở cột nguồn nó **sai**: hai câu Trung nối nhau không có khoảng trắng |
| **(b)** | nối, phân cách theo **ngôn ngữ**: `''` cho `zh`, `' '` cho `en`/`vi` | như (a) | Đúng hơn về chữ. Nhưng dựng một quy tắc theo ngôn ngữ ở đây là dựng một **nguồn sự thật thứ hai** cạnh `split_source_text` — nơi đã có `LANG_CHINESE` (`split.rs:48`) |
| **(c)** | nối cả hai | **mọi** mảnh nhận `target_text` rỗng | Nhất quán và không bịa. Cái giá: một lượt tách **xoá bản dịch** người dùng đã gõ — và AC không cho phép hoàn tác, xem #9 |

🔴 **Bất kể đường nào, AD-47 ① là bắt buộc và không có ngoại lệ ở đây:** lượt ghi này phải đặt **cả hai** — mốc so sánh **và** cột `translation_origin` — trong cùng một thao tác (`project-context.md:505-512`). Vế xuất xứ đã có luật sẵn, xem #4.

### #4 — Xuất xứ: luật ĐÃ CÓ, nhưng cái MỐC thì chưa có chỗ đứng

**Số đo, vế đã đóng.** AD-47 ④ khai bằng chữ, không cần Ice chọn: *"Mọi mảnh mang **cùng một** giá trị ⇒ segment mới giữ giá trị đó. **Bất kỳ bất đồng nào** ⇒ **người khác dịch**. Tách là ca tầm thường của luật này"* (`ARCHITECTURE-SPINE.md:715-717`). ⇒ **Dev thi hành, không hỏi lại.** Hằng đã có: `TRANSLATION_ORIGIN_OTHER = "other"` (`commands/segment.rs:1201`).

⚠️ AD-47 ④ còn ghi sẵn cái mất, chép vào đây để dev không tưởng là một lỗi: *"gộp một câu `''` (chưa dịch) với một câu **tôi dịch** cũng rơi vào nhánh bất đồng"*.

🔴 **Vế CÒN MỞ là cái mốc.** AD-47 ① đòi đặt lại **mốc so sánh** về đúng văn bản vừa ghi. Nhưng mốc **không sống trên đĩa** — Quyết định #2(b) của Story 2.7 (Ice ký 2026-08-16) đặt nó ở webview: mảng `segments` (`editorPanelState.ts:34`) giữ bản lúc nạp, và `confirmCurrentSegmentUnguarded:795` đọc từ đó rồi gửi `textAtLoad` qua dây. Segment mới sinh từ gộp/tách mang một **`id` chưa từng có trong mảng đó**.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Lệnh gộp/tách trả về hàng mới **đầy đủ**; webview chèn nó vào `segments` ⇒ mốc = văn bản vừa ghi, tự động | Đúng khuôn `replaceEditorSegment` đã có (`editorPanelState.ts:168-177`), nhưng hàm đó chỉ **thay** một hàng, không **thêm/bớt** — phải mở rộng |
| **(b)** | Nạp lại cả Chương sau mỗi lượt gộp/tách | Một dòng mã. Nhưng `ensureSegmentsLoaded` khoá bằng cờ `requested` (`:38, :92-116`) nên phải mở khoá đó — và trên 9.850 câu là một lượt nạp lại toàn bộ cho một thao tác sửa **một** chỗ |
| **(c)** | Rust tự giữ bản đồ mốc trong `OpenWork` | Bị bác ở Story 2.7 cho cùng câu hỏi (Quyết định #2), và bác lại vẫn đúng: nó dựng nguồn sự thật thứ hai cạnh mảng của webview |

### 🔴 #5 — `is_omitted` khi gộp một câu ĐÃ CẮT BỎ với một câu chưa — và đây là ứng viên CỬA CHẶN Task 0.4

**Số đo.** AD-5 (`ARCHITECTURE-SPINE.md:103-111`) khai segment mới là *"chưa xác nhận với lịch sử rỗng"* và **không một chữ** về cờ cắt bỏ. AD-47 ④ giải quyết đúng **một** cột — xuất xứ. Cột `is_omitted` (bước di trú 8, Story 2.5c) **không có luật nào** cho ca gộp.

🔴 **Đây đúng khuôn đã sinh ra AD-47.** Story 2.7 gặp một chỗ AD-5 im lặng về **một cột**, và Task 0.4 kích hoạt: Ice chốt giao Winston viết `AD-47` trước dòng mã đầu tiên (`2-7-...md:388-411`). Nếu chữ ký của #5 là một luật mới lấp im lặng của AD-5 theo cùng cách, thì **đó là một `AD` MỚI, không một dòng mã** (`project-context.md:461-463`) ⇒ **dừng story, báo Ice, viết AD trước.**

⚠️ Và cắt bỏ **không phải** một mức độ hoàn thành: `epics.md:2351` khai nó là *"một **trục độc lập** — câu vẫn giữ trạng thái riêng của nó trong bảng sáu giá trị"*, đúng khuôn `translate="no"` của XLIFF. Nên không thể mượn luật của `status`.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | **Bất kỳ** mảnh nào đã cắt ⇒ segment mới đã cắt | Chiều an toàn cho *"đừng đưa vào bản dịch thứ tôi đã bỏ"*. Nhưng nó **nuốt** một câu người dùng đang dịch |
| **(b)** | **Mọi** mảnh đã cắt ⇒ đã cắt; bất đồng ⇒ **không** cắt | Song song với AD-47 ④ về hình dạng *(đồng ý ⇒ giữ)* nhưng **ngược chiều** ở ca bất đồng — 47 ④ chọn chiều bi quan, đây chọn chiều lạc quan. Phải nói ra vì sao hai cột cùng ca lại khác chiều |
| **(c)** | Bất đồng ⇒ **từ chối thao tác**, báo người dùng bỏ cờ trước | 0 dữ liệu bị bịa. Cần một `MessageKey` mới; và nó là lần đầu một thao tác của story này **chặn** — ngược `EXPERIENCE.md:171` |

⚠️ Cùng câu hỏi, cùng hạng, nêu luôn để không phải hỏi hai lần: **`status`** của mảnh trước khi về hưu thì AD-5 đã trả lời (segment mới luôn *chưa xác nhận*), và **`is_target_paragraph_end`** thì AD-46 đã trả lời (ba ca biên áp y nguyên). Chỉ `is_omitted` là hở.

### #6 — Hàng ĐÃ VỀ HƯU có ở lại trong lưới không? Gộp hai câu hôm nay cho ra BA hàng

**Số đo, và không AC nào nêu nó.** `read_open_chapter_segments` (`commands/segment.rs:788-791`) chạy `SELECT ... FROM segment WHERE chapter_id = ?1 ORDER BY ord` — **không** `WHERE retired_at IS NULL`. Và `editorSegments.ts:161` đã cài `if (input.retiredAt !== null) return 'ornament'`, đứng **đầu** thứ tự sáu nhánh, với CSS `.rule-ornament` đã tồn tại (`GridPanel.vue:1327-1329`) mà comment tại chỗ ghi *"0 đường tới được trên dữ liệu thật"*.

⇒ Nối `retired_at` vào mà không làm gì thêm thì gộp hai câu cho **ba hàng** trên màn hình, và số hàng của một Chương **chỉ tăng, vĩnh viễn**.

⚠️ Hai tài liệu kéo ngược nhau: UX-DR19 (`epics.md:555`) liệt kê `ornament` *"mờ đã về hưu"* là một trong **sáu** giá trị vạch — tức hàng về hưu **có** xuất hiện. Còn AD-5 (`:109`) chỉ đòi *"chỗ đánh dấu khi đọc (FR119) trỏ tới segment về hưu thì **ở lại**, không bị xoá im lặng — hiện kèm ghi chú *câu này đã đổi*, vẫn **mở được về đúng vị trí trong Chương***" — một lời hứa về **điều hướng**, không nhất thiết về **hiển thị trong lưới**.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Lọc ở **Rust**: thêm `WHERE retired_at IS NULL` vào truy vấn nạp | Một dòng, đúng AD-1 *(quy tắc nghiệp vụ ở Rust)*. Cái giá: nhánh `ornament` của `resolveSegmentRule` thành **mã chết** — và `check-commands.mjs:2164` Kiểm I đối chiếu **hai chiều** `SEGMENT_RULE_VALUES` ↔ CSS, nên gỡ nó là một lượt sửa **ba chỗ** |
| **(b)** | Giữ hàng về hưu trong lưới, vẽ `ornament` mờ | Kênh thị giác đã dựng sẵn từ 2.5b, 0 dòng CSS mới. Cái giá: lưới phình theo số lần sửa, và `⌥↓` cùng mọi lệnh điều hướng phải học bỏ qua chúng |
| **(c)** | Lọc ở Rust **và** giữ nhánh `ornament` cho bề mặt lịch sử (2.6) đọc | Giữ cả hai. Cái giá: hai đường đọc khác nhau cho cùng một bảng |

🔴 Đường nào cũng phải trả lời **`ord` của hàng về hưu**: xem #7.

### #7 — Đánh lại `ord` thế nào, và có tiêu bước di trú 12 không

**Số đo.** `ord` cấp đúng **một** chỗ — `insert_segments:136`, `ord = index + 1` liên tục. Grep `SET ord|ord =` trên `src/commands/` và `src/core/segment/` = **1 kết quả duy nhất**, chính dòng đó. **Chưa đường mã nào đánh lại `ord`.** Cột **không** `UNIQUE`, và `schema.rs:279-282` ghi đó là chủ ý — để hở tạm trong một giao dịch nhiều bước.

**Vế di trú, đo từ nguồn:** `retired_at TEXT` đã có trong `SEGMENT_DDL` từ bước **5** (`schema.rs:351`), và `deferred-work.md:1961-1968` ghi thẳng lý do: *"Cột `retired_at` đã có sẵn trong `SEGMENT_DDL` để 2.8 **không phải mở một bước di trú thứ hai** chỉ để thêm một cột."* ⇒ **Story này có thể tiêu 0 bước di trú** — lượt đầu tiên của chuỗi 2.5c → 2.5d → 2.6 → 2.7 không tiêu số nào.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Đánh lại `ord` **liên tục 1..N** cho cả Chương trong cùng giao dịch, hàng về hưu nhận `ord` của hàng đầu nhóm | Bất biến đơn giản nhất, đọc là hiểu. Cái giá: một `UPDATE` chạm tới **9.850 hàng** cho một thao tác sửa một chỗ — phải đo, NFR2 là 50 ms/frame |
| **(b)** | Chỉ đánh lại **từ chỗ chạm về sau** | Rẻ hơn ở ca thường (sửa gần đầu Chương thì vẫn đắt). Cùng bất biến |
| **(c)** | Không đánh lại: chèn `ord` **phân số** hoặc để hở khoảng trống | 0 hàng bị chạm ngoài nhóm. Cái giá: `ord` thôi là số nguyên liên tục ⇒ **một bước di trú 12** đổi kiểu cột, và mọi mã giả định `ord` liên tục phải rà lại |

🔴 **Nếu chữ ký đòi một cột mới** *(vd một con trỏ dòng dõi `replaced_by`)* thì đó là bước **12** — và số đó phải đo lại từ `PROJECT_MIGRATIONS`, không chép dòng này. Kèm theo là **ba neo số học không cổng nào canh** (xem §Bẫy đã biết).

### #8 — Hợp âm phím: ba tài liệu nói ba điều, và `⌘M` đã bị một mockup khác chiếm

**Số đo.** Cả hai hợp âm **rảnh hôm nay**: grep `KeyM` trên `src/commands/index.ts` + `keys.ts` = **0**; `Slash` chỉ có trong bảng tra `NAMED_CODES`/`KEY_GLYPHS` (`keys.ts:114, 292`), chưa command nào dùng. Tiền tố `editor.` đã chốt đích danh cho story này (`commands/index.ts:977-980`).

⚠️ **Và luật vùng gõ cho phép chúng đi qua**: `keys.ts:510` chỉ chặn hợp âm **thiếu phím bổ trợ chính** (`lacksPrimaryMod = !meta && !ctrl`, `:415`) khi tiêu điểm ở vùng gõ. Mọi `Mod+…` chạy được ngay khi caret đang trong ô bản dịch — đúng chỗ story này cần.

**Ba tài liệu, ba câu:**

| Nguồn | Gộp | Tách |
|---|---|---|
| `epics.md:2498, 2502` *(AC — nguồn chính thức)* | `⌘M` | `⌘/` |
| `EXPERIENCE.md:169` | `⌘M` | `⌘/` |
| `EXPERIENCE.md:267` *(bảng Phím)* | — *(bảng ghi `Backspace`, tức Story 2.9)* | **`⌘T`** |
| `mockups/settings.html:276-277` | `⌘M` → `editor.segment.merge` | `⌘/` → `editor.segment.split` |

🔴 **`⌘M` đã bị chiếm trong một mockup khác**: `mockups/tm-manage.html:128` dùng `⌘M` mở màn hình Quản lý TM. Tài liệu **chưa gọi tên xung đột này** — trong khi xung đột `⌘⇧T` thì `settings.html:274-275` đã đánh dấu bằng `class="conflict"`. Va chạm này chưa xảy ra *(Quản lý TM là Epic 7)*, nhưng nó xảy ra **im lặng** ở Epic 7 nếu không ai ghi nợ hôm nay.

🔴 **Command id: hai nguồn nói hai tên, và một trong hai ĐÃ CÓ CHỮ KÝ.** `mockups/settings.html:276-277` viết `editor.segment.merge` / `editor.segment.split`. Nhưng `commands/index.ts:977-980` — doc-comment **Ice ký 2026-08-14, Quyết định #4 của Story 2.5** — khai đích danh: *"Story 2.8/2.9/2.10 sẽ dùng tiếp (`editor.merge_segments`, `editor.split_segment`, `editor.next_segment`…)"*.

⇒ Chữ ký cũ nghiêng về `editor.merge_segments` / `editor.split_segment`. Nêu ra vì command id nằm trong bảng keybinding của người dùng (Story 1.21) ⇒ đổi tên về sau là **mồ côi phím tắt người dùng đã gán, im lặng** — bài học Quyết định #5 của Story 2.5b. **Chốt một lần, ở đây**, và nếu chốt theo mã thì `settings.html` là tài liệu phải sửa.

Chữ ký cho #8 phải trả lời **ba** vế: ① hợp âm nào cho tách; ② command id chính xác; ③ ai sửa `EXPERIENCE.md:267` — 2.8 sửa tại chỗ kèm 🔵 và ngày *(khuôn `project-context.md:352-353`)*, hay ghi nợ.

### #9 — Dòng báo hệ quả và `⌘Z`: mockup vẽ cả hai, AC của story này đòi CẢ HAI ĐỀU KHÔNG

**Số đo.** Tám AC của Story 2.8 **không AC nào** nhắc dòng báo hay hoàn tác. Cả hai nằm ở **Story 2.9** (`epics.md:2558-2564`). Nhưng mockup `key-screen-workspace.html:128-129` vẽ chúng cạnh nhau: *"**Đã gộp hai câu.** Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được."* + *"Hoàn tác ⌘Z"*.

🔴 **Không FR, không AD, không UX-DR nào chốt mô hình undo** — grep toàn `prd.md` + `EXPERIENCE.md` + `DESIGN.md`: `⌘Z` chỉ xuất hiện như một **hệ quả UX được giả định**, chưa bao giờ được đặc tả. Câu hỏi chưa ai trả lời: hoàn tác một lượt gộp là **gỡ `retired_at` của hai hàng cũ và xoá hàng mới**, hay là **một lượt tách mới** *(tức lại về hưu + tạo mới, và `id` cũ không bao giờ quay lại — AD-3)*? Hai cách cho hai đĩa khác nhau.

⇒ **Nếu chữ ký chọn dựng undo, đó là một `AD` MỚI** ⇒ Task 0.4 kích hoạt, dừng story.

**Vế dòng báo, đo bề mặt đã có:** `StatusBar.vue` đã dựng (Story 2.3, UX-DR30) và chữ ký của Ice 2026-08-14 khai hợp đồng tối thiểu là *"cột nhãn trạng thái của chính hàng **CỘNG** một dòng ở thanh trạng thái"* (`StatusBar.vue:88-91`). Nhưng khe thông điệp là một `Record` **đóng** cho đúng ba giá trị của `ConfirmResult`, và tệp tự khai *"Không dựng khung mở rộng cho các thông điệp tương lai"* (`:17-19`) ⇒ có khuôn, **không** có chỗ trống để cắm.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | 2.8 không dựng dòng báo, không undo. Ghi nợ, chủ **Story 2.9** | Đúng phạm vi AC. Cái giá: một thao tác **phá huỷ** *(hai câu biến mất khỏi lưới)* chạy **im lặng và không lui được** — đúng lớp lỗi `project-context.md` gọi là *"rỗng im lặng"* |
| **(b)** | 2.8 mở rộng khe thông điệp của `StatusBar` và bắn một dòng báo; **không** undo | Đóng nửa nguy hiểm. `⌘Z` vẫn nợ, có chủ. Chi phí thật: mở `CONFIRM_NOTICE_KEYS` thành một danh mục mở + khoá `vi.json` mới |
| **(c)** | Dựng cả undo | **Một `AD` mới** ⇒ Task 0.4. Ghi ra để không ai đi lại bằng một lượt tiện tay |

---

## Tasks / Subtasks

- [x] **Task 0 — Chốt chín quyết định mở (CHẶN mọi task khác)**
  - [x] 0.1 Trình chín quyết định trên cho Ice, kèm số đo. Không tự chọn. *(Ice ký trọn gói 2026-08-17; cộng một chữ ký THỨ MƯỜI lật #6 sang (c) sau khi đo thêm.)*
  - [x] 0.2 Ghi chữ ký + ngày vào §Dev Agent Record.
  - [x] 0.3 Đo lại từ NGUỒN, không chép story này: bảy dòng của bảng §Điều kiện khởi hành. *(Sáu dòng đo được đều khớp; e2e hoãn tới Task 6.3.)*
  - [x] 0.4 🔴 **CỬA CHẶN** — chữ ký nào đòi thêm luật vào AD-5 (#5 `is_omitted`) hoặc dựng một mô hình undo (#9c) thì **dừng story**, báo Ice, `AD` viết trước dòng mã đầu tiên. Khuôn đã chạy một lần ở 2.7 (`2-7-...md:388-411`). *(Đã nêu #5(a) là ứng viên; Ice phán định KHÔNG kích hoạt. #9 ký (a) nên vế undo không phát sinh.)*
  - [x] 0.5 ⚠️ Đọc **nội dung** mọi dòng `grep` khớp, đừng đếm — 2.6 sập bẫy này khi một doc-comment chứa nguyên văn câu lệnh grep. *(Bắt được một tiền đề sai SỐ trong chính story — xem §Task 0.5.)*
  - [x] 0.6 Hỏi Ice về cây bẩn (§Điều kiện khởi hành) và commit riêng nếu Ice đồng ý. *(Không còn việc — cây sạch, các bản vá 2.7 đã ở `dfa9c95`.)*
- [x] **Task 1 — Bản đồ WKWebView (CHẶN Task 4; chữ ký #2 ký SAU phép đo)**
  - [x] 1.1 Cột nguyên văn: `Selection` đặt được ở đó không, trên **cả hai** engine? Ghi `Range`, `anchorOffset`, `activeElement`. *(Bằng CHUỘT: không — `"None"`/0 ở cả ba biến thể. Bằng SCRIPT: có — `Caret`/`Range`, offset đúng. **Một** engine, không hai: lý do ghi ở `2-8-ban-do/README.md` §Giới hạn ①.)*
  - [⊘] 1.2 Nếu (b): `tabindex="0"` trên cột nguồn có cướp tiêu điểm của ô bản dịch không (AD-34 §2, `focus.ts`)? *(**KHÔNG CHẠY** — điều kiện `nếu (b)` không xảy ra; Ice ký (e). Và phép đo còn bác luôn tiền đề của (b): `document.hasFocus()` là `false` ở mọi bước, kể cả bước ô bản dịch nhận caret ⇒ tiêu điểm không phải biến phân biệt.)*
  - [x] 1.3 Vùng chọn cột nguồn hôm nay **đang phục vụ Auto-Lookup** — đo xem một lượt bôi đen để tách có bắn một lượt tra từ điển không. *(**0** lượt `lookup_dictionary` — nhưng vì không vùng chọn nào được tạo ra, không phải vì đường tra bị chặn. Câu hỏi thật lớn hơn story: §Debug Log Ⓑ.)*
  - [x] 1.4 🔴 **LUẬT DỪNG**: ba vòng chẩn đoán bị phép đo bác ⇒ **dừng**, báo Ice. Đừng đi vòng thứ tư. *(**Không kích hoạt** — 0 vòng có một giả thuyết về sản phẩm bị bác; lý do phân biệt ghi ở §Debug Log Ⓐ. Nhưng tôi **vẫn dừng và báo Ice** vì phép đo lật một chữ ký.)*
  - [x] 1.5 Ghi mọi số vào §Debug Log kèm ngày + engine + phiên bản.
- [x] **Task 2 — Neo số học (làm TRƯỚC mã, kể cả khi #7 chốt 0 bước di trú)** *(AC: nền)*
  - [⊘] 2.1 Nếu có bước 12: nâng fixture "tương lai" `STEP_TWELVE` → `STEP_THIRTEEN` (`segment_contract.rs:1562-1578`) — **đổi tên + mảng +1 + số giả +1**. *(Điều kiện `nếu` không xảy ra — xem 2.3.)*
  - [⊘] 2.2 Nếu có bước 12: `vec![1,2,3,5,6,7,8,9,10,11]` (`segment_contract.rs:511`) và `pinned_contract.rs:174-185` (`len()` 10→11, `schema_version()` 11→12). *(Như trên.)*
  - [x] 2.3 Nếu **0 bước di trú**: ghi ra bằng chữ trong §Completion Notes rằng ba neo **cố ý không đổi**, kèm lý do — im lặng ở đây đọc giống một lượt quên.
- [x] **Task 3 — Tầng Rust: hàm thuần gộp/tách** *(AC: 1, 2, 3, 5, 6, 7)*
  - [x] 3.1 🔴 **Gọi `core::segment::paragraph`, đừng viết lại luật cờ** — `merged()`, `split_into()`, `at_end_of_chapter()` (`paragraph.rs:99, 111, 81`) dựng sẵn ở Story 2.5d **cho story này**.
  - [x] 3.2 🔴 **Hai cờ chạy ĐỘC LẬP.** *(Đọc riêng từng cột ở `load_segment_for_write`; ca hợp đồng dùng một cặp cờ **lệch nhau** nên một lượt chép cờ nguồn sang cờ đích là đỏ.)*
  - [x] 3.3 Ghi `retired_at` — **lượt đầu tiên của dự án**. Dùng cùng khuôn thời điểm: `strftime('%Y-%m-%dT%H:%M:%fZ','now')` trong SQL.
  - [x] 3.4 🔴 **KHÔNG tạo `segment_version`** cho lượt về hưu — AD-31. *(Ca riêng: `neither_merge_nor_split_ever_writes_a_segment_version_row`; đột biến ② chèn một lượt chụp ⇒ đỏ.)*
  - [x] 3.5 🔴 **KHÔNG chạm cặp TM** (AC5) — đóng bằng **cấu trúc**, ghi nợ có chủ Epic 7.
  - [x] 3.6 Áp AD-47 ④ cho `translation_origin`; đặt **cả hai** vế của AD-47 ① (mốc + xuất xứ) trong cùng thao tác. *(Vế mốc đi qua chữ ký #4(a) — lệnh trả hàng đầy đủ, webview chèn vào `segments`.)*
  - [x] 3.7 ⚠️ **`insert_segments` KHÔNG tái dùng được nguyên trạng** — đã **viết đường chèn thứ hai** (`write_regroup`), không mở rộng hàm cũ. Lý do ở §Completion Notes.
  - [x] 3.8 Đánh lại `ord` theo chữ ký #7, trong **một** giao dịch.
- [x] **Task 4 — Vỏ `wire` + dây IPC** *(AC: 1, 2)*
  - [x] 4.1 Khuôn hai lớp: hàm thuần nhận `Option<&Store>`, vỏ `#[tauri::command]` mỏng trong `mod wire` lấy `State` qua **`try_state`**.
  - [x] 4.2 Đăng ký lệnh mới ở `src-tauri/src/lib.rs`.
  - [x] 4.3 Từ chối segment đã về hưu ở đường **ghi**: dùng `segment_retired()` đã có, đừng dựng khoá thứ hai.
  - [x] 4.4 Adapter TS ở `src/config/segment.ts` — hình dạng ba trạng thái, **không bao giờ ném**.
  - [x] 4.5 ⚠️ **Kiểm payload lúc chạy hay tin payload?** — **TIN payload**, theo số đông (8/9 adapter) và theo chữ ký #7 của Story 2.7. Nói ra ở doc-comment của `mergeSegments`, kèm cái giá.
- [x] **Task 5 — Bề mặt lưới** *(AC: 8; và #1, #2, #6, #9 quyết phạm vi)*
  - [x] 5.1 Hai command đăng ký ở `src/commands/index.ts`, tiền tố `editor.`, id theo chữ ký #8.
  - [x] 5.2 🔴 `@click` phải là **đúng một** `dispatch('<id>')`. *(Không thêm `@click` nào — hai lệnh chỉ có đường phím; `@mouseup` của cột nguồn là một lượt **ghi nhận vị trí**, không một thao tác.)*
  - [x] 5.3 Đo lại `COMMAND_FLOOR` từ **số cổng in ra**, không chép — cổng in **46**, sàn 37 → **38** (82,6 %).
  - [x] 5.4 Khoá `vi.json` mới: nhãn lệnh. *(Bốn khoá: hai nhãn lệnh + hai khoá lỗi. #9(a) ⇒ không dòng báo.)*
  - [x] 5.5 ⚠️ **Đừng nhắc tên thẻ trong COMMENT của template `.vue`.** *(`check:i18n` xanh.)*
  - [⊘] 5.6 Nếu #6 chốt lọc hàng về hưu: gỡ nhánh `ornament` là **ba chỗ**. *(#6 ký **(b)** — KHÔNG lọc ⇒ không gỡ gì. Nhánh `ornament` nay có đường tới **thật** lần đầu.)*
- [x] **Task 6 — Nghiệm thu**
  - [x] 6.1 11 cổng npm *(9 đọc-tệp; `check:scope` + `check:scope:bundled` chạy tay)*.
  - [x] 6.2 `npm run build` + `vue-tsc --noEmit` · `npm run test` · `cargo test --locked`. Ghi số **trước/sau**.
  - [x] 6.3 e2e **trọn bộ**, trên máy rảnh, **giữ trọn output**.
  - [x] 6.4 Mỗi ca test mới đo **đỏ-rồi-xanh**: đột biến **mã sản phẩm**, không đột biến test.
  - [x] 6.5 Ghi mọi vế không nghiệm thu được vào `deferred-work.md` **kèm chủ**. Không tự chấm đạt.

---

## Dev Notes

### Đọc trước khi viết dòng đầu tiên

`project-context.md` §*Dữ liệu người dùng — chỗ hỏng là VĨNH VIỄN* (`:496-517`) · AD-5 (`ARCHITECTURE-SPINE.md:103-111`) · AD-47 toàn văn (`:675-744`) · AD-37 bảng ba ca (`:437-453`) · AD-46 (`:652-673`) · `core/segment/paragraph.rs` **toàn tệp**.

### 🔴 Luật lõi, chép nguyên văn vì nó định nghĩa cả story

> **Gộp/tách SEGMENT = về hưu + tạo mới** (trạng thái mới là *chưa xác nhận*, lịch sử rỗng). **Gộp/tách CHƯƠNG thì KHÔNG** — chỉ đổi `chapter_id` và `ord`, giữ nguyên `segment.id` và mọi dữ liệu gắn theo nó. Nhầm hai cái này phá sạch lịch sử của những Chương đã dịch xong.
> — `project-context.md:501-504`

⚠️ **AD-32 là cái bẫy song sinh** (`ARCHITECTURE-SPINE.md:394-398`): nó là luật cho gộp/tách **Chương** (FR15) và nói **ngược lại** — giữ nguyên `segment.id`. Đọc nhầm AD-32 thành luật của story này thì mọi AC hỏng cùng lúc mà mã vẫn biên dịch.

### Thứ ĐÃ DỰNG SẴN — đừng phát minh lại

| Đã có | Ở đâu | Ai dựng |
|---|---|---|
| Ba ca biên cờ kết đoạn, hàm thuần | `core/segment/paragraph.rs:81, 99, 111` | Story 2.5d, Quyết định #6(b) — **dựng cho 2.8** |
| Cột `retired_at` | `SEGMENT_DDL`, bước 5, `schema.rs:351` | Story 2.1 — **đặt sớm để 2.8 khỏi mở bước di trú** |
| Vạch `ornament` cho hàng về hưu | `editorSegments.ts:161` + `GridPanel.vue:1327-1329` | Story 2.5b — *"0 đường tới được trên dữ liệu thật"* |
| `retired_at` trên dây | `config/segment.ts:75` — *"`null` cho mọi segment hôm nay (Story 2.8)"* | Story 2.5 |
| Hàng rào từ chối segment về hưu | `segment_retired()`, `commands/segment.rs:1276-1283`; 4 lệnh ghi dùng | Story 2.5 → 2.7 |
| Danh mục xuất xứ đóng | `TRANSLATION_ORIGIN_*`, `commands/segment.rs:1201` | Story 2.7 |
| `SegmentRow` là **tuple struct** | `segment_contract.rs` | Story 2.7 — trần 12 phần tử của `std` đã gỡ, cột thứ 13 thêm được |

### Đường ĐỌC và đường GHI phải từ chối KHÁC NHAU

`read_segment_history` **không** hỏi `retired_at` (`commands/segment.rs:424-425`) — cố ý, đó là AC4. Bốn lệnh **ghi** thì từ chối. Story này thêm lệnh ghi ⇒ theo vế từ chối; và nó là lượt đầu tiên **tạo ra** trạng thái mà bốn hàng rào kia đang chờ.

### Lược đồ hôm nay

| `to_version` | hằng | story |
|---|---|---|
| 1 · 2 · 3 | `SCHEMA_MIGRATION_LOG_DDL` · `WORK_DDL` · `CHAPTER_DDL` | 1.15 |
| **5** | `SEGMENT_DDL` *(đã có `retired_at`)* | 2.1 |
| 6 · 7 | `SEGMENT_TARGET_TEXT_DDL` · `SEGMENT_STATUS_AND_VERSION_DDL` | 2.2 · 2.5 |
| 8 · 9 | `SEGMENT_OMITTED_DDL` · `SEGMENT_TARGET_PARAGRAPH_END_DDL` | 2.5c · 2.5d |
| 10 · 11 | `SEGMENT_VERSION_INDEX_DDL` · `SEGMENT_TRANSLATION_ORIGIN_DDL` | 2.6 · 2.7 |

Số **4 đã cháy**, vĩnh viễn không tái dùng (`segment_contract.rs:473-486` canh). Nguồn sự thật là `PROJECT_MIGRATIONS` (`schema.rs`), **không phải bảng này**.

### Bài học từ bốn story trước — đọc, đừng thi hành story này như mệnh lệnh

1. **Chữ ký thi hành đúng MỘT NỬA** — khuôn lặp **bốn lần** (2.5b ×3, 2.6 ×1) và một lần nữa ở 2.7 (cổng AC8 mù với ba cột). Nửa khó, có chú thích 🔵 đẹp thì làm; nửa là **một dòng chuỗi** hoặc **một câu phải xoá** thì rơi, và **không cổng nào canh nửa đó**.
2. **Story có thể nói SAI một điều kiện** — 2.5d ghi ra một điều kiện sai của chính nó và phát hiện là một khuyết tật thật. **Đọc mã mà xác nhận từng tiền đề ở trên**, đừng tin.
3. **Đọc nội dung dòng `grep` khớp, đừng đếm** — 2.6 sập bẫy này: `grep merge_segment` trả 1 kết quả, và dòng khớp là một doc-comment viết nguyên văn *"grep … cho 0"* (`paragraph.rs:10`).
4. **Neo số học không cổng nào canh** đã sai **ba lần liên tiếp** (2.5c · 2.5d · 2.6).
5. **e2e là lưới DUY NHẤT cho hình dạng dây** — lặp lại nguyên vẹn hai lần: cột `status` (2.5) và tham số `textAtLoad` (2.7). Cả 133 ca vitest đều mù, vì fixture chép tay luôn có sẵn trường.
6. **Lập luận cấu trúc không phải phép đo.** Ghi suy luận dưới nhãn suy luận.

### Bẫy đã biết

- **(a) Ba neo số học** — chỉ áp nếu #7 sinh bước 12: `vec![…]` (`segment_contract.rs:511`) · fixture "tương lai" `STEP_TWELVE` (`:1562-1578`, một neo **lúc biên dịch** — `E0080`, không một ca đỏ) · `pinned_contract.rs:174-185`.
- **(b) `insert_segments` phải set cột mới TƯỜNG MINH** — `DEFAULT` không với tới Chương nhập **sau** lượt di trú. Bài học 2.5d, lặp lại ở 2.7 cho cột thứ hai.
- **(c) `check-i18n.mjs` Kiểm A báo FAIL sai chỗ** với tên thẻ trong comment template `.vue` (chưa vá).
- **(d) Bộ e2e chập chờn vì bàn đo** — `devServerIsUp()` tin một Vite hấp hối; `FLUSH_WAIT_MS` thua một máy đang biên dịch. **Đừng nới hằng số cho hết đỏ** (`project-context.md:368-370`).
- **(e) `⌘M` sẽ va Quản lý TM ở Epic 7** — chưa ai ghi nợ. Ghi hôm nay.
- **(f) `EXPERIENCE.md:171` mang bản UX-DR32 CŨ** *("gõ đè lên đúng vị trí ranh giới")* — tiền đề đã bị Sprint Change Proposal 2026-08-14 bác và `epics.md` đã sửa. Sửa tại chỗ kèm 🔵 + ngày, hoặc ghi nợ.

### Project Structure Notes

Điểm nóng, khớp ba story gần nhất: `src-tauri/src/core/store/schema.rs` · `src-tauri/src/commands/segment.rs` · `src-tauri/src/core/segment/paragraph.rs` · `src-tauri/tests/{segment_contract,pinned_contract}.rs` · `src/config/segment.ts` · `src/panels/{editorPanelState.ts,editorSegments.ts,GridPanel.vue}` · `src/commands/index.ts` · `src/i18n/vi.json` · `scripts/check-commands.mjs`.

Quy ước ràng buộc: Rust `snake_case`, module theo **khái niệm miền** · command id khoá chấm cùng văn phạm với khoá i18n · chuỗi literal trong `src-tauri/src/**` viết **không dấu** *(comment tiếng Việt có dấu thì được)* · ISO-8601 UTC trong database, định dạng hiển thị **chỉ** ở frontend · `File List` kê từ `git status --porcelain`, kê thừa là một món nợ.

---

## Testing

| Mệnh đề | Đường đúng | Vì sao không đường khác |
|---|---|---|
| Bảng ba ca cờ kết đoạn, hai cờ độc lập | `cargo test` — `segment_contract.rs` | Hàm thuần, không cần webview. Bốn ca hợp đồng đã có sẵn từ 2.5d |
| Về hưu + tạo mới, lịch sử rỗng, `retired_at` khác NULL | `cargo test` | Hợp đồng lược đồ. Dựng bằng SQL trực tiếp **và** qua lệnh thật — AC4 đòi một segment về hưu **THẬT**, không một hàng dựng tay (`deferred-work.md:3675-3683`) |
| AD-47 ④: đồng ý ⇒ giữ · bất đồng ⇒ `other` | `cargo test` | Luật nghiệp vụ, AD-1 đặt nó ở Rust |
| Hình dạng dây của lệnh mới | **e2e** | Lưới duy nhất — hai tiền lệ (`status` 2.5, `textAtLoad` 2.7) đều lọt qua toàn bộ test Rust + vitest |
| Command đăng ký, không phải hệ quả phụ của gõ (AC8) | `check:commands` | Mệnh đề khai báo trên toàn cây |
| Vạch `ornament` hiện đúng cho hàng về hưu | `vitest` *(phân giải)* + **bàn đo tay** *(thị giác)* | `happy-dom` không phải WebKit — hình học thuộc bàn đo |
| Caret trong cột nguyên văn | **e2e / bàn đo** | Engine thật; Task 1 |
| Cặp TM ở lại nguyên (AC5) | **cấu trúc** + nợ có chủ | Bảng TM chưa tồn tại ⇒ không đường sản phẩm nào đối chứng được. Nghiệm thu lại ở Epic 7 |

---

## Nợ dự kiến

*(ghi vào `deferred-work.md` kèm chủ; không mục nào mồ côi. Đóng bằng cách **nối tiếp**, không xoá.)*

| Món | Hình dạng | Chủ |
|---|---|---|
| Chọn nhiều hàng — AC6 vế *"nhóm"* nếu #1 ký (a) | 🟡 nửa | một story sau của Epic 2 |
| Dòng báo hệ quả nếu #9 ký (a) | 🟡 nửa | Story 2.9 |
| `⌘Z` hoàn tác gộp/tách — chưa FR/AD/UX-DR nào chốt mô hình | 🔴 hở thật | Ice *(một `AD` mới)* |
| `⌘M` va Quản lý TM (`tm-manage.html:128`) | ⚠️ bẫy tương lai | Epic 7 |
| `EXPERIENCE.md:267` viết `⌘T`, `:171` mang UX-DR32 cũ | ⚠️ tài liệu lệch | 2.8 sửa tại chỗ, hoặc Ice |
| AC5 cặp TM — chưa bảng nào để đối chứng | 🟡 nửa | Epic 7 |
| `segment_version` không mang xuất xứ ⇒ khôi phục không trả xuất xứ về | 🟡 nửa *(đã có chủ từ 2.7)* | story nào cho `segment_version` một cột xuất xứ |
| `is_target_paragraph_end` không nằm trong `segment_version` ⇒ khôi phục không trả cờ về | 🟡 nửa *(đã có chủ từ 2.6)* | như trên |
| Bộ e2e chập chờn (`devServerIsUp`, `FLUSH_WAIT_MS`) | 🔴 hở thật *(đã có chủ)* | một story hạ tầng e2e |

---

## References

- `epics.md:2487-2528` — tám AC · `:2531-2556` Story 2.9 *(ranh giới phạm vi)* · `:2552` tách bắt buộc ở cột nguồn · `:555` UX-DR19 sáu giá trị
- `prd.md:437` — FR78 nguyên văn
- `ARCHITECTURE-SPINE.md:103-111` AD-5 · `:368-392` AD-31 · `:394-398` AD-32 *(bẫy song sinh)* · `:437-453` AD-37 · `:652-673` AD-46 · `:675-744` AD-47, đặc biệt `:694-699` ① và `:715-717` ④ · `:750-757` Consistency Conventions
- `project-context.md:496-517` dữ liệu người dùng · `:456-466` story và spec · `:140-149` khuôn hai lớp IPC · `:255-267` bốn đường nghiệm thu · `:425-426` cây bẩn
- `core/segment/paragraph.rs` toàn tệp · `core/store/schema.rs:344-355, 468-475, 849-911` · `commands/segment.rs:96-148, 788-791, 1201, 1276-1283`
- `config/segment.ts:66-120, 671-711` · `editorPanelState.ts:34, 57, 92-116, 168-177, 420-425` · `editorSegments.ts:161-175` · `GridPanel.vue:309-310, 1041-1049, 1122, 1327-1329` · `commands/index.ts:977-980` · `keys.ts:415, 502-528`
- `check-commands.mjs:252` COMMAND_FLOOR · `:2140-2184` Kiểm I
- `EXPERIENCE.md:105-114` bảng sáu giá trị · `:169` `⌘M`/`⌘/` · `:171` UX-DR32 cũ · `:259-268` bảng Phím · `DESIGN.md:183, 186, 193, 201, 391` token và vạch lề
- `mockups/settings.html:276-277` command id · `mockups/key-screen-workspace.html:128-129` dòng báo + `⌘Z` · `mockups/tm-manage.html:128` xung đột `⌘M`
- `deferred-work.md:1961-1968 · 2042-2058 · 3397-3424 · 3535-3555 · 3675-3683 · 3798-3806 · 3832-3841` — bảy món có chủ 2.8
- `2-7-...md` §Dev Agent Record *(khuôn Task 0.4 đã kích hoạt một lần)*

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, skill `bmad-dev-story`) — 2026-08-17.

### Baseline đo trước khi chạm dòng đầu tiên

**Đo 2026-08-17 trên HEAD `dfa9c95`, TRƯỚC khi chạm dòng mã đầu tiên.** Bảy dòng của §Điều kiện khởi hành đo lại **từ nguồn**, không chép từ bảng đó (Task 0.3).

| Đường | Story ghi | Đo được | Khớp |
|---|---|---|---|
| `cargo test --locked` | 383 / 0 / 5 | **383 passed / 0 failed / 5 ignored** | ✅ |
| `segment_contract` riêng | 103 / 103 | **103 / 103** *(`cargo test --test segment_contract`)* | ✅ |
| `npm run test` (vitest) | 133 / 133 · 12 tệp | **133 / 133 · 12 tệp** *(`npx vitest run`, 3,74 s)* | ✅ |
| 9 cổng đọc-tệp | xanh | **9 / 9 xanh** *(deps · tokens · i18n · commands · layout · dict · dict-manifest · gates · lint)* | ✅ |
| `COMMAND_FLOOR` | sàn 37 · cổng in 44 | **sàn 37** (`check-commands.mjs:252`) · **cổng in 44** | ✅ |
| Bước di trú kế tiếp | 12 | **12** — `PROJECT_MIGRATIONS` (`schema.rs:849-911`) dịch ở `to_version: 11`, mười bước `[1,2,3,5,6,7,8,9,10,11]` | ✅ |
| e2e | 8 / 8 spec | **chưa chạy** — cần máy rảnh + cổng 1420/4445 trống, chạy ở Task 6.3 | ⬜ |

⚠️ **Hai cổng chưa đo:** `check:scope` + `check:scope:bundled` dựng cửa sổ Tauri thật và cần cổng 1420 trống ⇒ chạy tay ở Task 6.1, không ở baseline.

🔵 **Cây làm việc SẠCH lúc khởi hành — §Điều kiện khởi hành đã hết đúng.** Story viết *"sáu tệp sửa chưa commit là các bản vá code review của Story 2.7"*; đo `git status --porcelain` ngày 2026-08-17: thứ duy nhất chưa theo dõi là **chính tệp story này**. Các bản vá 2.7 đã vào commit `dfa9c95` *("finalize story 2.7")*. ⇒ Task 0.6 không còn việc để làm, và số baseline ở trên đo đúng trên cây đã có các bản vá đó — đúng như ⚠️ của story dặn.

### Task 0.5 — Đọc NỘI DUNG dòng `grep` khớp, đừng đếm

Bốn tiền đề đo lại từ nguồn ngày 2026-08-17. **Ba đúng nguyên, một sai SỐ mà đúng KẾT LUẬN:**

- 🔵 **#1 — story ghi *"grep `selectedIds|multiSelect|rowSelection|Set<number>` trên `src/**` = 0 kết quả"*. Đo được: 2 kết quả.** Đọc nội dung cả hai: `wordBoundary.ts:111` (`ReadonlySet<number>` là kiểu TRẢ VỀ của `wordStartOffsets`) và `:112` (`new Set<number>()` gom offset đầu từ). Cả hai là **offset ký tự trong một chuỗi**, không phải chọn hàng. ⇒ Mệnh đề *"năng lực chọn nhiều hàng không tồn tại"* **vẫn đứng**; con số 0 thì sai. Đúng bài học #3 của chính story — và lần này bẫy nằm trong story chứ không trong mã.
- ✅ **#2** — `contenteditable="true"` viết cứng đúng một chỗ: `GridPanel.vue:1122`, trong khối `data-col="tgt"`. Cột nguyên văn (`:1041-1049`) không `contenteditable`, không `tabindex`.
- ✅ **#6** — `read_open_chapter_segments` (`commands/segment.rs:788-791`): `SELECT … FROM segment WHERE chapter_id = ?1 ORDER BY ord` — **không** `WHERE retired_at IS NULL`.
- ✅ **#7** — `ord` cấp đúng một chỗ: `commands/segment.rs:136` (`ord = index + 1`). Không đường mã nào đánh lại.
- ✅ **#8** — `KeyM` trên `src/commands/**` = **0**; `Slash` chỉ ở bảng tra `keys.ts:114, 292`. Cả hai hợp âm rảnh.

### Chữ ký của Ice cho chín quyết định mở

**Ice ký trọn gói chín quyết định ngày 2026-08-17**, trước dòng mã sản phẩm đầu tiên.

| # | Đường ký | Hệ quả thi hành |
|---|---|---|
| **#1** | **(a)** gộp đúng **hai** — câu đang có caret + câu liền trên | AC6 đóng **một nửa** theo chữ *"nhóm"* ⇒ ghi nợ có chủ. Tầng thuần `merged(&[…])` vẫn nhận `n` bất kỳ nên không khoá tương lai. Lặp lại chữ ký #1(b) của 2.5c |
| **#2** | **(a)** → 🔴 **BỊ PHÉP ĐO BÁC**, Ice ký lại **(e)**: sản phẩm **TỰ đặt** caret ở cột nguồn từ toạ độ cú bấm | Đường (e) **không có trong bảng bốn đường của story** — dev đề xuất từ số đo, Ice ký. Xem §Debug Log Ⓐ |
| **#3** | **(b)** nối theo **ngôn ngữ**: `''` cho `zh`, `' '` cho `en`/`vi`; tách ⇒ mảnh **đầu** giữ toàn bộ `target_text` | Phải nói ra vì sao **không** dựng nguồn sự thật thứ hai cạnh `LANG_CHINESE` (`split.rs:48`) — dùng lại chính hằng đó |
| **#4** | **(a)** lệnh trả về **hàng mới đầy đủ**; webview chèn vào `segments` | Mốc AD-47 ①(a) tự động đúng. Phải **mở rộng** `replaceEditorSegment` (hôm nay chỉ THAY, không THÊM/BỚT) |
| **#5** | **(a)** **bất kỳ** mảnh nào đã cắt ⇒ segment mới đã cắt | 🔴 Xem §Cửa chặn Task 0.4 ngay dưới |
| **#6** | **(a)** → **(c)** → **(b)** → 🔴 **LẬT LẦN CUỐI về LỌC**, bằng một lượt Ice **dùng thật** | Hàng về hưu **biến khỏi lưới**; vẫn nằm nguyên trên đĩa. Xem §Lượt lật thứ ba |
| **#7** | **(a)** đánh lại `ord` **liên tục 1..N** cả Chương trong một giao dịch | ⇒ **0 bước di trú**. Ba neo số học **cố ý không đổi** (Task 2.3). Phải **đo** chi phí `UPDATE` — NFR2 là 50 ms/frame |
| **#8** | `⌘M` / `⌘/` · id **`editor.merge_segments`** · **`editor.split_segment`** | Theo AC (nguồn chính thức) + chữ ký Ice 2026-08-14 (`commands/index.ts:977-980`). ⇒ `mockups/settings.html:276-277` là tài liệu phải sửa |
| **#8③** | **2.8 sửa tại chỗ**, kèm 🔵 + ngày | `EXPERIENCE.md:267` (`⌘T` → `⌘/`) và `:171` (bản UX-DR32 cũ) |
| **#9** | **(a)** không dòng báo, không `⌘Z` | Ghi nợ cả hai, chủ **Story 2.9**. 🔴 Và ghi thẳng cái mất: một thao tác **phá huỷ** chạy im lặng, không lui được |

#### 🔴 Cửa chặn Task 0.4 — nêu, và Ice phán định KHÔNG kích hoạt

Tôi trình chữ ký **#5(a)** như một **ứng viên cửa chặn**: nó lấp im lặng của AD-5 về một cột dữ liệu người dùng bằng một luật ràng buộc mọi story sau — **đúng hình dạng đã sinh ra AD-47** ở Story 2.7 (AD-5 im lặng về xuất xứ ⇒ Ice chốt giao Winston viết AD trước dòng mã đầu tiên).

**Ice phán định 2026-08-17: `#5(a)` nằm trong biên độ AD-5 đã có, KHÔNG phải một bất biến mới.** ⇒ Task 0.4 **không** kích hoạt, story đi tiếp. Tôi cài luật trong mã + một ca hợp đồng, và **ghi nợ có chủ** về việc luật này chưa có chỗ đứng trong spine — vì nó vẫn là một mệnh đề mà story sau sẽ đọc, và hôm nay nguồn duy nhất phát biểu nó là một ca test.

#### 🔴 Hệ quả chưa được giá ở #6(a) — nêu ra thay vì im lặng thi hành

Bảng đường của #6 định giá (a) là *"nhánh `ornament` thành mã chết ⇒ sửa ba chỗ"*. Đọc lại nguồn thì cái giá **rộng hơn một bậc**: `ornament` *"mờ đã về hưu"* là **một trong sáu giá trị vạch của UX-DR19** (`epics.md:555`). Gỡ nó khỏi mã là làm mã **lệch một UX-DR đang đứng** — mà `project-context.md:456-458` cấm sửa spec cho khớp mã.

⇒ Đây là **thông tin mới, không có trong bảng đường lúc Ice ký**, nên tôi trình lại một câu hỏi hẹp thay vì tự chọn.

✅ **Ice lật #6 từ (a) sang (c) ngày 2026-08-17** — lọc ở Rust **và** giữ nhánh `ornament` cho bề mặt lịch sử của Story 2.6 đọc. Lý do đường (c) thắng khi có thêm số: nhánh `ornament` thôi là mã chết **trong chính story này** — AC4 đòi tra lại được lịch sử của một segment **đã về hưu**, tức bề mặt lịch sử là đường tới thật đầu tiên kể từ khi 2.5b dựng vạch đó với ghi chú *"0 đường tới được trên dữ liệu thật"*. Cái giá đã biết và nhận: **hai đường đọc khác nhau cho cùng một bảng** — lưới lọc, lịch sử không.

⚠️ Đây là **lần thứ hai trong story này một chữ ký bị chính phép đọc nguồn lật** *(lần đầu: tiền đề `grep … = 0` của #1)*. Ghi ra vì nó là một mẫu, không một sự cố: bảng đường của một quyết định được viết **trước** khi dev đọc hết nguồn, nên cái giá trong bảng là **cận dưới**, không phải giá cuối.

#### 🔴 Lượt lật thứ ba của #6 — và lần này không một phép đọc nguồn nào thay được nó

**Ice dùng thật ngày 2026-08-17, sau khi `⌘/` chạy được:** *"đã tách ra 2 câu, nhưng câu cũ vẫn tồn tại và số thứ tự vẫn chiếm, gây rối nội dung. Tôi muốn khi tách ra thì xoá/ẩn câu cũ đi, và số thứ tự không tính câu cũ đó nữa."*

⇒ **Chốt cuối: LỌC.** `read_open_chapter_segments` thêm `AND retired_at IS NULL`; `applyRegroup` **gỡ** hàng về hưu khỏi ảnh chụp thay vì vá chúng vào.

🔴 **Điều đáng ghi nhất không phải quyết định, mà là cái giá đã được viết ra TRƯỚC KHI KÝ và vẫn không đủ.** Bảng đường của #6(b) có đúng câu này: *"lưới phình theo số lần sửa, và `⌥↓` cùng mọi lệnh điều hướng phải học bỏ qua chúng"*. Nó **đúng từng chữ** — nhưng nó không đọc được thành *"gây rối nội dung"* cho tới khi có một người thật nhìn vào một Chương thật. Ba lý lẽ đứng sau #6(b) *(nhánh `ornament` chỉ có một nơi gọi · UX-DR19 khai sáu giá trị · AD-5 hứa mở về đúng vị trí)* đều **vẫn đúng**, và cả ba cộng lại vẫn thua một lượt dùng.

⚠️ **"Xoá" ở đây là ẩn khỏi LƯỚI, không xoá khỏi ĐĨA** — và tôi nêu lằn ranh đó trước khi sửa một dòng nào. Hàng vẫn nằm trong `project.db` với `retired_at` khác `NULL`; `read_segment_history` không hỏi cột đó, nên **AC4** còn nguyên. Hai ca hợp đồng khoá cả hai vế cùng lúc: lưới rút xuống **2** hàng sau một lượt gộp, đĩa lên **4**.

⚠️ **Số thứ tự tự sửa theo, không một dòng nào cho riêng nó:** lưới đánh số bằng **chỉ số mảng**, và `ord` trên đĩa đã được chữ ký #7(a) đánh lại 1..N từ đầu. Ca e2e nay đọc **thẳng cột số người dùng nhìn thấy** (`.cell-num`) chứ không suy từ chỉ số — vế thứ hai của báo cáo phải có lưới riêng.

🔴 **Hệ quả CHƯA đóng:** nhánh `'ornament'` của `resolveSegmentRule` nay **không còn đường tới**. Tôi **không** gỡ nó — `ornament` là một trong sáu giá trị vạch mà UX-DR19 khai, và gỡ là làm mã lệch một UX-DR đang đứng (`project-context.md:456-458`). Món nợ có chủ, ghi ở `deferred-work.md`.

### Debug Log References

#### Ⓐ Task 1 — bàn đo WKWebView, ba vòng, và nó **lật chữ ký #2**

**Tạo tác đầy đủ: `2-8-ban-do/README.md`** *(bảng số từng vòng)* · `tach-cot-nguon-wkwebview.e2e.mjs` *(vòng 1–2)* · `tach-cot-nguon-vong3.e2e.mjs` *(vòng 3)*. WebKit **605.1.15**, cửa sổ Tauri thật, `--features wdio`, macOS 24.6.0, Node 22.22.2, 2026-08-17. Cổng 4445 bị `gdrive-su` **PID 91509** giữ ⇒ chạy ở **4467**.

**Số quyết định, ba biến thể trên cùng một ô `[data-col="src"]`:**

| Cử chỉ | `selectionType` | `rangeCount` | lượt `lookup_dictionary` |
|---|---|---|---|
| bấm đơn | **`"None"`** | **0** | 0 |
| kéo chọn, **6 bước** trung gian | **`"None"`** | **0** | 0 |
| kéo **sau khi** tài liệu đã có tiêu điểm | **`"None"`** | **0** | — |
| **đối chứng** — bấm ô `[data-col="tgt"]` | `"Caret"` | 1 | — |
| `setPosition` + `modify('extend','forward','word')` | **`"Range"`** — `"Một"` | 1 | — |

⇒ **Tiền đề của chữ ký #2(a) không tồn tại**: không cử chỉ chuột nào tạo ra *"vùng chọn đang có"* ở cột nguyên văn. Hai giả thuyết về **bàn đo** đã bị loại từng cái *(cú `blur()` của chính bàn đo; lượt kéo quá thô)*.

🔵 **Số phụ đáng giá hơn cả hai:** `document.hasFocus()` là **`false` ở MỌI bước** — kể cả bước mà ô bản dịch **vẫn** nhận được caret. ⇒ *"tài liệu có tiêu điểm"* **không** phải biến phân biệt hai cột; thứ phân biệt là **`contenteditable`**. Ai đọc bảng này sau đừng đi lại đường (b).

⚠️ **Và một số làm đường (e) chạy được, đo ở bước ⓪:** ô nguyên văn có `soPhanTuCon = 0` và ba text node `[0, 40, 0]` ⇒ tổng độ dài các text node **đứng trước** node dài nhất là **0** ⇒ `anchorOffset` **bằng đúng** chỉ số ký tự trong `source_text`. Phép ánh xạ offset → chỗ cắt là **tầm thường** — *hôm nay, với Hán Việt TẮT*. Với Hán Việt **BẬT** ô mang thêm `<ruby>`/`<rt>` và mệnh đề này **hết đúng**; chưa đo, ghi nợ.

🔵 **LUẬT DỪNG (Task 1.4) KHÔNG kích hoạt, và phải nói rõ vì sao** — ba vòng đã chạy nhưng **0** vòng có một *giả thuyết về sản phẩm* bị bác. Vòng 1 hỏng **thước** *(bàn đo không dọn vùng chọn giữa các bước ⇒ hai số hoán vị nhau; và `TreeWalker` lấy một text node rỗng)*; vòng 2 cho số đứng; vòng 3 loại hai ứng viên của **bàn đo**. Phân biệt *"số thật, trật câu hỏi"* này có tiền lệ ở `2-5d-ban-do/README.md` §Vòng 2.

#### Ⓑ Một ứng viên **không loại trừ được bằng bộ đo**, và nó lớn hơn story này

Mọi cử chỉ trên đi qua **WebDriver pointer actions**, không chuột vật lý — và bộ đo này đã mang **một** giới hạn cùng hạng từ 2026-08-12 (`browser.keys()` không đi vào đường nhập văn bản gốc của WKWebView). Ứng viên thứ ba — *"driver không lái được máy chọn văn bản của WebKit trong nội dung không sửa được, dù lái được tiêu điểm"* — **không loại trừ được bằng chính driver đó**, theo cấu tạo.

🔴 **Hệ quả nếu ứng viên đó SAI:** Auto-Lookup bằng chuột (FR21, Story 1.18, **đã phát hành**) đã chết trong bản đang chạy, và **bộ e2e hôm nay không có spec nào** cho vùng chọn ở cột nguồn. **Ice chốt 2026-08-17: tách hẳn thành một story hạ tầng**, gộp với hai món nợ *"bộ e2e chập chờn"* đã có chủ. Ghi nợ, không tự vá trong story này.

#### Ⓒ Chữ ký thứ mười một — đường (e), dev đề xuất từ số đo

Phép đo vừa **bác** một đường lại **cho phép** một đường khác: `setPosition`/`modify` chạy được ở cột nguyên văn. ⇒ Sản phẩm **tự đặt** caret ở cột nguồn từ toạ độ cú bấm — **đúng khuôn đường chuột mà Story 2.5b đã phải dựng cho ô BẢN DỊCH** (`setPosition` ở `mouseup`), vì `contenteditable` trần ở đó cũng không đủ. Cùng lớp khuyết tật, cùng lớp bản vá, lần thứ hai.


### Completion Notes List

#### ⓪ Task 2.3 — **BA NEO SỐ HỌC CỐ Ý KHÔNG ĐỔI**, và đây là lý do viết ra bằng chữ

Chữ ký **#7(a)** *(đánh lại `ord` liên tục 1..N)* **không cần một cột mới**, và `retired_at` đã có trong `SEGMENT_DDL` từ **bước 5** — `deferred-work.md:1961-1968` ghi thẳng rằng nó được đặt sớm *"để 2.8 không phải mở một bước di trú thứ hai"*. ⇒ Story này **tiêu 0 bước di trú**, lượt đầu tiên của chuỗi 2.5c → 2.5d → 2.6 → 2.7.

⇒ Ba neo giữ nguyên: `vec![1,2,3,5,6,7,8,9,10,11]` · fixture "tương lai" `STEP_TWELVE` · `pinned_contract.rs` (`len()` 10, `schema_version()` 11). **Số kế tiếp vẫn là 12.**

🔴 **Im lặng ở đây đọc giống một lượt quên** — đó là cả lý do Task 2.3 tồn tại. Neo số học của kho này đã sai **ba lần liên tiếp** (2.5c · 2.5d · 2.6) và **không cổng nào canh chúng**; một story không đụng tới chúng phải nói ra rằng nó **đã xét và quyết định không đụng**.

#### ① Task 3.7 — **viết đường chèn THỨ HAI, không mở rộng `insert_segments`**

Story đặt câu hỏi và đòi nói ra chọn cái nào. Chọn: **đường thứ hai** (`write_regroup`). Ba lý do, xếp theo sức nặng:

1. `insert_segments` nhận `&[SplitSegment]` — hình dạng của **bộ tách câu lúc nhập**, mang đúng `text` + `is_paragraph_end`. Một lượt gộp cần **năm** giá trị *(nguồn · đích · cặp cờ · cắt bỏ · xuất xứ)*. Mở rộng nó là biến một hàm một-việc thành một hàm hai-việc với một tham số cờ.
2. Hai hàm **nối tiếp `ord` khác nhau**: `insert_segments` cấp `ord = index + 1` cho một Chương **rỗng**; `write_regroup` chèn vào **giữa** một Chương đang có rồi đánh lại toàn bộ.
3. AD-4 *(ranh giới tính **một lần** lúc nhập, không bao giờ tính lại)*: `insert_segments` là đường **nhập**. Cho nó một chỗ gọi thứ hai từ một thao tác **sau khi nhập** làm mờ đúng lằn ranh đó.

⚠️ Cái giá, nhận chứ không giấu: **hai câu `INSERT` vào cùng một bảng**, và ngày cột thứ mười ba ra đời thì **cả hai** phải sửa. Đường đóng nó là cổng AC8 của Story 2.7 (`a_flush_touches_exactly_...`), thứ so **trọn hàng** — nhưng nó chỉ canh đường flush. Một cột mới thiếu ở `write_regroup` hôm nay **không cổng nào đỏ**.

#### ①b 🔵 **Chữ ký #4(a) hứa mở rộng `replaceEditorSegment`; thực tế viết `applyRegroup` mới**

*(Khai ở code review 2026-08-17 — lượt lệch này đáng lẽ phải nằm ở đây từ lượt dev.)*

Bảng chữ ký ghi cái giá của #4(a) là *"Phải **mở rộng** `replaceEditorSegment` (hôm nay chỉ
THAY, không THÊM/BỚT)"*. Cái giá đó **không được trả**: `replaceEditorSegment` không bị chạm
một dòng — nó vẫn đúng một định nghĩa (`editorPanelState.ts:170`) với các nơi gọi cũ ở
`segmentHistoryState.ts`. Thay vào đó `applyRegroup` được viết mới.

**Kết quả vẫn thoả AD-47 ①**, và hàm mới **có lẽ đúng hơn** cái giá đã hứa: một lượt gộp/tách
vừa **thêm** vừa **bớt** hàng, còn `replaceEditorSegment` là một hàm một-việc *(thay đúng một
hàng, dùng bởi đường khôi phục lịch sử)*. Nạp việc thứ hai vào nó là đúng thứ §① từ chối làm
với `insert_segments`, vì đúng lý do.

🔴 **Cái sai không phải quyết định, mà là sự im lặng.** Story khai **hai** lượt lật khác (#2,
#6) đầy đủ kèm lý do và số đo; lượt này đi qua không một dòng. Một chữ ký thi hành khác cách
đã hứa mà không ai khai là đúng khuôn *"chữ ký thi hành đúng MỘT NỬA"* mà §Bài học liệt kê —
chỉ khác là ở đây **nửa rơi lại là nửa dễ**, một câu phải viết ra.

#### ② Hai khoá `MessageKey` mới, và phép thử để không nói lại điều đã có

`segment.no_previous` · `segment.cut_leaves_empty_piece`. Cả hai qua đúng phép thử mà Story 2.5d dùng để dựng khoá thứ ba: chúng nói một sự thật mà **không khoá nào đang có nói được** — câu **tồn tại**, **còn sống**, và thao tác vẫn không chạy được. Ba khoá cũ nói *"không tìm thấy"* · *"đã về hưu"* · *"chưa có gì để xác nhận"*.

⚠️ `segment.no_previous` là một ca **thường nhật**, không một ca biên: chữ ký #1(a) chốt *"gộp với câu **liền trên**"*, nên nó xảy ra **mỗi lần** người dùng bấm `⌘M` ở câu đầu Chương.

#### ③ 🔴 MỘT KHUYẾT TẬT CỦA MÃ **ĐÃ PHÁT HÀNH**, tìm ra ở vòng chẩn đoán Ice cấp phép

**`document.caretPositionFromPoint` KHÔNG TỒN TẠI trên WKWebView này**, và một lời gọi trần **NÉM `TypeError`** — nó không trả `null`.

| API | `typeof`, đo 2026-08-17 trong cửa sổ Tauri thật |
|---|---|
| `document.caretPositionFromPoint` | **`"undefined"`** |
| `document.caretRangeFromPoint` | `"function"` — trả `offset: 19`, node **trong** ô |

`GridPanel.vue::placeCaretAtPoint` — viết ở **Story 2.5b**, đang chạy trong bản phát hành — gọi đúng API đó, **trần, ở dòng đầu**. ⇒ `onCellMouseUp` chết ngay tại đó ở **mọi** cú bấm vào ô bản dịch, và `ensureCaretNextFrame` — thứ mà doc-comment của chính nó gọi là *"đường DUY NHẤT chạy được khi engine không làm"* — **chưa bao giờ chạy**.

🔴 **Vì sao không ai thấy suốt hai story:** caret **vẫn hiện**. Đo cùng lượt, sau một cú bấm chuột thật vào ô bản dịch: `selectionType = "Caret"`, `rangeCount = 1`, `activeElement` = chính ô — **cộng** một `TypeError` và một `[Vue warn]` trong console. Caret ấy đến từ `cell.focus()` cộng hành vi mặc định của engine, **không** từ đường vá mà ba vòng chẩn đoán của Story 2.3 và 2.5b đã mua. `grid-empty-cell.e2e.mjs` xanh trên một sản phẩm mà **nửa cơ chế của nó đang chết**.

✅ **Ice chốt 2026-08-17: sửa trong story này.** Một hàm `caretPointAt` dò năng lực *(ưu tiên `caretRangeFromPoint`, thứ WebKit **thật sự có**)*, hai nơi gọi.

⚠️ **Hệ quả phải nói ra:** lượt vá này **đổi hành vi** một đường đã nghiệm thu ở 2.5b — `ensureCaretNextFrame` nay **sẽ chạy thật**, lần đầu tiên. Bộ e2e trọn bộ là đường canh vế đó.

#### ④ Ba giới hạn của BÀN ĐO, đo được — và một trong ba từng nói dối tôi ba vòng

1. **`browser.keys(['Meta','/'])` giao `code: "/"`**, không `"Slash"` ⇒ hợp âm không khớp, `defaultPrevented: false`, **0** command chạy. Đối chứng `⌘M` giao `code: "KeyM"` ⇒ khớp. ⇒ Spec dùng một `KeyboardEvent` tổng hợp mang `code: 'Slash'` cho **riêng** lượt tách, và ghi rõ vế nào vì thế không được phủ.
2. **Không giao được một cú bấm chuột tới cột nguyên văn** qua **ba** cách nhắm, trong khi cùng lệnh đó trên ô bản dịch thì ăn. ⇒ Spec bắn một `MouseEvent` tổng hợp lên ô, toạ độ lấy từ **hộp dòng thật**.
3. 🔴 **Và một khuyết tật của chính bàn đo tôi viết, thứ đã nói dối ba vòng liền:** listener chẩn đoán gọi `document.caretPositionFromPoint(e.clientX, e.clientY)` **trần** ⇒ nó **ném trước dòng `push`** ⇒ `chuot` rỗng ở mọi vòng, và tôi đọc con số rỗng ấy thành *"không một `mouseup` nào tới `document`"* — một mệnh đề về **engine** dựng từ một cú ném của **bàn đo**.
   ⚠️ Đúng lớp bẫy mà vòng 1 đã mắc một lần và `2-5d-ban-do` đã đặt tên: **một con số thật, trả lời sai câu hỏi**. Bài học rút ra, ghi vì nó sẽ gặp lại: **một listener chẩn đoán phải chịu được engine mà nó đang đo** — nó là mã chạy trên chính engine đó, không một cái nhìn từ bên ngoài.

#### ⑤ Số baseline → số sau

🔵 **2026-08-17, code review — BẢNG NÀY LÀ BẢNG TRƯỚC LƯỢT LẬT #6 VÀ NÓ ĐÃ HẾT ĐÚNG.** Số
đúng ở §⑥ *(nghiệm thu cuối)*: `cargo` **399**, `segment_contract` **119** — bảng dưới ghi 398
và 118, tức thiếu **một** ca sinh ra ở lượt lật. Sửa tại chỗ thay vì để hai bảng của cùng một
story nói hai số. 🔴 Ghi ra vì đây là **lần thứ tư** neo số học của kho sai *(2.5c · 2.5d ·
2.6, và nay là một bảng trong chính story)* và **không cổng nào canh chúng** — đó là cả lý do
Task 2.3 tồn tại.

| Đường | Trước | Sau | Chênh |
|---|---|---|---|
| `cargo test --locked` | 383 / 0 / 5 | ~~398~~ → **399 / 0 / 5** | ~~+15~~ **+16** |
| `segment_contract` riêng | 103 | ~~118~~ → **119** | ~~+15~~ **+16** |
| `npm run test` (vitest) | 133 / 133 · 12 tệp | **138 / 138 · 13 tệp** | **+5** |
| 9 cổng đọc-tệp | 9 xanh | **9 xanh** | — |
| `COMMAND_FLOOR` | sàn 37 · cổng in 44 | sàn **38** · cổng in **46** | +2 command |
| Bước di trú kế tiếp | 12 | **12** | **0 bước tiêu** |

**Đỏ-rồi-xanh:** 8 đột biến ở tầng thuần Rust · 6 ở đường SQL · 2 ở module frontend — **16 lượt, tất cả đột biến MÃ SẢN PHẨM**, không một lượt đột biến test.

#### ⑥ Nghiệm thu cuối — 2026-08-17 *(sau lượt lật #6)*

| Đường | Baseline | Kết quả |
|---|---|---|
| 11 cổng npm | 11 | **11 / 11 xanh** *(9 đọc-tệp + `check:scope` + `check:scope:bundled` chạy tay)* |
| `npm run build` · `vue-tsc --noEmit` | xanh | xanh · xanh |
| `npm run test` (vitest) | 133 · 12 tệp | **138 / 138** · 13 tệp |
| `cargo test --locked` | 383 / 0 / 5 | **399 / 0 / 5** *(+16)* |
| `segment_contract` riêng | 103 | **119** |
| `COMMAND_FLOOR` | sàn 37 · cổng in 44 | sàn **38** · cổng in **46** |
| Bước di trú kế tiếp | 12 | **12** *(tiêu 0 bước)* |

**e2e — HAI lượt trọn bộ, ghi cả hai:**

| Lượt | Kết quả | Ghi chú |
|---|---|---|
| ① *(trước lượt lật #6)* | **9 / 9 spec · 13 / 13 ca** (13m13) | — |
| ② *(sau lượt lật #6)* | **8 / 9 spec** (12m28) | `editor-typing-flush` đỏ |
| ③ chạy riêng spec đỏ | **2 / 2** (2m21) | — |

🔴 **Ca đỏ ở lượt ② KHÔNG phải một hồi quy, và đây là bằng chứng chứ không một lời trấn an:**
nguyên văn lỗi là `Couldn't find element for "pointerMove" action sequence` — **không tìm thấy
phần tử để bấm**, không một phép khẳng định nào về flush hay về lưới. Cùng spec đó xanh **2/2**
khi chạy riêng, và xanh ở lượt ① *(tức sau bản vá `placeCaretAtPoint`, trước lượt lật #6)*.
Khớp khuôn hai món nợ *"bộ e2e chập chờn"* đã có chủ — đặc biệt `deferred-work.md:3093-3115`
*(fixture không reset state panel giữa các spec)*.

⚠️ **KHÔNG chấm "đã chẩn đoán".** Luật sau Story 1.22 đòi **bắt nguyên văn trước**, và thứ tôi
có là một chuỗi lỗi cộng ba lượt chạy — đủ để nói *"không phải hồi quy của story này"*, **không**
đủ để nói *"nguyên nhân là X"*.

🔵 **Hai spec của 2.5b và 2.5d xanh SAU lượt vá `placeCaretAtPoint`** — `grid-empty-cell` và
`editor-typing-flush` *(cả hai ở lượt ①, và spec sau xanh lại ở lượt ③)*. Đó là đường canh duy
nhất cho hệ quả ở §Completion Notes ③: `ensureCaretNextFrame` nay **chạy thật** lần đầu, và nó
**không** làm hồi quy hai ca ấy.

### File List

*(kê từ `git status --porcelain`. Đường dẫn tương đối gốc kho.)*

**Mới**

- `src-tauri/src/core/segment/regroup.rs` — phép tính THUẦN cho hàng mới của gộp/tách
- `e2e/specs/segment-merge-split.e2e.mjs` — lưới duy nhất cho hình dạng dây
- `tests/frontend/editorSourceCut.test.ts` — phép ánh xạ offset → chỗ cắt
- `_bmad-output/implementation-artifacts/2-8-gop-va-tach-segment-tuong-minh.md` — chính story này
- `_bmad-output/implementation-artifacts/2-8-ban-do/README.md` — bảng số ba vòng bàn đo
- `_bmad-output/implementation-artifacts/2-8-ban-do/tach-cot-nguon-wkwebview.e2e.mjs`
- `_bmad-output/implementation-artifacts/2-8-ban-do/tach-cot-nguon-vong3.e2e.mjs`
- `_bmad-output/implementation-artifacts/2-8-ban-do/tach-chan-doan.e2e.mjs`
- `_bmad-output/implementation-artifacts/2-8-ban-do/caret-api-cot-dich.e2e.mjs`

**Sửa**

- `src-tauri/src/commands/segment.rs` — `merge_segments` · `split_segment` · `RegroupOutcome` · hai vỏ `wire` · hai khoá từ chối mới
- `src-tauri/src/core/segment/mod.rs` — khai `regroup`
- `src-tauri/src/core/i18n/mod.rs` — hai `MessageKey` mới
- `src-tauri/src/lib.rs` — đăng ký hai lệnh
- `src-tauri/tests/segment_contract.rs` — ~~+15 ca *(8 tầng thuần, 7 đường SQL)*~~ 🔵 **+16 ca
  *(8 tầng thuần, 8 đường SQL)*** — đếm lại ở code review 2026-08-17: `grep '^+#\[test\]'` trên
  diff cho **16**, và `hang_tho` là một hàm **phụ**, không một ca. Đúng bài học #3 của chính
  story *("đọc nội dung dòng khớp, đừng đếm")* — lần này sập bẫy trong File List.
  🔵 **+18 sau lượt vá code review** *(hai ca đa-mảnh: `splitting_at_many_cuts_…` ·
  `splitting_refuses_duplicate_cuts_…`)*
- `src/panels/GridPanel.vue` — `caretPointAt` *(dò năng lực)* · `onSourceCellMouseUp` · `placeCaretAtPoint` **sửa lỗi 2.5b**
- `src/panels/editorPanelState.ts` — `sourceCut` · `applyRegroup` · `regroup` · hai lệnh
- `src/panels/editorSegments.ts` — `sourceCutOffsetOf`
- `src/config/segment.ts` — `RegroupOutcome` · `mergeSegments` · `splitSegment`
- `src/commands/index.ts` — hai command + hai cổng dep
- `src/main.ts` — nối hai dep
- `src/i18n/vi.json` — 4 khoá *(2 nhãn lệnh, 2 lỗi)*
- `scripts/check-commands.mjs` — `COMMAND_FLOOR` 37 → 38
- `scripts/check-layout.mjs` — `document.createTreeWalker` vào danh sách cho phép
- `e2e/support/workspace.mjs` — tham số `text` **tuỳ chọn**, mặc định y nguyên chuỗi cũ
- `_bmad-output/implementation-artifacts/deferred-work.md` — 10 món, mỗi món một chủ
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `…/EXPERIENCE.md` — `:171` UX-DR32 cũ · `:267` `⌘T` → `⌘/`, cả hai kèm 🔵 + ngày
- `…/mockups/settings.html` — hai command id theo chữ ký 2026-08-14

🔵 **Thêm ở lượt vá code review 2026-08-17** *(kê từ `git status --porcelain`)*:

- `src-tauri/src/core/segment/paragraph.rs` — 🔵 ba mệnh đề đã hết đúng, sửa tại chỗ *(tệp này KHÔNG có trong File List của lượt dev — nó không bị chạm, mà ba câu của nó thì đã nói ngược cây nguồn)*
- `src-tauri/src/core/segment/regroup.rs` — `split_at` nhận **một tập** chỗ cắt · `.filter` mảnh rỗng khi nối `source_text`
- `src-tauri/src/commands/segment.rs` — `split_segment(… cuts: Vec<usize>)` + vỏ `wire` · `ORDER BY ord, id`
- `src-tauri/tests/segment_contract.rs` — +2 ca *(đa-mảnh · chỗ cắt trùng)*, và mọi nơi gọi `split_at`/`split_segment` đổi theo chữ ký mới
- `src/panels/editorSegments.ts` — `sourceCutOffsetOf` bỏ qua `<rt>` · đếm **code point** · hai hàm phụ `demKyTu`/`duoiRt`
- `src/panels/editorPanelState.ts` — `sourceCut` giữ một **tập** · `setEditorSourceCut` ba nhánh *(thay/gỡ/thêm)* · `regroup()` xoá tập **có điều kiện**
- `src/config/segment.ts` — `splitSegment(segmentId, cuts: readonly number[])`
- `src/panels/GridPanel.vue` — `pendingCuts` · `cutCountOf` · `sourcePiecesOf` · `data-cut-count` · `.cut-mark` · `.cell-src.has-cuts`
- `tests/frontend/editorSourceCut.test.ts` — +3 ca *(③b `<rt>` · ③c bấm vào `<rt>` · ③d ngoài BMP)*
- `e2e/specs/segment-merge-split.e2e.mjs` — +1 ca *(tách BA mảnh từ HAI điểm cắt)*
- `_bmad-output/implementation-artifacts/deferred-work.md` — 3 món đóng **tại chỗ** · 1 món chuyển chủ · 6 món mới có chủ

### Change Log

| Ngày | Việc |
|---|---|
| 2026-08-17 | Task 0 — Ice ký **chín** quyết định mở; cộng chữ ký thứ **mười** (lật #6 → (b)) và thứ **mười một** (lật #2 → (e), dev đề xuất từ số đo) |
| 2026-08-17 | Task 1 — bàn đo WKWebView ba vòng; **bác chữ ký #2(a)**: không cử chỉ chuột nào tạo vùng chọn ở cột nguyên văn |
| 2026-08-17 | Task 2 — **0 bước di trú**; ba neo số học cố ý không đổi, ghi ra bằng chữ |
| 2026-08-17 | Task 3 — tầng thuần `core::segment::regroup` + đường SQL về hưu/tạo mới/đánh lại `ord`; +15 ca hợp đồng |
| 2026-08-17 | Task 4 — hai vỏ `wire`, hai adapter TS, hai khoá `MessageKey` mới |
| 2026-08-17 | Task 5 — `editor.merge_segments` (`⌘M`) · `editor.split_segment` (`⌘/`); `COMMAND_FLOOR` 37 → 38 |
| 2026-08-17 | 🔴 **Sửa một khuyết tật của Story 2.5b đang chạy trong bản phát hành** — `caretPositionFromPoint` ném trên WKWebView (Ice chốt sửa trong story này) |
| 2026-08-17 | Task 6 — 9 cổng · build · `vue-tsc` · vitest 138 · cargo 398 · e2e trọn bộ |
| 2026-08-17 | Tài liệu: `EXPERIENCE.md:171` + `:267`, `mockups/settings.html:276-277` — sửa tại chỗ kèm 🔵 + ngày |
| 2026-08-17 | 🔴 **Code review BA TẦNG** (Blind Hunter · Edge Case Hunter · Acceptance Auditor, phiên sạch song song) — 10 phát hiện sau phân loại, 3 bác làm nhiễu |
| 2026-08-17 | 🔴 **Hai lỗi trong CÙNG một hàm `sourceCutOffsetOf`**, cùng biểu hiện *(cắt sai chỗ, im lặng, không hoàn tác được)*: `<rt>` bị đếm là ký tự nguồn · đếm code unit UTF-16 thay vì code point. Cả hai vá tại nguồn, đo đỏ-rồi-xanh |
| 2026-08-17 | 🔴 **AC7 vế *"nhiều mảnh"* — Ice ký DỰNG trong story này**, cơ chế **tích luỹ** điểm cắt. `cut` → `cuts` trên dây; kênh thị giác `.cut-mark` + `has-cuts` |
| 2026-08-17 | Tám bản vá: đơn vị đếm · điểm cắt bị xoá oan · `paragraph.rs` 🔵 · 4 món nợ đóng tại chỗ · khoá phụ `, id` · lọc mảnh rỗng khi nối nguồn · hai bảng số mâu thuẫn · khai lượt lệch #4(a) |
| 2026-08-17 | Nghiệm thu lượt vá: cargo **401** · `segment_contract` **121** · vitest **141** · 9 cổng · build · `vue-tsc` · e2e spec 2.8 **3/3** *(lượt ② — ca gộp chập chờn ở lượt ①, không hồi quy)* |

### Review Findings

**Code review ba tầng — 2026-08-17** *(Blind Hunter · Edge Case Hunter · Acceptance Auditor, chạy song song trên phiên sạch, cùng cấp mô hình `claude-opus-5`)*. Diff rà: 3.336 dòng, `git diff HEAD` trên `dfa9c95` cộng ba tệp mã mới. **10 phát hiện còn lại sau phân loại, 3 bác làm nhiễu.**

🔴 **Hai phát hiện nặng nhất nằm trong CÙNG MỘT HÀM** — `editorSegments.ts::sourceCutOffsetOf`, tức phép ánh xạ offset → chỗ cắt. Cả hai cho **cùng một biểu hiện**: `⌘/` cắt đúng chỗ người dùng **không** bấm, trên dữ liệu mà AD-5 không cho hoàn tác, và **không cổng nào đỏ**.

#### Quyết định cần Ice chốt

- [x] [Review][Decision] **`⌘/` cắt SAI VỊ TRÍ khi Hán Việt BẬT — và kho này đã đo đúng cái bẫy đó từ 2026-08-07** — `sourceCutOffsetOf` (`editorSegments.ts:257-266`) đi `TreeWalker(cell, SHOW_TEXT)` và cộng `textContent.length` của **mọi** text node trong ô. Khi Hán Việt bật, ô mang `<ruby>{chữ Hán}<rt>{âm}</rt></ruby>` (`SourceHanViet.vue`, render ở `GridPanel.vue:1166-1171` khi `showHanViet`) — và `<rt>` **là một text node**, bị đếm vào, dù nó không phải một ký tự nào của `source_text`. Đo bằng happy-dom trên đúng hình dạng DOM sản phẩm (`<ruby>京<rt>kinh</rt></ruby>都です。`): cú bấm đáng lẽ cho chỉ số **1** trả về **5**. ⇒ `split_at` cắt sai chỗ, hoặc chỉ số vượt biên và bị nuốt im lặng thành `'no-cut'`. 🔴 **Ba vế làm nó nặng hơn một món nợ đã khai:** ① story CÓ ghi nợ vế này *("chưa đo")* nhưng **không dòng mã nào chặn `⌘/` khi `showHanViet` bật** — tính năng phát hành trần; ② `SourceHanViet.vue:510` đã ghi nguyên văn *"🔴 `ruby.textContent` GỘP CẢ `<rt>` — đo được 2026-08-07 trên Chromium"*, tức bẫy đã có tên trong kho và hàm mới đi thẳng vào; ③ ca ③ của `editorSourceCut.test.ts:53-55` dựng một `<ruby>` **không có `<rt>`**, nên phép kiểm **không thể đỏ** — một lượt tin cậy giả. **Ice đã nhận vế này làm nợ, nhưng nhận trên một cơ sở thiếu ②③** ⇒ trình lại, không tự chọn.
- [x] [Review][Decision] **AC7 vế *"nhiều mảnh"* đóng một nửa — không quyết định, không chữ ký, không nợ** — `epics.md:2522` đòi *"tách một segment thành **nhiều** mảnh ⇒ cờ theo mảnh cuối, mọi mảnh trước nhận cờ tắt"*. Cài đặt chỉ ra **đúng hai** mảnh mỗi lượt: `split_at(part, cut: usize)` nhận **một** chỉ số và viết cứng `split_into(part.flags, 2)`; `sourceCut` giữ **một** điểm cắt. 🔴 **Đây đúng hình dạng khoảng hở của AC6** — mà AC6 thì được cả một Quyết định #1, một chữ ký của Ice, và một mục 🟡 có chủ. AC7 nhận **không gì cả**. Đúng lớp lỗi *"chữ ký thi hành đúng MỘT NỬA"* mà §Bài học của chính story này liệt kê là đã lặp năm lần — lần này lặp **bên trong** story viết ra để chống nó. `project-context.md:456-458` cấm sửa `epics.md` cho khớp mã ⇒ đường ra là một món nợ **có chủ**, nhưng ai nhận và có dựng đa-mảnh trong Epic 2 không là chữ ký của Ice.

#### Vá được, không cần Ice

- [x] [Review][Patch] **Chỉ số cắt lệch đơn vị: JS đếm UTF-16 code unit, Rust đếm scalar value** [src/panels/editorSegments.ts:263] — `n.textContent?.length` và `offsetInNode` đều là **UTF-16 code unit**; `regroup.rs:202` `source_text.chars().collect()` là **Unicode scalar value**. Hai đơn vị chỉ trùng khi mọi ký tự trước chỗ cắt nằm trong BMP. Một ký tự ngoài BMP — CJK Extension B (U+20000+), thứ có thật trong văn bản Hán cổ và tên riêng — là **2** đơn vị JS nhưng **1** `char` Rust ⇒ chỗ cắt lệch phải đúng bằng số ký tự astral đứng trước, thường **vẫn trong biên** nên không lỗi nào ném. Không ca test nào ở cả hai tầng dùng một ký tự ngoài BMP (`他走了。` bên Rust, `東京`/`京都`/`です` bên vitest — toàn BMP). Vá: đếm bằng code point (`[...s].length` / `Array.from`), kèm một ca có ký tự astral ở **cả hai** tầng.
- [x] [Review][Patch] **Một lượt `⌘M` không liên quan xoá mất điểm cắt đang chờ của `⌘/`** [src/panels/editorPanelState.ts:1178] — `regroup()` chạy `sourceCut.value = null` **vô điều kiện** sau mọi lượt gộp **hoặc** tách. Chú thích ④ biện minh cho nó bằng một mệnh đề **chỉ đúng ở đường tách**: *"nó trỏ một `segment.id` mà lượt này vừa cho về hưu"*. Ở đường gộp, điểm cắt có thể trỏ một segment **không nằm trong nhóm vừa về hưu**: bấm cột nguồn ở câu 5, rồi `⌘M` gộp câu 1+2 ⇒ câu 5 nguyên vẹn nhưng điểm cắt bay. `⌘/` kế tiếp trả `'no-cut'`, mà `'no-cut'` chỉ đi ra `console.warn` — người dùng không thấy gì. Vá: chỉ xoá khi `cut.segmentId` nằm trong tập vừa về hưu.
- [x] [Review][Patch] **`paragraph.rs` mang ba mệnh đề đã hết đúng, không 🔵, không ngày** [src-tauri/src/core/segment/paragraph.rs:11,26,31] — tệp **không bị chạm** trong diff, và doc-comment của nó vẫn viết *"Story 2.8 … là `backlog`"*, vẫn để hàng bảng *"Gộp → theo câu cuối | **Không** — mới là một bảng trong doc-comment | —"*, và vẫn kết *"AC3 vẫn không đóng trọn ở story này … ghi nợ có chủ **Story 2.8**"*. Cả ba sai từ hôm nay: `regroup::merge`/`split_at` gọi thẳng `merged()`/`split_into()`. `project-context.md:352-353` đòi **sửa tại chỗ kèm 🔵 + ngày** — story này đã áp đúng luật đó cho `EXPERIENCE.md` và `settings.html` rồi bỏ sót đúng tệp mà nó tiêu thụ.
- [x] [Review][Patch] **Bốn món nợ chủ *"Story 2.8"* không được đóng TẠI CHỖ trong `deferred-work.md`** [_bmad-output/implementation-artifacts/deferred-work.md:3397,3535,3675,3832] — diff chỉ **nối thêm** ở cuối (`@@ -3965,3 +3965,123 @@`), không chạm một mục cũ nào. `project-context.md:449-451` đòi đóng bằng cách **nối tiếp `→ ✅ ĐÃ ĐÓNG <ngày> (Story x.y)`** ngay tại mục. Đã đọc và phân loại từng món: `:3535-3555` *(vế "hai cờ chạy hai lần độc lập")* ✅ đóng bởi Task 3.2 · `:3675-3683` *(AC4 trên segment về hưu THẬT)* ✅ đóng bởi `the_history_of_a_genuinely_retired_segment_still_reads_back_after_a_real_merge` · `:3832-3841` hàng *"Gộp/tách segment | Story 2.8"* ✅ đóng bởi AD-47 ④ · `:3397-3424` *(dải câu, ứng viên 2.8)* **KHÔNG đóng** — chữ ký #1(a) chốt gộp đúng hai ⇒ mục này cần một dòng 🟡 nói 2.8 đã xét và từ chối, **không** một dấu ✅.
- [x] [Review][Patch] **Truy vấn lưới thiếu khoá phụ `, id` mà chính kho lập luận là phải có** [src-tauri/src/commands/segment.rs:827] — lưới đọc `ORDER BY ord`; truy vấn đánh lại số `:2233` dùng `ORDER BY ord, id`; truy vấn tìm câu liền trên `:2379` dùng `ORDER BY ord DESC, id DESC`. `ord` **cố ý không `UNIQUE`** (`schema.rs:279-282`), và chú thích `:2371` viết thẳng vì sao không được giả định `ord` liên tục. Lập luận đó không được áp cho truy vấn lưới ⇒ nếu hai hàng sống từng trùng `ord`, *"câu liền trên"* mà Rust gộp không bảo đảm là hàng người dùng **nhìn thấy** ở trên. Chưa đường mã nào sinh ra `ord` trùng hôm nay, nên đây là vá phòng thủ một dòng, không một lỗi có tầm với.
- [x] [Review][Patch] **`merge()` nối `source_text` không lọc mảnh rỗng, trong khi `join_targets` thì có** [src-tauri/src/core/segment/regroup.rs:168] — `join_targets` (`:121`) có `.filter(|t| !t.is_empty())` cộng nguyên một đoạn chú thích đặt tên cho hậu quả: *"Nối `'A'` với `''` bằng `' '` cho `'A '` — một ký tự người dùng CHƯA TỪNG GÕ, nằm trên đĩa vĩnh viễn"*. Cùng bản vá không được soi sang `source_text`, vốn nối **vô điều kiện**. Chưa với tới hôm nay *(`split_at` từ chối mảnh rỗng; đường nhập `trim()`)* và với Tác phẩm tiếng Trung `source_joiner` trả `""` nên vô hại — nhưng với nguồn `en` thì đó đúng là lỗi đã được vá một lần ở trường anh em.
- [x] [Review][Patch] **Hai bảng số trong story mâu thuẫn nhau, và File List chép nhầm bảng cũ** [_bmad-output/implementation-artifacts/2-8-gop-va-tach-segment-tuong-minh.md:611] — File List viết *"`segment_contract.rs` — +15 ca (8 tầng thuần, **7** đường SQL)"*. Đếm thật trên diff: **16** `#[test]` mới — 8 tầng thuần + **8** đường SQL *(`hang_tho` là hàm phụ, không phải ca)*. §Completion Notes ⑤ ghi `cargo 383 → 398 (+15)`; §Completion Notes ⑥ *(nghiệm thu cuối)* ghi `383 → 399 (+16)` và `segment_contract 103 → 119`. ⑥ đúng, ⑤ và File List là bảng trước lượt lật #6 chưa cập nhật. Neo số học của kho này đã sai ba lần liên tiếp và không cổng nào canh — đúng lý do Task 2.3 tồn tại.
- [x] [Review][Patch] **Chữ ký #4(a) hứa một cái giá cụ thể, cái giá đó không được trả, và lượt lệch không được khai** [src/panels/editorPanelState.ts] — bảng chữ ký (`:417`) ghi hệ quả thi hành của #4(a) là *"Phải **mở rộng** `replaceEditorSegment` (hôm nay chỉ THAY, không THÊM/BỚT)"*. Diff **không chạm** `replaceEditorSegment` — nó vẫn đúng một định nghĩa (`:170`) với các nơi gọi cũ ở `segmentHistoryState.ts`. Thay vào đó một hàm mới `applyRegroup` được viết. Kết quả vẫn thoả AD-47 ①, và viết hàm mới **có lẽ đúng hơn** là nạp thêm việc cho một hàm một-việc — nhưng story khai hai lượt lật khác (#2, #6) đầy đủ kèm lý do, còn lượt này im lặng. Vá bằng một dòng ở §Completion Notes, không bằng một lượt đổi mã.

#### Bác làm nhiễu *(3, ghi ra để không ai rà lại)*

- **Cắt giữa một cụm chữ cái** *(base + dấu tổ hợp)* — `split_at` cắt theo scalar value nên về lý thuyết chẻ được một cụm NFD. Bác: chỗ cắt đến từ caret của WebKit, và caret không đậu giữa một cụm. Nguồn duy nhất của `cut` là đường chuột đó.
- **`is_omitted` *(bất kỳ ⇒ cắt)* ngược chiều `translation_origin` *(bất đồng ⇒ `other`)*** — hai trường cạnh nhau giải "bất đồng" hai chiều. Bác: Ice ký #5(a) có ghi ngày, chú thích tại chỗ nói thẳng vì sao hai chiều khác nhau, và món nợ *"luật này chưa có chỗ đứng trong spine"* đã ghi kèm chủ.
- **AC6 vế *"nhóm"* đóng một nửa** — bác: không phải vi phạm giấu. Có Quyết định #1, có chữ ký, có mục 🟡 kèm chủ. Đây là **khuôn đúng** mà AC7 lẽ ra phải theo.

---

### Kết quả lượt vá — 2026-08-17

**Ice chốt hai quyết định:** ① sửa thật vế `<rt>` *(không chặn tính năng)* · ② **dựng đa-mảnh ngay trong 2.8**, cơ chế **tích luỹ** — mỗi cú bấm thêm một điểm, `⌘/` cắt hết. Cộng: vá cả 8 món `patch`, không đi từng món.

⚠️ **Tôi đã nêu một lo ngại về đường ② trước khi thi hành và Ice giữ nguyên quyết định:** cơ chế gom nhiều điểm cắt là một tương tác **chưa tài liệu nào của dự án mô tả** — đúng chỗ Quyết định #1 và Story 2.5c đã từ chối hai lần. Ice cân và ký; hình dạng cụ thể *(tích luỹ, bấm trùng thì gỡ)* do Ice chọn từ ba phương án kèm số đo, không do tôi tự chọn.

#### Đã sửa

| Món | Hình dạng bản vá |
|---|---|
| **D1** `<rt>` bị đếm | `sourceCutOffsetOf` bỏ qua mọi text node có tổ tiên `<rt>` **trong ô**; bấm thẳng vào `<rt>` ⇒ `null` *(không đoán bừa)*. Ca ③b dựng **đúng** hình dạng DOM sản phẩm |
| **D2** AC7 đa-mảnh | `split_at(part, cuts: &[usize])` — `n` chỗ cắt cho `n+1` mảnh trong **một** giao dịch. Dây đổi `cut: number` → `cuts: number[]`. `sourceCut` giữ một **tập** |
| **P1** đơn vị UTF-16 | `demKyTu()` đếm **code point**; `offsetInNode` đi qua nó thay vì cộng thẳng. Ca ③d dùng U+20000 |
| **P2** điểm cắt bị xoá oan | `regroup()` chỉ xoá tập khi `cut.segmentId` **nằm trong** tập vừa về hưu |
| **P3** `paragraph.rs` | Ba mệnh đề sửa tại chỗ kèm 🔵 + ngày + bảng *"mệnh đề cũ → hôm nay"* |
| **P4** sổ nợ | 3 món đóng tại chỗ `→ ✅ ĐÃ ĐÓNG`; món *"dải câu"* nhận 🟡 **chuyển chủ** *(2.8 đã xét và từ chối)*, không nhận ✅ |
| **P5** khoá phụ `, id` | `read_open_chapter_segments` nay `ORDER BY ord, id` |
| **P6** nối `source_text` | `.filter(|s| !s.is_empty())`, soi từ bản vá đã có ở `join_targets` |
| **P7** hai bảng số | ⑤ sửa kèm 🔵; File List `+15 (8+7)` → `+16 (8+8)` → `+18` sau lượt vá |
| **P8** lượt lệch #4(a) | Khai ở §Completion Notes ①b |

#### Kênh thị giác cho tập điểm cắt — bắt buộc, không trang trí

Một trạng thái sống **giữa hai thao tác** mà người dùng không thấy là đúng lớp *"im lặng"* mà `project-context.md` cấm. Ô nguyên văn nay mang `data-cut-count` + lớp `has-cuts` *(viền trái)*, và ở đường **chữ trần** mỗi ranh giới có một `.cut-mark` — một `<span>` **rỗng**, `aria-hidden`, màu từ token `--color-ornament`. 🔴 Rỗng chứ không một ký tự: một ký tự thật đi vào `Selection.toString()` của Auto-Lookup và vào mọi lượt sao chép của người dùng.

⚠️ **Giới hạn thật:** ở chế độ Hán Việt dấu cắt **không vẽ được** *(chỗ cắt rơi giữa các `<ruby>`)* — ô chỉ có viền và số đếm. Phép **ánh xạ** thì đúng ở cả hai chế độ; hở là vế **hiển thị**. Ghi nợ có chủ.

#### Đỏ-rồi-xanh — 4 đột biến, tất cả trên MÃ SẢN PHẨM

| Đột biến | Ca đỏ | Chỉ ca đó? |
|---|---|---|
| bỏ phép lọc `<rt>` | ③b | ✅ 1 đỏ / 7 xanh |
| `[...s].length` → `s.length` | ③d | ✅ 1 đỏ / 7 xanh |
| bỏ `.filter` mảnh rỗng khi nối nguồn | `merging_never_manufactures_whitespace…` | ✅ |
| `split_at` bỏ phép kiểm chỗ cắt trùng | `splitting_refuses_duplicate_cuts…` | ✅ |

#### Số sau lượt vá

| Đường | Trước lượt vá | Sau | Chênh |
|---|---|---|---|
| `cargo test --locked` | 399 / 0 / 5 | **401 / 0 / 5** | +2 |
| `segment_contract` riêng | 119 | **121** | +2 |
| `npm run test` (vitest) | 138 · 13 tệp | **141 / 141** · 13 tệp | +3 |
| 9 cổng đọc-tệp | 9 xanh | **9 / 9 xanh** | — |
| `npm run build` · `vue-tsc --noEmit` | xanh | **xanh · xanh** | — |
| `COMMAND_FLOOR` | sàn 38 · cổng in 46 | **sàn 38 · cổng in 46** | — *(không command mới)* |
| Bước di trú kế tiếp | 12 | **12** | **0 bước tiêu** |

#### e2e — ĐÃ CHẠY, hai lượt, ghi cả hai

Cổng 1420 và 4445 trống, WebKit 605.1.15, cửa sổ Tauri thật, `--features wdio`.

| Lượt | Kết quả | Ghi chú |
|---|---|---|
| ① | **2 / 3 ca** (43 s) | ca **gộp** đỏ |
| ② chạy lại nguyên spec | **3 / 3 ca** (3m02) | — |

🔴 **Ca `⌘/ tách BA mảnh từ HAI điểm cắt tích luỹ` XANH ở CẢ HAI lượt.** Đó là mệnh đề quan trọng nhất của lượt vá: lượt đổi hình dạng dây `cut: number` → `cuts: number[]` đi trọn từ `keydown` → command → adapter → `invoke` → `tauri-macros` → Rust → vá ảnh chụp → lưới, trên engine thật. Kho này đã để lọt đúng lớp lỗi ấy **hai lần** *(`status` 2.5, `textAtLoad` 2.7)* và cả hai lần toàn bộ test Rust + vitest đều xanh. Ca này cũng đọc **thẳng** kênh thị giác sản phẩm ghi ra *(`data-cut-count` = `'2'`, hai `.cut-mark`, lớp `has-cuts`)*, không suy từ một biến nội bộ.

🔴 **Ca gộp đỏ ở lượt ① KHÔNG phải hồi quy — bằng chứng, không một lời trấn an:**
- **Lượt vá này không chạm một dòng nào của đường gộp.** `merge_segments` · `merged` · `merged_origin` · `mergeCurrentSegment` đều nguyên trạng; hai thứ tôi sửa có chạm gộp là `.filter` mảnh rỗng khi nối `source_text` *(có ca hợp đồng riêng, xanh)* và khoá phụ `, id` *(một dòng SQL, 121/121 xanh)*.
- Nguyên văn lỗi là `Vào 'workspace' rồi mà không thấy [data-attribution-open] sau 30 giây` — một phép chờ **fixture** ở `workspace.mjs:89`, **không** một phép khẳng định nào về gộp, về lưới, hay về hình dạng dây.
- ⚠️ **Và giả thuyết trong chính câu lỗi SAI:** nó đoán *"kiểm `src-tauri/target/debug/dict/*.db` có mặt chưa"*. Đo: **cả bốn tệp có mặt** *(`dict-core` 195 MB · `dict-vietphrase` 160 MB · `dict-thieu-chuu` · `dict-tran-van-chanh`, ngày 10-08)*. Đúng khuôn *"một con số thật, trả lời sai câu hỏi"* mà §Completion Notes ④ đã đặt tên — lần này nằm trong **thông điệp** của bàn đo.
- Cùng ca xanh ở lượt ②, và ca này là ca **đầu tiên** của tệp *(cold start)*.

⚠️ **KHÔNG chấm "đã chẩn đoán".** Thứ tôi có là một chuỗi lỗi cộng hai lượt chạy — đủ để nói *"không phải hồi quy của lượt vá"*, **không** đủ để nói *"nguyên nhân là X"*. Khớp khuôn hai món nợ *"bộ e2e chập chờn"* đã có chủ, và lượt này thêm một quan sát cho món đó: **thông điệp trượt của `workspace.mjs` trỏ một nguyên nhân đã bị loại bằng đo**.

#### Bộ e2e TRỌN BỘ sau lượt vá — đã chạy, vì lượt vá đổi cách render ô nguyên văn của MỌI hàng

Lập luận *"chín spec kia không chạm mã tôi sửa"* **không dùng được** ở đây: `sourcePiecesOf` đổi hình dạng cây DOM của cột nguyên văn cho **mọi** hàng, không riêng hàng có điểm cắt. Đó là một mệnh đề về engine thật ⇒ phải đo, không suy *(§Bài học #6)*.

| Lượt | Kết quả | Thời gian |
|---|---|---|
| trọn bộ | **8 / 9 spec** · 13 ca *(12 xanh)* | 13m12 |
| `editor-typing-flush` chạy riêng | **2 / 2** | 1m57 |

🔴 **Ca đỏ: `editor-typing-flush.e2e.mjs:184`, và nó KHÔNG do lượt vá — bằng chứng là một phép so, không một lời trấn an.**

Nguyên văn: `expect(received).toContain(expected)` · `Expected substring: "Bản dịch gõ trong WKWebView thật."` · `Received string: ""`.

- 🔴 **Đúng spec này ĐÃ ĐỎ trong bộ trọn bộ ở lượt ② của chính lượt dev** *(§⑥, `8 / 9 spec`)* — tức **trước** khi tôi chạm một dòng nào. Một ca đã đỏ trước bản vá không thể là hồi quy của bản vá. Đây là vế mạnh nhất và nó là một phép so trực tiếp, không một suy luận.
- Xanh **2/2** khi chạy riêng — khuôn y hệt lượt ②/③ của lượt dev.
- ⚠️ **Nhưng BIỂU HIỆN khác lượt ②**, và tôi ghi ra thay vì làm phẳng: lượt ② trượt bằng `Couldn't find element for "pointerMove" action sequence` *(không tìm thấy phần tử)*; lượt này trượt bằng một **phép khẳng định thật** *(chữ không tới đĩa)*. Hai triệu chứng khác nhau ⇒ tôi nói được *"không do lượt vá"*, **không** nói được *"cùng một nguyên nhân"*.
- 🔵 **Một quan sát MỚI cho món nợ, đo được:** trước ca đỏ có **hơn 20** dòng `WARN tauri-service:window: Failed to get window states: Error: Tauri core.invoke not available after 5s timeout`. Cầu IPC của bàn đo **không lên** trong suốt spec đó. Chuỗi rỗng nhận được vì thế có một ứng viên chưa từng được nêu trong hai món nợ cũ — và nó là một mệnh đề về **bàn đo**, không về flush.

⚠️ **KHÔNG chấm "đã chẩn đoán"** *(luật sau Story 1.22)*. Ba lượt chạy cộng một chuỗi lỗi nguyên văn cộng một phép so với lượt ② — đủ để loại lượt vá khỏi danh sách nghi phạm, không đủ để đặt tên nguyên nhân. Ghi vào món nợ *"bộ e2e chập chờn"* đã có chủ, kèm quan sát `core.invoke` ở trên.

✅ **Ca 2.8 xanh ở CẢ HAI lượt trọn bộ và cả lượt riêng** — 3/3, kể cả ca đa-mảnh. Và `grid-empty-cell` *(2.5b)* xanh, tức lượt đổi render cột nguyên văn **không** làm hồi quy đường chuột của ô bản dịch.

## Nhật ký sprint-status

Gỡ nguyên văn từ `sprint-status.yaml` ngày 2026-08-19: tệp đó giữ TRẠNG THÁI, nội dung story thuộc về tệp này. Không sửa một ký tự.

```
  # 🔵 2026-08-16 — create-story: chuyen sang ready-for-dev. Story mang CHIN quyet dinh mo
  #   phai co chu ky cua Ice TRUOC dong ma dau tien (Task 0 chan moi task khac), va Task 0.4
  #   la mot CUA CHAN — khuon nay DA KICH HOAT THAT mot lan o 2.7 (AD-47 giao Winston).
  # 🔴 DIEU KIEN KHOI HANH: cay lam viec BAN, va thu ban KHONG phai tao tac cua story nay —
  #   sau tep sua chua commit la ban va code review cua Story 2.7 (`segment.rs` +
  #   `segment_contract.rs`: `trim()` hai ve cua phep so moc, chu ky thu MUOI cua Ice).
  #   Theo project-context.md:425-426: hoi Ice, commit rieng, TRUOC dong ma dau tien.
  # 🔵 STORY NAY CO THE TIEU 0 BUOC DI TRU — lan dau cua chuoi 2.5c/2.5d/2.6/2.7. Cot
  #   `retired_at` DA CO tu buoc 5 (SEGMENT_DDL, schema.rs:351), va deferred-work.md:1961-1968
  #   ghi thang rang no duoc dat som CHINH DE 2.8 khoi mo mot buoc thu hai. So ke tiep neu
  #   can van la **12** — do lai tu PROJECT_MIGRATIONS (schema.rs), dung doc dong nay.
  # ✅ BON THU DA DUNG SAN, dev KHONG duoc phat minh lai:
  #   - `core/segment/paragraph.rs` — ba ca bien cua AD-37 (`merged` :99 · `split_into` :111 ·
  #     `at_end_of_chapter` :81), Quyet dinh #6(b) cua 2.5d dung SAN CHO story nay. Doc-comment
  #     :88-98 ghi thang cai bay: lay co nguon cua cau cuoi roi suy "co dich chac cung vay"
  #     XOA quyet dinh ngat doan cua nguoi dung, va khong cong nao do.
  #   - Vach `ornament` cho hang ve huu — `editorSegments.ts:161` + CSS GridPanel.vue:1327-1329,
  #     dung tu 2.5b voi ghi chu "0 duong toi duoc tren du lieu that".
  #   - `retired_at` tren day (`config/segment.ts:75`) va hang rao tu choi `segment_retired()`
  #     (`commands/segment.rs:1276-1283`, 4 lenh ghi dang dung).
  #   - `SegmentRow` da la tuple struct tu 2.7 ⇒ tran 12 phan tu cua std khong con chan.
  # 🔴 XUAT XU KHONG PHAI QUYET DINH MO — AD-47 ④ (SPINE:715-717) da khai bang chu: moi manh
  #   cung mot gia tri ⇒ giu; BAT KY bat dong nao ⇒ `other`. Tach la ca tam thuong. Dev THI
  #   HANH, khong hoi lai. Va AD-47 ① buoc dat CA HAI (moc + xuat xu) trong cung thao tac —
  #   gop/tach nam trong danh muc DONG (project-context.md:505-512).
  # 🔴 CHIN QUYET DINH MO, hai cai co the de ra mot AD MOI:
  #   #1 gop HAI hay gop mot NHOM — AC1 va AC6 cua CUNG story noi hai dieu khac nhau, va nang
  #      luc chon nhieu HANG khong ton tai (`editorPanelState.ts:57` la Ref<number|null>;
  #      grep selectedIds|multiSelect|Set<number> tren src/ = 0). Lap lai #1 cua 2.5c.
  #   🔴 #2 TACH O DAU — diem chan that: dac ta buoc tach o COT NGUYEN VAN (epics.md:2552,
  #      EXPERIENCE.md:267) ma cot do KHONG `contenteditable`, KHONG `tabindex`
  #      (GridPanel.vue:1041-1049; contenteditable="true" viet cung chi o :1122, cot tgt)
  #      ⇒ khong co caret de cat. Task 1 (ban do WKWebView) CHAN task cai dat cua #2.
  #   #3 `target_text` cua segment moi — 8 AC im lang, ma AD-47 ③ liet gop/tach vao danh muc
  #      cac luot GHI `target_text`. Gop va tach KHONG doi xung: noi thi xac dinh, tach thi
  #      khong co phep chieu nao tu ban dich sang cho cat ben nguon.
  #   #4 MOC so sanh cho segment MOI — moc song o webview (mang `segments`, Quyet dinh #2(b)
  #      cua 2.7), ma segment moi mang mot `id` chua tung co trong mang do.
  #   🔴 #5 `is_omitted` khi gop mot cau DA CAT voi mot cau chua — AD-5 im lang, AD-47 ④ chi
  #      giai MOT cot. Day DUNG KHUON da sinh ra AD-47 ⇒ UNG VIEN CUA CHAN Task 0.4.
  #   #6 hang DA VE HUU co o lai trong luoi khong — do duoc: `read_open_chapter_segments`
  #      (segment.rs:788-791) KHONG co `WHERE retired_at IS NULL`, va `resolveSegmentRule`
  #      da co nhanh ornament ⇒ gop hai cau hom nay cho ra BA hang. Khong AC nao neu.
  #      UX-DR19 (epics.md:555) keo mot chieu, AD-5 (:109) keo chieu kia.
  #   #7 danh lai `ord` + co tieu buoc 12 khong — chua duong ma nao danh lai `ord`
  #      (grep 'SET ord' = 1 ket qua duy nhat, chinh luc INSERT). Cot khong UNIQUE, co chu y.
  #   #8 hop am phim — BA tai lieu noi BA dieu: epics/`EXPERIENCE.md:169`/settings.html deu
  #      `⌘M`+`⌘/`, nhung bang Phim `EXPERIENCE.md:267` viet `⌘T`. Ca hai hop am RANH hom nay
  #      (grep KeyM = 0; Slash chi o bang tra). Va command id co HAI ten: settings.html viet
  #      `editor.segment.merge`, con `commands/index.ts:977-980` — ICE KY 2026-08-14 — viet
  #      `editor.merge_segments`. Chot mot lan; doi ten ve sau la mo coi phim tat da gan.
  #   🔴 #9 dong bao + `⌘Z` — mockup ve ca hai, AC cua 2.8 doi CA HAI DEU KHONG (chung thuoc
  #      2.9). Va KHONG FR/AD/UX-DR nao chot mo hinh undo ⇒ chon dung undo la mot AD MOI.
  #      `StatusBar.vue:17-19` tu khai khe thong diep la mot Record DONG, khong co cho trong.
  # ⚠️ Bay da ghi trong story: (a) ba neo so hoc chi ap neu #7 sinh buoc 12 — im lang o day
  #   doc giong mot luot quen, nen phai ghi ra bang chu; (b) `insert_segments` KHONG tai dung
  #   duoc nguyen trang (viet cung `translation_origin = ''` va soi guong co nguon sang co
  #   dich, ca hai SAI cho segment sinh tu gop/tach); (c) `⌘M` da bi mockup tm-manage.html:128
  #   chiem cho Quan ly TM — va chua ai ghi no, trong khi xung dot `⌘⇧T` thi da duoc danh dau;
  #   (d) `EXPERIENCE.md:171` con mang ban UX-DR32 CU ("go de len ranh gioi"), epics.md da sua.
  # ⚠️ AD-32 la BAY SONG SINH: no la luat cho gop/tach CHUONG va noi NGUOC LAI (giu nguyen
  #   `segment.id`). Doc nham no thanh luat cua story nay thi moi AC hong ma ma van bien dich.
  # 🔴 AC5 (cap TM o lai nguyen) KHONG co duong san pham nao doi chung — bang TM chua ton tai.
  #   Dong bang CAU TRUC (khong cau SQL nao cham bang do) va ghi no, chu: Epic 7.
  # 🔵 Baseline khoi hanh: cargo test --locked 383/0/5 · segment_contract 103/103 ·
  #   vitest 133/133 (12 tep, do lai bang `npx vitest run`) · 11 cong npm · COMMAND_FLOOR
  #   san 37 / cong in 44 · e2e 8/8 spec. Do lai ca bay dong tu NGUON o Task 0.3.
  # 🔵 2026-08-17 — 2.8 chuyen sang in-progress (dev-story). Task 0 CHAN moi task khac: chin
  #   quyet dinh mo cho Ice ky, va Task 0.4 la mot CUA CHAN (#5 `is_omitted` va #9c undo deu
  #   co the de ra mot AD MOI — khuon nay DA KICH HOAT THAT mot lan o 2.7).
  #   🔵 DIEU KIEN KHOI HANH HET DUNG: story viet "cay lam viec BAN, sau tep chua commit".
  #   Do 2026-08-17: cay SACH, thu duy nhat chua theo doi la chinh tep story. Cac ban va code
  #   review cua 2.7 da vao commit `dfa9c95`. => Task 0.6 khong con viec.
  #   Baseline do TRUOC khi cham dong dau tien, tren HEAD dfa9c95: cargo test --locked 383/0/5 ·
  #   segment_contract 103/103 · vitest 133/133 (12 tep) · 9 cong doc-tep xanh · COMMAND_FLOOR
  #   san 37 / cong in 44 · buoc di tru ke tiep **12** (PROJECT_MIGRATIONS dich o to_version 11,
  #   schema.rs:849-911). KHOP ca bay dong cua bang §Dieu kien khoi hanh.
  #   🔵 MOT TIEN DE CUA STORY SAI SO ma dung ket luan (Task 0.5): grep selectedIds|multiSelect|
  #   rowSelection|Set<number> tren src/ story ghi 0, do duoc **2** — ca hai o wordBoundary.ts
  #   (offset ky tu trong mot chuoi, khong phai chon hang). Menh de "khong co chon nhieu hang"
  #   VAN DUNG. Dung bai hoc #3 cua chinh story, lan nay bay nam TRONG story.
  # ✅ 2026-08-17 — 2.8 XONG, chuyen sang `review`. Moi task tick tru HAI o `[⊘]` co ly do
  #   (dieu kien `neu` khong xay ra: buoc di tru 12 khong sinh, va #6 ky (b) nen khong go
  #   nhanh `ornament`). Mot dau [x] o do la mot loi khai sai.
  #   🔵 STORY DAU TIEN CUA CHUOI TIEU **0 BUOC DI TRU** (2.5c/2.5d/2.6/2.7 deu tieu mot buoc):
  #   `retired_at` da co trong SEGMENT_DDL tu buoc 5, dat som CHINH DE 2.8 khoi mo buoc thu hai.
  #   So ke tiep VAN LA **12**. Ba neo so hoc CO Y khong doi — ghi ra bang chu o Completion
  #   Notes ⓪, vi im lang o do doc giong mot luot quen (neo nay da sai BA lan lien tiep).
  #   Nghiem thu: 11 cong npm (9 doc-tep + check:scope + check:scope:bundled chay tay) · build ·
  #   vue-tsc · vitest **138/138** (13 tep) · cargo test --locked **399/0/5**.
  #   Baseline 383 + 133 => +16 Rust, +5 vitest. COMMAND_FLOOR 37 -> **38** (cong in 46).
  #   e2e: HAI luot tron bo, ghi ca hai. ① truoc luot lat #6: **9/9 spec, 13/13 ca** (13m13).
  #   ② sau luot lat: **8/9** — `editor-typing-flush` do voi nguyen van "Couldn't find element
  #   for pointerMove", tuc KHONG tim thay phan tu de bam, khong mot khang dinh nao ve luoi.
  #   ③ chay rieng spec do: **2/2** (2m21). => Khong phai hoi quy; khop khuon hai mon no "bo
  #   e2e chap chon" da co chu (:3093-3115 fixture khong reset state panel).
  #   ⚠️ KHONG cham "da chan doan" — luat sau 1.22 doi BAT NGUYEN VAN truoc, va thu toi co du
  #   de noi "khong phai hoi quy", KHONG du de noi "nguyen nhan la X".
  #
  # ✅ ICE KY **MUOI MOT** chu ky, khong chin: chin cua Task 0, cong hai chu ky LAT do chinh
  #   phep do de ra.
  #   #1(a) gop dung hai · #2(a) -> **LAT sang (e)** · #3(b) noi theo ngon ngu · #4(a) lenh tra
  #   hang moi day du · #5(a) bat ky manh nao da cat · #6(a) -> **LAT sang (b)** · #7(a) danh
  #   lai ord 1..N · #8 ⌘M/⌘/ + editor.merge_segments/editor.split_segment · #8③ 2.8 sua tai
  #   cho · #9(a) khong dong bao khong undo.
  #
  # 🔴 CUA CHAN TASK 0.4 NEU RA, ICE PHAN DINH KHONG KICH HOAT: chu ky #5(a) lap im lang cua
  #   AD-5 ve mot cot du lieu nguoi dung — dung khuon da sinh ra AD-47 o 2.7. Ice chot no nam
  #   TRONG bien do AD-5 => khong AD moi, story di tiep. Mon no ghi rieng (chu: Ice) vi hom nay
  #   nguon DUY NHAT phat bieu luat ay la mot ca test.
  #
  # 🔴 HAI CHU KY BI CHINH PHEP DO LAT, ca hai deu la thong tin MOI khong co trong bang duong:
  #   ① #2(a) "lay cho cat tu vung chon dang co o cot nguyen van" — ban do WKWebView (ba vong,
  #     `2-8-ban-do/`) do: KHONG cu chi chuot nao tao duoc vung chon o cot do (bam don "None"/0,
  #     keo 6 buoc "None"/0, keo sau khi tai lieu co tieu diem "None"/0); doi chung o ban dich
  #     "Caret" => thuoc tot. Hai gia thuyet ve ban do da bi loai tung cai. => Tien de KHONG TON
  #     TAI. Ice ky duong (e) — **san pham TU dat caret tu toa do cu bam**, dung khuon duong
  #     chuot ma 2.5b da phai dung cho o BAN DICH. Duong (e) KHONG co trong bang bon duong cua
  #     story; dev de xuat tu so do.
  #   ② #6(a) "loc hang ve huu o Rust" — dev NEU MOT DINH CHINH: bang duong cua chinh dev khai
  #     duong (c) cho nhanh `ornament` "mot duong toi that qua be mat lich su 2.6". SAI. Do lai:
  #     `resolveSegmentRule` co DUNG MOT noi goi (GridPanel.vue:214); be mat lich su doc
  #     `SegmentVersionRow`, khong dung vach le. => Luoi la nguon DUY NHAT cua nhanh do. Ice ky
  #     lai **(b)**: giu hang ve huu trong luoi, ve `ornament` mo. Gia da nhan: luoi PHINH theo
  #     so lan sua, VINH VIEN (gop hai cau cho BA hang).
  #
  # 🔴 MOT KHUYET TAT CUA MA **DA PHAT HANH** bi bat, va no to hon story nay:
  #   `document.caretPositionFromPoint` **KHONG TON TAI** tren WKWebView (typeof = "undefined"),
  #   va mot loi goi TRAN **NEM TypeError** — no khong tra `null`.
  #   `GridPanel.vue::placeCaretAtPoint` (Story **2.5b**, dang chay trong ban phat hanh) goi
  #   dung API do, tran, o DONG DAU => `onCellMouseUp` chet ngay tai do o MOI cu bam vao o ban
  #   dich, va `ensureCaretNextFrame` — thu ma doc-comment cua chinh no goi la "duong DUY NHAT
  #   chay duoc khi engine khong lam" — **CHUA BAO GIO CHAY**.
  #   ⚠️ Vi sao khong ai thay suot hai story: caret VAN HIEN (do: selectionType "Caret",
  #   rangeCount 1, activeElement = chinh o) — no den tu `cell.focus()` cong hanh vi mac dinh
  #   cua engine, KHONG tu duong va ma ba vong chan doan cua 2.3 va 2.5b da mua.
  #   `grid-empty-cell.e2e.mjs` xanh tren mot san pham ma NUA CO CHE cua no dang chet.
  #   ✅ Ice chot: SUA TRONG STORY NAY. Mot ham `caretPointAt` do nang luc (uu tien
  #   `caretRangeFromPoint`, thu WebKit that su co), hai noi goi. Hai spec cua 2.5b/2.5d xanh
  #   SAU luot va => `ensureCaretNextFrame` nay chay that lan dau, va khong lam hoi quy.
  #
  # 🔴 BA GIOI HAN CUA BAN DO do duoc, va MOT trong ba da NOI DOI dev ba vong lien:
  #   (1) `browser.keys(['Meta','/'])` giao `code: "/"`, khong `"Slash"` => hop am khong khop,
  #       0 command chay. Doi chung `⌘M` giao `code: "KeyM"` => khop.
  #   (2) KHONG giao duoc mot cu bam chuot toi cot nguyen van qua BA cach nham, trong khi cung
  #       lenh do tren o `[data-col="tgt"]` thi an.
  #   (3) 🔴 Listener chan doan cua CHINH BAN DO goi `caretPositionFromPoint` tran => no NEM
  #       truoc dong `push` => `chuot` rong o moi vong, va dev doc con so rong ay thanh "khong
  #       mot mouseup nao toi document" — mot menh de ve ENGINE dung tu mot cu nem cua BAN DO.
  #       Dung lop bay ma 2-5d-ban-do da dat ten: mot con so THAT, tra loi SAI cau hoi.
  #       Bai hoc: mot listener chan doan phai CHIU DUOC engine ma no dang do.
  #
  # 🔴 CON MOT MON CHO ICE, va no la mot cau hoi ve MOT PHIM TAT DANG SONG: `⌘/` co the la mot
  #   phim tat CHET tren ban phim that. Bo do khong phan biet duoc "driver gui sai `code`" voi
  #   "engine bao vay", theo cau tao. Ice da thu tay va bao "khong co gi xay ra" — nhung luot
  #   thu do dien ra KHI MA CON MANG khuyet tat `caretPositionFromPoint`, nen no khong tach
  #   duoc hai kha nang. Thu lai SAU ban va nay, tren mot cau da bam vao cot nguyen van.
  #
  # 🔴 LUOT LAT THU BA CUA #6, va no den tu MOT LUOT DUNG THAT — khong mot phep doc nguon nao
  #   thay duoc no. Ice bao sau khi `⌘/` chay duoc: "da tach ra 2 cau, nhung cau cu van ton tai
  #   va so thu tu van chiem, gay roi noi dung". => Chot cuoi: **LOC**.
  #   `read_open_chapter_segments` them `AND retired_at IS NULL`; `applyRegroup` GO hang ve huu
  #   khoi anh chup thay vi va chung vao.
  #   🔴 "Xoa" o day la AN KHOI LUOI, khong xoa khoi DIA — hang van nam trong project.db voi
  #   `retired_at` khac NULL, `read_segment_history` khong hoi cot do, nen AC4 con nguyen. Mot
  #   ca hop dong khoa CA HAI ve cung luc: luoi 3 -> 2, dia 3 -> 4.
  #   ⚠️ So thu tu tu sua theo, khong mot dong nao cho rieng no: luoi danh so bang CHI SO MANG,
  #   va `ord` tren dia da duoc chu ky #7(a) danh lai 1..N tu dau. Ca e2e nay doc THANG cot so
  #   nguoi dung nhin thay (`.cell-num`).
  #   🔴 DIEU DANG GHI NHAT khong phai quyet dinh, ma la: cai gia nay DA DUOC VIET RA BANG CHU
  #   TRUOC KHI KY (bang duong cua #6(b) co dung cau "luoi phinh theo so lan sua") va van khong
  #   du de thay. Ba ly le dung sau #6(b) deu VAN DUNG hom nay; ca ba cong lai thua mot luot
  #   nguoi that nhin vao mot Chuong that.
  #   🟡 He qua CHUA dong: nhanh `ornament` cua `resolveSegmentRule` nay KHONG con duong toi.
  #   KHONG go no — `ornament` la mot trong SAU gia tri vach ma UX-DR19 khai, va go la lam ma
  #   lech mot UX-DR dang dung (project-context.md:456-458). Mon no rieng, chu: Ice.
  #
  # 🟡 MUOI MOT mon no vao deferred-work.md, moi mon mot chu. Mot mon DA DONG bang cach noi
  #   tiep (khong xoa muc goc): "luoi phinh theo so lan sua" — dong boi chinh luot lat tren. Trong do hai mon DOI CHU chu khong
  #   dong: "chon nhieu hang" (2.5c -> mot story sau cua Epic 2) va "Auto-Lookup bang chuot o
  #   cot nguon chua co duong nghiem thu" (Ice chot: tach han thanh mot story ha tang e2e).
  # 🔵 Tai lieu sua tai cho kem 🔵 + ngay: EXPERIENCE.md:171 (ban UX-DR32 CU) · :267 (`⌘T` ->
  #   `⌘/`) · mockups/settings.html:276-277 (command id theo chu ky 2026-08-14 cua Ice).
  # ✅ 2026-08-17 — code review BA TANG (Blind Hunter · Edge Case Hunter · Acceptance Auditor,
  #   phien sach song song, cung cap mo hinh). 10 phat hien sau phan loai, 3 bac lam nhieu.
  #   Ice chot HAI quyet dinh; ca 8 ban va da ap. Story sang `done`.
  #   🔴 HAI LOI NANG NHAT NAM TRONG CUNG MOT HAM — `editorSegments.ts::sourceCutOffsetOf`,
  #   phep anh xa offset -> cho cat. Cung mot bieu hien: `⌘/` cat dung cho nguoi dung KHONG
  #   bam, im lang, tren du lieu AD-5 khong cho hoan tac, va khong cong nao do.
  #   ① `<rt>` (am Han Viet) bi dem la ky tu nguon — do bang happy-dom tren dung hinh dang DOM
  #     san pham: cu bam dang le cho 1 tra ve 5. 🔴 Kho DA DO va GHI RA dung cai bay do tu
  #     2026-08-07 (`SourceHanViet.vue` §"`ruby.textContent` GOP CA `<rt>`") — va ca test cua
  #     ham moi dung mot `<ruby>` KHONG CO `<rt>`, nen phep kiem KHONG THE do.
  #   ② Dem code unit UTF-16 trong khi Rust dem code point (`chars()`). Mot ky tu ngoai BMP
  #     (CJK Ext B, co that trong van ban Han co) day cho cat lech mot, va thuong VAN TRONG
  #     BIEN nen khong loi nao nem.
  #   🔴 AC7 ve "nhieu manh" DONG MOT NUA ma khong quyet dinh, khong chu ky, khong mot dong no
  #   — trong khi AC6 cung hinh dang thi co ca Quyet dinh #1, chu ky Ice, va mot muc 🟡 co chu.
  #   Ice ky 2026-08-17: DUNG da-manh ngay trong story nay, co che TICH LUY (moi cu bam them
  #   mot diem, `⌘/` cat het). Day tren doi `cut: number` -> `cuts: number[]`.
  #   ⚠️ Toi da neu lo ngai truoc khi thi hanh — co che gom nhieu diem cat la mot tuong tac
  #   CHUA tai lieu nao mo ta, dung cho Quyet dinh #1 va Story 2.5c da tu choi hai lan. Ice can
  #   va giu quyet dinh; hinh dang cu the do Ice chon tu ba phuong an kem so do.
  #   Sau lat: cargo 399 -> 401 · segment_contract 119 -> 121 · vitest 138 -> 141 · 9 cong xanh
  #   · build + vue-tsc xanh · 4 dot bien MA SAN PHAM, moi cai do dung mot ca cua no.
  #   e2e TRON BO 8/9 spec (13m12) — ca do la `editor-typing-flush`, thu DA DO trong bo tron bo
  #   o luot ② cua chinh luot dev, tuc TRUOC ban va; xanh 2/2 khi chay rieng. Ca 2.8 xanh 3/3
  #   ca hai luot, ke ca ca da-manh tren WKWebView that.
  #   🔵 So no: 3 mon chu 2.8 DONG TAI CHO (`→ ✅ ĐÃ ĐÓNG`) — luot dev chi noi them o cuoi tep
  #   va bo sot ca bon. Mon "dai cau" KHONG dong: 2.8 da xet va tu choi (chu ky #1(a)) ⇒ 🟡
  #   chuyen chu. Cong 7 mon moi tu luot ra, moi mon mot chu.
```

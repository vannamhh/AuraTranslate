---
baseline_commit: d813118c08dcaca04d479d330e4c663e58582062
---
# Story 2.5d: Ngắt đoạn của bản dịch

Status: done

**Covers:** FR134 · AD-46 · bước di trú **9**
**Epic:** 2 — Biên tập theo segment, một vòng dịch tay hoàn chỉnh
**Story trước:** 2.5c (Cắt bỏ câu khỏi bản dịch) — `done` 2026-08-15
**Story sau:** 2.6 (Lịch sử phiên bản segment và khôi phục)

---

## Story

As a **người dịch**,
I want **tách một đoạn nguồn dài thành hai đoạn trong bản dịch**,
so that **bản dịch có nhịp của tiếng Việt chứ không phải nhịp của bản gốc**.

---

## Acceptance Criteria

Chép nguyên văn từ `epics.md:2386-2409`. Số hiệu AC1–AC6 là của story này, dùng để tham chiếu trong Tasks.

**AC1 — `Enter` là xuống dòng trong ô**
**Given** con trỏ trong ô bản dịch
**When** người dùng bấm `Enter`
**Then** **xuống dòng trong ô**, ký tự lưu vào `target_text`

**AC2 — cờ đích mặc định soi gương cờ nguồn**
**Given** cờ kết đoạn của bản dịch
**When** một Chương được nhập
**Then** **mặc định bằng cờ nguồn** — bản dịch soi gương bản gốc cho tới khi người dùng đổi

**AC3 — ba ca biên của AD-37 áp y nguyên**
**Given** ba ca biên của AD-37 *(gộp → theo câu cuối · tách → theo mảnh cuối, các mảnh trước tắt · segment cuối Chương → tắt, luôn luôn)*
**When** áp cho cờ đích
**Then** **y nguyên**, không một ca nào xử lý khác

**AC4 — đọc dữ liệu đã lưu, không suy ra**
**Given** bất kỳ đường mã nào
**When** cần cấu trúc đoạn của bản dịch
**Then** đọc **dữ liệu đã lưu**, **không** suy ra từ nội dung nguồn

**AC5 — bước di trú 9**
**Given** lược đồ `project.db`
**When** thêm cột cờ đích
**Then** một `ALTER TABLE` đánh số **9**

**AC6 — gỡ hai lớp chặn `Enter`, đúng một chỗ**
**Given** hai lớp chặn `Enter` hiện có — `EditorPanel.vue:769` *(tầng `beforeinput`, `inputType === 'insertParagraph'`)* và `:842` *(tầng phím)*
**When** story này chạy
**Then** gỡ **ở ô bản dịch**, giữ nguyên ở mọi nơi khác
⚠️ Hai lớp đó được đặt ra **vì AD-37** — chúng đúng ở thời điểm viết. AD-46 là thứ mở khoá chúng, và **chỉ ở đúng một chỗ**.

---

## 🔴 Quyết định mở — Ice chốt TRƯỚC khi viết dòng mã đầu tiên

Sáu chỗ dưới đây có từ hai phương án hợp lệ và **đặc tả không chọn hộ**. Luật của dự án: *"Ice là người chốt các quyết định mở. Gặp một chỗ hai phương án đều hợp lệ: nêu cả hai kèm số đo, đừng tự chọn rồi đi tiếp — và cũng đừng loại một phương án chỉ vì nó đắt"* (`project-context.md:464-466`).

Dev agent **dừng ở đây** và trình sáu quyết định. Không tự chọn.

### Quyết định #1 — `Enter` chèn cái gì vào DOM: để engine làm, hay tự chèn `\n`?

Đây là quyết định nặng nhất của story, và nó **không** phải "xoá hai dòng `preventDefault`".

**Số đo có sẵn trong chính tệp sẽ sửa.** `GridPanel.vue:686-689` ghi lại một phép đo của Story 2.5b về nhánh ② *(chèn từ ngoài bàn phím)*: *"không có nhánh này thì **cả hai engine tiêm markup** — `<pre>`, `<span style>`, và trên WebKit cả `<div>` khối — cộng một `\n` thật vào `target_text`"*. Lượt `insertParagraph` của engine đi **đúng cùng đường tiêm markup đó**: nó dựng `<div>` hoặc `<br>` con bên trong ô.

**Và hệ quả đo được ngay trên đường ghi hiện có:** đường ghi đọc `cell.textContent` (`GridPanel.vue:770-774`, `reportEdit`). `textContent` của `<div>a</div><div>b</div>` là `"ab"` — **không một ký tự xuống dòng nào**. Tức nếu để engine làm, AC1 hỏng ở vế *"ký tự lưu vào `target_text`"* mà **không cổng nào đỏ**: DOM có hai dòng, đĩa có một chuỗi liền.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Gỡ hẳn nhánh ① của `onBeforeInput` (`:699-703`) — thả engine chạy `insertParagraph` mặc định | Rẻ nhất về số dòng. Nhưng mở lại **đúng** lớp lỗi tiêm markup mà nhánh ② tồn tại để chặn, và `textContent` nuốt mất ranh giới ⇒ AC1 hỏng im lặng. Hai engine cho hai hình dạng DOM khác nhau |
| **(b)** | Giữ `preventDefault()`, **tự chèn một text node `"\n"`** theo đúng khuôn nhánh ② (`:730-742`) rồi gọi `reportEdit(cell)` | DOM giữ **một** text node phẳng ⇒ `textContent` đọc ra `\n` thật ⇒ đĩa nhận đúng thứ màn hình hiện. Khuôn đã chạy và đã đo. Cái giá: phải tự đặt lại caret, và **bắt buộc** đi kèm Quyết định #2 *(không có `white-space` thì `\n` không vẽ ra dòng nào)* |
| **(c)** | Chèn `<br>`, rồi dựng một tầng chuyển đổi hai chiều `<br>` ⇄ `\n` ở `reportEdit` và `restoreEditedText` | Là cách trình duyệt tự làm nên caret cư xử tự nhiên. Nhưng dựng **hai** phép chuyển đổi phải luôn đồng ý với nhau, trên hai engine — đúng hình dạng "hai nguồn sự thật" mà `AD-37` và `AD-46` đều tồn tại để cấm |

⚠️ **Dù chọn đường nào cũng phải ĐO trên WKWebView thật trước khi tin.** Tiền lệ trực tiếp: `deferred-work.md:3012-3037` — `Backspace` ở offset 0 **không phát `beforeinput`** trên WebKit *(0 sự kiện; Blink thì có)*, và phép đo đó đã **lật** một tiền đề mà Story 2.5b đã viết ra bằng chữ. Không suy diễn hành vi `insertParagraph` của WebKit từ Blink.

### Quyết định #2 — `white-space` cho ô bản dịch: đây là điểm chặn thật, không phải một dòng CSS

**Số đo:** grep `white-space` trên toàn `src/` cho **0** kết quả trong `GridPanel.vue`. `.cell-tgt` là một `<div>` khối, mặc định trình duyệt là `white-space: normal` ⇒ mọi `\n` bị **gộp thành một khoảng trắng** lúc hiển thị. Tức AC1 có thể đúng trọn ở tầng dữ liệu *(đĩa có `\n`)* mà **màn hình không đổi một pixel**.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | `white-space: pre-wrap` trên `.cell-tgt` | Giữ **mọi** khoảng trắng đúng như đã gõ, gồm khoảng trắng đầu và cuối dòng. Bản dịch cũ trên đĩa có khoảng trắng thừa sẽ **hiện ra** — đó là sự thật, nhưng là một lượt đổi hình dạng của dữ liệu đã có |
| **(b)** | `white-space: pre-line` | Chỉ giữ xuống dòng, vẫn gộp khoảng trắng liên tiếp. Ít làm dữ liệu cũ đổi hình. Nhưng nó **gộp im lặng** thứ người dùng gõ ⇒ màn hình và `target_text` nói hai chuyện khác nhau, cùng lớp lỗi Quyết định #1(a) |

⚠️ Cổng `check:tokens` **không** soi `white-space` (`PURE_COLOR_PROPS`/`COMPOSITE_COLOR_PROPS`, `check-tokens.mjs:880-912`, không liệt thuộc tính này) ⇒ viết thẳng trong `<style scoped>` là hợp lệ, **không** cần token mới, **không** cần miễn trừ có tên.
⚠️ Cột nguyên văn **không** nằm trong quyết định này. `SourceHanViet.vue:825,880` đã có `pre-wrap` riêng; `.cell-src` thì không, và story này **không** có AC nào đòi đụng nó.

### Quyết định #3 — cơ chế người dùng ĐỔI cờ đích: sáu AC không AC nào nói

FR134 (`prd.md:462`) hứa *"mặc định soi gương bản gốc **cho tới khi người dùng đổi**"*, và AD-46 (`ARCHITECTURE-SPINE.md:669`) lặp lại y hệt. Nhưng **AC2 chỉ khai mặc định**, và **không AC nào của story này mô tả một thao tác đổi cờ**. Đã grep `EXPERIENCE.md` · `DESIGN.md` · `prd.md` · `epics.md` cho "đổi cờ" · "ngắt đoạn" · "gộp đoạn": bảng phím của `EXPERIENCE.md:262-268` có sáu hàng và **không hàng nào** là "đổi ranh giới đoạn".

Ràng buộc đã cố định (không phải chỗ chọn): nếu có thao tác thì nó **bắt buộc** là một command đăng ký trong `CommandRegistry` (AD-34; `ARCHITECTURE-SPINE.md:689` — *"Không thao tác nào chỉ tồn tại trong một handler chuột"*), id theo văn phạm `^[a-z0-9]+(\.[a-z0-9_]+)+$` (`registry.ts:125`), tiền tố miền `editor.`.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Story này **chỉ** lưu cột và mặc định soi gương. Quyền đổi ghi nợ có chủ | Đóng đúng sáu AC đã viết, không dựng một tương tác chưa ai đặc tả. Nhưng FR134 còn hở một vế, và cột mới thành **dữ liệu không ai đổi được** cho tới một story sau |
| **(b)** | Một lệnh bập bênh `editor.toggle_target_paragraph` | Một phím, một nhãn. ⚠️ Quyết định #3 của Story 2.5c đã **bác** hình dạng bập bênh một lần, đúng lý do này: *"nhãn của một phím bập bênh không nói được nó sắp làm gì"* (commit `073981d`) |
| **(c)** | Hai lệnh, khuôn `editor.omit_segment`/`editor.restore_segment` mà 2.5c vừa dựng | Nhất quán với tiền lệ gần nhất. Cái giá: hai id, hai nhãn, hai hợp âm phải chưa dùng — và `src/commands/index.ts:880-883` kiểm trùng hợp âm **trên toàn registry**, không theo chế độ |

🔴 Nếu Ice chọn (a): phần còn hở ghi vào `deferred-work.md` kèm chủ, **không** sửa `epics.md` cho khớp mã (`project-context.md:456-458`).

### Quyết định #4 — cờ đích hiển thị thế nào, khi "khoảng thở" là thuộc tính của HÀNG

**Số đo, và nó là một ràng buộc hình học chứ không phải một sở thích.** Khoảng thở của cờ nguồn hôm nay là `.cell.para-end { padding-bottom: 14px }` (`GridPanel.vue:1115-1118`), nhân ra **cả năm** cột (`:897,913,923,975,993`). Nhưng năm cột là năm `subgrid` chia **chung một tập track hàng** (`:1085-1090`), và `.cell` không khai `align-self` nên mặc định `stretch`. ⇒ Một `padding-bottom` đặt **chỉ** ở ô bản dịch vẫn làm **track hàng** cao lên, và bốn ô kia giãn theo. **Cấu trúc đoạn khác nhau giữa hai cột không biểu diễn được bằng khoảng thở.**

⚠️ Đây là **suy luận hình học từ mã đã đọc, chưa phải một phép đo**. Luật của kho: *"không đánh dấu đạt bằng suy luận"*. Ai đi đường nào cũng phải đo lại trên bản dựng thật trước khi tin — và tiền lệ đứng ngay đây: `deferred-work.md:3131-3162` đo được `subgrid` **ép ô bản dịch cao 388 px** theo cột nguyên văn, một cái giá mà nợ gốc **không** nêu.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Story này **không** hiển thị cờ đích. Nó là dữ liệu cho đường xuất (Epic 8) | Rẻ nhất, và trung thực với việc chưa ai đặc tả hình dạng. Nhưng một quyết định của người dùng **không nhìn thấy được** là đúng lớp lỗi mà FR133 sinh ra để chặn *(*"không biến thành một lỗ hổng im lặng"*)* |
| **(b)** | Một chỉ báo **phi hình học** ở ô bản dịch hoặc cột nhãn trạng thái *(một ký tự, một đường kẻ đáy đậm hơn)* | Nhìn thấy được mà không đụng chiều cao track ⇒ không lật phép đo 388 px. Cái giá: một từ vựng thị giác mới chưa có trong `DESIGN.md`, và `EXPERIENCE.md:99` gọi bảng vạch lề là *"tài nguyên đã tiêu hết"* |
| **(c)** | Đo trước, quyết sau: dựng một bàn đo hai engine cho ba hình dạng rồi mới chọn | Đúng luật của kho, và đây là story đầu tiên hai cấu trúc đoạn cùng tồn tại. Cái giá: một Task đo trước Task hiển thị |

### Quyết định #5 — tên cột, kiểu cột, và cách backfill "bằng cờ nguồn"

**Số đo:** `DEFAULT` của SQLite **phải là hằng** — nó không nhận `DEFAULT is_paragraph_end`. Mà AC2 đòi backfill **bằng cờ nguồn**, tức một giá trị **theo hàng**. Hai bước 6 và 7 đều ghi lại ràng buộc này tại chỗ (`schema.rs:367-370`, `:409-414`).

**Đường thoát đã có sẵn và đã có tiền lệ:** `migrate` chạy `tx.execute_batch(m.sql)` (`schema.rs:739`) ⇒ một hằng DDL chứa **nhiều câu** ngăn bằng `;` chạy trọn trong **một** giao dịch. `SEGMENT_STATUS_AND_VERSION_DDL` (bước 7) đã dùng đúng cơ chế đó cho hai câu DDL.
⚠️ Nhưng tiền lệ đó là **DDL + DDL**. Bước 9 cần **DDL + DML** (`ALTER TABLE …; UPDATE segment SET <cột mới> = is_paragraph_end;`) — chưa bước nào của kho làm vậy, và `schema.rs:417-424` khai một lằn ranh: thứ Quyết định #4 của Story 2.1 cấm nhét vào một bước là một **quy tắc nghiệp vụ**. Một lượt backfill theo hàng nằm đâu giữa hai vế đó là một chỗ phải ký, không phải một chỗ tự quyết.

| Đường | DDL | Ghi chú |
| --- | --- | --- |
| **(a)** | `ALTER TABLE segment ADD COLUMN target_paragraph_end INTEGER NOT NULL DEFAULT 0;` **+** `UPDATE segment SET target_paragraph_end = is_paragraph_end;` trong **cùng** hằng, cùng giao dịch | Đúng AC2 cho **21** `project.db` thật đang có dữ liệu. Tên cột là **đúng cái tên** mà `schema.rs:284` viết ra để cấm — xem §Dev Notes, dòng đó phải sửa cùng lượt |
| **(b)** | Chỉ `ALTER TABLE … DEFAULT 0`, backfill làm ở một đường Rust riêng sau khi mở kho | Giữ bước di trú thuần DDL. Cái giá: một lượt ghi **ngoài** giao dịch di trú ⇒ có một cửa sổ mà đĩa mang cờ đích sai, và không `PRAGMA user_version` nào nói ra điều đó |
| **(c)** | Tên khác: `is_target_paragraph_end` *(giữ tiền tố `is_` như `is_paragraph_end`, `is_omitted`)* | Thuần đặt tên. ⚠️ Chọn xong thì **dùng một từ**, không đặt từ thứ hai ở tầng TS hay tầng lệnh — đúng luật Quyết định #5 của 2.5c |

🔴 Dù chọn gì: **không** `CHECK` trong DDL — giá trị hợp lệ cưỡng chế ở tầng Rust, đúng khuôn `status`, `is_omitted` và `chapter.status` (`schema.rs:400-402`).

### Quyết định #6 — AC3: hai trong ba ca biên **không có bề mặt** để áp

**Số đo:** `grep "fn merge_segments\|merge_segment\|MergeSegment"` trên `src-tauri/src/**` cho **0** kết quả. Story 2.8 *(gộp và tách segment tường minh)* là `backlog`. Ba ca biên của AD-37 hôm nay đứng như sau:

| Ca | Có mã thi hành? | Ở đâu |
| --- | --- | --- |
| Segment cuối Chương → cờ tắt, luôn luôn | **Có** | `split.rs:258-263`, test `the_last_segment_never_ends_a_paragraph` |
| Tách → theo mảnh cuối, các mảnh trước tắt | **Có, nhưng chỉ ở đường NHẬP** | `split.rs:377-390` (`mark_paragraph_end`). Đường tách **do người dùng gọi** thì chưa tồn tại |
| Gộp → theo câu cuối | **Không** — mới chỉ là một bảng trong doc-comment `split.rs:168-181`, ghi sẵn *"Chủ: Story 2.8"* | — |

⚠️ Nếu cờ đích **mặc định bằng cờ nguồn lúc nhập** (AC2) thì ba ca biên đúng cho cờ đích **theo dẫn xuất**, không cần một dòng mã thứ hai. Vế còn hở là ngày người dùng đã **đổi** cờ đích rồi mới gộp hoặc tách — lúc đó hai cờ khác nhau và bảng ba ca phải chạy **hai lần**.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Khẳng định bằng test hợp đồng rằng cờ đích = cờ nguồn lúc nhập ⇒ ba ca biên đúng theo dẫn xuất. Ghi nợ có chủ **Story 2.8** cho vế gộp/tách | AC3 đóng **một nửa** 🟡, đúng và trung thực. Phụ thuộc người sau đọc sổ nợ |
| **(b)** | Dựng sẵn một **hàm thuần** ở `core/segment/` áp bảng ba ca cho **một cặp cờ**, + test hợp đồng. Story 2.8 chỉ việc gọi | Khuôn đã có: Quyết định #2(b) của 2.5c dựng `core::segment::omit` đúng cách này cho một nghĩa vụ chưa có bề mặt tiêu thụ. Cái giá: ~1 hàm + ~2 test, cho một đường chưa ai gọi |

---

## Tasks / Subtasks

> Task 0 chạy **trước** mọi task khác và **chặn** chúng.

- [x] **Task 0 — Trình sáu quyết định mở cho Ice** (AC1, AC2, AC3, AC5, và hai chỗ hình dạng)
  - [x] 0.1 Trình Quyết định #1–#6 ở mục trên, kèm số đo đã ghi. Không tự chọn đường nào
  - [x] 0.2 Ghi chữ ký của Ice vào `§Dev Agent Record` kèm ngày
  - [x] 0.3 Nếu một chữ ký làm hẹp phạm vi *(ví dụ #3 đường (a), #6 đường (a))*: ghi phần còn hở vào `deferred-work.md` **kèm chủ**, ngay lúc ký. Không sửa `epics.md` — ⇒ **không chữ ký nào làm hẹp phạm vi**: `#3(c)` đóng vế *"cho tới khi người dùng đổi"* của FR134, `#6(b)` dựng sẵn hàm thuần thay vì hoãn. Hai món nợ **có sẵn từ trước** vẫn giữ chủ cũ *(AC3 vế gộp/tách → Story 2.8 · AC4 vế đường xuất → Epic 8)*
  - [x] 0.4 Đo baseline **trước khi chạm dòng đầu tiên** và ghi vào story: `cargo test --locked` *(ghi chép nói **347/0/5**)* · `npm run test` *(ghi chép nói **101/101**)*. Số lệch thì **dừng và nói**, đừng sửa cho khớp — ⇒ **khớp cả hai**, ghi ở `§Baseline`

- [x] **Task 1 — Bàn đo: `insertParagraph` trên WKWebView thật** (AC1, chặn Task 3)
  - [x] 1.1 Dựng một bàn đo trong `2-5d-ban-do/` theo khuôn `2-5b-ban-do/` và `2-5c-ban-do/`, chạy trên webview thật *(`npm run test:e2e` với một spec riêng, hoặc bàn đo tay theo khuôn `2-5b-ban-do/do-hang-va-hieu-nang.e2e.mjs`)* — ⇒ `insertparagraph-wkwebview.e2e.mjs` + `README.md`, và nó chạy trên **lưới thật** chứ không trên hình dạng chép tay
  - [x] 1.2 Đo **năm** mệnh đề, mỗi mệnh đề một số, trên **cả hai** engine nếu tới được — ⇒ năm mệnh đề **cộng hai** phép đo phát sinh; vế Blink là **khoảng mù có tên** *(Blink chỉ tới qua WebView2/Windows — nếu không tới được thì ghi **khoảng mù có tên**, đúng khuôn `deferred-work.md:3155-3157`)*:
        ① `Enter` trong một ô có chữ phát `beforeinput` với `inputType === 'insertParagraph'` hay `'insertLineBreak'`?
        ② Nếu **không** `preventDefault`, engine dựng cái gì trong DOM — `<div>`, `<br>`, hay một text node `\n`?
        ③ `cell.textContent` sau lượt đó đọc ra gì?
        ④ Tự chèn một text node `"\n"` rồi `setPosition` sau nó: caret có đúng chỗ không? *(Đây là vế đo cho Quyết định #1(b) — `setPosition`, **không** `addRange`, lý do ở `GridPanel.vue:331-336`)*
        ⑤ `white-space: pre-wrap` trên `.cell-tgt`: ô có vẽ ra hai dòng không, và chiều cao **track hàng** đổi bao nhiêu px?
  - [x] 1.3 🔴 **LUẬT DỪNG.** Nếu ba vòng chẩn đoán liên tiếp bị phép đo bác thì **dừng**, báo Ice, 2.5d quay về `backlog`. Đây là luật đã cứu Story 2.4 khỏi sản xuất một bảng số vô nghĩa — ⇒ **KHÔNG kích hoạt.** Bốn vòng chạy, **0** vòng bị bác; ba lần đổi thước là *"số thật, trật câu hỏi"*, một loại khác hẳn. Phân biệt ghi ở `§Debug Log Ⓐ`
  - [x] 1.4 ⚠️ Bảo đảm **cổng 1420 trống** trước khi chạy — ⇒ 1420 trống; **4445 bị `gdrive-su` PID 48486 giữ** ⇒ đặt `TAURI_WEBDRIVER_PORT=4467`, **không** giết tiến trình của Ice: `wdio.conf.mjs::devServerIsUp()` chỉ hỏi `res.ok`, một Vite hấp hối vẫn trả 200 và làm **7/7 spec đỏ oan** (`deferred-work.md:3274-3330`, chưa vá). Nếu **4445** bị chiếm: đặt `TAURI_WEBDRIVER_PORT`, **không** giết tiến trình của người dùng (`deferred-work.md`, bài học ④ của 2.5b)
  - [ ] 1.5 🔴 **CÒN MỘT MÓN CHO ICE** — gõ tiếng Việt **bằng bộ gõ thật** rồi bấm `Enter` giữa câu, trên máy thật. Không bộ chạy test nào của dự án mô phỏng được một bộ gõ tiếng Việt (`EXPERIENCE.md:271`), và `Enter` là **phím chốt dấu** của Telex

- [x] **Task 2 — Bước di trú 9** (AC2, AC5)
  - [x] 2.1 Thêm một hằng `SEGMENT_TARGET_PARAGRAPH_END_DDL` *(tên theo Quyết định #5)* trong `src-tauri/src/core/store/schema.rs`, cạnh `SEGMENT_OMITTED_DDL` (`:460-511`)
  - [x] 2.2 Doc-comment theo đúng khuôn ba hằng trước: vì sao số **9**, vì sao `ALTER TABLE` chứ không sửa `SEGMENT_DDL` *(vết sẹo số 4)*, và — mới ở bước này — **vì sao một câu `UPDATE` đi cùng** nếu Ice chọn #5(a)
  - [x] 2.3 Thêm `Migration { to_version: 9, sql: … }` vào cuối `PROJECT_MIGRATIONS` (`schema.rs:580-618`), kèm comment một dòng nêu Story + lý do số
  - [x] 2.4 Cập nhật khối `🔵 CẬP NHẬT` ở doc-comment đầu `PROJECT_MIGRATIONS` (`schema.rs:513-531`): đích 8 → **9**, bảy bước → **tám** bước
  - [x] 2.5 Đổi **tên hàm** và mảng của `the_project_migration_set_reaches_eight_through_seven_steps` (`segment_contract.rs:497-503`, hiện khẳng định `vec![1,2,3,5,6,7,8]`). ⚠️ Tên hàm này **mang số hiệu** và vì thế sai lại ở **mỗi** story thêm một bước — 2.5c đã gỡ số khỏi bốn tên khác vì đúng lý do này nhưng để sót tên này. Cân nhắc gỡ số luôn
  - [x] 2.6 🔴 Nâng fixture `STEP_NINE` của `a_project_database_newer_than_the_app_is_refused_and_never_written_to` (`segment_contract.rs:1104-1164`, `to_version: 9` tại `:1128`) lên **10**. Không nâng thì cổng AD-30 *(db mới hơn app bị từ chối mở)* **vẫn xanh** mà chạy trên một db **không** mới hơn app — chết lâm sàng. Doc-comment tại `:1110-1115` đã ghi luật *"số của fixture phải luôn là `target + 1`"*; đây là lượt lặp lại thứ hai của nó
  - [x] 2.7 Rà `pinned_contract.rs` — 2.5c đã dời hai hằng neo 7/8 → 8/9; lượt này 8/9 → **9/10**
  - [x] 2.8 Test hình dạng cột theo khuôn `a_fresh_project_database_carries_an_is_omitted_column_with_the_shape_ice_signed` (`segment_contract.rs:934`): đọc `pragma_table_info('segment')` khẳng định kiểu, `notnull`, `dflt_value`, và đọc `sqlite_master.sql` khẳng định **không** `CHECK`
  - [x] 2.9 🔴 Test **backfill** theo khuôn `a_project_database_at_version_seven_migrates_up_and_no_old_row_is_omitted` (`:1004`), nhưng mệnh đề khác hẳn: dựng fixture ở phiên bản 8 có **cả hai** loại hàng *(`is_paragraph_end` bật và tắt)*, di trú lên 9, khẳng định cờ đích **bằng cờ nguồn từng hàng** — không phải "mọi hàng nhận 0". Fixture dựng từ các bước **THẬT**, không chép tay DDL
  - [x] 2.10 ⚠️ Dự kiến **có** ca đỏ ngoài danh sách: mọi ca neo vào *"đích là 8"* và mọi ca đếm số cột của `SegmentRow` *(11 → 12)*. 2.5c gặp **bốn** ca như vậy. Ghi ra ở `§Debug Log References`, đừng sửa im lặng

- [x] **Task 3 — `Enter` là xuống dòng** (AC1, AC6)
  - [x] 3.1 🔴 **Ba mốc dòng của AC6 trỏ vào một tệp KHÔNG CÒN TỒN TẠI.** `EditorPanel.vue` bị xoá ở Story 2.5b (commit `ca33072`). Hai lớp chặn hôm nay ở `src/panels/GridPanel.vue`: nhánh ① của `onBeforeInput` (`:699-703`) và `onEditKeydown` (`:764-767`). Sửa đúng hai chỗ đó
  - [x] 3.2 ⚠️ Vế *"giữ nguyên ở mọi nơi khác"* của AC6 **đã đúng theo cấu trúc**, và phải giữ cho nó đúng: `targetCellOf` (`:316-319`) chỉ khớp `[data-col="tgt"]`, và cả ba handler đăng ký trên **cột bản dịch** (`:955-957`). Không nới phạm vi của `targetCellOf`
  - [x] 3.3 Cài theo chữ ký #1. Nếu là (b): chèn text node `"\n"` theo khuôn nhánh ② (`:730-742`) — `range.deleteContents()` → `insertNode` → `setStartAfter` → `collapse` → `setPosition`, rồi gọi `reportEdit(cell)` **bằng tay** *(vì `preventDefault()` đã cắt lượt `input` của engine — lý do ghi ở `:743`)*. ⚠️ Mọi thành viên `window.*`/`document.*` phải nằm trong danh sách cho phép của `check:layout`; `document.createTextNode` và `window.getSelection` đã có mặt *(dùng ở `:720,737`)*, một API Selection/Range **mới** thì chưa
  - [x] 3.4 🔴 **`event.isComposing` đứng TRƯỚC mọi nhánh khác** trong `onEditKeydown`, cùng dòng và cùng lý do `keys.ts:506`. Một lượt commit composition của bộ gõ tiếng Việt phát `keydown` mang `code` vật lý; **ăn nó là ăn mất chữ**. Dòng `if (event.isComposing) return` (`:765`) **không được** chạm
  - [x] 3.5 ⚠️ Gỡ `preventDefault` ở tầng phím làm `Enter` trần **rơi tiếp** xuống global keymap. Đã đọc: `keys.ts:510` bỏ qua hợp âm không mod khi `isTypingZone(event.target)`, và `isTypingZone` (`:434-439`) trả `true` cho `el.isContentEditable === true` ⇒ ô bản dịch **được** che. **Đo lại** thay vì tin dòng này — `Mod+Enter` (`editor.confirm_segment`) vẫn phải chạy, và `Enter` trần vẫn phải **không** ký câu nào
  - [x] 3.6 Sửa `tests/frontend/editorTypingZone.test.ts:168-191` — ca *"`Enter` bị CHẶN trong vùng gõ"*. Nó sẽ đỏ, và **đúng vai**: mệnh đề của nó vừa bị AD-46 lật. Viết lại thành mệnh đề mới, **giữ** vế *"không `data-segment-id` nào nhân đôi"* *(xuống dòng trong ô **không** tách câu)*
  - [x] 3.7 Thêm ca vitest cho đường ghi: một lượt `Enter` ⇒ `noteEditorEdit` nhận chuỗi **có** `\n`. ⚠️ `happy-dom` **không phải** WebKit — ca này canh **đường ghi**, không canh hành vi engine. Hành vi engine thuộc Task 1

- [x] **Task 4 — `\n` sống sót từ DOM tới đĩa và ngược lại** (AC1)
  - [x] 4.1 Rà `reportEdit` (`GridPanel.vue:770-774`) đọc `cell.textContent` — xác nhận nó mang `\n` sau lượt chèn của Task 3
  - [x] 4.2 Rà `restoreEditedText` (`:598-612`) ghi `el.textContent = text` — một chuỗi có `\n` đi ngược vào DOM thành **một** text node, đúng hình dạng cần
  - [x] 4.3 Rà đường ghi Rust: `save_segment_targets` (`segment.rs:450-545`) ghi `target_text` **nguyên văn**, không `trim`, không `replace` — đã đọc và xác nhận. Thêm một ca hợp đồng khẳng định `\n` đi trọn một vòng đĩa
  - [x] 4.4 ⚠️ `confirm_segment` (`segment.rs:750`) từ chối khi `target_text.trim().is_empty()`. `"\n"` trim ra rỗng ⇒ một ô chỉ có xuống dòng **vẫn** bị từ chối ký. Đó là hành vi **đúng**; thêm một ca canh nó thay vì để một story sau "sửa" nhầm
  - [x] 4.5 `white-space` theo chữ ký #2, đặt trên `.cell-tgt` trong `<style scoped>` của `GridPanel.vue`

- [x] **Task 5 — Cột mới đi qua dây IPC** (AC2, AC4)
  - [x] 5.1 Thêm trường vào struct `ChapterSegment` (`segment.rs:164-174`). **Không** thêm `#[serde(rename_all)]` — tên trả về giữ `snake_case`
  - [x] 5.2 Thêm cột vào câu `SELECT` của `read_open_chapter_segments` (`segment.rs:326-329`) và phép đọc/ép kiểu `INTEGER → bool` (`:331-347`)
  - [x] 5.3 🔴 Test hợp đồng theo khuôn `the_load_command_carries_the_is_omitted_column_over_the_wire`. **Đây là cổng chống một lỗi ĐÃ XẢY RA HAI LẦN SUÝT BA:** bản đầu Story 2.5 quên `status` ở đúng hai chỗ 5.1/5.2 ⇒ `undefined` phía webview, **74/74 test frontend vẫn xanh**, chỉ e2e bắt được. Nguyên vụ ghi ở `segment.rs:144-153`
  - [x] 5.4 ⚠️ `insert_segments` (`segment.rs:96-117`) — nếu Ice chọn #5(b) *(backfill ngoài giao dịch di trú)* thì câu `INSERT` phải set cờ đích tường minh `= segment.is_paragraph_end`. Hai chỗ gọi: `project.rs:153` (`create_work`) và `segment.rs:279` (`split_chapter_into_segments`) — **cả hai**
  - [x] 5.5 Thêm trường vào type `ChapterSegment` phía TS (`src/config/segment.ts:66-102`), **snake_case**, khớp đúng tên trên dây
  - [x] 5.6 *(hệ quả của 5.5)* **Ba** fixture chép tay phía frontend sẽ đỏ dưới `vue-tsc`, và đó **đúng vai** — đây là lớp lỗi mà Story 2.5 phải nhờ e2e mới thấy: `tests/frontend/support/segmentFixture.ts:42-78` *(ba object)* · `tests/frontend/segmentNavigation.test.ts:142-154` *(hàm `row()`)* · `tests/frontend/editorSegmentRule.test.ts:23-35` *(hàm `segment()`)*

- [x] **Task 6 — Ba ca biên AD-37 cho cờ đích** (AC3 — hình dạng theo Quyết định #6)
  - [x] 6.1 Test hợp đồng: một Chương vừa nhập có cờ đích **bằng** cờ nguồn từng hàng, gồm cả hàng cuối *(cả hai cờ **tắt**)*
  - [x] 6.2 Nếu Ice chọn #6(b): hàm thuần ở `core/segment/` áp bảng ba ca cho **một cặp cờ**, + test. 🔴 Logic ở **Rust**, không một `v-if` ở Vue (AD-1)
  - [x] 6.3 Cập nhật bảng ba ca trong doc-comment `split.rs:168-181` — nó hiện khai *"Chủ: Story 2.8"* cho hai ca, và sau story này nó phải nói rõ bảng chạy cho **hai** cờ
  - [x] 6.4 Ghi nợ có chủ **Story 2.8** cho vế gộp/tách khi hai cờ đã khác nhau

- [x] **Task 7 — Hai doc-comment trong mã nói NGƯỢC với AD-46** (AC4)
  - [x] 7.1 🔴 `schema.rs:283-286` khai nguyên văn *"**Một** cột, dùng chung cho nguyên văn và bản dịch; **không** `source_paragraph_end`/`target_paragraph_end`"*. Story này thêm **đúng** cột thứ hai đó. Sửa dòng này, dẫn AD-46, và ghi rõ AD-37 **vẫn sở hữu** cờ nguồn
  - [x] 7.2 🔴 `split.rs:118-119` khai *"không `source_paragraph_end`/`target_paragraph_end`"*. Cùng lượt sửa, cùng lý do
  - [x] 7.3 ⚠️ **Không** sửa `ARCHITECTURE-SPINE.md`. AD-46 (`:658`) khai bằng chữ *"AD-37 **không sửa một chữ**"* — câu *"một cờ duy nhất dùng chung"* ở `AD-37 §Rule` (`:443`) ở lại **có chủ ý**, và AD-46 là thứ nới nó. Đây khác hẳn hai doc-comment trên: chúng mô tả **mã**, và sau story này chúng **sai về mã**
  - [x] 7.4 Rà xem có đường mã nào **suy** cấu trúc đoạn bản dịch từ nội dung nguồn không *(một `split('\n')` lúc render)*. `GridPanel.vue:1108-1109` đã cảnh báo lớp lỗi này và ghi rằng nó **đi qua mọi cổng**. Grep `split('\n')` · `split("\n")` trên `src/**` và `src-tauri/src/**`; nếu sạch, ghi số 0 vào `§Debug Log References` thay vì im lặng
  - [x] 7.5 Cập nhật `src/panels/README.md` nếu story thêm một khái niệm mới

- [x] **Task 8 — Hiển thị cờ đích** (theo chữ ký #4 — cả task này có thể **không tồn tại** nếu Ice chọn (a))
  - [x] 8.1 🔴 **Không có phần tử "hàng" để gắn class.** Năm cột là năm con của `.grid` (`:889`), mỗi cột `grid-row: 1 / -1` + `grid-template-rows: subgrid` (`:1085-1090`). Một hàng chỉ là track thứ *i*. Khuôn có sẵn để chép: `.cell.para-end` lặp ở cả năm `v-for`
  - [x] 8.2 ⚠️ **Không** thêm giá trị vào `SEGMENT_RULE_VALUES` (`editorSegments.ts:69-76`) và **không** dựng khối `.rule-<x>` mới. Kiểm I (`check-commands.mjs:2116-2269`) đối chiếu **ba chiều** và **cả chiều ngược lại** — một `.rule-<x>` lạ trong CSS làm cổng FAIL
  - [x] 8.3 ⚠️ Nếu cần một màu: **không** `ornament`. `check-tokens.mjs:1300-1334` cấm nó làm màu chữ *(đo 2026-08-15: 2,44 sáng / 2,64 tối, trượt sàn AA 4,5)*, và đây là lượt thứ **năm** `DESIGN.md` §components chỉ vào một token cổng cấm. Đường đã giải: `on-surface-variant` (5,60 / 5,56)
  - [x] 8.4 ⚠️ **Không** `opacity` trung gian cho "mờ đi": `DESIGN.md:230` + Kiểm D (`check-tokens.mjs:1345-1396`). Đừng nhét miễn trừ để cổng hết đỏ

- [x] **Task 9 — Lệnh đổi cờ đích** (theo chữ ký #3 — task này **không tồn tại** nếu Ice chọn (a))
  - [x] 9.1 Hàm thuần Rust nhận `Option<&OpenWork>`, ghi cờ. Vỏ `#[tauri::command]` **mỏng** trong `pub mod wire`, lấy `State` qua **`try_state`**, không `state()`
  - [x] 9.2 🔴 Ghi **rời rạc**: một `open.store.write(|tx| …)`, một transaction — khuôn `set_segment_omitted` (`segment.rs:868-943`). **Không** định tuyến qua bộ đệm gõ / `saveSegmentTargets`: một thao tác người dùng *thấy đã xong* nằm chờ tới 5 giây rồi biến mất nếu app sập (`project-context.md:520-522`)
  - [x] 9.3 Từ chối có hình dạng: segment không tồn tại · segment đã về hưu. Dùng lại `SegmentNotFound` · `SegmentRetired` (`core/i18n/mod.rs:209,220`). Khoá `err.segment.*` mới **chỉ** khi có nhánh từ chối thật sự mới, và phải qua `message_keys!`
  - [x] 9.4 Đăng ký command trong `installCommands()` ở `src/commands/index.ts` — **chỉ ở đó** *(một lượt HMR sẽ gọi lần hai và `register()` ném vì id trùng)*. Nhãn vào `vi.json` khoá `command.<id>`
  - [x] 9.5 ⚠️ Hợp âm phải chưa dùng — `index.ts:880-883` kiểm trùng **trên toàn registry**, không theo chế độ. Đã dùng: `Mod+Enter` · `Alt+ArrowDown` · `Mod+Alt+X` · `Mod+Alt+R`
  - [x] 9.6 ⚠️ Nâng `COMMAND_FLOOR` (`check-commands.mjs:235`) kèm một dòng ghi **số thật mới**. Comment tại chỗ hiện ghi *"35 command (sau Story 2.5b)"* và **đã hết đúng** — 2.5c thêm hai lệnh. **Đo lại, đừng chép**
  - [x] 9.7 Adapter IPC ở tầng TS **không bao giờ ném**: một `invoke`, một `try/catch`, trả hình dạng ba trạng thái
  - [x] 9.8 Ảnh chụp hiển thị dựng bằng **mảng mới** *(trải phần tử cũ)* — `shallowRef` không theo dõi sửa tại chỗ (`editorPanelState.ts:704-709`)

- [x] **Task 10 — Nghiệm thu**
  - [x] 10.1 `npm run check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:layout` · `check:dict` · `check:dict-manifest` · `check:lint` · `check:gates`, **cộng** `check:scope` + `check:scope:bundled` chạy tay *(cần **cổng 1420 trống**)*
  - [x] 10.2 `npm run test` (vitest) · `npm run build` *(`vue-tsc` hai lượt + `vite build`)* · `cargo test --locked`
  - [x] 10.3 e2e chạy tay. Thêm một ca vào `e2e/specs/editor-typing-flush.e2e.mjs` *(hoặc một spec mới)*: gõ, `Enter`, gõ tiếp, flush, rồi đọc lại qua `read_open_chapter_segments` và khẳng định `target_text` **mang `\n`**. 🔴 Đây là đường duy nhất bắt được lớp lỗi *"DOM có hai dòng, đĩa có một chuỗi"* — vitest với fixture chép tay **không đủ**
  - [x] 10.4 ⚠️ **Đo lại chiều cao hàng** nếu Task 4.5 hoặc Task 8 chạy. `deferred-work.md:3131-3162` đo `subgrid` ép ô bản dịch cao **388 px** theo cột nguyên văn; story này thêm áp lực **ngược chiều** *(ô bản dịch nhiều dòng đẩy track cao lên)* — một chiều đo mà phép đo cũ **chưa tính**. Ghi số, đừng suy luận
  - [x] 10.5 ⚠️ **Đo lại độ trễ dời con trỏ** nếu Task 8 thêm `:class` vào các `v-for`. 2.5b đo **706–770 ms** trên 9.850 câu — vượt trần NFR2 (50 ms/frame) **~15 lần**, còn hở, chủ là Story 2.4 (`deferred-work.md:3164-3194`)
  - [x] 10.6 Mỗi vế không nghiệm thu được ở tầng này ⇒ `deferred-work.md` kèm chủ. **Không tự chấm đạt**

---

## Dev Notes

### Đọc trước khi viết dòng đầu tiên

`_bmad-output/project-context.md` — 130 luật. Ba mục sát story này: §Critical Don't-Miss Rules (*"Dữ liệu người dùng — chỗ hỏng là VĨNH VIỄN"*), §Testing Rules *(bốn đường nghiệm thu, bốn vai không chồng nhau)*, §Code Quality *(chú thích nói **lý do**, kèm **phép đo**, không sở thích)*.

### 🔴 Story này KHÔNG thêm phụ thuộc nào

Mọi thứ cần đã có: `rusqlite` cho cột mới · `CommandRegistry` cho lệnh · `vitest` cho test frontend · khuôn `set_segment_omitted` cho đường ghi · khuôn nhánh ② của `onBeforeInput` cho lượt chèn text node.

⚠️ Và có một cám dỗ **cụ thể** ở story này: *"một ô soạn thảo nhiều dòng thì nên dùng một thư viện editor"*. Đừng. Món nợ *"thư viện editor"* đã đổi chủ một lần (`deferred-work.md:2969-2981`, Story 2.4 → 2.5b) đúng vì hình dạng DOM đổi, và 2.5b **đã** kết luận không cần. Nếu dev agent thấy mình muốn thêm một gói, **dừng lại** — cửa NFR15 vẫn đứng với ba bước bắt buộc (`project-context.md:92-100`), và sáu tên bị cấm cưỡng chế bằng `npm run check:deps`.

### 🔴 Ba mốc dòng của AC6 trỏ vào một tệp không còn tồn tại

AC6 viết `EditorPanel.vue:769` và `:842`. `EditorPanel.vue` **bị xoá** ở Story 2.5b (commit `ca33072`) — `ls src/panels/` hôm nay không có nó. Hai lớp chặn sống ở `src/panels/GridPanel.vue`:

| Lớp | Ở đâu hôm nay | Chặn gì |
| --- | --- | --- |
| ① tầng `beforeinput` | `onBeforeInput`, `GridPanel.vue:699-703` | `insertParagraph` · `insertLineBreak` ⇒ `preventDefault()`, không xử lý gì thêm |
| ② tầng phím | `onEditKeydown`, `GridPanel.vue:764-767` | `Enter` trần khi **không** composing |

Nhánh ② của `onBeforeInput` (`:705-746`) chặn `insertFromPaste`/`insertFromDrop`/`insertReplacementText` rồi **tự chèn** text thuần đã làm phẳng. Nhánh đó **ở lại** — nó là một mệnh đề khác, và nó là **khuôn** cho Quyết định #1(b).

🔴 `GridPanel.vue:684` đã viết sẵn địa chỉ của story này: *"Quyền xuống dòng **trong ô bản dịch** là FR134/AD-46, **Story 2.5d**. Đừng mở sớm."*

### Lược đồ và di trú

`PROJECT_MIGRATIONS` (`schema.rs:580-618`) hôm nay có **bảy** bước, đích **8**:

| `to_version` | hằng | story |
| --- | --- | --- |
| 1 | `SCHEMA_MIGRATION_LOG_DDL` | 1.15 |
| 2 | `WORK_DDL` | 1.15 |
| 3 | `CHAPTER_DDL` | 1.15 |
| **5** | `SEGMENT_DDL` | 2.1 |
| 6 | `SEGMENT_TARGET_TEXT_DDL` | 2.2 |
| 7 | `SEGMENT_STATUS_AND_VERSION_DDL` | 2.5 |
| 8 | `SEGMENT_OMITTED_DDL` | 2.5c |

**Số 4 đã cháy** (`schema.rs:553-579`). Cổng thật là `segment_contract.rs::the_project_migration_set_never_reuses_the_burned_number_four` (`:474`); `validate_strictly_increasing` (`:636-652`) chỉ đòi tăng dần nghiêm ngặt và **không** bắt được việc tái dùng.

🔴 **Nguồn sự thật cho số kế tiếp là `PROJECT_MIGRATIONS`, không phải một ghi chép ở nơi khác — kể cả story này.** Đo lại lúc bắt đầu.

**Bảng `segment` hôm nay** (mười một cột): `id` · `chapter_id` · `ord` · `source_text` · `is_paragraph_end` · `retired_at` · `created_at` · `updated_at` · `target_text` · `status` · `is_omitted`. Một chỉ mục: `idx_segment_chapter_ord`. Không `FOREIGN KEY`, không `CHECK`.

**Cách di trú chạy** (`schema.rs:733-770`): lọc `m.to_version > from`, mỗi bước **một** `Transaction` riêng, `tx.execute_batch(m.sql)` — **nhiều câu ngăn bằng `;` chạy trọn trong một giao dịch** — rồi một dòng vào `schema_migration_log`, rồi `PRAGMA user_version`, rồi commit. Backup (`:617-648` khuôn cũ) chạy khi `found >= 1 && found < target`.

⚠️ **21** `project.db` thật và **10.477** hàng `segment` từ 21 Chương *(đo 2026-08-12)* sẽ chạy bước 9. Backfill là một **quyết định nghiệp vụ**: cờ đích soi gương cờ nguồn nói *"bản dịch giữ nhịp bản gốc cho tới khi người dùng đổi"*; một `DEFAULT 0` trần nói *"không đoạn nào của bản dịch kết thúc"* — và câu thứ hai **sai** với mọi Chương đã có.

### Dây IPC — và một lỗi đã xảy ra thật

`ChapterSegment` (`segment.rs:164-174`) đi trên dây **không** có `#[serde(rename_all)]`, nên trường trả về giữ `snake_case`. ⚠️ Chiều ngược lại khác: `invoke()` gửi **tham số** dạng camelCase dù hàm Rust nhận `snake_case`.

🔴 **Tiền lệ lỗi phải chặn, ghi nguyên vụ tại `segment.rs:144-153`:** bản đầu Story 2.5 thêm `status` vào kiểu TypeScript nhưng quên thêm vào **struct** và vào câu `SELECT`. Kết quả: `segment.status` luôn `undefined` phía webview, `isConfirmed` luôn `false` **trên sản phẩm thật** — và **74/74 test frontend vẫn xanh**, vì fixture chép tay có sẵn cột. Chỉ e2e bắt được (`editor-confirm-segment.e2e.mjs`, 2026-08-14). Story 2.5c thêm cột thứ hai vào **đúng đường đó** và đã dựng lưới riêng. Story này là cột thứ **ba**.

### Đường ghi: `\n` có sống sót tới đĩa không?

Đã đọc từng chặng, và câu trả lời là **có, không một chặng nào cắt nó** — nhưng ba chặng đầu phụ thuộc chữ ký #1:

| Chặng | Ở đâu | Xử lý `\n` |
| --- | --- | --- |
| DOM → state | `reportEdit`, `GridPanel.vue:770-774` | đọc `cell.textContent`. Mang `\n` **chỉ khi** DOM là text node phẳng |
| state → tập chờ | `noteEditorEdit`, `editorPanelState.ts:251-267` | chuỗi nguyên văn vào `Map`, rồi `flush.markChanged` |
| tập chờ → IPC | `flushEditorNow` → `saveSegmentTargets`, `config/segment.ts:319-340` | `invoke('save_segment_targets', { chapterId, edits })`, một lô một lượt |
| IPC → đĩa | `save_segment_targets`, `segment.rs:450-545` | `UPDATE segment SET target_text = ?1` — **không** `trim`, **không** `replace` |
| đĩa → DOM | `SELECT` (`:326-329`) → `{{ s.target_text }}` (`:983`) → `restoreEditedText` (`:598-612`) | interpolation và `el.textContent =` đều dựng **một** text node mang `\n` |

⚠️ Chặng hiển thị là chỗ `\n` **biến mất khỏi mắt** chứ không khỏi đĩa: không `white-space` nào được khai ⇒ mặc định `normal` ⇒ gộp thành khoảng trắng. Xem Quyết định #2.
⚠️ `confirm_segment` (`segment.rs:750`) dùng `target_text.trim().is_empty()` để từ chối ký một câu rỗng, và `str::trim()` cắt theo `char::is_whitespace` của Unicode ⇒ một ô **chỉ có** `\n` vẫn bị từ chối. Hành vi đúng, nhưng chưa ca nào canh.

### Lưới: hình dạng thật, và vì sao "khoảng thở" là thuộc tính của HÀNG

`GridPanel.vue:889` là `<div class="grid">` cha khai `grid-template-rows` động. **Năm cột là năm con trực tiếp**, mỗi cột `grid-row: 1 / -1` + `grid-template-rows: subgrid` (`:1085-1090`). Thứ tự: vạch trạng thái · số câu · nguyên văn · bản dịch · nhãn trạng thái.

🔴 **Một hàng KHÔNG phải một phần tử DOM** (`:5-19`). Mọi kiểu dáng cấp hàng phải nhân ra từng ô.

⚠️ **Hệ quả cho AC2 và Quyết định #4, và nó là một ràng buộc chứ không một lựa chọn:** `.cell.para-end` là `padding-bottom: 14px` (`:1115-1118`) đặt trên **năm** ô. Năm cột chia **chung** một tập track hàng, và `.cell` không khai `align-self` nên mặc định `stretch`. ⇒ Một `padding-bottom` chỉ đặt ở ô bản dịch vẫn làm **track** cao lên và bốn ô kia giãn theo. **Hai cấu trúc đoạn khác nhau không biểu diễn được bằng hai khoảng thở khác nhau trong cùng một lưới.** Đây là suy luận từ mã, **chưa** phải phép đo — Task 1.2⑤ là chỗ đo nó.

**Neo:** `data-segment-id` xuất hiện **hai lần** mỗi câu (ô nguyên văn + ô bản dịch), phân biệt bằng `data-col="src"|"tgt"`.

`contenteditable`**:** đặt **tĩnh** trong template (`:982`), trên **mọi** ô bản dịch, không binding động (`:962-964`). Mỗi ô là một editing host riêng ⇒ một `Range` soạn thảo **không** bắc cầu qua hai ô (`:690-693`). Đừng chạm mệnh đề này.

**Ba handler sống ở CỘT, không ở từng ô** (`:949-957`) — sự kiện nổi bọt, và N listener cho 9.850 hàng là N lượt đăng ký mà một lượt là đủ.

### AD-37 ba ca biên: chỉ **một** ca có mã thi hành

`grep "fn merge_segments\|merge_segment\|MergeSegment"` trên `src-tauri/src/**` cho **0** kết quả. Story 2.8 là `backlog`, và `segment.rs:31-32` xác nhận *"nút tái tách kèm cảnh báo… thuộc **Story 2.8** — hôm nay chưa có `SegmentVersion` để mà giữ lại"*. `retired_at` là `None` cho **mọi** segment hôm nay (`segment.rs:138`).

Xem Quyết định #6 cho bảng đầy đủ. Điểm phải nhớ: AC3 nói *"áp y nguyên"*, và hai trong ba ca **chưa có chỗ để áp**. Đó là *"năng lực chưa dựng ≠ lệch spec"* (`project-context.md:456`), **không** phải cớ để tự chấm đạt, và cũng **không** phải lý do sửa `epics.md`.

### Hai doc-comment trong mã nói ngược với AD-46

| Ở đâu | Nói gì | Sau story này |
| --- | --- | --- |
| `schema.rs:283-286` | *"**Một** cột, dùng chung cho nguyên văn và bản dịch; **không** `source_paragraph_end`/`target_paragraph_end`"* | **sai về mã** — phải sửa |
| `split.rs:118-119` | *"không `source_paragraph_end`/`target_paragraph_end`"* | **sai về mã** — phải sửa |
| `ARCHITECTURE-SPINE.md:443` (AD-37 §Rule) | *"Một cờ duy nhất dùng chung cho cả nguyên văn và bản dịch"* | **giữ nguyên** — AD-46 (`:658`) khai bằng chữ *"AD-37 không sửa một chữ"* |

⚠️ Đây là lần thứ **năm** kho gặp khuôn *"một dòng đặc tả hoặc doc-comment trôi khỏi mã"*: bốn lần trước ở `DESIGN.md` §components (2.5b Quyết định #9, 2.5c Quyết định #6, cộng hai dòng `:145`/`:146`). Khác biệt lần này: hai dòng trên là doc-comment **trong mã**, nên chúng thuộc lượt sửa của story; dòng ở spine thì **không**.

### Bài học từ hai story trước

**① Đo trước khi vá, và luật dừng là thật.** 2.5b tốn **bốn** lượt vá "hợp lý" cho một chẩn đoán sai *(caret mất khi bấm vào lưới)*; cả bốn bị phép đo bác vì chúng chạy **trước** lượt `enterFocus` thật sự gây lỗi. 2.4 chạy và **treo** ở chỗ không tiêm được `bench.js` vào webview release. Task 1.3 của story này mang luật dừng vì đúng lớp rủi ro đó.

**② `happy-dom` không phải WebKit.** Mọi mệnh đề về hình học hoặc hành vi engine thuộc bàn đo hoặc e2e. `editorSegmentRule.test.ts:9-11` ghi thẳng giới hạn này.

**③ Số của fixture "tương lai" phải luôn là `target + 1`.** 2.5c nâng `STEP_NINE` từ 8 lên 9 và ghi luật tại chỗ (`segment_contract.rs:1110-1115`). Story này là lượt lặp lại đầu tiên của luật đó — nâng 9 lên **10**.

**④ Kiểu bắt được thứ e2e từng phải bắt.** 2.5c thêm một trường vào `ChapterSegment` phía TS và `vue-tsc` đỏ **ba** fixture chép tay ngay lượt đầu — bắt được **chỉ vì** `tsconfig.json` include cây test. Đừng "sửa cho hết đỏ" bằng một `as any`.

**⑤ File List kê thừa là một món nợ.** Kê từ `git diff --stat`, không từ trí nhớ.

**⑥ Cổng 4445 suýt cho số của app khác.** Máy chủ WebDriver bám cổng cố định; nếu bị chiếm, phiên nối nhầm vào webview app khác và **vẫn trả số hợp lệ**. Đặt `TAURI_WEBDRIVER_PORT`, đừng giết tiến trình của người dùng.

### Luật "erasable-only" — một `import` sai giết ba phép kiểm

Tệp phải **nạp được bằng Node trần** *(cổng `import()` chúng để chạy kiểm **hành vi** trên chính mã sản phẩm)*: `src/commands/{registry,focus,keys,index}.ts` · `src/panels/editorSegments.ts` · `src/panels/segmentNavigation.ts` · `src/layout/{workspaceLayout,writeSchedule}.ts`.

⇒ Không `import` **giá trị** của `vue`/`dockview`/`@tauri-apps/api`; không `enum`, `namespace`, parameter property. Kiểm I `abort()` chứ không FAIL — nó **dừng hẳn** CI (`check-commands.mjs:794-813`).

`editorPanelState.ts` · `editorFlush.ts` · `GridPanel.vue` **không** chịu luật này.

### Project Structure Notes

- Test frontend ở `tests/frontend/**` **phẳng**, không đồng vị trí trong `src/` — bốn cổng đếm quần thể `src/**` và một tệp test đổ vào đó **thổi phồng mẫu số** (`vitest.config.ts:12-38`).
- Chuỗi literal trong `src-tauri/src/**` viết **KHÔNG DẤU** (`check-i18n` Kiểm A); `tests/**` được miễn trừ nên giữ dấu. Comment tiếng Việt có dấu thì được ở cả hai.
- Chẩn đoán `console.warn` trong `.vue` viết **tiếng Anh** — cùng luật Kiểm A. Khuôn có sẵn: `GridPanel.vue:722,728`.
- Tên hàm test là một **câu khẳng định**, không `test_foo`. ⚠️ Và **không mang số hiệu** nếu số đó sẽ trôi — xem Task 2.5.
- Hai họ test Rust: `*_contract.rs` (hợp đồng) · `*_boundary.rs` (ranh giới module).
- Bàn đo đi vào một thư mục `2-5d-ban-do/` cạnh story, khuôn `2-5b-ban-do/` và `2-5c-ban-do/`.
- Commit: `type(scope): câu tiếng Việt`, `scope = story-2.5d`. Câu sau dấu hai chấm **nói ĐIỀU ĐÃ TÌM RA**, không chỉ điều đã sửa. Mỗi lớp một commit sạch.

### Bẫy đã biết, ghi ra thay vì để phát hiện lại

- ⚠️ **`textContent` nuốt ranh giới `<div>`.** `textContent` của `<div>a</div><div>b</div>` là `"ab"`. Đây là bẫy trung tâm của Quyết định #1 và nó **không** làm cổng nào đỏ.
- ⚠️ **WebKit im lặng ở biên.** `Backspace` offset 0 phát **0** `beforeinput` trên cả WKWebView lẫn Playwright-WebKit, trong khi Blink phát và huỷ được (`deferred-work.md:3012-3037`). Không suy hành vi `insertParagraph` của WebKit từ Blink.
- ⚠️ **`subgrid` ép ô bản dịch cao theo cột nguyên văn** — đo được **388 px / 11,47 dòng** khi bật Hán Việt song song (`deferred-work.md:3131-3162`). Story này thêm áp lực **ngược chiều**, một chiều mà phép đo đó chưa tính. Vế Blink là **khoảng mù có tên**, chưa đo.
- ⚠️ **`epics.md:2329` mang một ước lượng đã bị đo sai** *(đoán ~330 px / 6–7 dòng)*. Dùng số đã đo, không dùng ước lượng trong epics.
- ⚠️ **Fixture e2e không reset state panel giữa các spec** (`deferred-work.md:3093-3115`, chủ Story 1.22). Spec mới nên tự nạp lại webview sau khi tạo Tác phẩm.
- ⚠️ **Command id nằm cứng trong spec e2e và không cổng nào canh** (`deferred-work.md:3117-3129`). `check:commands` **không đọc `e2e/**`** ⇒ thêm hoặc đổi id thì phải tự rà `e2e/**` bằng tay.
- ⚠️ **`editorOmitError` và `editorConfirmError` được export mà chỉ một chỗ đọc.** Nếu Task 9 dựng một lệnh mới, đừng nhân thêm một ô lỗi thứ ba không ai đọc.

### References

- `_bmad-output/planning-artifacts/epics.md:2376-2410` — Story 2.5d, sáu AC
- `_bmad-output/planning-artifacts/epics.md:132` — FR134 · `:280` — FR121 sửa 2026-08-14 · `:407` — AD-46 trong bảng bất biến · `:986` — *"đường xuất phải đọc CẢ HAI nguồn"*
- `_bmad-output/planning-artifacts/epics.md:2487-2529` — Story 2.8 (gộp/tách, `backlog`) · `:2531-2570` — Story 2.9 · `:2572-2621` — Story 2.10
- `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md:462-464` — FR134 nguyên văn kèm ràng buộc kiến trúc
- `.../architecture/.../ARCHITECTURE-SPINE.md:652-670` — AD-46 đầy đủ · `:437-453` — AD-37 và bảng ba ca biên · `:689` (Consistency Conventions) · `:75-79` (AD-1)
- `.../ux-designs/.../EXPERIENCE.md:236` — ngắt đoạn là khoảng thở, không một hàng rỗng · `:262-271` — bảng phím và vì sao `Enter` trần không ký · `:273-275` — tách ở cột nguyên văn, gộp từ cả hai phía
- `_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-14.md:103-107` · `:225-256` · `:291` · `:337-344`
- `src-tauri/src/core/store/schema.rs:278-295` · `:335-346` · `:380-424` · `:460-511` · `:513-531` · `:553-579` · `:580-618` · `:624-652` · `:733-770`
- `src-tauri/src/commands/segment.rs:96-117` · `:144-153` · `:164-174` · `:316-357` · `:450-545` · `:740-760` · `:860-950`
- `src-tauri/src/core/segment/split.rs:104-120` · `:168-181` · `:258-263` · `:377-390`
- `src-tauri/src/core/segment/omit.rs` — khuôn hàm thuần lọc đầu ra (Quyết định #2(b) của 2.5c)
- `src-tauri/src/core/export/mod.rs` — **6 dòng, toàn doc-comment, 0 dòng mã**
- `src-tauri/tests/segment_contract.rs:474` · `:497-503` · `:934` · `:1004` · `:1104-1164` · `:2218-2260`
- `src/panels/GridPanel.vue:5-19` · `:316-319` · `:331-336` · `:598-612` · `:674-746` · `:748-774` · `:889` · `:949-983` · `:1069-1090` · `:1099-1118` · `:1236-1240`
- `src/panels/editorPanelState.ts:56` · `:251-267` · `:288-349` · `:704-709` · `:790-811`
- `src/config/segment.ts:66-102` · `:200-215` · `:319-340`
- `src/commands/keys.ts:434-439` · `:506-510` · `src/commands/index.ts:880-883` · `:917` · `:955`
- `tests/frontend/support/segmentFixture.ts:38-78` · `tests/frontend/editorTypingZone.test.ts:168-191`
- `scripts/check-commands.mjs:235` · `:794-813` · `:1876-2113` (Kiểm F) · `:2116-2269` (Kiểm I)
- `scripts/check-tokens.mjs:880-912` · `:1300-1334` · `:1345-1396`
- `_bmad-output/implementation-artifacts/2-5c-cat-bo-cau-khoi-ban-dich.md` — năm chữ ký + Quyết định #6 phát sinh
- `_bmad-output/implementation-artifacts/2-5b-luoi-hai-cot-doi-chieu.md:775-814` — bàn đo hai engine, năm mệnh đề
- `_bmad-output/implementation-artifacts/deferred-work.md:2969-2981` · `:3012-3037` · `:3093-3129` · `:3131-3162` · `:3164-3194` · `:3274-3330` · `:3496-3507`
- MDN `Element: beforeinput` / `InputEvent.inputType` — https://developer.mozilla.org/en-US/docs/Web/API/InputEvent/inputType
- SQLite `ALTER TABLE ADD COLUMN` *(ràng buộc `DEFAULT` phải là hằng)* — https://www.sqlite.org/lang_altertable.html

---

## Testing

Bốn đường nghiệm thu, **bốn vai không chồng nhau**. Chọn sai đường là dựng nguồn sự thật thứ hai — trước khi viết một phép kiểm mới, hỏi: **mệnh đề này đã có chủ ở đường nào chưa?**

| Mệnh đề của story này | Đường đúng |
| --- | --- |
| Bước 9 tồn tại, danh sách di trú đúng, **backfill = cờ nguồn từng hàng** trên db thật | `cargo test` — `segment_contract.rs` |
| Cột mới đi qua dây IPC (không `undefined` phía webview) | `cargo test` — khuôn `:2218-2260` |
| `target_text` mang `\n` đi trọn một vòng đĩa, không bị `trim` | `cargo test` |
| Một ô chỉ có `\n` **vẫn** bị `confirm_segment` từ chối | `cargo test` |
| Cờ đích **bằng** cờ nguồn ngay sau một lượt nhập, gồm hàng cuối | `cargo test` |
| `Enter` đi vào đường ghi và tập chờ nhận chuỗi có `\n` | `vitest` — `editorTypingZone.test.ts` |
| `Enter` **không** tách câu *(không `data-segment-id` nào nhân đôi)* | `vitest` |
| `Enter` trần **không** ký câu nào; `Mod+Enter` vẫn ký | `vitest` + bàn đo |
| `insertParagraph` phát sự kiện gì, engine dựng DOM gì, `textContent` đọc ra gì | **bàn đo hai engine** — không `vitest` |
| Ô vẽ ra hai dòng thật; chiều cao track hàng đổi bao nhiêu px | **bàn đo tay / e2e** — `happy-dom` không phải WebKit |
| `\n` đi từ phím tới đĩa **trên sản phẩm thật** rồi đọc lại được | **e2e** — đây là lớp lỗi *"DOM hai dòng, đĩa một chuỗi"* |
| Gõ tiếng Việt bằng **bộ gõ thật** rồi `Enter` | **tay Ice** — không bộ chạy test nào mô phỏng được |

**Luật cổng:** mã thoát là phán quyết; lỗi hạ tầng **không phải** một phép kiểm đỏ (`abort()`, thoát khác 0, nói rõ *"đây là lỗi hạ tầng"*); không phán quyết nào đọc tham số từ chính thứ nó đang kiểm.

**Luật đo:** không đánh dấu đạt bằng suy luận. Số đo ghi kèm **phiên bản toolchain và ngày** — *"số đo không truy nguyên được thì không phải số đo"*.

**Thêm một cổng = sửa BA danh sách** (`package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`), `check:gates` canh cả ba. Story này nhiều khả năng **không** thêm cổng nào.

⚠️ **`npm run test:e2e` không nằm trong CI và không nằm trong pre-push** — nó chỉ chạy tay. Bốn lớp lỗi nặng nhất của Epic 2 tới nay đều **chỉ** e2e bắt được.

---

## Nợ dự kiến (ghi vào `deferred-work.md` kèm chủ, không tự chấm đạt)

| Món | Trạng thái dự kiến | Chủ |
| --- | --- | --- |
| AC3 vế **gộp** và vế **tách do người dùng gọi** — không có bề mặt để áp ba ca biên | 🟡 | Story 2.8 |
| AC4 vế *"đường xuất đọc cả hai nguồn"* — `core/export/mod.rs` là **6 dòng doc-comment** | 🟡 | Epic 8 (Story 8.3 · 8.4 · 8.6) |
| Nghĩa vụ AD-46 với FR121 có phát biểu **một chiều** không — không AC nào của Epic 8 tham chiếu ngược lại AD-46? *(Cùng khuôn lỗi mà 2.5c tìm ra cho FR133 — kiểm lại, đừng giả định)* | 🔴 hở nếu xác nhận | Ice |
| FR134 vế *"cho tới khi người dùng đổi"* — nếu Quyết định #3 chọn (a) | 🟡 | theo chữ ký của Ice |
| Hiển thị cờ đích trong lưới — nếu Quyết định #4 chọn (a) | 🟡 | theo chữ ký của Ice |
| Chiều cao hàng khi ô bản dịch nhiều dòng — một chiều đo mới mà `:3131-3162` chưa tính | phụ thuộc | Ice *(quyết định UX)* + Story 2.4 *(bộ đo NFR2)* |
| Vế **Blink** của mọi phép đo hình học — chỉ tới được qua WebView2/Windows, chưa có đường nghiệm thu tại chỗ | khoảng mù có tên | Story 1.22 |
| Tên test mang số hiệu di trú — sai lại ở **mỗi** story thêm một bước | 🟡 | story này *(Task 2.5)*, nếu không gỡ thì ghi nợ |

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, skill `bmad-dev-story`) — 2026-08-15.

### Baseline đo trước khi chạm dòng đầu tiên

**Đo 2026-08-15, trước khi chạm một dòng nào.** Cây sạch — `git status --porcelain` chỉ có `?? _bmad-output/implementation-artifacts/2-5d-ngat-doan-ban-dich.md` *(chính tệp story)*.

| Phép đo | Ghi chép trong story | Đo lại | |
| --- | --- | --- | --- |
| `cargo test --locked` | 347 / 0 / 5 | **347 / 0 / 5** | khớp |
| `npm run test` (vitest) | 101 / 101 | **101 / 101** *(10 tệp, 2,36 s)* | khớp |

**Toolchain:** `rustc 1.97.1 (8bab26f4f 2026-07-14)` · `cargo 1.97.1` · Node `v22.22.2` · npm `10.9.7` · vitest `4.1.10`.
**HEAD lúc đo:** `0069808`.

⚠️ **Một chỗ lệch, ghi ra thay vì im lặng:** `baseline_commit` ở frontmatter là `d813118` *(chốt lúc `create-story`)*, còn HEAD hôm nay là `0069808` — một commit **chỉ sửa `sprint-status.yaml`**, không chạm mã. Giữ nguyên `baseline_commit` theo luật của workflow; ghi số HEAD thật ở đây để hai con số không âm thầm được coi là một.

### Bốn tiền đề đo lại từ NGUỒN, không từ ghi chép

Story tự dặn *"nguồn sự thật là `PROJECT_MIGRATIONS`, không phải một ghi chép ở nơi khác — **kể cả story này**"*. Đã đo:

| Mệnh đề | Nguồn đo | Kết quả |
| --- | --- | --- |
| Số di trú kế tiếp | `PROJECT_MIGRATIONS`, `schema.rs:582-618` | đích **8** qua bảy bước `[1,2,3,5,6,7,8]` ⇒ kế tiếp là **9** ✅ đúng như story nói |
| `white-space` trong `GridPanel.vue` | `grep -rn "white-space" src/` | **0** kết quả trong tệp đó *(16 kết quả ở tệp khác, gồm `SourceHanViet.vue:825,880` đã có `pre-wrap`)* ✅ Quyết định #2 là điểm chặn thật |
| Bề mặt gộp segment | `grep "merge_segment\|MergeSegment"` trên `src-tauri/src/` | **0** kết quả ✅ hai trong ba ca biên AD-37 chưa có chỗ áp |
| `EditorPanel.vue` | `ls src/panels/` | **không tồn tại** ✅ ba mốc dòng của AC6 trỏ vào tệp đã xoá |

**Hai lớp chặn `Enter` hôm nay, đã đọc tận nơi:** nhánh ① của `onBeforeInput` (`GridPanel.vue:700-703`) và `onEditKeydown` (`:765-767`, đúng hai dòng: `if (event.isComposing) return` rồi `if (event.key === 'Enter') event.preventDefault()`).

⚠️ **Một chi tiết của story cần chỉnh khi thi công:** Task 3.3 mô tả khuôn nhánh ② là *"`setStartAfter` → `collapse` → `setPosition`"*. Đọc mã thật (`:736-743`): nhánh ② kết thúc bằng `removeAllRanges()` + `addRange(range)`, **không** `setPosition`. Mà `setCaret` (`:328-336`) mang một phép đo ghi rõ **`setPosition`, KHÔNG `removeAllRanges`+`addRange`** — vì trên WKWebView với phần tử soạn thảo **rỗng**, WebKit **bỏ qua** `addRange`. ⇒ Nếu Ice ký #1(b) thì lượt chèn `"\n"` phải dùng `setCaret`, **không** chép nguyên đuôi của nhánh ②. *(Nhánh ② không lộ lỗi này vì nó chỉ chạy khi ô đã có vùng chọn hợp lệ từ một lượt dán.)*

### Chữ ký của Ice cho sáu quyết định mở

| \# | Nội dung | Đường Ice chọn | Ngày |
| --- | --- | --- | --- |
| 1 | `Enter` chèn cái gì vào DOM | ~~**(c)** chèn `<br>` + hai phép chuyển~~ → 🔵 **LẬT sang (d)** *(đường thứ tư, phép đo Task 1 lật)*: chặn `insertParagraph`, gọi `insertLineBreak` của chính engine | (c) 2026-08-15 → **(d) 2026-08-16** |
| 2 | `white-space` cho ô bản dịch | **(b)** `pre-line` | 2026-08-15 |
| 3 | Cơ chế đổi cờ đích | **(c)** hai lệnh tường minh, khuôn `omit_segment`/`restore_segment` | 2026-08-15 |
| 4 | Hiển thị cờ đích trong lưới | **(c)** đo trước, quyết sau — một Task đo đứng trước Task hiển thị | 2026-08-15 |
| 5 | Tên, kiểu cột, cách backfill | **(c)** `is_target_paragraph_end`, DDL **+** DML trong cùng một hằng, cùng giao dịch | 2026-08-15 |
| 6 | AC3 — hai ca biên không có bề mặt | **(b)** dựng sẵn hàm thuần ở `core/segment/` + test hợp đồng | 2026-08-15 |

#### 🔵 Hai hệ quả của bộ chữ ký này, ghi ra vì chúng KHÔNG hiển nhiên

**① 🔵 `#1` và `#2` KHÔNG độc lập — và sau lượt lật sang (d) thì `#2(b)` là ĐIỀU KIỆN của `#1`.**
*(Ghi lúc ký, khi #1 còn là (c): với đường `<br>`, `white-space` chỉ cần cho đường **render đầu tiên** — template `{{ s.target_text }}` (`GridPanel.vue:983`) đổ **thẳng chuỗi** có `\n` vào DOM dưới dạng text node, và không `pre-line` thì một Chương vừa mở hiện bản dịch **mất hết ngắt đoạn**. Vế đó vẫn đúng nguyên văn.)*

🔴 **Sau khi #1 lật sang (d), ràng buộc chặt hơn hẳn.** Đối chứng E3 của Task 1 đo được: hình dạng mà engine dựng **phụ thuộc `white-space`** — `normal` ⇒ `<br>` *(`textContent = "AB"`, hỏng)*; `pre-line` ⇒ text node `\n` *(`textContent = "A\nB"`, đúng)*. ⇒ `white-space: pre-line` **không** là một dòng CSS trang trí, nó là **tiền đề vận hành** của Quyết định #1. Một lượt đổi `white-space` sau này **lật cả hai**. Chú thích tại chỗ trong `GridPanel.vue` phải nói đúng điều đó, không nói ít hơn.

**② 🔵 Mối lo của `#1(c)` được nhận — và Task 1 ĐO ĐƯỢC nó, rồi lật chữ ký.** Bảng quyết định đã ghi cái giá của (c): hai phép chuyển phải luôn đồng ý với nhau. Phép đo trên WKWebView thật *(2026-08-15, `2-5d-ban-do/README.md`)* cho thấy cái giá đó **cao hơn** bảng ước: `A<br>` hiển thị **1 dòng**, phải `A<br><br>` mới ra 2 ⇒ hai phép chuyển còn phải thoả thuận về một **sentinel** không có trong dữ liệu, ở đúng ca *thường nhất* của tính năng *(bấm `Enter` cuối câu)*.

Cùng lượt đo lộ ra một **đường thứ tư** mà story không liệt vì bảng viết trước khi có số: dưới `white-space: pre-line` *(chính chữ ký #2(b))*, `execCommand('insertLineBreak')` của WebKit dựng **text node `\n`** — `0` phần tử con, `textContent = "A\nB"`, và ở ca cuối nội dung engine **tự thêm `\n` canh chót**. ⇒ **Ice ký (d) ngày 2026-08-16.** Chi tiết và bảng số ở `§Debug Log References Ⓐ`.

**③ Không món nợ nào phát sinh *lúc ký*** — `#3(c)` đóng vế *"cho tới khi người dùng đổi"* của FR134 thay vì hoãn nó; `#6(b)` dựng sẵn thay vì ghi nợ. Hai món vẫn còn chủ và **không** đổi: vế **gộp/tách do người dùng gọi** của AC3 *(chủ: Story 2.8 — chưa có bề mặt)*, và vế **đường xuất đọc cả hai nguồn** của AC4 *(chủ: Epic 8 — `core/export/mod.rs` là 6 dòng doc-comment)*.
🔴 `#4(c)` **chưa** là một quyết định về hình dạng — nó là lệnh đo trước. Sau khi có số, tôi quay lại hỏi Ice, **không** tự chọn.

### Debug Log References

#### Ⓐ Task 1 — bàn đo `Enter` trên WKWebView thật (2026-08-15, bốn vòng)

Tạo tác đầy đủ: **`2-5d-ban-do/README.md`** + `insertparagraph-wkwebview.e2e.mjs`.
WebKit **605.1.15** *(WKWebView thật của Tauri)*, macOS, `--features wdio`, cổng WebDriver **4467** *(4445 bị `gdrive-su` PID 48486 giữ — đúng ca 2.5b đã ghi tên)*. Tự kiểm danh tính phiên ĐẠT: `href = http://localhost:1420/`, `#app` có mặt.

🔵 **Bàn đo này chạy trên LƯỚI THẬT, không trên một hình dạng chép tay** — gỡ được §Giới hạn ① mà bốn bàn đo trước của kho đều mang. Đổi lại nó phải vô hiệu hoá hai lớp chặn `Enter` của sản phẩm, và nó làm bằng **thứ tự sự kiện** *(listener `document` pha capture + `stopPropagation`)*, không bằng một lượt sửa mã. Lượt chạy kết thúc bằng một số **đối chứng** với chặn bật lại: `innerHTML = "AB"` không đổi ⇒ hai lớp chặn **còn sống**.

**Năm mệnh đề của Task 1.2, mỗi mệnh đề một số:**

| # | Số đo |
| --- | --- |
| ① | `Enter` qua `browser.keys()` ⇒ **0** `beforeinput` *(chỉ `keydown`)* — giới hạn **bộ đo**. `execCommand('insertParagraph')` ⇒ `beforeinput insertParagraph`, **cancelable** |
| ② | engine dựng **`A<div>B</div>`** — `<div>` khối, `soPhanTuCon = 1` |
| ③ | 🔴 `textContent = "AB"` — **không một `\n` nào**. Bẫy trung tâm **tái lập được** |
| ④ | tự chèn text node `"\n"` ⇒ `textContent = "A\nB"` ✅, caret `Caret` neo trong ô, anchor là text node |
| ⑤ | **số dòng thật**: text `\n` + `normal` = **1** · + `pre-line` = **2** · + `pre-wrap` = **2**; `<br>` = **2** ở cả ba |

**Cộng hai phép đo mà Task 1.2 không đòi, và cả hai đều đổi một quyết định:**

⑥ `<br>` cuối nội dung: `A<br>` = **1 dòng**, `A<br><br>` = **2 dòng**.
⑦ **áp lực ngược chiều lên track hàng** *(chiều đo mà `deferred-work.md:3131-3162` chưa tính)*: ô bản dịch 1 → 2 → 5 dòng ⇒ track **38,00 → 63,00 → 150,00 px**, lệch `top` **0 px** mọi lượt.

🔴 **BỐN VÒNG, VÀ BA LẦN ĐỔI THƯỚC — không lần nào là một chẩn đoán bị bác.** Ghi rõ vì Task 1.3 mang một luật dừng đếm theo *"vòng chẩn đoán bị phép đo bác"*, và **không** vòng nào của lượt này rơi vào loại đó:
- **Vòng 1** đo *"ô có vẽ ra hai dòng không"* bằng **chiều cao ô** ⇒ `71,00 px` ở **mọi** lượt. Số thật, **trật câu hỏi**: `subgrid` đang ghim ô bản dịch theo cột nguyên văn. ⇒ Số chiều cao của vòng 1 **bị rút**, không được trích.
- **Vòng 2** đổi sang thước **đếm hộp dòng** (`Range.getClientRects()`) — không bị `stretch` làm nhiễu — và thêm phép đo áp lực ngược chiều.
- **Vòng 3** đo lại riêng ca `<br>` **cuối nội dung** vì vòng 1 trả lời nó bằng đúng cái thước hỏng, và vì đó là thao tác **thường nhất** của FR134.
- **Vòng 4** đo **đường thứ tư** mà vòng 3 vô tình lộ ra.

🔵 **Đường thứ tư, và vì sao Quyết định #1 lật:**

| | E1 giữa nội dung, `pre-line` | E2 **cuối** nội dung, `pre-line` | E3 giữa nội dung, `normal` *(đối chứng)* |
| --- | --- | --- | --- |
| DOM | `TEXT("A")·TEXT("\n")·TEXT("B")` | `TEXT("A")·TEXT("\n")·TEXT("\n")` | `A<br>B` |
| phần tử con | **0** | **0** | 1 |
| `textContent` | ✅ `"A\nB"` | ✅ `"A\n\n"` | 🔴 `"AB"` |
| số dòng | 2 | **2** | 2 |

E4 *(vế ngược)*: `el.textContent = "A\nB"` dưới `pre-line` ⇒ **một** text node, 2 dòng ⇒ `restoreEditedText` **không phải sửa một dòng**.

⇒ Đường (d) đóng AC1 mà `reportEdit` và `restoreEditedText` **giữ nguyên**. Đường (c) cần hai phép chuyển **cộng** một sentinel `<br>` canh chót. Ice ký (d) **2026-08-16**.

⚠️ **Cái giá của (d), không miễn phí:** `execCommand('insertLineBreak')` tự phát một `beforeinput` `insertLineBreak` — đúng inputType mà nhánh ① đang chặn ⇒ cài đặt cần một **chốt chống đệ quy**. E1 đo được **đúng một** sự kiện mỗi lượt.

**Bốn giới hạn của lượt đo, chép sang đây vì chúng ràng buộc cách đọc số trên:** ① `browser.keys()` không vào đường nhập văn bản gốc *(chủ: Story 1.22)* · ② vế **bộ gõ tiếng Việt thật** không mô phỏng được — **Task 1.5, chủ ICE** · ③ vế **Blink** là **khoảng mù có tên** *(chủ: Story 1.22)*; tiền lệ `Backspace` offset 0 cho thấy hai engine **nói ngược nhau** ở đúng địa hạt này · ④ fixture có **một** segment nên vế *"cả năm cột giãn theo"* là **suy một bước** từ `lechTopPx = 0` + cấu trúc `subgrid`, không đo trực tiếp.

### Completion Notes List

**Story 2.5d XONG 2026-08-16.** Sáu quyết định mở của Task 0 có chữ ký, **một** trong sáu bị
chính phép đo của Task 1 **lật** và được ký lại, và Quyết định #4 được ký **sau** khi có số
đúng như đường (c) đòi.

#### Nghiệm thu

| Đường | Kết quả |
| --- | --- |
| 9 cổng `check:*` | **XANH** |
| `check:scope` + `check:scope:bundled` *(chạy tay, cổng 1420 trống)* | **XANH** |
| `npm run build` *(`vue-tsc` hai lượt + `vite build`)* | **XANH** |
| `npm run test` (vitest) | **102/102** *(baseline 101, +1 ca)* |
| `cargo test --locked` | **359/0/5** *(baseline 347, +12 ca)* |
| `npm run test:e2e` | **7/7 spec · 9/9 ca** (8m25) |

#### Sáu chữ ký, và cái đã lật

- **#1 → (d)**, lật từ (c) ngày 2026-08-16 **bằng phép đo**. Chi tiết ở `§Debug Log Ⓐ`. Hai số
  lật nó: `A<br>` hiển thị **1 dòng** *(phải `A<br><br>` mới ra 2)* ⇒ đường `<br>` cần một
  sentinel mà hai phép chuyển phải thoả thuận, ở đúng ca **thường nhất** *(bấm `Enter` cuối
  câu)*; và `execCommand('insertLineBreak')` dưới `pre-line` dựng **text node `\n`**
  *(0 phần tử con, `textContent === "A\nB"`)*, engine tự thêm `\n` canh chót ở ca cuối nội dung.
  ⇒ Đường (d) đóng AC1 mà `reportEdit` và `restoreEditedText` **không sửa một dòng**.
- **#2 (b)** `pre-line`. 🔴 Và nó **không** là một dòng CSS: đối chứng E3 đo được hình dạng DOM
  mà engine dựng **phụ thuộc `white-space`** ⇒ `pre-line` là **tiền đề vận hành** của #1(d).
  Hai chỗ đó phải đọc cùng nhau, và cả hai chú thích đều nói ra điều đó.
- **#3 (c)** hai lệnh: `editor.end_target_paragraph` (`Mod+Alt+P`) · `editor.join_target_paragraph`
  (`Mod+Alt+U`).
- **#4 (Ⓑ)**, ký **sau** phép đo vòng 5: `padding-bottom` chỉ ở ô bản dịch đẩy track **38,00 →
  46,00 px** *(ô nguyên văn cũng 46)*; đổi kiểu đường kẻ đáy và một ký tự ở cột nhãn đều **0 px**.
  ⇒ Chỉ báo là **màu đường kẻ đáy** của riêng ô bản dịch.
- **#5 (c)** `is_target_paragraph_end`, DDL **+** DML trong cùng một hằng, cùng giao dịch.
- **#6 (b)** hàm thuần `core::segment::paragraph` + 4 ca hợp đồng.

#### Ba thứ PHÉP ĐO tìm ra mà đặc tả không nêu

**① Story nói SAI một điều kiện, và nó là một khuyết tật thật.** Task 5.4 viết *"nếu Ice chọn
#5(b) thì câu `INSERT` phải set cờ đích tường minh"*. Điều kiện đó **sai**: bước di trú 9
backfill các hàng **đã có trên đĩa**, còn một Chương nhập **sau** lượt di trú đi qua
`insert_segments`, nơi `DEFAULT 0` là thứ duy nhất cấp giá trị. ⇒ Với **mọi** đường ký,
`INSERT` phải set tường minh, nếu không **mọi Chương mới có cờ đích tắt hết** — ca **thường
nhất** của AC2. Đo được, không suy: ca
`a_freshly_imported_chapter_mirrors_the_source_flag_into_the_target_flag_row_by_row` đỏ với
`[(2, true, false), (3, true, false)]` trước lượt vá.

**② Vòng 1 của bàn đo đo TRÚNG số nhưng TRẬT câu hỏi.** Nó hỏi *"ô có vẽ ra hai dòng không"*
bằng **chiều cao ô** ⇒ `71,00 px` ở mọi lượt, vì `subgrid` đang ghim ô bản dịch theo cột nguyên
văn. Số đó **bị rút**, và ba vòng sau đổi thước *(đếm hộp dòng)*. Ghi rõ vì Task 1.3 đếm *"vòng
chẩn đoán bị phép đo **bác**"* — **0** vòng thuộc loại đó, nên luật dừng **không** kích hoạt.

**③ Một khuyết tật của `check-i18n.mjs`.** Viết một tên thẻ *(ví dụ `<style>`)* trong **comment**
của template `.vue` làm `scanTemplate` báo FAIL ở **một comment khác cách đó 20 dòng**. Cổng
**đỏ** chứ không xanh, nhưng nó **đỏ sai chỗ** — mất một vòng chẩn đoán. Ghi nợ có chủ.

#### Bảy món nợ, mỗi món một chủ *(`deferred-work.md`)*

🟡 AC3 vế gộp/tách do người dùng gọi *(Story 2.8)* · 🟡 AC4 vế đường xuất *(Epic 8)* · 🟡 lượt
đổi cờ bị từ chối không có đường ra màn hình *(một story dựng đường báo lỗi dùng chung)* ·
🔴 khuyết tật `check-i18n` *(story hạ tầng cổng)* · ⚠️ vế **Blink** của mọi phép đo — khoảng mù
có tên *(Story 1.22)* · ⚠️ độ trễ dời con trỏ sau khi thêm một nhánh `:class` — **giao lại,
không tự chấm** *(Story 2.4)* · ⚠️ fixture e2e không reset state panel *(Story 1.22)*.

#### 🟡 Ba AC đóng MỘT NỬA, ghi đúng mức, KHÔNG tự chấm đạt

- **AC3** — ba ca biên áp *"y nguyên"*: hai trong ba **chưa có bề mặt** để áp *(`merge_segment`
  = 0 kết quả, Story 2.8 là `backlog`)*. Bảng đã thành mã + test; vế áp thật còn hở.
- **AC4** — vế *"đường xuất đọc dữ liệu đã lưu"*: `core/export/mod.rs` vẫn **6 dòng doc-comment,
  0 dòng mã**. Cột, dây IPC và lưới đã đọc cột; bề mặt tiêu thụ thật thì chưa tồn tại.
- **AC6** — vế *"giữ nguyên ở mọi nơi khác"* đúng **theo cấu trúc** *(`targetCellOf` chỉ khớp
  `[data-col="tgt"]`)*, nhưng không có bề mặt soạn thảo thứ hai nào để đối chứng.

#### 🔴 CÒN MỘT MÓN CHO ICE — Task 1.5

Gõ tiếng Việt **bằng bộ gõ thật** rồi bấm `Enter` **giữa câu**, trên máy thật. `Enter` là **phím
chốt dấu** của Telex, và không đường nghiệm thu nào của dự án mô phỏng được một bộ gõ — chữ ký
của Ice **là** đường nghiệm thu duy nhất. Đây là ca rủi ro nhất của story: nếu một lượt chốt dấu
bị lượt xuống dòng ăn mất, triệu chứng là **mất chữ** chứ không phải một lỗi.

### File List

Kê từ `git status --porcelain`, không từ trí nhớ. **24 sửa · 3 thêm.**

**Rust — lược đồ và lệnh**
- `src-tauri/src/core/store/schema.rs` *(sửa)* — hằng `SEGMENT_TARGET_PARAGRAPH_END_DDL` (bước **9**, DDL+DML), mục `PROJECT_MIGRATIONS`, doc-comment đích 8→9, **và** sửa dòng `:283-286` nói ngược AD-46
- `src-tauri/src/commands/segment.rs` *(sửa)* — trường `is_target_paragraph_end` vào `ChapterSegment` + câu `SELECT`; `insert_segments` set cờ tường minh; `ParagraphEndOutcome`; hàm thuần `set_segment_paragraph_end` + vỏ `wire`
- `src-tauri/src/core/segment/paragraph.rs` *(**thêm**)* — bảng ba ca biên AD-37 cho một **cặp** cờ
- `src-tauri/src/core/segment/mod.rs` *(sửa)* — khai `pub mod paragraph`
- `src-tauri/src/core/segment/split.rs` *(sửa)* — doc-comment `:118-119` nói ngược AD-46, và bảng ba ca nay chạy cho **hai** cờ
- `src-tauri/src/lib.rs` *(sửa)* — đăng ký `wire::set_segment_paragraph_end`

**Rust — test**
- `src-tauri/tests/segment_contract.rs` *(sửa)* — +12 ca; `SegmentRow` 11→12 cột; `STEP_NINE`→`STEP_TEN`; gỡ số khỏi tên `..._matches_the_declared_ladder_step_for_step`; 5 neo phiên bản 8→9
- `src-tauri/tests/pinned_contract.rs` *(sửa)* — neo 7/8 → **8/9**

**Frontend**
- `src/panels/GridPanel.vue` *(sửa)* — nhánh ① của `onBeforeInput` đổi nghĩa + chốt chống đệ quy; `onEditKeydown` bỏ `preventDefault`; `white-space: pre-line`; class `tgt-para-end`
- `src/panels/editorPanelState.ts` *(sửa)* — `setCurrentSegmentParagraphEnd`
- `src/config/segment.ts` *(sửa)* — trường TS, `ParagraphEndOutcome`, adapter `setSegmentParagraphEnd`
- `src/commands/index.ts` *(sửa)* — cổng dep + hai lệnh
- `src/main.ts` *(sửa)* — cắm dep
- `src/i18n/vi.json` *(sửa)* — hai nhãn lệnh
- `src/panels/README.md` *(sửa)* — bảng năng lực + mục *"một `\n` và một cờ, HAI khái niệm"*

**Cổng**
- `scripts/check-layout.mjs` *(sửa)* — `document.execCommand` vào danh sách cho phép, kèm phép đo
- `scripts/check-commands.mjs` *(sửa)* — `COMMAND_FLOOR` 29 → **33** *(đo lại: **39** command thật)*

**Test frontend**
- `tests/frontend/editorTypingZone.test.ts` *(sửa)* — viết lại ca `Enter` theo AD-46, +1 ca đường ghi
- `tests/frontend/support/setup.ts` *(sửa)* — lớp giả `document.execCommand` cho `happy-dom`
- `tests/frontend/support/segmentFixture.ts` · `segmentNavigation.test.ts` · `editorSegmentRule.test.ts` *(sửa)* — ba fixture `vue-tsc` bắt được

**e2e và bàn đo**
- `e2e/specs/editor-typing-flush.e2e.mjs` *(sửa)* — ca `Enter` viết lại thành ca của Task 10.3
- `_bmad-output/implementation-artifacts/2-5d-ban-do/` *(**thêm**)* — `insertparagraph-wkwebview.e2e.mjs` + `README.md`

**Tài liệu**
- `_bmad-output/implementation-artifacts/deferred-work.md` *(sửa)* — 6 mục nợ mới
- `_bmad-output/implementation-artifacts/sprint-status.yaml` *(sửa)*
- `_bmad-output/implementation-artifacts/2-5d-ngat-doan-ban-dich.md` *(**thêm**)*


### Change Log

| Ngày | Việc |
| --- | --- |
| 2026-08-15 | `create-story` — sáu quyết định mở, ba mốc dòng của AC6 bị bác *(tệp đã bị xoá)*, và một điểm chặn mà AC không nêu: `white-space` |
| 2026-08-15 | Task 0 — baseline **khớp** (347/0/5 · 101/101); bốn tiền đề đo lại từ nguồn; Ice ký sáu quyết định |
| 2026-08-15 | Task 1 — bàn đo trên **lưới thật**, bốn vòng: `insertParagraph` dựng `<div>` và `textContent` đọc `"AB"`; `A<br>` = **1 dòng**; và một **đường thứ tư** mà story không liệt |
| 2026-08-16 | 🔵 **Quyết định #1 LẬT (c) → (d) bằng phép đo** — chặn `insertParagraph`, gọi `insertLineBreak` của chính engine |
| 2026-08-16 | Task 2 — bước di trú **9**, bước ĐẦU TIÊN của kho mang DDL + DML trong một hằng; 6 ca đỏ ngoài danh sách, đúng như Task 2.10 dự đoán |
| 2026-08-16 | Task 3–4 — `Enter` xuống dòng, `white-space: pre-line` là **tiền đề vận hành** chứ không một dòng CSS |
| 2026-08-16 | Task 5 — 🔴 story nói **sai một điều kiện**: `insert_segments` phải set cờ tường minh với **mọi** đường ký, không chỉ #5(b) |
| 2026-08-16 | Task 6–7 — hàm thuần `core::segment::paragraph`; hai doc-comment nói ngược AD-46 đã sửa; rà `split('\n')` ⇒ **0** |
| 2026-08-16 | Task 8 — Quyết định #4 ký **sau** phép đo: `padding-bottom` riêng ở ô bản dịch đẩy track **38 → 46 px** ⇒ chỉ báo phi hình học |
| 2026-08-16 | Task 9 — hai lệnh `Mod+Alt+P` / `Mod+Alt+U`; `COMMAND_FLOOR` đo lại 29 → **33** (39 command thật) |
| 2026-08-16 | Task 10 — 11 cổng · build · vitest **102** · cargo **359/0/5** · e2e **7/7 spec, 9/9 ca**. Story chuyển `review` |

### Review Findings

Rà soát mã ngày **2026-08-16** — ba tầng song song (Blind Hunter · Edge Case Hunter · Acceptance
Auditor), mỗi tầng khởi động lạnh. Không tầng nào trượt. 11 phát hiện thô ⇒ **9** còn lại sau khi
khử trùng lặp và tự kiểm chứng tại mã nguồn; **1** loại làm nhiễu.

🔵 **2026-08-16 — ba mục `[Decision]` đã được Ice chốt và chuyển thành `[Patch]`**; chữ ký ghi tại
chỗ ở đầu mỗi mục. Tổng còn **9 mục vá**, **1** hoãn.

🔵 Ba tầng **mâu thuẫn nhau về AC3**: Acceptance Auditor chấp nhận lời tự nhận *"đóng một nửa, ghi
nợ đúng"*, Edge Case Hunter đo ra một vi phạm AC3 **đang sống trong mã hôm nay** mà món nợ đã ghi
**không** phủ. Phép đo thắng — xem `[Decision]` đầu tiên.

- [x] [Review][Patch] 🔴 **Ice ký 2026-08-16 — đường (a): lệnh TỪ CHỐI, kèm một `message_key` MỚI.** **`set_segment_paragraph_end` cho phép BẬT cờ đích trên segment CUỐI Chương** — AC3 đòi ca *"segment cuối Chương → tắt, LUÔN LUÔN"* áp y nguyên cho cờ đích, và `paragraph.rs::at_end_of_chapter` đã dựng sẵn hàm thuần cho ca đó. Nhưng lệnh `Mod+Alt+P` (`src-tauri/src/commands/segment.rs:419-473`) chỉ kiểm `retired_at` và giá trị hiện tại — **không** hỏi segment có phải câu cuối Chương không, và không gọi `at_end_of_chapter`. Cờ có bề mặt hiển thị THẬT hôm nay (`GridPanel.vue:1063` gắn `tgt-para-end`, `:1429` vẽ đường kẻ đáy) ⇒ người dùng bấm `Mod+Alt+P` trên câu cuối vẽ được một ranh giới đoạn dưới câu cuối cùng, đúng thứ AC3 nói không bao giờ được có. Món nợ AC3 đã ghi chỉ phủ **gộp/tách** (chủ Story 2.8), **không** phủ ca này. ⇒ Vá theo (a). ⚠️ Khoá thứ ba **không** phá lý lẽ của doc-comment hiện có (*"đừng dựng khoá thứ ba nói cùng một chuyện"*): hai khoá cũ nói *không tìm thấy* và *đã về hưu*; đây là một sự thật **khác** — câu tồn tại, còn sống, và vẫn không được mang cờ. Ba nhánh từ chối, ba sự thật.
- [x] [Review][Patch] 🔴 **Ice ký 2026-08-16 — đường (b): cho DÁN giữ `\n`.** **Đường DÁN vẫn làm phẳng `\n` thành khoảng trắng, trong khi `Enter` gõ tay nay giữ nguyên** — `GridPanel.vue:771` (`raw.replace(/[\r\n]+/g, ' ')`) **không** bị story này chạm. Chú thích tại chỗ nói lý do: dán hai đoạn vào một ô thì hai chữ ở hai đầu ranh giới không được dính. Lý do đó viết khi `\n` **không thể** tồn tại trong ô; AC1 vừa làm nó tồn tại được. ⇒ Hai đường vào cùng một ô nay mang hai luật khác nhau, và lượt dán mất ranh giới dòng **không một lời cảnh báo**. ⇒ Vá theo (b) — một đường vào ô, một luật. ⚠️ Vế *"hai chữ ở hai đầu ranh giới không được dính"* mà chú thích cũ bảo vệ **vẫn được giữ**: `\n` là một dấu tách thật, không phải bị bỏ đi — đó chính là điều đã đổi kể từ khi lý lẽ cũ được viết. Phép gộp khoảng trắng ngang (`[ \t]+`) giữ nguyên.
- [x] [Review][Patch] 🔵 **Ice ký 2026-08-16 — GIỮ `--color-primary`, chỉ ghi rõ đây là lượt tái dùng CÓ CHỦ Ý.** **`--color-primary` nay mang hai nghĩa trong cùng một khung nhìn** — cột vạch dùng nó cho *"hàng đang có con trỏ"* (`GridPanel.vue:1251`); story này dùng **đúng token đó** cho *"cờ kết đoạn bản dịch đang BẬT"* (`:1430`). Doc-comment của Quyết định #4 đo rất kỹ vế **hình học** (vì sao không `padding-bottom`) nhưng không bàn vế **token màu**. Hai chỗ khác hình dạng (nền vạch vs viền dưới) nên nhầm lẫn là có thể, không chắc chắn. ⇒ Không đổi token. Vá: một dòng chú thích tại `:1430` nói thẳng lượt tái dùng là có chủ ý và vì sao (hai hình dạng khác nhau, cùng nghĩa rộng *đang bật*), để lượt rà sau không hỏi lại cùng một câu.
- [x] [Review][Patch] Nợ **AC6** được tự chấm 🟡 bằng lời nhưng KHÔNG có mục trong sổ nợ [`_bmad-output/implementation-artifacts/deferred-work.md`] — diff thêm **7** mục (AC3 · AC4 · ô lỗi lệnh · khuyết tật `check-i18n` · e2e Blink · fixture · NFR2), không mục nào là AC6; trong khi `sprint-status.yaml` viết *"BA AC đóng một nửa, cả ba ghi nợ có chủ"*. Câu đó sai với chính diff của nó. Vá: thêm mục AC6 kèm chủ, và sửa câu ở `sprint-status.yaml`.
- [x] [Review][Patch] Hai tầng đọc hai định nghĩa **"ô rỗng"** khác nhau [src/panels/GridPanel.vue:1060] — lưới dùng `=== ''`, Rust dùng `target_text.trim().is_empty()` (`commands/segment.rs:886`). Ô chỉ chứa `"\n"` (bấm `Enter` trong ô rỗng) **mất** class `.cell-tgt.empty` nên trông đã dịch, nhưng `confirm_segment` từ chối ký nó — không giải thích vì sao. Cộng thêm: bàn đo Task 1 chạy cả bốn vòng trên ô **đã có** nội dung `"A"`, **không** vòng nào đo `Enter` là thao tác gõ đầu tiên trong ô rỗng.
- [x] [Review][Patch] Hai chuỗi từ chối dùng chung mang từ vựng **"xác nhận"** [src/i18n/vi.json:19-20] — `"…không có gì được xác nhận."` / `"…không xác nhận được nữa."`. Doc-comment của lệnh mới khẳng định *"cùng ngữ nghĩa, cùng thông điệp"* — đúng ở mức `message_key`, **sai ở mức câu chữ**. Hôm nay vô hình (lệnh chưa có đường ra màn hình); ngày món nợ *"đường báo lỗi dùng chung"* được đóng, người bấm `Mod+Alt+P` trên câu đã về hưu đọc một câu nói về xác nhận. Test chỉ so `message_key` enum nên không bắt được.
- [x] [Review][Patch] Chú thích MỚI gọi sai cơ chế thật đang giữ AC11 [src/panels/GridPanel.vue] — doc-comment của `onEditKeydown` nói `Enter` trần bị `keys.ts::isTypingZone` chặn. Đọc `keys.ts:508-510`: vòng lặp `continue` ở `entry.code !== event.code || !sameMods(...)` **trước** khi chạm `isTypingZone`; lệnh duy nhất gắn `Enter` là `Mod+Enter`, nên `Enter` trần không bao giờ khớp và không bao giờ tới dòng đó. AC11 giữ đúng — nhưng vì `sameMods`, không vì `isTypingZone`. Hậu quả hôm nay bằng 0; nó dạy sai cho người thêm một hợp âm `Enter` trần sau này.
- [x] [Review][Patch] Chú thích MỚI khẳng định sai phạm vi Kiểm A [src/main.ts:293] — `check-i18n.mjs:839,860-861` cho thấy Kiểm A quét đúng `rsFiles` (`.rs`) + `vueFiles` (`.vue`). `src/main.ts` là `.ts`, **ngoài phạm vi**: dòng chẩn đoán này có dấu cũng không cổng nào đỏ. Quy ước viết không dấu vẫn tốt; chỉ **lý do** ghi ra là sai.
- [x] [Review][Patch] Giá trị trả về của `document.execCommand('insertLineBreak')` không được kiểm [src/panels/GridPanel.vue:748] — `preventDefault()` đã chạy trước đó, nên nếu lệnh (API đã bị khai tử trong đặc tả) trả `false` thay vì ném, `Enter` **biến mất không dấu vết**: chốt `insertingLineBreak` chỉ canh ca `execCommand` NÉM. Bàn đo chứng minh nó chạy trên WebKit; nửa Blink là món nợ đã ghi. Vá tối thiểu: một dòng chẩn đoán khi trả `false`.
- [x] [Review][Defer] Cùng khẳng định sai về Kiểm A đã có sẵn ba chỗ [src/main.ts:248,264,281] — deferred, pre-existing (Story 2.5 · 2.5c). Diff này chỉ lặp lại mẫu, không tạo ra nó.

---
baseline_commit: 26d89d1de15b74e1f84a6d00bc081b76c4b8d63b
---

# Story 2.1: Tách segment cấp câu và cờ kết đoạn

Status: ready-for-dev

**Covers:** FR23 (`prd.md:421-427`) · A4 — giả định *"tách câu tự động đúng ở tỷ lệ chấp nhận được"* (`prd.md:1075`)
**Epic:** 2 — Biên tập theo segment · **story ĐẦU của epic** (epic chuyển `backlog → in-progress` ở lượt này)
**Nguồn:** `epics.md:1986-2032` · AD-3 · AD-4 · AD-5 · AD-11 · AD-21 · AD-28 · AD-30 · AD-31 · AD-37 · AD-39 (`ARCHITECTURE-SPINE.md`)
**Nợ đóng ở đây:** action item **A6** của Epic 1 *(vết sẹo `PROJECT_MIGRATIONS` số 4)* · `deferred-work.md:542` *(`segment_count = 0` trên mọi Chương Epic 1)* · `deferred-work.md:561` *(`\r` của CRLF chưa chuẩn hoá — phần thuộc 2.1)*
**Nợ ĐI QUA đây mà KHÔNG đóng:** `deferred-work.md:254` *(bản sao lưu trước di trú không nguyên tử, không xác minh lại)* · `:195-206` *(sáu số Tuning, chủ là 2.4)* · `:1169-1180` *(không cổng nào cấm tái dùng một số di trú — xem Task 3.4)*

---

## Điều kiện khởi hành — ĐỌC TRƯỚC KHI GÕ MỘT DÒNG

### 1. Cây làm việc SẠCH, và đây là mốc gốc

`git status --porcelain` trả **0 dòng** lúc dựng story này (2026-08-12). `baseline_commit` ở frontmatter là SHA thật của `HEAD`: `26d89d1`. Không có món vá cũ nào phải commit riêng trước — khác Story 1.21, bắt được đi thẳng vào Task 0.

### 2. Bước di trú kế tiếp của `project.db` là số **5**, KHÔNG phải 4

Đây là **action item A6 của Epic 1**, chủ là Dev, còn mở, và điều kiện đóng của nó viết bằng chữ là *"story 2.1 dựng ra đã mang mệnh đề đó"*. Số 4 là một số **đã cháy**, và vết sẹo đã nằm sẵn trong mã — `schema.rs:280-296`:

> *"Bản đầu của Story 1.20 (2026-08-10) thêm **bước 4** đặt `PINNED_ENTRY_DDL` vào bộ này... Ngày 2026-08-11 Ice ký lại... và bước 4 **bị gỡ**. […] Số **4** vì thế là một số **đã cháy**: bước di trú kế tiếp của `project.db` phải đánh số **5**, không được tái dùng 4."*

`PROJECT_MIGRATIONS` hôm nay có **đúng ba** bước (`schema.rs:297-310`): `to_version: 1` `SCHEMA_MIGRATION_LOG_DDL` · `2` `WORK_DDL` · `3` `CHAPTER_DDL`.

🔴 **`validate_strictly_increasing` KHÔNG bắt được lỗi này.** Nó chỉ kiểm tăng dần nghiêm ngặt — `[1, 2, 3, 4]` là một danh sách hợp lệ hoàn hảo. Viết `to_version: 4` sẽ đi qua **mọi** cổng hiện có mà không một dòng đỏ nào, và hậu quả chỉ lộ ra ở máy người dùng từng chạy bản v4 cũ. Xem Task 3.4 — story này dựng cổng còn thiếu đó.

### 3. Dữ liệu THẬT trên đĩa mang `\r\n` chưa chuẩn hoá

`deferred-work.md:561` giao món nợ này đích danh cho 2.1:

> *"Chuẩn hoá xuống dòng (CRLF → LF) và khoảng trắng CỐ Ý KHÔNG làm ở Story 1.15 — `core::segment::import::import_text` giữ nguyên byte văn bản sau khi giải mã... Hệ quả phải biết: mọi Chương nhập từ một tệp Windows ở Epic 1 mang `\r\n` trong `chapter.source_text`, và Story 2.1 (tách câu) phải xử lý `\r` như khoảng trắng, không để nó dính vào cuối segment. Chủ: Story 2.1 + Story 6.4/6.5."*

Chuẩn hoá THẬT (FR124/FR125) là Epic 6. Story 2.1 **không** chuẩn hoá `chapter.source_text`; nó chỉ phải **tự phòng thủ** trong bộ tách. Lý do phải phòng thủ ngay bây giờ chứ không đợi Epic 6: AD-4 đóng băng ranh giới **vĩnh viễn** lúc ghi, nên một `\r` dính vào cuối segment hôm nay là một segment sai không sửa lại được bằng Epic 6.

### 4. Mọi bằng chứng của story này chỉ xanh trên macOS

Ice chốt 2026-08-12 (`deferred-work.md:1861-1918`): trọn phần Windows dời về **cuối dự án**, Ice sẽ tự dựng máy. Và CI thôi tự chạy lúc push — `ci.yml` giờ khai `workflow_dispatch`, tiêu 0 phút cho tới khi có người bấm.

> *"Hệ quả phải nói thẳng: mọi thứ Epic 2 → Epic 9 thêm vào sẽ chạy **chỉ trên macOS** cho tới lượt đó. Khoảng mù không đứng yên — nó dày lên theo từng epic."*

⇒ Sau khi push, **phải tự bấm `workflow_dispatch`** để có bằng chứng CI. Không giả định. Bài học §8.1 của retro: 12 lượt CI đỏ trôi qua 6 ngày vì không ai đọc.

### 5. Một khái niệm TRÙNG TÊN đã tồn tại — đừng nhầm

`src/panels/wordBoundary.ts` và `SourceHanViet.vue:118` đã có kiểu **`Segment`** ở TypeScript: `{ kind:'han'; chars: string[]; readings: (string|null)[] }`. Đó là segment **cấp TỪ**, sinh bằng `Intl.Segmenter('zh', {granularity:'word'})`, sống ở webview, tính lại mỗi lần Chương nạp — Story 1.18b.

Segment của story này là **cấp CÂU**, ở Rust, trong `project.db`, tính **một lần** rồi đóng băng. Hai thứ khác hẳn nhau về tầng, đơn vị và vòng đời. Đừng đặt tên đụng nhau, và **đừng** lấy `Intl.Segmenter` làm cơ chế tách câu (xem Task 0 · Quyết định #1).

---

## Story

As a người dịch,
I want văn bản được chia thành từng câu ngay khi nhập và ranh giới đó ổn định mãi mãi,
So that lịch sử và trạng thái công việc của tôi không bao giờ trỏ sai chỗ.

---

## Acceptance Criteria

Nguyên văn từ `epics.md:1994-2032`, đánh số để tham chiếu:

**AC1** — **Given** văn bản tiếng Trung · **When** tách segment · **Then** tách theo `。！？；`

**AC2** — **Given** văn bản tiếng Anh · **When** tách segment · **Then** tách theo `. ! ?` có xử lý các trường hợp viết tắt không phải kết câu

**AC3** — **Given** một Chương được nhập · **When** tách segment · **Then** kết quả **lưu xuống** `project.db` · **And** không đường mã nào tính lại ranh giới lúc nạp Chương

**AC4** — **Given** mỗi segment · **When** tạo · **Then** mang `segment.id` bất biến · **And** thứ tự trong Chương là cột riêng `ord`, sắp lại được mà không đụng `id`

**AC5** — **Given** một `segment.id` đã về hưu · **When** cấp id mới · **Then** id đó **không bao giờ** được tái dùng

**AC6** — **Given** mỗi segment · **When** tạo · **Then** mang **cờ kết đoạn** tính cùng lượt với ranh giới câu và lưu xuống đĩa · **And** là **một cờ duy nhất dùng chung** cho cả nguyên văn và bản dịch

**AC7** — **Given** segment cuối cùng của một Chương · **When** tính cờ kết đoạn · **Then** cờ **tắt, luôn luôn**

**AC8** — **Given** quy tắc tách câu được cải thiện về sau · **When** áp dụng · **Then** chỉ áp qua thao tác **tái tách chủ động** của người dùng trên từng Chương, kèm cảnh báo về dữ liệu sẽ về hưu · **And** không có đường nào tự động tách lại toàn bộ Thư viện

### AC bổ sung — dẫn xuất từ kiến trúc và từ đo đạc mã nguồn

Tám AC trên không nói hết thứ phải đúng để tính năng chạy được trong hệ thống đang có. Bảy AC dưới đây **cùng hạng ràng buộc**, mỗi cái neo vào một nguồn kiểm chứng được:

**AC9 — `AUTOINCREMENT` là cơ chế DUY NHẤT thoả AC5, và nó phải nằm trong DDL.** `schema.rs:225-231` đã phân xử bằng chữ cho bảng `chapter`: *"`INTEGER PRIMARY KEY` trần là bí danh của `rowid`, và SQLite **tái dùng** rowid đã xoá khi nó là rowid lớn nhất từng cấp — cụ thể, xoá hàng cuối rồi chèn hàng mới sẽ nhận lại đúng `id` vừa mất."* AC5 không phải một lời hứa ở tầng Rust; nó là một thuộc tính của DDL. Nghiệm thu: tạo 3 segment, xoá segment cuối, tạo segment mới ⇒ `id` mới **phải** là 4, không phải 3. *(Đây là bản sao đúng khuôn `project_contract.rs::a_retired_chapter_id_is_never_handed_out_again`.)*

**AC10 — bước di trú mới đánh số `5`, và một cổng cấm số 4 quay lại.** Xem §Điều kiện khởi hành mục 2. `deferred-work.md:1169-1180` ghi món nợ *"không cổng nào cấm tái dùng một số di trú"* và gợi ý chủ là *"story đầu tiên thêm bước di trú cho `project.db`"* — tức chính story này. Nghiệm thu: một test khẳng định `PROJECT_MIGRATIONS` không chứa `to_version == 4`, và test đó **đỏ** khi cố tình đổi thành 4.

**AC11 — `\r` không bao giờ dính vào một segment.** Xem §Điều kiện khởi hành mục 3. Nghiệm thu: tách `"Câu một.\r\nCâu hai."` ⇒ segment thứ nhất là `"Câu một."`, **không** `"Câu một.\r"`; và đối chứng âm — một assertion kiểm rằng không segment nào chứa `\r`.

**AC12 — bộ tách sống ở Rust, trong `core/segment/`, và không một đường nào ở TypeScript.** `EXPERIENCE.md:23` khai đích danh: *"Frontend **không chứa quy tắc nghiệp vụ** (AD-1). Tách câu, khớp ngôn ngữ, phân giải scope đều nằm ở Rust. Ngoại lệ tường minh duy nhất: văn bản đang gõ trong Editor là state cục bộ, đẩy xuống Rust theo hợp đồng flush của AD-35."* — ngoại lệ đó là Story 2.3, không phải chỗ này. Bản đồ năng lực của kiến trúc đặt C2 Workspace ở `core/segment/` (`ARCHITECTURE-SPINE.md:861`). Nghiệm thu: một test biên đúng khuôn `store_boundary.rs` — không tệp nào ngoài `core/segment/**` nhắc tới bảng chữ cái kết câu, và không tệp `.ts`/`.vue` nào chứa `Intl.Segmenter` với `granularity: 'sentence'`.

**AC13 — segment ghi xuống CÙNG một giao dịch với hàng `chapter` sinh ra chúng.** `commands/project.rs:119-133` hôm nay đã ghi `work` + `chapter` trong một `store.write` duy nhất. Một Chương tồn tại mà segment của nó chưa tồn tại là đúng trạng thái `segment_count = 0` mà `deferred-work.md:542` đang bắt story này dọn — dựng lại nó ở đường nhập mới là dựng lại chính món nợ. Nghiệm thu: một test giả lập lỗi giữa chừng ⇒ **không** hàng `chapter` nào và **không** hàng `segment` nào còn lại.

**AC14 — lỗi mới qua IPC theo AD-21, và không chữ tiếng Việt có dấu ở vị trí mã `.rs`.** AD-21 (`ARCHITECTURE-SPINE.md:302-306`): hình dạng `{ code, message_key, params, retryable }`. `check-i18n.mjs` Kiểm A (`:836`) quét **mọi** tệp `.rs` dưới `src-tauri/src/**`, gồm cả `debug_assert!`. Nghiệm thu: `npm run check:i18n` xanh; khoá chuỗi mới có mặt trong `src/i18n/vi.json`.

**AC15 — mọi sàn `*_FLOOR` bị vượt được nâng theo SỐ THẬT, đo chứ không ước.** Story này thêm tệp `.rs` (ít nhất `core/segment/split.rs` và `tests/segment_contract.rs`). Sàn phải rà: `RS_FLOOR = 35` (`check-i18n.mjs:276`, số thật sau 1.21 là 41) · `RS_FLOOR = 20` trong `store_boundary.rs:52`, `scope_boundary.rs:50`, và sàn tương ứng ở `dict_boundary.rs:306`. Nếu story thêm tệp `.ts` thì `TS_FLOOR = 26` (`check-commands.mjs:212`); nếu thêm command thì `COMMAND_FLOOR = 29` (`:219`). Số thật đo được ghi vào §Completion Notes, không ước.

---

## Task 0 — NĂM QUYẾT ĐỊNH, chốt TRƯỚC dòng mã đầu tiên

Khuôn cố định của mọi story lớn trong dự án (1.17 · 1.18 · 1.19 · 1.20 · 1.21). Mỗi quyết định có **đề xuất mặc định kèm lý lẽ đo được**. Dev đọc, xác nhận hoặc phản biện **bằng số** — không im lặng thi hành, và không tự đổi sau khi đã gõ mã.

### Quyết định #1 — Bộ tách câu: viết mới ở Rust, KHÔNG mượn thư viện

Ba đường đã cân, hai đường bị **loại bằng phép đo**:

**(a) `unicode-segmentation` — `unicode_sentences()` theo UAX #29. LOẠI.**
Đo trên `unicode-segmentation v1.13.3` ngày 2026-08-12, chạy thật:

| Đầu vào | Kết quả | Phán quyết |
|---|---|---|
| `他走了；她笑了。` | `n=1` → `["他走了；她笑了。"]` | **trượt AC1** — UAX #29 không coi `；` là ranh giới câu |
| `Mr. Smith went home. He slept.` | `n=3` → `["Mr. ", "Smith went home. ", "He slept."]` | **trượt AC2** — cắt ngay sau `Mr.` |
| `他走了。她笑了。` | `n=2` | đúng |
| `真的吗？太好了！` | `n=2` | đúng |
| `It costs 3.50 dollars. That is fine.` | `n=2` | đúng, luật số thập phân có sẵn |

Hai trên năm ca là **đúng hai AC bắt buộc của story**. Ghi thêm một dữ kiện để không ai phải đo lại: crate này **đã nằm trong cây mặc định** qua `tauri → muda → keyboard-types` (`cargo tree --locked -e normal -i unicode-segmentation`), nên nếu về sau có việc cần tới nó thì đó là 0 byte payload — cùng tiền lệ `uuid` của Story 1.15. Nhưng việc đó **không** phải tách câu.

**(b) `Intl.Segmenter('zh', {granularity:'sentence'})` ở webview. LOẠI, hai lý do độc lập.**
① AD-1 + `EXPERIENCE.md:23` đặt tách câu ở Rust bằng chữ (AC12). ② Nó chạy **mỗi lần Chương nạp** — đúng thứ AC3 cấm (*"không đường mã nào tính lại ranh giới lúc nạp Chương"*). Thêm một món nợ đã ghi: `Intl.Segmenter` trên WKWebView **chưa từng đo** (chỉ đo trên Chromium, Story 1.18b), và dự án **không có bộ test frontend** để bắt lệch.

**(c) Viết mới, thuần Rust, trong `core/segment/split.rs` — ĐỀ XUẤT.**
Không phụ thuộc mới ⇒ không phải mở một hàng bảng Stack, không phải rà NFR15 (giấy phép), không đụng `check-deps.mjs`. Luật của AC1/AC2 là một tập hữu hạn và đã viết sẵn trong PRD; viết tay 150 dòng rẻ hơn uốn một thư viện không khớp.

🔴 **Đừng thêm crate.** Bảng Stack là thứ `Cargo.lock` xác nhận (`Cargo.toml:24-27`), và mọi crate mới kéo theo một lượt rà NFR15 đọc thân tệp LICENSE. Nếu dev tin rằng cần một crate, **dừng lại và hỏi Ice bằng số** trước khi thêm.

### Quyết định #2 — Hình dạng bảng `segment`

**Đề xuất** — bước di trú `to_version: 5`, hằng `SEGMENT_DDL` riêng (cùng khuôn "mỗi bước một hằng" mà `schema.rs:263-273` đã phân xử: `Migration::sql` là `&'static str`, `concat!` chỉ nhận literal, nên nối hai DDL buộc phải chép lại nguyên văn):

```sql
CREATE TABLE segment (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  chapter_id       INTEGER NOT NULL,
  ord              INTEGER NOT NULL,
  source_text      TEXT    NOT NULL,
  is_paragraph_end INTEGER NOT NULL,
  retired_at       TEXT,
  created_at       TEXT    NOT NULL,
  updated_at       TEXT    NOT NULL
);
```

Từng cột, và vì sao:

| Cột | Lý do neo vào đâu |
|---|---|
| `AUTOINCREMENT` | AC5 + AC9 — cơ chế duy nhất, `schema.rs:225-231` |
| `chapter_id` | AD-32: gộp/tách **Chương** chỉ đổi `chapter_id` và `ord`, giữ nguyên `segment.id` |
| `ord` cột riêng | AC4 + AD-3. **Không** `UNIQUE` trên `ord` — cùng lý do `CHAPTER_DDL` đã ghi (`schema.rs:234-235`): Epic 2 tự quyết cơ chế sắp lại, có thể để hở tạm trong một giao dịch nhiều bước |
| `is_paragraph_end` | AC6, AD-37. **Một** cột, dùng chung nguồn và đích — không `source_paragraph_end`/`target_paragraph_end` |
| `retired_at` | AD-5 "về hưu = tombstone". Story 2.1 **không** cho về hưu segment nào; cột có mặt để 2.8 không phải mở bước di trú thứ hai chỉ để thêm một cột, và để `ornament` (giá trị vạch lề thứ 5, `EXPERIENCE.md:105-115`) có chỗ đọc |
| `created_at` / `updated_at` | Cùng khuôn `chapter`: sinh ở tầng SQL bằng `strftime('%Y-%m-%dT%H:%M:%fZ','now')` (`project.rs:128-129`), không truyền từ Rust |

**Ba chi tiết nhỏ mà bỏ sót sẽ tốn một lượt review:**
- **`ord` đánh số từ 1**, liên tục, không lỗ — cùng gốc với `chapter` (`project.rs:127` chèn `VALUES (1, ...)`). Story 2.10 điều hướng *"segment kế tiếp"* đứng trên giả định này.
- **`is_paragraph_end` là `INTEGER` 0/1** — SQLite không có kiểu boolean. Tầng Rust cưỡng chế giá trị hợp lệ, cùng khuôn `chapter.status` đã phân xử (`schema.rs:237-239`).
- **Chọn nhánh ngôn ngữ bằng `work.source_lang`**, và giá trị phân biệt đang dùng trong kho là chuỗi `'zh'` (`sourcePanelState.ts:285`, `dict.ts:83`). Mọi giá trị khác đi nhánh tiếng Anh — quyết định này phải nằm trong doc-comment, vì FR23 chỉ khai hai ngôn ngữ mà cột thì nhận chuỗi tự do.

**Ba thứ CỐ Ý không có, và mỗi thứ có chủ:**
- `target_text` (bản dịch) — **Story 2.2/2.3** sở hữu, đi kèm bước di trú 6. Thêm hôm nay là đoán trước hợp đồng flush (AD-35) mà 2.3 chưa chốt.
- `status` (máy trạng thái AD-31) — **Story 2.5** sở hữu. AC của 2.1 không nhắc một giá trị trạng thái nào.
- `role` (`alt` | `caption`, AD-42) — **Story 6.13** sở hữu.

⚠️ Nếu dev thấy một cột trong ba cột trên là **bắt buộc** để 2.1 chạy, đó là dấu hiệu phạm vi đang trôi — dừng và hỏi, đừng tự thêm.

### Quyết định #3 — Cờ kết đoạn tính thế nào

AD-37 (`ARCHITECTURE-SPINE.md:437-453`) định nghĩa: cờ mô tả *"sau câu này là xuống dòng"*, tính **cùng lượt** với ranh giới câu, lưu xuống đĩa, và **không đường mã nào suy ra đoạn từ nội dung lúc xuất, lúc nạp hay lúc render**.

**Đề xuất:** một lượt quét duy nhất trả `Vec<(String, bool)>`. Cờ của segment thứ *i* bật khi phần văn bản nằm **giữa** cuối segment *i* và đầu segment *i+1* chứa ít nhất một ký tự xuống dòng (`\n`, sau khi coi `\r` là khoảng trắng theo AC11). Segment cuối Chương: cờ **tắt, luôn luôn** (AC7) — kể cả khi văn bản gốc kết thúc bằng một dòng trống.

Bảng ba ca biên của AD-37 (`:449-453`) — hai hàng đầu thuộc Story 2.8, ghi ra ở đây để 2.8 không phải đi tìm lại:

| Ca | Cờ đi đâu | Chủ |
|---|---|---|
| Gộp segment | theo **câu cuối** của nhóm gộp | Story 2.8 |
| Tách segment | theo **mảnh cuối**; mọi mảnh trước nhận cờ **tắt** | Story 2.8 |
| Segment cuối Chương | **tắt, luôn luôn** | **Story 2.1 — AC7** |

### Quyết định #4 — 25 Chương Epic 1 đã có trên đĩa được tách bằng đường nào

Đây là quyết định **đắt nhất** của story, và AC không nói thẳng. `deferred-work.md:542` để ngỏ đúng hai đường: *"một thao tác tách TƯỜNG MINH trong giao diện (hoặc một bước di trú dữ liệu)"*.

**(a) Nhét phép tách vào bước di trú 5. KHÔNG đề xuất.**
Bước di trú là DDL; chạy một quy tắc nghiệp vụ trong đó trộn hai tầng, và nó chạy **im lặng** lúc mở Tác phẩm — khó phân biệt với đúng cái *"đường tính ngầm lúc nạp Chương"* mà AC3 cấm. Thêm một rủi ro đã ghi và chưa ai vá (`deferred-work.md:254`): bản sao lưu trước di trú **không nguyên tử, không xác minh lại**, và đây sẽ là lượt di trú thật đầu tiên trên một `project.db` **đã có dữ liệu người dùng**.

**(b) Một lệnh IPC tách tường minh — ĐỀ XUẤT.**
- Chương **mới** nhập: tách chạy tự động trong `create_work`, cùng giao dịch (AC3, AC13). Đây là *"khi nhập"* theo đúng chữ của AC3 và của AD-39.
- Chương **cũ** (`segment_count = 0`): tách bằng một lệnh gọi tường minh, một Chương một lượt.
- Lệnh **từ chối** một Chương đã có segment, kèm `message_key` nói rõ lý do. Không ghi đè im lặng.

Bước di trú 5 vì thế chỉ làm **một việc**: `CREATE TABLE segment`. Sạch, và giữ đúng luật `schema.rs:275` — *"Không thêm bước cho một lược đồ chưa tồn tại"*.

⚠️ **Phần AC8 mà story này KHÔNG dựng, và nó không phải lệch spec.** AC8 đòi *"tái tách chủ động kèm cảnh báo về dữ liệu sẽ về hưu"*. Ngữ nghĩa **về hưu** (AD-5: segment cũ thành tombstone, lịch sử vẫn tra được, segment mới bắt đầu với lịch sử rỗng) thuộc **Story 2.8**, và hôm nay chưa có `SegmentVersion` để mà giữ lại. Story 2.1 giao **nửa cưỡng chế được ngay**: đường tự động tách lại **không tồn tại**, và có một test biên khẳng định điều đó (AC8 vế hai, Task 6). Nửa còn lại — nút tái tách kèm cảnh báo — ghi vào `deferred-work.md` với chủ là **Story 2.8**, không sửa `epics.md`. *(Cùng khuôn phân loại "năng lực chưa dựng khác AC sai" mà Ice đã chốt ở review Story 1.20.)*

### Quyết định #5 — Bảng viết tắt tiếng Anh của AC2

AC2 nói *"có xử lý các trường hợp viết tắt không phải kết câu"* mà không liệt kê. FR78 (`prd.md:429`) nói vì sao đường lui tồn tại: *"tách câu tự động luôn sai ở một tỷ lệ nhất định — nhất là với dấu chấm trong viết tắt, số thập phân và hội thoại."* Và A4 (`prd.md:1075`) khai đây là một **giả định**, không một sự thật.

**Đề xuất — bốn luật, theo thứ tự, chỉ áp cho nhánh tiếng Anh:**
1. **Bảng viết tắt** đóng, hằng `&[&str]` đặt tên, sắp xếp: danh xưng (`Mr.` `Mrs.` `Ms.` `Dr.` `Prof.` `St.` `Jr.` `Sr.`), tháng/thứ viết tắt, `etc.` `vs.` `e.g.` `i.e.` `cf.` `al.`, `Inc.` `Ltd.` `Co.`
2. **Chữ cái đầu đơn** — một chữ HOA đứng trước dấu chấm (`J. R. R. Tolkien`) không kết câu
3. **Số** — dấu chấm có chữ số ở cả hai bên (`3.50`) không kết câu
4. **Dấu ba chấm** — `...` và `…` không kết câu *(chọn ở đây, không đợi ca hỏng)*

Sau dấu kết câu thật, ranh giới chỉ chốt khi ký tự **không trắng** kế tiếp là chữ HOA hoặc mở ngoặc kép — hoặc khi đã hết văn bản.

🔴 **A4 là một giả định, và story này là chỗ đầu tiên nó chạm dữ liệu thật.** Task 8 đòi đo tỷ lệ sai trên các Chương Epic 1 có thật và **ghi số vào Completion Notes**. Nếu số đó xấu, đó là một phát hiện có giá trị cho Ice, không phải một thất bại phải giấu — FR78 tồn tại chính vì lý do đó.

---

## Tasks / Subtasks

- [ ] **Task 0 — Chốt năm quyết định** (không AC — điều kiện của mọi task sau)
  - [ ] Đọc §Task 0, xác nhận hoặc phản biện **bằng số** từng quyết định
  - [ ] Ghi phán quyết vào §Dev Agent Record trước dòng mã đầu tiên

- [ ] **Task 1 — Bộ tách câu thuần ở `core/segment/split.rs`** (AC1, AC2, AC11, AC12)
  - [ ] Tạo tệp mới; `pub mod split;` trong `core/segment/mod.rs` *(hôm nay tệp đó đúng 10 dòng, chỉ có `pub mod import;`)*
  - [ ] Hàm **thuần**, không I/O, không `Connection` — cùng khuôn `import_text` (`import.rs:197`)
  - [ ] Nhánh tiếng Trung: `。！？；` (AC1)
  - [ ] Nhánh tiếng Anh: bốn luật của Quyết định #5 (AC2)
  - [ ] Chọn nhánh theo `source_lang` của `work` — **không** đoán ngôn ngữ từ nội dung
  - [ ] `\r` xử lý như khoảng trắng, không dính vào cuối segment (AC11)
  - [ ] **Bốn ca biên phải có test, mỗi ca một quyết định ghi vào doc-comment:** ① văn bản rỗng hoặc chỉ khoảng trắng ⇒ **0 segment** (và Task 4 phải chịu được một Chương 0 segment mà không hỏng) · ② văn bản không kết thúc bằng dấu kết câu ⇒ phần đuôi vẫn là **một segment** · ③ nhiều dấu kết câu liền nhau (`真的吗？？！`) ⇒ **một** ranh giới, không segment rỗng · ④ **không segment nào rỗng hoặc chỉ khoảng trắng**, bất kể đầu vào
  - [ ] Test bảng: mỗi hàng của Quyết định #5 một ca, kèm **đối chứng âm** (không segment nào chứa `\r`)

- [ ] **Task 2 — Cờ kết đoạn, cùng lượt quét** (AC6, AC7)
  - [ ] Trả `Vec<(String, bool)>` từ **một** lượt — không lượt quét thứ hai suy ra đoạn
  - [ ] Segment cuối Chương: cờ tắt, luôn luôn (AC7) — test riêng cho ca "văn bản kết thúc bằng dòng trống"
  - [ ] Doc-comment ghi bảng ba ca biên của AD-37, đánh dấu hai hàng thuộc Story 2.8

- [ ] **Task 3 — Lược đồ `segment` và bước di trú 5** (AC3, AC4, AC5, AC9, AC10)
  - [ ] 3.1 Hằng `SEGMENT_DDL` trong `schema.rs`, hình dạng theo Quyết định #2, kèm doc-comment nêu lý do từng cột
  - [ ] 3.2 Thêm `Migration { to_version: 5, sql: SEGMENT_DDL }` vào `PROJECT_MIGRATIONS` — 🔴 **số 5, không phải 4**
  - [ ] 3.3 Sửa dòng tiêu đề doc-comment `schema.rs:256` (*"Hôm nay **ba** bước"*) cho khớp số mới — code review 2026-08-11 đã bắt đúng lỗi rot này một lần
  - [ ] 3.4 **Cổng còn thiếu:** test khẳng định `PROJECT_MIGRATIONS` không chứa `to_version == 4`, kèm lý do vết sẹo. Chạy đỏ-rồi-xanh: đổi thành 4, xác nhận test đỏ, đổi lại (AC10)
  - [ ] 3.5 Test: xoá segment cuối rồi chèn mới ⇒ `id` không tái dùng (AC5, AC9)

- [ ] **Task 4 — Nối vào đường nhập, cùng giao dịch** (AC3, AC13)
  - [ ] Gọi bộ tách trong `create_work` (`commands/project.rs:94-188`), **trước** `store.write`
  - [ ] `INSERT INTO segment` nằm trong **cùng** closure `store.write` với `INSERT INTO chapter` (`:119-133`)
  - [ ] 🔴 Closure ghi **chỉ SQL** — phép tách chạy **ngoài** nó *(Quyết định #3 của Story 1.15, và `Store::write` giữ writer nối tiếp: CPU trong closure chặn mọi lượt ghi khác)*
  - [ ] Test: lỗi giữa chừng ⇒ không hàng `chapter` nào và không hàng `segment` nào còn lại (AC13)

- [ ] **Task 5 — Lệnh tách tường minh cho Chương đã có** (AC3, AC8, AC14)
  - [ ] Hàm thuần + vỏ `#[tauri::command]` trong `mod wire` — khuôn `commands/chapter.rs:63-110`
  - [ ] Từ chối Chương đã có segment, `message_key` nói rõ lý do; hình dạng lỗi theo AD-21 (AC14)
  - [ ] Wrapper TS `src/config/segment.ts` theo khuôn `src/config/chapter.ts`, hằng tên lệnh khớp hai phía
  - [ ] Khoá chuỗi mới vào `src/i18n/vi.json`; **không** chữ có dấu ở vị trí mã `.rs` (AC14)
  - [ ] Chạy trên 25 Chương thật của Epic 1, đếm segment sinh ra

- [ ] **Task 6 — Cấm đường tính lại tự động** (AC3, AC8, AC12)
  - [ ] `tests/segment_boundary.rs` theo khuôn `store_boundary.rs`: không tệp nào ngoài `core/segment/**` mang bảng chữ cái kết câu
  - [ ] Khẳng định không tệp `.ts`/`.vue` nào dùng `Intl.Segmenter` với `granularity: 'sentence'`
  - [ ] Khẳng định đường đọc Chương (`read_open_chapter`) **không** gọi bộ tách
  - [ ] Sàn `RS_FLOOR` của tệp test mới đặt **dưới** số thật, cùng khuôn `store_boundary.rs:52`

- [ ] **Task 7 — Cổng và sàn** (AC15)
  - [ ] Chạy đủ 11 cổng: `check:deps` `check:tokens` `check:i18n` `check:commands` `check:layout` `check:scope` `check:scope:bundled` `check:dict` `check:dict-manifest` `check:gates` `check:lint`
  - [ ] `cargo test` toàn bộ trong `src-tauri/`
  - [ ] Rà mọi `*_FLOOR` bị vượt, nâng theo **số thật đo được**, ghi số vào Completion Notes
  - [ ] Push xong **tự bấm `workflow_dispatch`** và đọc kết quả — không giả định

- [ ] **Task 8 — Bàn đo tay, và số của giả định A4** (AC1, AC2)
  - [ ] Chạy bộ tách trên các Chương thật của Epic 1; đếm tổng segment
  - [ ] Rà tay một mẫu, đếm ranh giới **sai**, ghi **tỷ lệ đo được** vào Completion Notes
  - [ ] Nếu tỷ lệ xấu: nói thẳng với Ice kèm số, đừng vá bằng cách nới bảng viết tắt cho tới khi mẫu xanh

---

## Dev Notes

### Cái đã có, cái chưa có — đo ngày 2026-08-12

| Thứ | Trạng thái | Nguồn |
|---|---|---|
| `core/segment/mod.rs` | **10 dòng**, chỉ `pub mod import;`. Doc-comment tự nói: *"Không tạo `segment` nào ở đây — tách segment thật là Story 2.1 (FR23)"* | đọc tệp |
| `core/segment/import.rs` | 300 dòng. `import_text(raw) -> ImportedChapter` (`:197`) chỉ chạy `strip_bom`; `import_file` (`:241`) kiểm cỡ (`MAX_IMPORT_BYTES = 100 MB`), phần mở rộng `.txt`/`.md`, giải mã UTF-8 nghiêm | đọc tệp |
| bảng `segment` | **CHƯA CÓ**. `CHAPTER_DDL` doc-comment (`schema.rs:241-244`) nói thẳng: *"`source_text` mang **nguyên khối**... Story 2.1 sở hữu bước tách tường minh biến nó thành các hàng `segment`"* | `schema.rs` |
| `PROJECT_MIGRATIONS` | **3 bước** (`schema.rs:297-310`). Kế tiếp là **5** | đọc tệp |
| `ProjectStore` trait | `ports/project_store.rs`, 44 dòng, **chưa có implementor nào**. Chỉ khai `meta()` và `chapter_source_text()`. Story 2.1 **không cần** sửa tệp này — thêm `segment_*` vào đây là một quyết định kiến trúc, không phải mặc định | đọc tệp |
| `EditorPanel.vue` | Khung **rỗng có chủ ý**, 39 dòng. Comment: *"Editor thật... là **Epic 2**. Ở đây thân panel để trống có chủ ý"* | đọc tệp |
| kiểu `Segment` phía TS | Có, nhưng là **cấp từ** (`wordBoundary.ts`, Story 1.18b) — xem §Điều kiện khởi hành mục 5 | đọc tệp |
| vitest / test frontend | **CHƯA CÓ**. Không `vitest` trong `package.json`, không `*.test.ts` nào trong `src/` | đọc tệp |

### Hợp đồng tầng ghi — API chính xác phải dùng

`Store::write` (`core/store/mod.rs:612-618`):

```rust
pub fn write<T, F>(&self, job: F) -> Result<T, StoreError>
where
    F: FnOnce(&Transaction<'_>) -> SqlResult<T> + Send + 'static,
    T: Send + 'static,
```

Mỗi job là **một giao dịch**: `Ok` ⇒ commit, `Err` ⇒ rollback. Đọc dùng `Store::read` (`:632`), mượn từ reader pool `query_only = 1`.

🔴 **Một writer duy nhất, nối tiếp** (AD-11): một `Connection` `move` vào một thread lúc `Writer::spawn`, job đi qua `mpsc::channel`. Nghĩa là **thời gian CPU trong closure ghi chặn mọi lượt ghi khác của tiến trình**. Phép tách một Chương dài không được nằm trong đó — Task 4 nói rõ.

Ví dụ dùng thật để chép khuôn — `commands/project.rs:119-133`:

```rust
let write_result = store.write(move |tx: &Transaction<'_>| {
    tx.execute("INSERT INTO work (...) VALUES (...)", ...)?;
    tx.execute("INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) ...", ...)?;
    Ok(())
});
```

### Hợp đồng dữ liệu mà 2.2 → 2.12 sẽ tiêu thụ

Đây là lý do hình dạng bảng ở Quyết định #2 phải đúng **ngay lượt đầu** — mười một story sau đứng trên nó:

| Hợp đồng | Ai tiêu thụ |
|---|---|
| `segment.id` bất biến, không tái dùng | 2.5 (`SegmentVersion` gắn theo id) · 2.6 (lịch sử tra được **kể cả sau khi về hưu**) · 2.7 (xuất xứ) · 2.8/2.9 (về hưu + tạo mới) · 5.13 (đánh dấu trỏ tới segment đã về hưu vẫn ở lại) · 9.2 (phát hiện proofreader tham chiếu id, **không** tham chiếu vị trí) |
| `ord` cột riêng, sắp lại không đụng `id` | 2.10 (điều hướng) · 2.12 (sync scrolling) · 5.8 (tổ chức lại Chương chỉ đổi `chapter_id`/`ord`) |
| Cờ kết đoạn đã lưu | 2.2 (render trang liền mạch) · 2.8 (ba ca biên) · 6.16 (nhập song ngữ lấy cờ từ ranh giới hàng) · 8.4/8.6 (xuất `.docx`/`.md` đọc cờ **đã lưu**, không suy ra lúc xuất) |
| Ranh giới cố định, không tính lại | 2.3/2.4 (flush và NFR18/NFR2 — không phải trả phí tính lại ranh giới trong vòng lặp gõ) |
| Segment độc lập với TM | 7.1 (TM khoá theo cặp văn bản, **độc lập hoàn toàn** `segment.id` — AD-6, nên TM sống sót qua gộp/tách) |

Hai chỗ **cố ý KHÔNG** dùng `segment.id`, ghi ra để dev không "sửa" nhầm: TM khoá theo cặp văn bản (AD-6, `epics.md:4817-4824`), và ghi nhớ proofreader khoá theo `(work, chữ ký phát hiện)` (`epics.md:6039-6050`) — cả hai để sống sót qua gộp/tách.

### UX ràng buộc ngược lên mô hình dữ liệu

- `DESIGN.md:380` — vạch lề dọc 2px, máng 22px, **cao đúng bằng câu tương ứng**, và đây là *"cách duy nhất trạng thái segment được hiển thị"*. Hệ quả cho 2.1: một hàng `segment` = đúng một câu hiển thị được, không nhóm.
- `DESIGN.md:382` — ranh giới câu là ký tự `⏐`, `opacity: 0` mặc định. Ranh giới do 2.1 tính là **dữ liệu hiển thị trực tiếp**, không phải thứ frontend suy ra.
- `DESIGN.md:400` — *"Không chia Editor thành ô hay bảng"*. Segment là câu trong một dòng chảy liên tục.
- `EXPERIENCE.md:99` — vạch lề **đã dùng hết năm giá trị**; proofreader phải dùng gạch chân. Năm giá trị là tài nguyên hữu hạn, đừng tiêu thêm.
- Mockup chạm: `key-screen-workspace.html` · `workspace-dark.html` · `bilingual-import.html` · `reading-mode.html`.

### Chuẩn kiểm thử của kho

- **Hai loại tệp test**, phân theo hậu tố: `*_contract.rs` = hành vi lúc chạy · `*_boundary.rs` = kiểm tĩnh trên cây nguồn (grep chuỗi cấm ở vị trí mã).
- **Tên hàm test** là một câu mô tả hành vi, `snake_case`, **không** tiền tố `test_`. Ví dụ có thật: `creating_a_work_lays_down_exactly_three_things_on_disk` · `a_retired_chapter_id_is_never_handed_out_again` · `the_semantics_table_matches_ad_18_row_by_row`.
- Khuôn *"đối chiếu một bảng trong tài liệu từng hàng"* (AD-18 / `scope_contract.rs`) là khuôn nên theo cho bảng ba ca biên của AD-37 và bảng viết tắt của Quyết định #5.
- **Không có** script `test` trong `package.json` — Rust test chạy thẳng `cargo test` trong `src-tauri/`.
- e2e: 4 spec, chạy **tuần tự** (`maxInstances: 1`), chỉ macOS/WKWebView, ~3 phút một lượt. Nếu story này viết e2e, **bắt buộc** `realClick()` — ESLint cấm `.click()` trong `e2e/**` kể từ Story 1.22, sau khi một `element.click()` che giấu đúng một khuyết tật sản phẩm thật (UX-DR17 hỏng trên WKWebView).
- 🔴 Bộ e2e **chập chờn**: 8 lượt gần nhất 6 xanh / 2 đỏ, một lần đỏ chưa chẩn đoán. Luật đã ghi vào `wdio.conf.mjs`: gặp lượt đỏ không tái lập được thì **bắt nguyên văn lỗi TRƯỚC**, đừng chạy lại ngay.

### Bài học Epic 1 áp thẳng vào cách làm story này

1. **Đo trước khi tin** (retro §7.1) — Story 1.20 và 1.21 tự bác chính đề xuất của mình bằng phép đo, và lời giải sau đó đơn giản hơn. Quyết định #1 của story này đã được quyết bằng một phép đo chạy thật, không bằng lý lẽ.
2. **Cổng mới phải vào CI, không chỉ chạy tay** (retro §4) — `check:lint` từng sống một ngày ngoài CI. Cổng của Task 3.4 phải là một `cargo test` thật, không một lời nhắc trong doc-comment.
3. **Nợ nghiệm thu thị giác có hệ số nhân** (retro §5) — Story 1.21 đi từ 12 lên 19 hàng bàn đo treo. Story 2.1 gần như không có bề mặt thị giác; giữ nguyên như thế, đừng nhân tiện dựng UI cho 2.2.
4. **`in-progress` không phải chỗ đậu** (retro §8.2) — nếu phải để dở, ghi **nguyên nhân cụ thể** trong story file, không chỉ đổi nhãn.
5. **Ký hiệu cấm** — emoji "biển cấm" `U+26D4` đã gỡ khỏi toàn kho (8.298 ca, 0 còn lại, 2026-08-07). Viết `không`/`KHÔNG` thẳng. Thấy nó bò ngược vào một bản vá thì gỡ ngay.

### Git intelligence — 5 commit gần nhất

`26d89d1` gitignore · `4a118d7` CI chỉ chạy khi bấm tay · `78ee81d` nâng tầng vỏ giao diện lên mốc macOS 13px · `f729cc2` C3 đóng bằng một phép đo · `404d3c3` luật trả tiêu điểm dùng chung + fixture workspace.

Đọc được từ đó: **không commit nào chạm `src-tauri/src/core/**` hay `schema.rs`** trong 5 lượt gần nhất — toàn bộ là e2e, token giao diện, CI và tài liệu. Story 2.1 vào một vùng mã đang **yên**, và mọi thứ nó sửa ở tầng lưu trữ là mới. Khuôn thông điệp commit của kho: `<type>(<scope>): <câu tiếng Việt mô tả điều đã thay đổi>`.

### Phụ thuộc mới — không có, và đó là chủ ý

Bảng Stack ghim **chính xác** bằng `=` (`Cargo.toml:24-27`), và lý do ghi ngay trong tệp: *"bảng Stack trở thành thứ `Cargo.lock` xác nhận, không phải một danh sách trong tài liệu mà mỗi story diễn giải lại."* Story này thêm **0 crate**. `jieba-rs` (tách **từ** tiếng Trung, AD-17, `core/matching`) và `tantivy-stemmers` không dùng ở đây — chúng phục vụ khớp thuật ngữ cho Glossary/TM, chưa có consumer thật.

---

### Project Structure Notes

Tệp **mới** story này tạo:

```
src-tauri/src/core/segment/split.rs      # bộ tách câu + cờ kết đoạn (Task 1, 2)
src-tauri/tests/segment_contract.rs      # hành vi lúc chạy (Task 1, 2, 3.4, 3.5)
src-tauri/tests/segment_boundary.rs      # kiểm tĩnh cây nguồn (Task 6)
src/config/segment.ts                    # wrapper IPC (Task 5)
```

Tệp **sửa**:

```
src-tauri/src/core/segment/mod.rs        # + pub mod split;
src-tauri/src/core/store/schema.rs       # + SEGMENT_DDL, + Migration to_version: 5, sửa dòng "ba bước"
src-tauri/src/commands/project.rs        # nối bộ tách vào create_work, cùng giao dịch
src-tauri/src/commands/mod.rs            # đăng ký lệnh mới
src-tauri/src/lib.rs                     # invoke_handler
src/i18n/vi.json                         # khoá chuỗi lỗi mới
scripts/check-i18n.mjs                   # RS_FLOOR nếu vượt
```

**Không** đụng: `ports/project_store.rs` (xem bảng trên) · `core/matching/**` · `src/panels/**` (Editor là 2.2) · `EditorPanel.vue` · hình dạng `OpenChapter` trong `commands/chapter.rs` — Story 2.2 sở hữu việc đưa segment lên giao diện, và đổi hình dạng đó hôm nay là dựng nửa hợp đồng của 2.2 mà không có gì tiêu thụ nó.

Quy ước đặt tên đã đo: Rust `snake_case` · Vue `PascalCase.vue` · khoá i18n phẳng theo dấu chấm (`lookup.empty_result`) · module Rust đặt theo **khái niệm miền**, không theo nhóm năng lực (`ARCHITECTURE-SPINE.md:656-659`).

---

### References

- FR23 — `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md:421-427`
- FR78 (gộp/tách là đường lui), FR125 (thứ tự chuẩn hoá trước tách), A4 — `prd.md:429`, `:347-349`, `:1075`
- AC nguyên văn — `_bmad-output/planning-artifacts/epics.md:1986-2032`
- AD-3 · AD-4 · AD-5 — `ARCHITECTURE-SPINE.md:89-111`
- AD-11 (một writer) — `:153-157` · AD-21 (hình dạng lỗi IPC) — `:302-306`
- AD-28 (id cục bộ) — `:350-354` · AD-30 (lược đồ có phiên bản) — `:362-366` · AD-31 — `:368-392` · AD-32 — `:394-398`
- AD-37 (cờ kết đoạn, ba ca biên) — `:437-453`
- AD-39 (pipeline nhập có thứ tự cố định) — `:465-504`
- Bản đồ năng lực C2 → `core/segment/` — `:861`
- Vết sẹo `user_version = 4` — `src-tauri/src/core/store/schema.rs:280-296`
- Doctrine `AUTOINCREMENT` — `schema.rs:225-231` · `CHAPTER_DDL` — `:245-254`
- `Store::write` — `src-tauri/src/core/store/mod.rs:612-618`
- `create_work` một giao dịch — `src-tauri/src/commands/project.rs:119-133`
- `segment_count = 0` — `_bmad-output/implementation-artifacts/deferred-work.md:542`
- Nợ CRLF giao cho 2.1 — `deferred-work.md:561`
- Hai quyết định của Ice (CI · Windows) — `deferred-work.md:1861-1918`
- Action item A6 + retro §10 — `_bmad-output/implementation-artifacts/epic-1-retro-2026-08-11.md:229-231`, `:291`, `:305-306`
- UX vạch lề và ranh giới câu — `ux-designs/.../DESIGN.md:380`, `:382`, `:400`; `EXPERIENCE.md:23`, `:99`, `:105-115`

---

## Dev Agent Record

### Agent Model Used

*(điền lúc thực thi)*

### Debug Log References

### Completion Notes List

Bắt buộc có mặt trước khi story rời `in-progress`:
- Phán quyết cho từng quyết định của Task 0
- Số thật của mọi `*_FLOOR` bị chạm (AC15)
- **Tỷ lệ ranh giới sai đo được** trên Chương Epic 1 thật — số của giả định A4 (Task 8)
- Tổng segment sinh ra khi tách 25 Chương cũ
- Kết quả lượt `workflow_dispatch` đã bấm tay

### File List

### Change Log

| Ngày | Mốc gốc | Ghi chú |
|---|---|---|
| 2026-08-12 | `26d89d1` | Story dựng, cây làm việc sạch 0 dòng |
| 2026-08-12 | `5ec8e3d` | **Mốc THẬT để dev bắt đầu.** Giữa lúc dựng story và lúc dev khởi hành có một commit không liên quan hạ cánh: gỡ 3.098 tệp `.claude/` · `.agents/` · `_bmad/` khỏi index. Không chạm một tệp nào story này sửa. `baseline_commit` ở frontmatter giữ nguyên giá trị lúc dựng |

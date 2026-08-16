---
baseline_commit: 440c6d5cb079ea8212eecbd64c8f03e8e8c13f34
---

# Story 2.7: Xuất xứ bản dịch cấp segment

Status: done

**Epic:** 2 — Biên tập theo segment · **Covers:** FR117
**Story trước:** 2.6 (done 2026-08-16) · **Story sau:** 2.8 (backlog)
**baseline_commit:** `440c6d5` — cây làm việc **SẠCH** lúc dựng story (`git status --porcelain` trả 0 dòng, không cần commit vá riêng)

---

## Story

As a **người dịch làm cả vai biên tập**,
I want **hệ thống tự biết câu nào là chữ của tôi và câu nào là của người khác**,
so that **kho Translation Memory về sau không bị trộn phong cách**.

---

## Acceptance Criteria

**AC1** — **Given** người dùng gõ bản dịch rồi xác nhận · **When** ghi xuất xứ · **Then** là **tôi dịch**

**AC2** — **Given** người dùng **sửa** một câu sẵn có rồi xác nhận · **When** ghi xuất xứ · **Then** là **tôi dịch** — câu sau khi sửa là chữ của họ

**AC3** — **Given** người dùng duyệt **nguyên văn** một câu sẵn có, không sửa gì, rồi xác nhận · **When** ghi xuất xứ · **Then** **giữ nguyên xuất xứ lúc nạp segment**

**AC4** — **Given** hệ thống xác định câu có bị sửa hay không · **When** thực hiện · **Then** so **văn bản đích hiện tại với bản lúc nạp segment** · **And** **không dùng cờ dirty**

**AC5** — **Given** người dùng gõ rồi hoàn tác về đúng nguyên trạng rồi xác nhận · **When** ghi xuất xứ · **Then** coi như **không sửa**

**AC6** — **Given** xuất xứ · **When** ghi · **Then** ghi **cùng lúc với chuyển tiếp sang đã xác nhận**, không ở chỗ nào khác

**AC7** — **Given** người dùng · **When** dùng tính năng này · **Then** **không có thao tác nào thêm** — hệ thống không hỏi

---

## Điều kiện khởi hành

- ✅ **Không cửa chặn nào.** Story 2.5 (máy trạng thái + `segment_version`) và 2.6 (đường đọc + index bước 10) đều `done`; đường ghi mà story này móc vào đã tồn tại và có test hợp đồng.
- ⚠️ **Story 2.3 và 2.4 vẫn `in-progress`.** Story này **không tự chấm đạt** món nào của hai story đó, đúng cách 2.5 → 2.6 đã đi qua. Cụ thể: vế *"báo lỗi flush ra màn hình"* (`deferred-work.md:2882-2886`) và mọi số hiệu năng NFR2/NFR18 vẫn thuộc 2.4.
- ⚠️ **Không món nào của story này cần một máy Windows** — nửa Windows đang gom về cuối dự án (action item Epic 1, chủ Ice).
- ⚠️ Cây làm việc phải **sạch** trước khi bắt đầu; diff của story phải đọc được một mình. Lúc dựng story: sạch trên `440c6d5`.

---

## 🔴 Quyết định mở — Ice chốt TRƯỚC dòng mã đầu tiên

> *"Ice là người chốt các quyết định mở. Gặp một chỗ hai phương án đều hợp lệ: nêu cả hai kèm số đo, đừng tự chọn rồi đi tiếp — và cũng đừng loại một phương án chỉ vì nó đắt"* (`project-context.md:464-466`).
>
> Dev agent **dừng ở đây** và trình tám quyết định. Không tự chọn. **Task 0 chặn mọi task khác.**

### Quyết định #1 — Cột xuất xứ nằm ở BẢNG NÀO

**Số đo.** Hai tài liệu chỉ vào hai bảng khác nhau, và cả hai đều là nguồn hợp lệ:

- `ARCHITECTURE-SPINE.md:368-392` (AD-31) viết *"Xuất xứ ghi vào **segment** và vào **cặp TM**"*, và ERD `:818-844` viết `CHAPTER ||--o{ SEGMENT : "chứa — segment mang xuất xứ và cờ kết đoạn"`.
- `schema.rs:449-452` lại đặt cột xuất xứ ở `segment_version`: *"ĐÚNG BỐN CỘT… Xuất xứ (FR117, Story 2.7) và cặp TM (FR56, Epic 7) ghi tại cùng một chuyển tiếp, nhưng cột của chúng thuộc story chủ của chúng."*

**Phép đo bác một đường.** AC3 đòi một giá trị đọc được **LÚC NẠP**, trước bất kỳ lượt xác nhận nào. Một segment chưa từng xác nhận có **0 hàng** `segment_version` — đường `INSERT INTO segment_version` duy nhất nằm trong nhánh chuyển tiếp thật của `confirm_segment` (`commands/segment.rs:1367-1371`). ⇒ Đường *"chỉ `segment_version`"* **không biểu diễn được AC3**. `segment.<cột xuất xứ>` là bắt buộc; câu hỏi còn lại là `segment_version` **có thêm nữa hay không**.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Chỉ `segment.<xuất xứ>` | Rẻ nhất, một cột, khớp AD-31 + ERD. ⚠️ Món nợ "bốn nhãn" của 2.6 (`deferred-work.md:3685-3697`) **không đóng được** — lớp phủ lịch sử vẫn không hiện xuất xứ **của từng phiên bản** |
| **(b)** | Cả `segment` **và** `segment_version` | Đóng được cả nhãn theo phiên bản (mockup `data-integrity.html:221` vẽ `Xuất xứ: tôi dịch` cho **một hàng phiên bản**) và mở đường cho Quyết định #5. ⚠️ Hai cột trong một bước; và nó **đoán trước** một hợp đồng mà chính `schema.rs:449-452` cấm đoán trước cho Epic 7 |
| **(c)** | Chỉ `segment_version` | **BỊ PHÉP ĐO BÁC** — xem trên. Ghi ra để không ai đi lại |

🔴 **Một tiền đề của món nợ 2.6 KHÔNG đứng được khi đọc lại, và Ice cần biết trước khi ký.** `deferred-work.md:3794-3796` viết rằng *"cột xuất xứ (FR117) làm câu hỏi 'hàng nào đang dùng' trả lời được theo `id` chứ không theo nội dung"*. Đọc lại: một cột **xuất xứ** trên `segment_version` **không** nói hàng nào đang dùng — hai phiên bản cùng mang `tôi dịch` thì nhãn lại khớp nhiều hàng, đúng lớp lỗi mà chính món nợ đó tồn tại để chống. Thứ trả lời được câu đó là một **con trỏ** (ví dụ `segment.current_version_id`), không phải một cột xuất xứ. ⇒ Nếu Ice muốn đóng nhãn *"đang dùng"* ở story này thì đó là **một cột thứ ba**, không phải hệ quả miễn phí của (b). Nếu không, món nợ giữ nguyên và **đổi chủ**, không tự chấm đạt.

### Quyết định #2 — Ai giữ mốc "bản lúc nạp", và ai chạy phép so

**Số đo — mốc KHÔNG tồn tại trên đĩa.** `unconfirm_edited_segments` (`commands/segment.rs:1653`) *có* so nội dung, nhưng so với **đĩa tại lượt flush** — câu `UPDATE … AND target_text <> ?5` ở `commands/segment.rs:1737-1741` — và đĩa bị ghi đè dần theo từng lượt flush AD-35. Đánh dấu xuất xứ theo từng lượt flush **phá AC5**: gõ `AB` (flush: khác ⇒ đánh dấu), hoàn tác về `A` (flush: lại khác ⇒ đánh dấu), văn bản cuối y hệt bản lúc nạp mà cờ đã bật. Đây **đúng** ca mà AD-31 §Hợp đồng phụ gọi tên: *"cờ dirty nói đã sửa, so sánh văn bản nói không đổi"*.

**Số đo — mốc CÓ tồn tại ở webview, và nó SỐNG SÓT qua lượt gõ + flush.** `ensureSegmentsLoaded` khoá bằng cờ `requested` (`editorPanelState.ts:93`) nên nạp **đúng một lần** mỗi phiên panel; `confirmCurrentSegmentUnguarded` vá ảnh chụp **cố ý không đụng** `target_text` (`editorPanelState.ts:678-684`). Và `editorPanelState.ts:194-207` khai bằng chữ, từ trước story này, rằng `segments` giữ bản lúc nạp **dành cho FR117** và phải tách rời `editedText`.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Webview tính `edited: bool` rồi gửi qua dây | Rẻ nhất trên dây. 🔴 Nhưng nó đặt **một quy tắc nghiệp vụ** vào TypeScript — va thẳng AD-1 (`:75-79`), mà ngoại lệ tường minh duy nhất của AD-1 là *"văn bản đang gõ là state cục bộ frontend"*, không phải *"phép phân xử xuất xứ"* |
| **(b)** | Webview gửi **văn bản lúc nạp**, Rust so | Quy tắc ở lại Rust; TS chỉ chở **dữ liệu nó sở hữu hợp pháp**. Giá: một chuỗi nữa trên dây mỗi lượt xác nhận, và Rust tin một giá trị do webview khai |
| **(c)** | Rust tự giữ bản đồ mốc trong `OpenWork`, nạp ở `read_open_chapter_segments` | Cả quy tắc lẫn dữ liệu ở Rust, dây không đổi một byte. Giá: state phiên **mới** ở tầng Rust (chưa có tiền lệ), bộ nhớ cho tới 9.850 `target_text`, và một luật ghi rõ *"lần đọc đầu thắng"* — lượt đọc thứ hai **không** được làm mới mốc, nếu không nó tự xoá chính thứ nó giữ |

⚠️ Cả ba đường đều không cưỡng chế được ở tầng Rust cái thứ tự *flush trước, xác nhận sau* — món nợ `deferred-work.md:2731-2739` (chủ: *"story nào dựng bề mặt xác nhận thứ hai"*) áp nguyên vào đây và **nặng thêm**: một bề mặt tương lai `invoke('confirm_segment')` thẳng sẽ ghi **cả một xuất xứ sai**, không chỉ một văn bản cũ.

### Quyết định #3 — Tập giá trị, kiểu cột, tên cột

**Số đo.** FR117 (`prd.md:443`) khai **đúng ba** giá trị: *tôi dịch* · *người khác dịch* · *nhập từ tài liệu song ngữ*. Hôm nay **không đường mã nào sinh ra hai giá trị sau**: FR115 (nhập song ngữ) là Epic 6, FR58 (điền sẵn từ TM) là Epic 7, AI là Epic 4. Một Chương vừa nhập có `target_text = ''` (mặc định của bước 6, `schema.rs:383-384`) — và xuất xứ của một bản dịch **rỗng** là một câu hỏi đặc tả không trả lời.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | `TEXT NOT NULL DEFAULT '<tôi dịch>'`, ba giá trị, khuôn `status` của bước 7 | Đúng ba giá trị như FR117 khai. ⚠️ Mọi câu **chưa dịch** mang sẵn nhãn *tôi dịch* — một lời khai sai về một câu chưa ai viết, và nó đi thẳng vào Epic 7 |
| **(b)** | `TEXT` **NULL-able**, `NULL` = *"chưa có bản dịch"* | Không nói dối về câu rỗng. ⚠️ Thành **bốn** trạng thái thật trong khi FR117 khai ba — một lượt nới đặc tả, phải viết ra |
| **(c)** | `INTEGER` mã hoá, khuôn `is_omitted` của bước 8 | Gọn trên đĩa. ⚠️ Kho đã có **hai** khuôn và bước 7 (`status TEXT`) là khuôn cho một **enum nhiều giá trị**; số trần trong SQL là thứ không đọc được lúc soi `.db` bằng tay |

🔴 **Tên cột: `origin` đã ĐÔNG NGHĨA trong kho, đo được.** Frontend dùng `origin === 'user'` cho lượt kích hoạt panel dockview (`WorkspaceDock.vue:416,593,609` · `commands/focus.ts:209`), và `editorPanelState.ts:407,744` gọi nhánh gốc của flush là *originator*. Tầng Rust thì `dict_citation` đã có cột `author`/`work` mang nghĩa **xuất xứ trích dẫn từ điển** (`core/dict/senses.rs:91,97,213`). ⚠️ Và PRD dùng đúng chữ *"xuất xứ"* cho **bốn** thực thể rời nhau: bản dịch (FR117) · mục Glossary (FR47) · tài liệu nguồn (FR128/FR131) · trích dẫn từ điển (FR30). ⇒ Tên phải tự phân biệt được; `translation_origin` / `translated_by` là ứng viên, `origin` trần thì không.

### Quyết định #4 — Story này có bề mặt NHÌN THẤY không

**Số đo.** Bảy AC của story **không AC nào** đòi hiển thị, và AC7 đòi *"không có thao tác nào thêm"*. Về phía thiết kế thì không gian thị giác đã cạn: vạch lề segment khai **đúng năm giá trị** khác trống (`DESIGN.md:391`), danh sách đó bị `check-commands.mjs` Kiểm I đối chiếu **hai chiều** với `SEGMENT_RULE_VALUES` (`DESIGN.md:199`), cột nhãn trạng thái rộng 108px đã có chủ (`DESIGN.md:146`), và trục **trạng thái hoàn thành** vuông góc với trục **xuất xứ** — không token nào dành sẵn cho trục thứ hai.

⚠️ Mockup gần nhất có màu xuất xứ là `tm-manage.html` (màn hình Story 7.9), và nó dùng `#b99a5e` cho *người khác dịch* — **trùng giá trị** với token `tm-rule` mà `DESIGN.md:190,391` đã khoá nghĩa *"gợi ý TM chờ xác nhận"*. Bê nguyên bảng màu đó vào lưới là dựng một màu mang hai nghĩa.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Không bề mặt nào — story thuần dữ liệu | Khớp đúng bảy AC. ⚠️ e2e **không quan sát được** một mệnh đề nào của story; nghiệm thu chỉ bằng test hợp đồng Rust. Và bốn nhãn của `deferred-work.md:3685-3697` giữ nguyên, đổi chủ |
| **(b)** | Thêm dòng xuất xứ vào `SegmentHistoryOverlay.vue` (khuôn `data-integrity.html:221`) | Đóng được một trong bốn nhãn, và e2e quan sát được. Giá: một khoá `vi.json` mới + phụ thuộc vào Quyết định #1(b) |
| **(c)** | Thêm cả chỉ dấu vào lưới | 🔴 Đòi một token màu **thứ 18** hoặc một cột thứ sáu; `DESIGN.md:196` đã cảnh báo về token thứ 17, và Kiểm I sẽ đỏ. Đây là **công việc thiết kế mới**, không phải một lượt render |

### Quyết định #5 — Khôi phục (Story 2.6) có trả xuất xứ về không

**Số đo.** `restore_segment_version` **sửa văn bản thật** và vì thế **có** đụng `updated_at` (`deferred-work.md:2812-2816`), khác `confirm_segment` (không đụng). Ở webview, `replaceEditorSegment` vá `target_text` vào ảnh chụp ⇒ nó **định nghĩa lại mốc "bản lúc nạp"** của segment đó giữa phiên. Không AC nào của 2.6 lẫn 2.7 nói tới xuất xứ ở đường này.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Khôi phục trả **cả** xuất xứ của phiên bản đó về | Nhất quán nhất: một phiên bản là một cặp (văn bản, xuất xứ). **Đòi Quyết định #1(b)** |
| **(b)** | Khôi phục **chỉ** đụng văn bản, xuất xứ giữ nguyên, và khai bằng chữ tại chỗ | Rẻ, và đúng khuôn Story 2.6 đã chọn cho `is_target_paragraph_end` (`deferred-work.md:3709-3721`, đường ②). ⚠️ Lặp lại **đúng cái hở** mà món nợ đó đang mở với chủ là Ice |
| **(c)** | Khôi phục hạ xuất xứ về mặc định | Mất dữ liệu, đường tệ nhất. Ghi ra để loại tường minh |

### Quyết định #6 — Backfill cho hàng đã nằm trên đĩa

**Số đo.** Bước 9 là **bước đầu tiên** của kho trộn DDL với DML trong một hằng (`schema.rs:591-594`), chạy trọn trong một giao dịch qua `tx.execute_batch` (`schema.rs:923`) — tiền lệ đã có. Và bài học đắt của 2.5d: `DEFAULT` **không với tới** Chương nhập **sau** lượt di trú, nên `insert_segments` (`commands/segment.rs:96-127`) phải set cột mới **tường minh**, nếu không mọi Chương mới mang giá trị mặc định câm.

Hôm nay mọi segment `confirmed` trên đĩa **chắc chắn** do người dùng gõ — không nguồn nào khác tồn tại (đo ở Quyết định #3).

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Backfill theo hàng: `confirmed ⇒ tôi dịch`, phần còn lại theo mặc định của #3 | Trung thực nhất với dữ liệu đang có. Đòi DDL+DML, khuôn bước 9 |
| **(b)** | Backfill **đồng loạt** một giá trị | Một câu `UPDATE` không điều kiện. ⚠️ Nó khai *tôi dịch* cho cả những câu chưa ai viết |
| **(c)** | Không backfill, để `DEFAULT` lo | Thuần DDL, rẻ nhất. ⚠️ Chỉ hợp lệ nếu #3 chọn (b) NULL-able — với #3(a) thì nó **chính là** đường (b) ở một cái tên khác |

### Quyết định #7 — Adapter mới: kiểm payload lúc chạy hay tin payload

Món nợ **chủ Ice** đang mở nguyên văn (`deferred-work.md:3770-3781`): sau 2.6, `src/config/segment.ts` có **hai loại** adapter — sáu cái tin payload, `readSegmentHistory` kiểm nó lúc chạy bằng `isSegmentVersionArray` (`config/segment.ts:303-325`). Lý do lệch là một lớp lỗi **đã xảy ra thật** (bản đầu 2.5 quên `status` ⇒ `isConfirmed` luôn `false` trên sản phẩm thật trong khi 74/74 vitest xanh). Nếu story này thêm một trường vào `ChapterSegment` hoặc `ConfirmOutcome`, câu hỏi quy ước đó chạm ngay và **không cổng nào canh sự nhất quán này**.

### Quyết định #8 — Phạm vi: story này có chạm đường Review Mode / gộp-tách không

**Số đo — một lỗ đọc ra được từ chính AD-31.** AD-31 liệt kê *"Chấp nhận thay đổi từ Review Mode (FR94)"* là một trong sáu sự kiện của máy trạng thái, nhưng bảng xuất xứ ngay dưới chỉ có **hai hàng dựa trên phép so văn bản**. Đi theo đúng chữ: người dùng chấp nhận nguyên văn đề xuất của reviewer (không gõ một ký tự nào) ⇒ văn bản **khác** bản lúc nạp ⇒ ghi **tôi dịch**. Điều đó ngược đúng câu biện minh của FR117 (`prd.md:452`): *"hệ thống biết được vì nó thấy bạn có gõ hay không"*, và nó thủng đúng lời hứa R13 mà FR117 sinh ra để giữ.

⚠️ Im lặng thứ hai, cùng hạng: AD-5 (`:103-111`) và Story 2.8 định nghĩa segment mới sau gộp/tách là *"chưa xác nhận, lịch sử rỗng"* và **không nói gì** về xuất xứ. Gộp một câu *tôi dịch* với một câu *người khác dịch* cho ra xuất xứ gì?

Đường (a): ghi **hai món nợ có chủ** (Epic 8 · Story 2.8), story giữ phạm vi bảy AC. Đường (b): 2.7 tự chốt luôn ngữ nghĩa cho cả hai. 🔴 Đường (b) là **sửa AD-31**, tức một `AD` mới — `project-context.md:461-463` bắt nó đi qua thủ tục viết ra, không qua một lượt tiện tay.

🔴 **Task 0.4 — CỬA CHẶN.** Nếu chữ ký của Ice cho #1, #5 hoặc #8 đòi sửa một mệnh đề của AD-31 hay AD-5, thì đó là **một `AD` MỚI, không một dòng mã**. **Dừng story lại**, báo Ice, viết `AD` trước. Đây đúng cửa mà Story 2.6 đi qua được nhờ chữ ký #1(a) và `git diff --stat` trên `planning-artifacts` trả **RỖNG**.

---

## Tasks / Subtasks

- [x] **Task 0 — Chốt tám quyết định mở (CHẶN mọi task khác)**
  - [x] 0.1 Trình tám quyết định trên cho Ice, kèm số đo. Không tự chọn.
  - [x] 0.2 Ghi chữ ký + ngày vào §Dev Agent Record.
  - [x] 0.3 Đo lại từ NGUỒN, không chép từ story này: số bước di trú kế tiếp (`PROJECT_MIGRATIONS`, `schema.rs`) · baseline `cargo test --locked` và `npm run test` · `COMMAND_FLOOR` thật do cổng in ra.
  - [x] 0.4 🔴 **CỬA CHẶN** — chữ ký nào đòi sửa AD-31/AD-5 thì **dừng**, viết `AD` mới trước.
  - [x] 0.5 ⚠️ Đọc **nội dung** mọi dòng `grep` khớp, đừng đếm. Kho này ghi kết quả đo vào chú thích rất dày; 2.6 đã sập bẫy này (`paragraph.rs:10` là một doc-comment viết nguyên văn *"grep … cho 0"*).

- [x] **Task 1 — Bước di trú mới (AC1·AC3·AC6)**
  - [x] 1.1 Thêm một hằng DDL mới. 🔴 **KHÔNG** sửa `SEGMENT_STATUS_AND_VERSION_DDL` (bước 7) tại chỗ — một `project.db` đã ở v7 không bao giờ chạy lại hằng đó, sửa tại chỗ cho ra **hai lược đồ khác nhau mang cùng số 7** (`schema.rs`, và `deferred-work.md:2790-2793`).
  - [x] 1.2 Thêm `Migration { to_version: <số đo ở 0.3>, sql: … }` vào cuối `PROJECT_MIGRATIONS`.
  - [x] 1.3 Backfill theo chữ ký #6. Nếu là DDL+DML thì theo khuôn bước 9 (`schema.rs:591-594`).
  - [x] 1.4 🔴 Set cột mới **tường minh** trong `insert_segments` (`commands/segment.rs:96-127`) — bài học 2.5d: backfill không với tới Chương nhập sau lượt di trú.
  - [x] 1.5 Doc-comment cho hằng mới theo văn hoá kho: **vì sao hình dạng này**, phương án bị loại **bị loại bằng gì**, và giới hạn thật.

- [x] **Task 2 — Ba neo số học KHÔNG cổng nào canh (đã sai 3 lần liên tiếp)**
  - [x] 2.1 `segment_contract.rs` — `vec![1,2,3,5,6,7,8,9,10]` thêm số mới, cập nhật câu thông báo.
  - [x] 2.2 `segment_contract.rs` — fixture "tương lai": đổi tên hằng, kích thước mảng `[Migration; N]` **+1**, và số giả **+1**. ⚠️ Đây là neo **lúc biên dịch** (`E0080`), không phải một ca đỏ — gỡ sai thì lỗi biên dịch, không phải test đỏ.
  - [x] 2.3 `pinned_contract.rs` — `PROJECT_MIGRATIONS.len()` **+1** và `schema_version()` **+1**, cập nhật hai câu thông báo.
  - [x] 2.4 Một test `a_project_database_at_version_<N-1>_…` dựng fixture ở bậc trước rồi mở lên đích mới, khẳng định dữ liệu cũ còn nguyên (khuôn của bước 8/9/10).

- [x] **Task 3 — Đường ghi xuất xứ trong `confirm_segment` (AC1·AC2·AC3·AC5·AC6)**
  - [x] 3.1 Sửa **hàm thuần** `confirm_segment` (`commands/segment.rs:1300-1396`) theo chữ ký #2. Vỏ `wire` (`:1877-1891`) chỉ được mang **một lời gọi và không một quyết định nào**.
  - [x] 3.2 🔴 Ghi xuất xứ **trong cùng giao dịch** và **chỉ** ở nhánh ④ (`Ok(true)`, `:1362-1373`) — nhánh chuyển tiếp thật. Nhánh ③ (AC13 của 2.5: đã `confirmed` thì không ghi một byte) **giữ nguyên**, đó là AC6.
  - [x] 3.3 Gỡ/viết lại khối *"MỐI NỐI ĐỂ MỞ"* (`commands/segment.rs:1288-1299`): vế xuất xứ nay **đã đóng**, vế cặp TM (Epic 7) **ở lại**. Sửa tại chỗ kèm 🔵 và ngày, đừng để mệnh đề lặng lẽ sai.
  - [x] 3.4 Nếu chữ ký #2 là (b)/(c): cập nhật `ConfirmOutcome` và/hoặc chữ ký hàm. Nếu là (c): viết luật *"lần đọc đầu thắng"* thành mã, không thành một câu chú thích.

- [x] **Task 4 — Dây IPC và tầng webview (AC4·AC7)**
  - [x] 4.1 Nếu cột mới đi qua dây: thêm trường vào `ChapterSegment` (`commands/segment.rs:187-198`) **và** vào câu `SELECT` của `read_open_chapter_segments` (`:762-765`) **cùng lượt**, cộng một test `the_load_command_carries_the_<x>_column_over_the_wire` (khuôn đã lặp 3 lần: `:2785` · `:2848` · `:2924`).
  - [x] 4.2 Adapter ở `src/config/segment.ts` theo chữ ký #7.
  - [x] 4.3 Nếu chữ ký #2 là (b): `confirmCurrentSegmentUnguarded` (`editorPanelState.ts:734-811`) đọc mốc **trước** bước ③ vá ảnh chụp (`:678-684`), và gửi kèm. 🔴 Mốc phải lấy từ `segments`, **không** từ `editedText`.
  - [x] 4.4 🔴 **AC7 là một mệnh đề PHỦ ĐỊNH và phải được canh:** không hộp thoại, không nút, không phím tắt mới. `EXPERIENCE.md:261-268` không có hàng nào cho xuất xứ và story này **không xin** hàng nào.
  - [x] 4.5 Nếu thêm command: đo lại `COMMAND_FLOOR` từ **số cổng in ra**, không chép — 2.6 dựng sàn trên một số đo sai (chú thích ghi 41, cổng in 44).

- [⊘] **Task 5 — Bề mặt hiển thị — KHÔNG CHẠY.** Chữ ký **#4(a)** (Ice ký 2026-08-16): story
  thuần dữ liệu, không bề mặt nhìn thấy. Bốn subtask dưới đây **không được tick** — chúng là
  công việc chưa làm, không công việc đã xong, và một dấu `[x]` ở đây là một lời khai sai.
  - [⊘] 5.1 Khoá `vi.json` mới — **0** khoá thêm. *(Đo: `git diff --stat src/i18n/vi.json` = rỗng.)*
  - [⊘] 5.2 `@click` ⇒ `dispatch('<id>')` — **0** command mới, `COMMAND_FLOOR` không đổi.
  - [⊘] 5.3 Bẫy `check-i18n` Kiểm A trong comment `.vue` — **0** tệp `.vue` bị chạm.
  - [⊘] 5.4 Token màu — **0** token chạm; `SEGMENT_RULE_VALUES` giữ đúng năm giá trị, Kiểm I xanh.

- [x] **Task 6 — Test**
  - [x] 6.1 Test hợp đồng Rust cho AC1·AC2·AC5 (gõ · sửa · gõ-rồi-hoàn-tác).
  - [x] 6.2 🔴 AC3 dựng bằng **SQL trực tiếp** (không đường sản phẩm nào sinh được xuất xứ khác mặc định hôm nay) — đúng tiền lệ chữ ký #8(a) của Story 2.6 cho `retired_at`.
  - [x] 6.3 AC6: một ca khẳng định xác nhận lại một segment **đã** `confirmed` **không** ghi một byte nào.
  - [x] 6.4 Mỗi ca phải **đỏ được**: gỡ bản vá ⇒ đỏ, trả lại ⇒ xanh. Ghi cặp số vào Debug Log.
  - [x] 6.5 `tests/frontend/support/segmentFixture.ts` (`:42-87`): nếu trường mới qua dây thì thêm vào **cả ba** hàng fixture — một fixture chép tay thiếu trường là đúng lớp lỗi đã cho 74/74 xanh trên sản phẩm hỏng ở 2.5.
  - [x] 6.6 e2e chỉ khi chữ ký #4 khác (a). ⚠️ Bộ e2e đang chập chờn vì hai món nợ hạ tầng còn mở (`deferred-work.md:3345-3354` · `:3117-3139`) — gặp một lượt đỏ không tái lập được thì **BẮT NGUYÊN VĂN TRƯỚC**, và đừng `tail` output.

- [x] **Task 7 — Tài liệu và sổ nợ**
  - [x] 7.1 Ghi mọi vế không nghiệm thu được vào `deferred-work.md` **kèm chủ**. Đóng bằng cách **nối tiếp**, không xoá mục gốc.
  - [x] 7.2 Cập nhật `src/panels/README.md` nếu tầng panel có khái niệm mới.
  - [x] 7.3 🔴 **Không sửa `epics.md`/`prd.md` cho khớp mã đã viết** — năng lực chưa dựng khác lệch spec (`project-context.md:456-458`).

- [x] **Task 8 — Nghiệm thu cuối**
  - [x] 8.1 11 cổng npm (9 đọc-tệp + `check:scope` + `check:scope:bundled` chạy tay, cần cổng 1420 trống) · `build` · `vue-tsc` · `vitest` · `cargo test --locked`.
  - [x] 8.2 Ghi số kèm **ngày và toolchain** — *"số đo không truy nguyên được thì không phải số đo"*.

---

## Dev Notes

### Đọc trước khi viết dòng đầu tiên

`_bmad-output/project-context.md` (130 luật) · `ARCHITECTURE-SPINE.md` §AD-31 (`:368-392`), AD-1 (`:75-79`), AD-5 (`:103-111`), AD-11 (`:153-157`), AD-35 (`:419-425`) · doc-comment của chính tệp đang sửa.

### 🔴 Story này KHÔNG thêm phụ thuộc nào

Không gói Rust, không gói npm. Cửa rà giấy phép NFR15 vì thế **không mở** ở story này. ⚠️ Và đặc biệt: **không cài `similar` lẫn `dissimilar`** — `Cargo.toml:86-89` ghi sẵn cả hai số và **cố ý không cài cái nào**; cài một cái là âm thầm đóng một quyết định kiến trúc chốt ở Story 8.1.

### Ba tầng tài liệu KHỚP nhau — đọc kỹ trước khi tưởng có mâu thuẫn

Bảng thô của FR117 (`prd.md:443`) viết hàng *"duyệt nguyên văn ⇒ **người khác dịch**"*, còn AC3 của story viết *"giữ nguyên xuất xứ lúc nạp"*. Hai câu này **không** mâu thuẫn: AD-31 (`ARCHITECTURE-SPINE.md:368-392`) hoà giải sẵn bằng một bảng riêng — *"**người khác dịch** hoặc **nhập từ tài liệu song ngữ**, giữ nguyên xuất xứ nạp vào"*. ⇒ AC3 là bản **chính xác hơn**, bảng PRD là bản thô. **Đừng sửa `epics.md` cho khớp PRD**, và đừng làm ngược lại.

FR117 ↔ AD-31 ↔ Story 2.7 ↔ Story 7.2 (`epics.md:5066-5096`) đồng thuận rất chặt về: ba giá trị, suy ra tự động, ghi đúng tại chuyển tiếp, so văn bản thay vì cờ dirty. Vùng xám nằm ở chỗ khác (Quyết định #8).

⚠️ **Story 7.4 (`epics.md:5168-5170`) thêm một luật KHÔNG có trong FR117 gốc:** xác nhận một segment **điền sẵn từ TM khớp 100%** mà không sửa ⇒ *"giữ nguyên xuất xứ của cặp TM nguồn"*. Bảng bốn hàng của FR117 chỉ nói tới ca nạp từ FR115, không nói ca FR58. Đây là một mở rộng ngầm của Epic 7, chưa được ghi ngược vào FR117 hay AD-31 — nhưng nó **tương thích** với AC3 nếu #1 và #3 chọn hình dạng đủ chở một giá trị nạp vào.

### Trạng thái hôm nay của khái niệm "xuất xứ" — nó chưa tồn tại ở đâu cả

- Tầng Rust: **0** kết quả cho `origin`/`provenance` mang nghĩa xuất xứ bản dịch. Năm dòng khớp đều thuộc `dict_citation` (xuất xứ **trích dẫn từ điển**, `core/dict/senses.rs:91,97,213`).
- Tầng frontend: **0** kết quả. Mọi `origin` là dockview panel origin hoặc nhánh *originator* của flush.
- `vi.json`: không tiền tố nào cho xuất xứ. Tiền tố đang dùng: `err.segment.*` · `command.editor.*` · `command.history.*` · `panel.grid.*` · `history.*`.
- `MessageKey`: bảy khoá `err.segment.*`, không khoá nào về xuất xứ. 🔴 Luật của danh mục đóng: **không thêm khoá cho một nhánh không chỗ gọi nào đi qua** — chỉ thêm cùng lượt với lệnh dùng nó.

### Lược đồ và di trú

`PROJECT_MIGRATIONS` hôm nay: `1 · 2 · 3 · 5 · 6 · 7 · 8 · 9 · 10` — **số 4 là số đã cháy**, bỏ trống vĩnh viễn. 🔴 **Đọc `PROJECT_MIGRATIONS` trong `schema.rs` mà lấy số kế tiếp, đừng đọc dòng này** — chính `schema.rs:395` và `:485` viết luật đó ra bằng chữ, và `sprint-status.yaml` đã có một dòng hết đúng vì lý do này.

Bảng `segment` đầy đủ hôm nay: `id · chapter_id · ord · source_text · is_paragraph_end · retired_at · created_at · updated_at · target_text · status · is_omitted · is_target_paragraph_end`.
Bảng `segment_version`: `id · segment_id · target_text · created_at` — đúng bốn cột, cộng `idx_segment_version_segment_created (segment_id, created_at DESC)` đến ở bước 10.

⚠️ **`updated_at` có hai hành vi ngược nhau và cả hai đều đúng** (`deferred-work.md:2799-2816`): `confirm_segment` **không** đụng (nó không sửa chữ); `restore_segment_version` **có** đụng (nó sửa chữ). Story này ghi xuất xứ ở đường xác nhận ⇒ **không** đụng `updated_at`.

### Dây IPC — và một lỗi đã xảy ra thật

Khuôn hai lớp: hàm thuần nhận `Option<&OpenWork>` (thứ `tests/**` gọi được không cần webview) + vỏ mỏng trong `mod wire` lấy `State` qua **`try_state`**. Không struct nào đặt `#[serde(rename_all)]` — bốn tên trường của `IpcError` là **dây**.

🔴 **Lớp lỗi đã xảy ra thật, và nó là lý do Task 4.1 gộp ba việc vào một lượt:** bản đầu Story 2.5 thêm cột `status` vào DB nhưng quên thêm vào struct `ChapterSegment` **và** vào câu `SELECT` ⇒ `undefined` phía webview ⇒ `isConfirmed` **luôn false trên sản phẩm thật**, trong khi 74/74 vitest vẫn xanh vì fixture chép tay có sẵn cột. Chỉ e2e bắt được.

`ConfirmOutcome::version_created` (`commands/segment.rs:1170-1181`) đã tồn tại đúng cho việc này: *"Story 2.7 (xuất xứ) cùng Epic 7 (cặp TM) móc vào **chuyển tiếp**, không móc vào trạng thái."*

### Hợp đồng flush và thứ tự — thứ story này KHÔNG được phá

AD-35: idle **2 s** · trần cứng **5 s không reset bởi phím gõ** · xác nhận · rời segment · đóng Tác phẩm. `EDITOR_IDLE_MS`/`EDITOR_HARD_CAP_MS` ở `editorFlush.ts:43,56`. Flush **không** tạo `SegmentVersion`.

`flush_segment_targets` chạy **hai giao dịch nối tiếp**: `unconfirm_edited_segments` **trước**, `save_segment_targets` **sau**. Thứ tự này nằm ở **hàm thuần**, không ở vỏ — vì đo 2026-08-14 cho thấy đảo hai dòng trong vỏ vẫn **54/54 xanh** (một vỏ `#[tauri::command]` cần `AppHandle` nên `tests/**` gọi không được). 🔴 Đừng chuyển một quyết định nào lên vỏ.

`confirmCurrentSegmentUnguarded` gọi `flushEditorBeforeDiscreteWrite()` (`editorPanelState.ts:420-425`) trước khi `invoke`. ⚠️ Hàm đó chạy **hai** lượt flush có chủ ý — code review 2.6 bắt được nó từng chạy **một** lượt trong khi doc-comment khai hai. Nếu story này chép khuôn đó, **chép cả mã**, không chỉ chú thích.

### Bài học từ bốn story trước — đây là chỗ hay hỏng nhất

① **Chữ ký thi hành đúng MỘT NỬA** — khuôn lặp **bốn lần** (2.5b ×3, 2.6 ×1). Nửa khó, có chú thích 🔵 đẹp thì làm; nửa là **một dòng chuỗi** hoặc **một câu phải xoá** thì rơi, và **không cổng nào canh nửa đó**. Hậu quả nặng nhất đã xảy ra: một nhãn bố cục **nói dối** với người dùng.
② **Story có thể nói SAI một điều kiện.** 2.5d Task 5.4 viết sai một tiền đề và chính phép đo bác nó. **Đọc mã mà xác nhận từng tiền đề của story này**, đừng thi hành nó như một mệnh lệnh. Story này đã tự bác một tiền đề của món nợ 2.6 ở Quyết định #1 — kiểm lại cả phép bác đó.
③ **Đọc nội dung dòng `grep` khớp, đừng đếm** — 2.6 sập bẫy này.
④ **Neo số học không cổng nào canh** đã sai **ba lần liên tiếp** (2.5c · 2.5d · 2.6). Task 2 tồn tại chỉ vì lớp lỗi đó.
⑤ **Mã chết do chính mình viết** kèm lời biện minh *"giữ phòng khi Ice đổi ý"* đã bị từ chối **hai lần** — gỡ, đừng giữ.
⑥ **Lập luận cấu trúc không phải phép đo.** Nói *"0 phép tính mới nên NFR2 không đổi"* là suy luận; ghi nó dưới nhãn suy luận, đừng ghi dưới nhãn số đo.

### Project Structure Notes

Nhóm tệp "điểm nóng" xuyên bốn story gần nhất, và gần như chắc chắn là nhóm story này chạm: `src-tauri/src/core/store/schema.rs` · `src-tauri/src/commands/segment.rs` · `src-tauri/tests/{segment_contract,pinned_contract}.rs` · `src/config/segment.ts` · `src/panels/editorPanelState.ts` · `src/commands/index.ts` · `src/i18n/vi.json`.

Cây test frontend ở `tests/frontend/**`, **không** đồng vị trí trong `src/**` (bốn cổng đếm quần thể `src/**`). Vá `happy-dom` sống ở `tests/frontend/support/setup.ts` — 🔴 khoảng thiếu của bản mô phỏng vá **ở đó**, khuyết tật sản phẩm vá **trong `src/`**; đừng thêm `?.` vào mã sản phẩm cho hết đỏ.

### Bẫy đã biết, ghi ra thay vì để phát hiện lại

- ⚠️ Fixture "tương lai" là neo **lúc biên dịch** (`E0080`), không phải một ca đỏ.
- ⚠️ Phép so **chuỗi DDL** sẽ XANH trên một index sai thứ tự cột — đọc hình dạng bằng `pragma_index_info(...) ORDER BY seqno`.
- ⚠️ `check-i18n.mjs` Kiểm A báo FAIL **sai chỗ** khi một tên thẻ được nhắc trong comment của template `.vue` (chưa vá).
- ⚠️ Kiểm A **không** quét `.ts` — một dòng chẩn đoán tiếng Việt có dấu ở `main.ts` không cổng nào canh.
- ⚠️ Chuỗi literal trong `src-tauri/src/**` viết **KHÔNG DẤU**; `tests/**` được miễn trừ nên giữ dấu. Comment tiếng Việt có dấu thì hoàn toàn được.
- ⚠️ Bộ e2e cần cổng 1420 trống và **ghi vào `global.db` cùng thư mục Library THẬT** nếu hai biến chuyển hướng không xuống được tiến trình con. Cổng 4445 đã từng bị `gdrive-su` chiếm.
- ⚠️ Ca `toISOString()` **rỗng nghĩa trên CI** (runner chạy UTC) — nếu story chạm định dạng thời gian, mệnh đề chỉ được canh ở `pre-push` trên máy Ice.
- ⚠️ `mockups/data-integrity.html:259-269` vẽ `segments.db`/`history.db`/`tm.db` rời nhau — **ĐÃ LỖI THỜI**, thực tế là **một** `project.db` (AD-9). `key-screen-workspace.html` còn ở hình dạng văn bản liền mạch **trước lưới** — cũng lỗi thời, đừng dùng làm tham chiếu.

### References

- FR117: `planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md:443-452` · FR56 `:588` · FR118 `:590` · FR62 `:610` · FR70 `:644` · R13 `:1126` · vai biên tập `:98`
- AD-31 (máy trạng thái + **bảng xuất xứ** + hợp đồng phụ): `ARCHITECTURE-SPINE.md:368-392` · AD-1 `:75-79` · AD-3 `:89-93` · AD-5 `:103-111` · AD-11 `:153-157` · AD-34 `:406-417` · AD-35 `:419-425` · ERD `:818-844` · capability map `:895`
- Story 2.7: `epics.md:2446-2485` · ghi chú cài đặt Epic 2 `:873` · Story 2.8 `:2487-2529` · Story 7.2 `:5066-5096` · Story 7.4 `:5168-5170`
- UX: `EXPERIENCE.md:261-268` (bảng phím) · `:466` (mockup tham chiếu) · `DESIGN.md:146` · `:190` · `:199` · `:213` · `:391` · `mockups/data-integrity.html:221`
- Rust: `schema.rs:344-355` · `:383-384` · `:436-475` · `:591-594` · `:748-802` · `:903-957` · `commands/segment.rs:96-127` · `:187-198` · `:752-796` · `:1165-1181` · `:1288-1299` · `:1300-1396` · `:1594-1602` · `:1653+` · `:1877-1891`
- Test Rust: `segment_contract.rs:474-486` · `:506-515` · `:1552-1602` · `:2785` · `:2848` · `:2924` · `pinned_contract.rs:145-182`
- Frontend: `editorPanelState.ts:34` · `:92-116` · `:194-207` · `:209-211` · `:420-425` · `:678-684` · `:686-729` · `:734-811` · `editorFlush.ts:43,56` · `GridPanel.vue:316-319` · `:892-897` · `:1411-1413` · `config/segment.ts:178-190` · `:247-261` · `:303-325` · `:519-539` · `commands/index.ts:827` · `:974-1014` · `SegmentHistoryOverlay.vue:264-277`
- Sổ nợ: `deferred-work.md:2731-2739` · `:2799-2816` · `:3117-3139` · `:3345-3354` · `:3575-3586` · `:3685-3697` · `:3709-3721` · `:3752-3768` · `:3770-3781` · `:3785-3796`

---

## Testing

| Mệnh đề | Đường đúng | Vì sao không đường khác |
|---|---|---|
| Xuất xứ ghi đúng ở ba ca gõ/sửa/hoàn-tác | **Test hợp đồng Rust** | Quy tắc sống ở Rust (AD-1); test gọi được hàm thuần không cần webview |
| Ghi **chỉ** tại chuyển tiếp, không ở chỗ khác | **Test hợp đồng Rust** | Một mệnh đề về giao dịch, không về màn hình |
| Bậc thang di trú và lược đồ | **Test hợp đồng Rust** + neo lúc biên dịch | Cổng tĩnh không đọc SQL |
| Mốc "bản lúc nạp" sống sót qua gõ + flush | **vitest** | Hành vi của module thuần frontend |
| Trường mới đi qua **dây** | **e2e** hoặc test hợp đồng đọc thật | 🔴 vitest **không bắt được** lớp lỗi này — fixture chép tay luôn có sẵn trường |
| Vế thị giác (nếu #4 khác (a)) | **e2e / bàn đo tay** | `happy-dom` không phải WebKit |

**Luật của một cổng:** mã thoát là phán quyết · mỗi cổng phải **đỏ được** và không đỏ oan · lỗi hạ tầng **không phải** một phép kiểm đỏ.

**Luật đo:** không đánh dấu đạt bằng suy luận. Vế nào không nghiệm thu được ở tầng đang làm thì ghi `deferred-work.md` **kèm chủ**, không tự chấm đạt. Số đo ghi kèm toolchain và ngày.

---

## Nợ dự kiến

| Món | Trạng thái dự kiến | Chủ |
|---|---|---|
| AC3 chỉ đối chứng được bằng fixture SQL — **không đường sản phẩm nào** sinh xuất xứ khác mặc định hôm nay | 🟡 nửa | Epic 6 (FR115) · Epic 7 (FR58) |
| Xuất xứ khi **chấp nhận thay đổi từ Review Mode** — AD-31 im lặng, đọc theo chữ thì ghi *tôi dịch* cho một câu người dùng không gõ | 🔴 hở thật | Epic 8 (FR94) |
| Xuất xứ của segment sinh ra từ **gộp/tách** — AD-5 và Story 2.8 im lặng | 🔴 hở thật | Story 2.8 |
| Nhãn *"đang dùng"* trong lớp phủ lịch sử — tiền đề của món nợ 2.6 **không đứng được** (cần một con trỏ, không một cột xuất xứ) | 🟡 nửa, đổi lý do | Ice |
| Ba nhãn còn lại của `data-integrity.html` (`từ bản review` · `từ AI` · `từ TM`) | 🟡 nửa | Epic 8 · Epic 4 · Epic 7 |
| Hai loại adapter trong `config/segment.ts` — câu hỏi quy ước | 🟡 nửa, **KHÔNG nới thêm** (story này thêm 0 adapter) | Ice |
| 🔵 **PHÁT SINH:** cổng AC8 mù với `.10` · `.11` · `.12` suốt hai story | ✅ **ĐÃ ĐÓNG** — phép so trọn hàng, tự kiểm đỏ-rồi-xanh đã chạy | — |
| 🔵 **PHÁT SINH:** một hàng tự mâu thuẫn (`confirmed` + có chữ + xuất xứ `''`) | ✅ **ĐÃ ĐÓNG** — nhánh sửa sentinel + một ca hợp đồng | — |
| 🔵 **PHÁT SINH:** biên `FLUSH_WAIT_MS` chỉ 1.500 ms trên idle AD-35 ⇒ e2e đỏ trên máy có tải | ✅ **ĐÃ CHẨN ĐOÁN** (không do 2.7 — đo: baseline 8/8, nhánh 8/8 khi máy rảnh); 🟡 **bản vá chưa làm** — chờ **sự kiện**, không nới hằng số | story hạ tầng e2e |
| Chữ *"xuất xứ"* mang **bốn** nghĩa rời nhau trong PRD — nguy cơ dùng chung một enum cho hai khái niệm | ⚠️ bẫy | ghi tại chỗ |

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, dev-story) — 2026-08-16.

### Baseline đo trước khi chạm dòng đầu tiên

**Đo lại từ nguồn 2026-08-16** trên HEAD `440c6d5`, trước khi chạm dòng mã đầu tiên. Toolchain:
rustc/cargo 1.97.1 · vitest 4.1.10.

| Đường | Số ghi trong story | **Đo lại** | Khớp |
|---|---|---|---|
| `cargo test --locked` | 372 / 0 / 5 | **372 / 0 / 5** | ✅ |
| `npm run test` (vitest) | 130 / 130 | **130 / 130** (12 tệp) | ✅ |
| `COMMAND_FLOOR` | sàn 37, cổng in 44 | sàn **37** (`check-commands.mjs:252`), cổng in **44** | ✅ |
| Bước di trú kế tiếp | 11 | **11** — `PROJECT_MIGRATIONS` dịch ở `to_version: 10` (`schema.rs:799`), chín bước `[1,2,3,5,6,7,8,9,10]` | ✅ |
| e2e | 8/8 spec · 11/11 ca | **8/8 spec** (9m28, máy rảnh) — năm lượt, sổ đầy đủ ở §Debug Log Ⓗ | ✅ |

**Cây làm việc lúc khởi hành:** không sạch, nhưng thứ bẩn **chính là tạo tác của story này** —
`2-7-….md` (untracked) + entry `2-7` trong `sprint-status.yaml`, cả hai do `create-story` sinh ra.
Không có thay đổi lạ nào ⇒ **không cần commit vá riêng**; diff của story vẫn đọc được một mình.

#### Tiền đề của tám quyết định — đo lại từ NGUỒN (Task 0.5: đọc nội dung dòng khớp, không đếm)

| # | Tiền đề story khai | Kết quả đo | Phán quyết |
|---|---|---|---|
| 1 | Đường `INSERT INTO segment_version` **duy nhất** ở `confirm_segment` nhánh ④ | `grep` toàn `src-tauri/src`: **đúng một** dòng, `commands/segment.rs:1368`, nằm trong nhánh ④ sau chốt AC13 (`:1358`) | ✅ đứng — đường (c) **bị bác** |
| 1 | AD-31 + ERD nói ghi vào `segment` | `SPINE:383` *"Xuất xứ (FR117) ghi cùng lúc với cặp TM"*, bảng `:385-388` ghi *"vào segment và vào cặp TM"*; ERD `:827` *"segment mang xuất xứ và cờ kết đoạn"* | ✅ đứng |
| 2 | `segments` giữ bản lúc nạp và **sống sót** qua gõ + flush | Sáu chỗ ghi `segments.value`: nạp (`:108`) · `replaceEditorSegment` (`:176`) · reset (`:441`) · ba lượt vá **một trường** (`:792` status · `:885` is_omitted · `:939` is_target_paragraph_end). **Không** đường flush nào chạm. `editorPanelState.ts:197-201` khai bằng chữ, từ trước story này, rằng mốc này dành cho FR117 | ✅ đứng |
| 2 | Mốc **không** tồn tại trên đĩa | `confirm_segment` đọc `target_text` từ **đĩa** (`:1318`) — không nguồn nào khác trong hàm thuần | ✅ đứng |
| 3 | FR117 khai **đúng ba** giá trị | `prd.md:443-448`: *tôi dịch* · *người khác dịch* · *nhập từ tài liệu song ngữ*, bảng bốn hàng | ✅ đứng |
| 3 | `origin` đã đông nghĩa | Frontend: `WorkspaceDock.vue:609` `e.origin !== 'user'` (dockview) · `editorPanelState.ts:407,744` nhánh *originator*. 🔵 **Đo hẹp hơn story:** tầng Rust có **0** định danh `origin`/`provenance` — khái niệm xuất xứ ở đó mang tên `work`/`author` (`core/dict/senses.rs:91`) | ✅ đứng, thu hẹp |
| 4 | Vạch lề có **đúng năm** giá trị, cổng đối chiếu hai chiều | `DESIGN.md:391` liệt kê 5; `check-commands.mjs:2146` Kiểm I ①: *"`SEGMENT_RULE_VALUES` có ĐÚNG năm phần tử, và đúng năm cái tên đó"* ⇒ giá trị thứ sáu làm cổng **đỏ** | ✅ đứng |
| 5 | `replaceEditorSegment` vá `target_text` vào ảnh chụp ⇒ định nghĩa lại mốc | `editorPanelState.ts:168-177`, chỗ gọi **duy nhất** `segmentHistoryState.ts:351` (đường khôi phục 2.6) | ✅ đứng |
| 6 | Bước 9 là tiền lệ DDL+DML trong một hằng | `SEGMENT_TARGET_PARAGRAPH_END_DDL` (`schema.rs:591-594`) = `ALTER TABLE …` + `UPDATE segment SET …` | ✅ đứng |
| 7 | `config/segment.ts` có hai loại adapter | 🔵 **Story ghi "sáu cái tin payload" — đo được là BẢY.** Tám adapter: `splitChapterIntoSegments` · `readOpenChapterSegments` · `saveSegmentTargets` · `confirmSegment` · `setSegmentOmitted` · `setSegmentParagraphEnd` · `restoreSegmentVersion` **tin payload**; chỉ `readSegmentHistory` kiểm lúc chạy (`:665`) | ✅ đứng, **số sai +1** |
| 8 | AD-5 im lặng về xuất xứ của segment gộp/tách | `SPINE:103-111`: *"segment mới bắt đầu ở trạng thái chưa xác nhận với lịch sử rỗng"* — **không** một chữ về xuất xứ | ✅ đứng |

### Chữ ký của Ice cho tám quyết định mở

**Ice ký 2026-08-16.** Tám chữ ký, và **hai đường bị chính chữ ký #1(a) giết** trước khi được hỏi.

| # | Ice ký | Ghi chú |
|---|---|---|
| **#1** | **(a)** chỉ `segment.<xuất xứ>` | Đường (c) đã bị phép đo bác **trước** khi hỏi (một `INSERT INTO segment_version` duy nhất, `segment.rs:1368`, nằm trong nhánh chuyển tiếp ⇒ segment chưa ký có 0 hàng ⇒ AC3 không biểu diễn được). Khớp AD-31 + ERD. |
| **#2** | **(b)** webview gửi **văn bản lúc nạp**, Rust so | Quy tắc nghiệp vụ ở lại Rust (AD-1). TS chỉ chở dữ liệu nó sở hữu hợp pháp. |
| **#3** | **(b′)** `TEXT NOT NULL DEFAULT ''` | 🔵 **Đường (b′) KHÔNG có trong story — dev đề xuất, Ice ký.** `''` = *chưa có bản dịch*, ba giá trị kia y FR117. Lý do nó thắng cả (a) lẫn (b): (a) khai *tôi dịch* cho câu chưa ai viết; (b) NULL-able **mâu thuẫn một quyết định đã ký của chính bảng này** — `schema.rs:368-374` cấm bằng chữ hình dạng `Option<String>` cho `target_text` *("`None` và `Some("")` là hai cách nói cùng một điều")*. (b′) lấy vế đúng của (b) mà không phá tiền lệ. |
| **#4** | **(a)** không bề mặt — story thuần dữ liệu | ⚠️ Đường (b) **chết theo #1(a)**, không phải bị loại: nó vẽ xuất xứ cho **một hàng phiên bản**, mà `segment_version` nay không mang xuất xứ. |
| **#5** | **(b)** khôi phục chỉ đụng văn bản, xuất xứ giữ nguyên, khai bằng chữ tại chỗ | ⚠️ Đường (a) **chết theo #1(a)**: không có xuất xứ theo phiên bản để trả về. |
| **#6** | **(a)** backfill theo hàng: `confirmed ⇒ tôi dịch` | Khuôn DDL+DML của bước 9 (`schema.rs:591-594`). |
| **#7** | **tin payload** — theo đa số hiện có | ⚠️ Đo lại: tỉ lệ thật là **7 tin / 1 kiểm**, không phải 6/1 như story ghi. Món nợ quy ước (chủ Ice) **không nới thêm**. |
| **#8** | 🔴 **(b)** — 2.7 tự chốt ngữ nghĩa cho **cả** FR94 lẫn gộp/tách | 🔴 **CỬA CHẶN Task 0.4 KÍCH HOẠT.** Xem mục ngay dưới. |

#### 🔴 Task 0.4 — cửa chặn đã kích hoạt, story DỪNG ở đây

Chữ ký #8(b) đòi thêm luật vào **hai** bất biến đang đứng:

- **AD-31** (`ARCHITECTURE-SPINE.md:383-390`) — bảng xuất xứ có **đúng hai hàng**, cả hai dựa trên
  phép so văn bản. Ca *"chấp nhận thay đổi từ Review Mode (FR94)"* **có** trong bảng máy trạng
  thái ngay trên (`:380`) nhưng **không** có trong bảng xuất xứ. Đọc theo đúng chữ hôm nay: văn bản
  khác bản lúc nạp ⇒ **tôi dịch**, cho một câu người dùng **không gõ một ký tự nào** — ngược đúng
  câu biện minh của FR117 (`prd.md:452`: *"hệ thống biết được vì nó thấy bạn có gõ hay không"*).
- **AD-5** (`:103-111`) — đọc lại toàn văn 2026-08-16: *"segment mới bắt đầu ở trạng thái chưa xác
  nhận với lịch sử rỗng"*, và **không một chữ** về xuất xứ.

⇒ Theo `project-context.md:461-463` (*"Đổi một bất biến kiến trúc là một `AD` MỚI, không phải một
dòng mã"*) và theo chính Task 0.4 của story: **dừng story, viết `AD` trước dòng mã đầu tiên.**
Không task nào khác được chạy — Task 0 chặn toàn bộ.

**Trạng thái lúc dừng:** 0 dòng mã sản phẩm bị chạm. `git diff --stat` trên `src/` · `src-tauri/` ·
`scripts/` · `tests/` · `e2e/` = **rỗng**. Cây nguồn còn nguyên baseline `440c6d5`.

#### Bàn giao — Ice chốt 2026-08-16: `AD` mới giao cho **Winston** (architect)

Ice chọn đường 2 trong ba đường được trình *(dev tự soạn `AD` · giao Winston · quay lại #8(a))*.
Lý do đường này đúng vai: một `AD` đi qua thủ tục kiến trúc đầy đủ thay vì một lượt tiện tay từ
dev-story.

**Hồ sơ bàn giao:** `_bmad-output/planning-artifacts/ad-brief-2026-08-16-xuat-xu-ban-dich.md`.
Nó chở: bảy chữ ký đã có **và ràng buộc chúng đặt lên `AD`** · hai câu hỏi phải trả lời kèm số đo và
đường ứng viên · thứ `AD` **không** được đụng · năm điều kiện nghiệm thu để 2.7 mở lại được · toàn
bộ số baseline để Winston khỏi đo lại.

🔴 **Một số đo chỉ lộ ra SAU lúc Ice ký #8(b), đã ghi vào hồ sơ:** mockup `data-integrity.html` vẽ
**bốn** nhãn xuất xứ (`từ bản review` `:195` · `từ AI` `:203` · `từ TM` `:207` · `tôi dịch` `:221`),
trong khi FR117 khai **ba**. ⇒ Nếu `AD` chọn một giá trị riêng cho ca FR94 thì nó **nới tập giá trị
của FR117**, và lượt nới đó chạm Epic 4 và Epic 7 chứ không chỉ Epic 8. Đây là thông tin có thể làm
Ice muốn cân lại phạm vi — nêu ra chứ không tự quyết.

⚠️ **Hai mục metadata của spine đã lệch, chủ Winston, ghi trong hồ sơ:** frontmatter
`updated: '2026-08-11'` trong khi AD-46 dẫn phiên 2026-08-14; và `project-context.md:590` viết
*"45 `AD`"* trong khi đếm thật `grep -c "^### AD-"` = **46**.

**Điều kiện mở lại Story 2.7:** `AD` mới có mặt trong `ARCHITECTURE-SPINE.md`, trả lời **cả hai**
câu hỏi (FR94 · gộp/tách), và `lint_spine.py` chạy sạch. Lúc đó Task 0.4 mở, và bảy chữ ký kia còn
nguyên hiệu lực — không phải ký lại.

#### 🔵 CỬA CHẶN ĐÃ MỞ 2026-08-16 — `AD-47` có mặt, đo lại chứ không đọc ghi chép

Winston giao `AD-47 — Mốc so xuất xứ là lượt ghi KHÔNG-PHẢI-NGƯỜI-DÙNG gần nhất, không phải
lượt nạp`. **Năm điều kiện nghiệm thu của hồ sơ bàn giao, đo lại từ nguồn:**

| # | Điều kiện | Đo được |
|---|---|---|
| 1 | `### AD-NN` với **Binds · Prevents · Rule** | ✅ `ARCHITECTURE-SPINE.md:675`, đủ ba mục |
| 2 | Trả lời **cả hai** câu hỏi | ✅ FR94 ở bảng ③ *(người khác dịch)* · gộp/tách ở ④ |
| 3 | Nói bằng chữ AD-31/AD-5 đổi gì | ✅ mục ⑦, cộng AD-18 · AD-14 · AD-6 |
| 4 | Nếu nới FR117 thì ghi ai chịu | ✅ **không nới** — ⑥ giữ ba giá trị, điều kiện rơi |
| 5 | `lint_spine.py` sạch | ✅ `{"ok": true, "total_findings": 0}` |
| — | `grep -c "^### AD-"` | **47** |

🔵 **`AD-47` làm story RẺ HƠN chứ không đắt hơn, và đó là kết quả của việc dời story lại.** Nó
**dời mốc** thay vì thêm giá trị: mốc là *"bản do lượt ghi không-phải-người-dùng gần nhất đặt"*.
Hôm nay lượt nạp là lượt ghi loại đó **duy nhất đã cài**, nên hai cách đọc cho **cùng kết quả
trên toàn bộ mã đang chạy** ⇒ **0 giá trị mới**, **0 bước di trú thêm**, và bảy AC hiện có đủ
cho phần 2.7 phải cài. Bảy chữ ký cũ **không** phải ký lại.

**Cây làm việc:** tạo tác của Winston *(spine · `project-context.md` · hồ sơ bàn giao)* được
commit **riêng** trước dòng mã đầu tiên — Ice chốt 2026-08-16, đúng luật *"diff của story phải
đọc được một mình"*. Commit `5a7e007`. ⇒ **Baseline thật của diff story này là `5a7e007`**, còn
`baseline_commit` ở frontmatter giữ nguyên `440c6d5` *(mốc số đo, không mốc diff)*.

🔵 **CHỮ KÝ THỨ MƯỜI — 2026-08-16, từ lượt code review ba tầng, SAU khi story chuyển `review`.**

**Ice chốt: `trim()` cả hai vế của phép so mốc** (`segment.rs:1493`).

Lượt rà tìm ra nhánh ② *(`target_text.trim().is_empty()`)* và nhánh ④ *(`target_text !=
text_at_load`)* cách nhau 22 dòng mà **xử lý khác nhau cùng một nguồn hiểm hoạ**: nhánh ② đã
được một lượt code review TRƯỚC (2026-08-14) vá vì `contenteditable` để lại `U+00A0`; nhánh ④
thì chưa ai xét.

🔴 **Đây là một lượt đọc rộng AC4, nên nó phải được viết ra chứ không lặng lẽ vào mã.** AC4
viết *"so văn bản đích hiện tại với bản lúc nạp"* và chữ đó là so **nguyên văn**. Sau chữ ký
này, một khoảng trắng bao ngoài — kể cả do người dùng **cố ý** gõ — **thôi** được coi là một
lượt sửa. Bảy AC còn lại không đổi.

⚠️ **Vế CHƯA phủ, ghi ra thay vì để người sau tưởng đã xét:** chuẩn hoá Unicode (NFC/NFD). Hai
chuỗi giống hệt nhau trên màn hình vẫn khác nhau từng byte nếu một bên dùng ký tự dựng sẵn còn
bên kia dùng dấu kết hợp. Phủ nốt cần một **phụ thuộc mới** ⇒ phải qua cửa NFR15 ba bước ⇒ ghi
nợ, **chủ: Ice**.

### Debug Log References

#### Ⓐ Bốn phép tự kiểm ĐỎ-RỒI-XANH ở tầng Rust (Task 6.4)

Mỗi lượt: gỡ đúng một mảnh của bản vá → chạy → trả lại → chạy.

| Gỡ cái gì | Ca phải đỏ | Đo được |
|---|---|---|
| `\|\| translation_origin.is_empty()` | `a_signed_sentence_can_never_be_left_claiming_it_has_no_translation` | **1 đỏ / 0 xanh** |
| cả vế ghi xuất xứ khỏi câu `UPDATE` | `confirming_text_the_user_typed_records_it_as_their_own` | **1 đỏ / 0 xanh** |
| phép phân xử → `if true` *(luôn `self`)* | lọc `origin` | **2 đỏ / 4 xanh** — đúng hai ca giữ nhánh *giữ nguyên* (AC3 · AC5) |
| câu `UPDATE` backfill khỏi bước 11 | `a_project_database_at_version_ten_backfills_the_origin_only_for_signed_rows` | **1 đỏ / 0 xanh** |

Trả lại cả bốn ⇒ `segment_contract` **102/102**.

#### Ⓑ Hai phép tự kiểm ĐỎ-RỒI-XANH ở tầng frontend

| Gỡ cái gì | Đo được |
|---|---|
| mốc `loaded.target_text` → `editedText.value.get(id) ?? …` | **2 đỏ / 15 xanh** |
| mốc → `''` cố định | **2 đỏ / 15 xanh** |

Hai lượt đỏ ở **hai cặp ca khác nhau**, và đó là bằng chứng ba ca ④ không trùng vai: lượt gửi
nhầm `editedText` không làm ca *gõ-rồi-hoàn-tác* đỏ *(sau khi hoàn tác, hai giá trị bằng nhau)*,
còn lượt gửi `''` thì không làm ca *câu chưa dịch* đỏ *(mốc đúng của nó **là** `''`)*.

#### Ⓒ 🔴 MỘT TRẦN CỦA NGÔN NGỮ, không một lượt gõ nhầm — `SegmentRow` chạm 12

Cột thứ **mười ba** làm bản cũ *(`type SegmentRow = (…)`, một tuple trần)* **không biên dịch
được nữa**: `std` chỉ `impl` `PartialEq`/`Debug` cho tuple **tới 12 phần tử**. Triệu chứng là
mười mấy `E0369` + `E0277` cùng lúc, không một ca đỏ.

⚠️ **Đường sai rẻ ở đây là gộp hai cột vào một tuple lồng cho đủ 12** — nó biên dịch, và nó làm
chính cổng `the_raw_column_reader_sees_every_column_...` **đếm sai số cột**, tức lượt né trần sẽ
tắt đúng cái lưới tồn tại để bắt cột mới. ⇒ Đổi sang một **tuple struct** với `#[derive]`: `.0` …
`.12` giữ nguyên ở **mọi** chỗ gọi *(0 chỗ dùng phải sửa)*, `derive` chạy ở **mọi** arity, và
phép đếm cột giữ nguyên sự thật.

#### Ⓓ 🔴 MỘT KHUYẾT TẬT CÓ SẴN, bắt được lúc đọc chứ không lúc chạy — cổng AC8 mù với BA cột

`a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else` khẳng định **từng cột**
cho hàng bị sửa, và danh sách đó dừng ở `.9`. Đo 2026-08-16: `.10` (`is_omitted`, Story 2.5c) ·
`.11` (`is_target_paragraph_end`, Story 2.5d) · `.12` (`translation_origin`, story này) — **cả
ba** không có phép khẳng định nào.

🔴 Doc-comment của `SegmentRow` khai rằng bộ đọc được nâng *"CÙNG LƯỢT với bước di trú sinh ra
nó"* **để cổng này không mù**. Vế đó đã làm đúng hai lần; vế còn lại — thêm một dòng
`assert_eq!` — **rơi cả hai lần**. Đúng khuôn *"chữ ký thi hành đúng MỘT NỬA"* đã lặp bốn lần ở
2.5b và 2.6, lần này lặp trên **chính cái lưới dựng ra để chống lớp lỗi đó**.

⇒ Vá bằng một phép khẳng định **không mục lại được**: dựng hàng kỳ vọng bằng chính hàng trước
đó, thay đúng hai trường mà AC8 cho phép đổi, rồi so **trọn hàng**. Một cột thứ mười bốn mai sau
đi vào ca này **miễn phí**. Các `assert_eq!` từng cột ở lại vì **câu thông báo** của chúng.
Tự kiểm: cho câu `UPDATE` của flush chạm `translation_origin` ⇒ **đỏ** ở đúng câu này; trả lại ⇒
**102/102**.

#### Ⓔ 🔴 MỘT CA THƯỜNG NHẬT KHÔNG AC NÀO NÊU — `''` trên một câu ĐÃ có bản dịch

Tìm ra lúc đọc lại đường ghi, **không** lúc thi hành đặc tả. Kịch bản, mọi bước đều có thật hôm
nay: gõ bản dịch → flush ghi xuống đĩa → **đóng Tác phẩm mà chưa xác nhận** → mở lại. Lúc này
mốc lúc nạp **bằng** văn bản trên đĩa, còn `translation_origin` vẫn `''` *(bước 11 chỉ backfill
hàng `confirmed`; flush **không** đụng cột này — AD-47 ① nói flush chở đúng bộ đệm gõ)*.
⇒ Xác nhận mà không sửa đi vào nhánh *"y hệt mốc"* ⇒ một hàng **tự mâu thuẫn** trên đĩa người
dùng: `status = 'confirmed'`, `target_text` có chữ, xuất xứ nói *"chưa có bản dịch"*.

⚠️ Nhánh vá **không** là một đầu vào thứ hai cho phép phân xử *(thứ sẽ đụng hợp đồng phụ AD-31)*
— nó **sửa một sentinel**, và lập luận đọc thẳng từ AD-47 ①(b): mỗi lượt ghi
không-phải-người-dùng đặt mốc **và** đặt xuất xứ trong cùng thao tác ⇒ *"có văn bản mà xuất xứ
rỗng"* nói **không lượt ghi loại đó nào** đặt văn bản này ⇒ nó đến từ bộ đệm gõ ⇒ *tôi dịch*.
Nhánh ② đã loại mọi câu rỗng *(`trim().is_empty()`)* trước điểm này, nên *"có văn bản"* là
**bất biến**, không giả định.

#### Ⓕ Tiền đề bị THU HẸP, ghi vì nó đổi cách đọc một con số

Story viết *"`Cargo.toml:86-89` … không cài `similar` lẫn `dissimilar`"* và story này **không
thêm phụ thuộc nào** — đo lại: `git diff` trên `Cargo.toml`, `Cargo.lock`, `package.json`,
`package-lock.json` = **rỗng**. ⇒ Cửa rà giấy phép NFR15 **không mở** ở story này.

#### Ⓖ 🔴 E2E BẮT MỘT KHUYẾT TẬT MÀ CẢ 382 CA RUST LẪN 133 CA VITEST BỎ LỌT

Lượt e2e đầu: **7/8 spec**, và spec đỏ **không** phải một lượt chập chờn. Nguyên văn, bắt trọn
trước khi chẩn đoán *(luật sau Story 1.22 — và lần này tôi **không** `tail` output)*:

```
Error in "Story 2.6 — …ký → sửa → ký lại → đọc lịch sử → khôi phục → đọc lại từ đĩa"
Error: invalid args `textAtLoad` for command `confirm_segment`:
       command confirm_segment missing required key textAtLoad
    at e2e/specs/segment-history-restore.e2e.mjs:105
```

**Nguyên nhân, một dòng:** `signWith` của spec đó gọi `invoke('confirm_segment')` **thẳng**, đi
vòng qua adapter `src/config/segment.ts`. Tức *"một bề mặt thứ hai gọi lệnh thẳng"* — cái lỗ mà
tôi vừa ghi vào `deferred-work.md` như một **rủi ro tương lai** — **đã tồn tại từ Story 2.6**.

🔴 **Hai điều đọc ra được, và cả hai đáng giữ hơn bản vá:**
1. Đường e2e là lưới **duy nhất** cho hình dạng dây. Đúng khuôn vụ cột `status` của Story 2.5,
   lặp lại **nguyên vẹn**: mọi đường khác xanh, chỉ engine thật đỏ.
2. **Tauri từ chối một tham số THIẾU một cách ồn ào**, không âm thầm cấp một giá trị mặc định.
   ⚠️ Nếu nó im lặng cấp `""`, lượt này đã **XANH** trong khi mọi câu *duyệt nguyên văn* bị gắn
   nhãn *tôi dịch* — tức đúng lớp hỏng mà cả story tồn tại để chống, đi thẳng qua tám spec.

⚠️ **Vế VẪN HỞ, ghi ra thay vì để người sau tự phát hiện:** một chỗ gọi truyền một mốc **sai
kiểu đúng** *(một chuỗi hợp lệ, chỉ không phải bản lúc nạp)* thì Tauri **không có gì để nói**,
và không cổng nào bắt được. Ghi vào `deferred-work.md`, chủ giữ nguyên.

**Bản vá:** `signWith(chapterId, id, text, textAtLoad = '')` cộng một chỗ gọi trực tiếp nữa;
`''` là mốc **đúng sự thật** ở đây *(Chương tạo trong chính ca test ⇒ bản dịch lúc nạp rỗng)*,
không một giá trị mồi cho hết đỏ. Chạy lại spec đó riêng: **2/2 passing (40,6 s)**.

#### Ⓗ 🔵 `editor-typing-flush` — ĐÃ CHẨN ĐOÁN, và nguyên nhân là **PHƯƠNG PHÁP ĐO CỦA TÔI**, không phải mã

**Sổ e2e đầy đủ, năm lượt trọn bộ, ghi cả năm** *(luật sau Story 1.22; không `tail` output)*:

| # | Cây nguồn | Máy | Kết quả | Spec đỏ |
|---|---|---|---|---|
| ① | Story 2.7 | **bận** *(`cargo test` chạy song song)* | 7/8 | `segment-history-restore` — 🔴 **ĐỎ THẬT**, đã vá (Ⓖ) |
| ② | Story 2.7 | **bận** | 7/8 | `editor-typing-flush` `:184` `toContain` |
| ③ | Story 2.7 | **bận** | 7/8 | `editor-typing-flush` `:293` `toBe` |
| ④ | **baseline `5a7e007`** | **rảnh** | **8/8** (10m30) | — |
| ⑤ | **Story 2.7** | **rảnh** | **8/8** (9m28) | — |
| riêng | Story 2.7 | rảnh | `segment-history-restore` 2/2 · `editor-typing-flush` 2/2 ×2 | — |

🔴 **Lượt ④ là phép đo mà tôi đã khai là "bị chặn quyền" — Ice cấp quyền, tôi chạy, và nó LẬT
kết luận tạm của tôi.** Bản trước của mục này viết *"chưa chẩn đoán"* và nghiêng về *"chập chờn
môi trường"*. Sai ở chỗ quan trọng: baseline **8/8** ở lượt đầu tiên làm giả thuyết *"chập chờn
sẵn có"* yếu hẳn, và buộc phải đi tìm biến thật.

**Biến thật, và nó đo được:** cả ba lượt đỏ đều chạy trong lúc tôi cho `cargo test --locked`
chạy **vòng lặp song song** để giết thời gian chờ; lượt ④ và ⑤ chạy trên máy **rảnh**.

⇒ **`FLUSH_WAIT_MS = 3.500 ms` mà idle của AD-35 là 2.000 ms — biên chỉ 1.500 ms**, và trong
biên đó phải lọt: timer idle · một lượt `invoke` · `Store::write` **nối tiếp** của AD-11 · một
lượt fsync WAL. Một máy đang biên dịch Rust ăn hết biên đó. Hai lượt đỏ rơi vào **hai phép khẳng
định khác nhau** *(`:184` và `:293`)*, và **cả hai** đều là *"chữ chưa tới đĩa sau `browser.pause`"*
— tức **một** triệu chứng, không hai.

⇒ **Kết luận: Story 2.7 KHÔNG gây ra hai lượt đỏ đó.** Và câu đó nay dựa trên một **phép đo**
*(⑤ 8/8 trên chính cây nguồn này)*, không một lập luận cấu trúc.

🔴 **PHÁT HIỆN CHO KHO, to hơn story này, và nó ĐẶT TÊN cho một món nợ cũ:** hai món *"bộ e2e
chập chờn"* đã ghi ở `deferred-work.md` nay có một **nguyên nhân đo được** cho ít nhất một phần:
biên 1.500 ms của `FLUSH_WAIT_MS` **không chịu được một máy có tải**. Đó cũng là lý do các lượt
đỏ ấy *"không tái lập được"* — người chẩn đoán sau đó chạy lại trên một máy đã rảnh.
⚠️ Bản vá **không** phải nới hằng số cho hết đỏ *(nới là hạ ngưỡng để cổng thôi đỏ — đúng thứ
`project-context.md` cấm)*; đường đúng là **chờ một sự kiện** thay vì chờ một khoảng thời gian.
Ghi nợ có chủ, không tự sửa trong story này.

⚠️ **Bài học về phương pháp, ghi ra vì tôi vừa mắc:** đừng chạy việc nặng song song với một bộ đo
**thời gian thực trên engine thật**. Bộ đo đó không có cách nào phân biệt *"sản phẩm hỏng"* với
*"máy bận"*, nên mọi lượt đỏ nó cho đều là một cuộc chẩn đoán vô ích. Đúng lớp mà Story 2.4 đã
ghi bằng chữ khi nó dựng *"cổng đo tranh chấp CPU/I-O hai chế độ"*.

### Completion Notes List

**Story 2.7 hoàn tất 2026-08-16.** Bảy AC, mười bốn ca mới, `0` phụ thuộc mới, `0` bề mặt mới.

**Bước di trú 11 ĐÃ TIÊU** (`segment.translation_origin`). Số kế tiếp là **12**. 🔴 Nguồn sự thật
vẫn là `PROJECT_MIGRATIONS` (`schema.rs`), **không** phải dòng này.

| Vế | Cài ở đâu |
|---|---|
| **AC1** gõ ⇒ *tôi dịch* | `confirm_segment` nhánh ④, `target_text != text_at_load` |
| **AC2** sửa ⇒ *tôi dịch* | cùng nhánh, cùng phép so |
| **AC3** duyệt nguyên văn ⇒ giữ nguyên | nhánh *else* của cùng phép so |
| **AC4** so văn bản, không cờ dirty | mốc đến từ `segments` qua dây; **0** cờ dirty ở bất kỳ tầng nào |
| **AC5** gõ rồi hoàn tác ⇒ không sửa | hệ quả của AC4 — mốc là bản **lúc nạp**, không bản lúc flush |
| **AC6** ghi **chỉ** tại chuyển tiếp | ghi trong cùng câu `UPDATE` với `status`, chỉ ở nhánh ④; vế phủ định canh bởi cổng AC8 trọn hàng |
| **AC7** không thao tác nào thêm | **0** command · **0** khoá `vi.json` · **0** hàng bảng phím · `COMMAND_FLOOR` không đổi |

**Nghiệm thu cuối 2026-08-16** — toolchain `rustc/cargo 1.97.1` · `Node 22.22.2` · `npm 10.9.7`
· `vitest 4.1.10`:

| Đường | Baseline | Sau |
|---|---|---|
| 11 cổng npm *(9 đọc-tệp + `check:scope` + `check:scope:bundled` chạy tay)* | 11 xanh | **11 xanh** |
| `npm run build` · `vue-tsc --noEmit` | xanh | **xanh** |
| `npm run test` (vitest) | 130/130 | **133/133** *(+3)* |
| `cargo test --locked` | 372/0/5 | **382/0/5** *(+10)* |
| `COMMAND_FLOOR` | sàn 37 · cổng in 44 | **không đổi** — AC7 |
| e2e | 8/8 (Story 2.6) | **8/8** (9m28, máy rảnh) — xem §Debug Log Ⓖ và Ⓗ |

**Ba quyết định thi hành cần gọi tên, vì chúng không đọc thẳng ra từ AC:**
- **Tên cột `translation_origin`** — ✅ **ICE DUYỆT 2026-08-16**, sau khi được nêu rõ rằng chữ
  ký #3(b′) khai **kiểu và giá trị mặc định**, *không* khai cái tên. Đây là một chữ ký **thứ
  chín**, ghi riêng chứ không gộp vào #3: nó đóng một cột **nằm trên đĩa người dùng**, và cửa sổ
  đổi rẻ *(chưa `.atproj` nào chạy bước 11)* đóng lại ngay lượt mở app đầu tiên sau bản này.
  Lý do đường này thắng, giữ lại để story sau đọc được: không `origin` trần — §Consistency Conventions của spine đòi
  định danh **tự phân biệt được** giữa bốn nghĩa của chữ *"xuất xứ"*, và `origin` trần đã đông
  nghĩa ở frontend *(dockview panel origin · nhánh `originator` của flush)*. Ứng viên còn lại
  `translated_by` bị loại: nó đọc như một **tên người**, mà cột chở một **hạng mục** — và `''`
  không phải câu trả lời cho *"ai dịch"*. ⚠️ **Chữ ký #3 của Ice là (b′) — nó khai kiểu và giá
  trị mặc định, không khai cái TÊN.** Ice muốn tên khác thì đây là chỗ đổi, và đổi bây giờ còn
  rẻ *(chưa `.atproj` nào chạy bước 11)*.
- **Bốn giá trị trên đĩa**: `''` · `self` · `other` · `bilingual_import`. `TRANSLATION_ORIGINS`
  là danh mục đóng, và nó có một **cổng** — không phải một hằng chỉ test đọc.
- **`insert_segments` set cột mới tường minh** dù giá trị đúng **trùng** `DEFAULT ''` hôm nay ⇒
  bỏ nó đi thì **không ca nào đỏ**. Nó vẫn ở đó vì ngày Epic 6 dựng FR115, `DEFAULT` thôi là giá
  trị đúng — đúng bài học 2.5d, cột thứ hai liên tiếp.

🔴 **HAI phát hiện của lượt này KHÔNG nằm trong bảy AC**, cả hai ghi đầy đủ ở §Debug Log:
Ⓓ cổng AC8 mù với ba cột suốt hai story · Ⓔ một hàng tự mâu thuẫn ở ca thường nhật.

🟡 **Không vế nào tự chấm đạt.** Bảy món vào `deferred-work.md`, mỗi món một chủ — trong đó món
*"phiên bản nào đang được dùng"* của 2.6 được **đính chính chứ không đóng**: tiền đề của nó
không đứng được, và chữ ký #1(a) đẩy nó xa thêm một bậc.

### File List

**Rust — nguồn**
- `src-tauri/src/core/store/schema.rs` — hằng `SEGMENT_TRANSLATION_ORIGIN_DDL` (bước 11, DDL+DML) + mục vào `PROJECT_MIGRATIONS`
- `src-tauri/src/core/store/mod.rs` — tái xuất hằng mới
- `src-tauri/src/commands/segment.rs` — bốn hằng giá trị + `TRANSLATION_ORIGINS`; `insert_segments` set tường minh; `confirm_segment` nhận `text_at_load` và phân xử; vỏ `wire`; một doc-comment hết đúng sửa tại chỗ

**Rust — test**
- `src-tauri/tests/segment_contract.rs` — `SegmentRow` thành tuple struct 13 trường; 27 chỗ gọi nhận mốc; ba neo số học; cổng AC8 thêm phép so trọn hàng; **10 ca mới**
- `src-tauri/tests/pinned_contract.rs` — hai neo (`len()` 9→10, `schema_version()` 10→11)

**Frontend — nguồn**
- `src/config/segment.ts` — `confirmSegment` nhận `textAtLoad`, gửi qua dây
- `src/panels/editorPanelState.ts` — đọc mốc từ `segments` trước bước ③; đường từ chối khi không có mốc

**Frontend — test**
- `tests/frontend/editorConfirmSegment.test.ts` — harness ghi lại mốc; **3 ca mới** (khối ④)

**e2e**
- `e2e/specs/segment-history-restore.e2e.mjs` — `signWith` nhận `textAtLoad` (mặc định `''`) + một chỗ gọi trực tiếp nữa; khối lý do ghi lại phép đo ở §Debug Log Ⓖ

**Tài liệu**
- `_bmad-output/implementation-artifacts/deferred-work.md` — bảy món, mỗi món một chủ
- `_bmad-output/implementation-artifacts/sprint-status.yaml` · tệp story này

⚠️ **KHÔNG chạm:** `epics.md` · `prd.md` · `ARCHITECTURE-SPINE.md` *(cả ba là tạo tác quy hoạch;
`AD-47` do Winston viết và đã commit riêng ở `5a7e007`)* · `vi.json` · `src/commands/index.ts` ·
mọi tệp `.vue` · `Cargo.toml` · `package.json`.
⚠️ **`src/panels/README.md` không sửa, có lý do:** tầng panel **không** nhận một khái niệm mới —
mốc là một **chỗ đọc mới** của một bất biến đã có tên và đã được khai bằng chữ ở doc-comment của
`editedText` từ Story 2.3. README của thư mục nói về **vai panel**, không về state module.

### Change Log

| Ngày | Việc |
|---|---|
| 2026-08-16 | Task 0 — tám quyết định trình Ice; tám chữ ký; **cửa chặn Task 0.4 kích hoạt** ở #8(b) |
| 2026-08-16 | Bàn giao hồ sơ `AD` cho Winston; story **DỪNG**, 0 dòng mã sản phẩm bị chạm |
| 2026-08-16 | `AD-47` về; năm điều kiện nghiệm thu đo lại **đạt**; cửa chặn **MỞ** |
| 2026-08-16 | Tạo tác của Winston commit riêng (`5a7e007`) trước dòng mã đầu tiên — Ice chốt |
| 2026-08-16 | Task 1–4 · 6–8; Task 5 **KHÔNG CHẠY** (chữ ký #4(a)) |
| 2026-08-16 | e2e lượt ①: **7/8** — `segment-history-restore` đỏ THẬT, `invoke` thẳng thiếu `textAtLoad` |
| 2026-08-16 | Vá spec; chạy lại riêng **2/2**; chạy lại trọn bộ |
| 2026-08-16 | e2e lượt ② ③: 7/8, `editor-typing-flush` đỏ — chẩn đoán tạm *"chập chờn"* |
| 2026-08-16 | Ice cấp quyền ⇒ đo baseline `5a7e007`: **8/8**. Chẩn đoán tạm bị **lật**; đi tìm biến thật |
| 2026-08-16 | Biến thật = **tải CPU do chính tôi tạo**. Chạy lại nhánh trên máy rảnh: **8/8**. Đã chẩn đoán |
| 2026-08-16 | ✅ Ice duyệt **tên cột `translation_origin`** — chữ ký thứ chín, ngoài tám quyết định của Task 0 |
| 2026-08-16 | Story chuyển sang `review` |
| 2026-08-16 | **Code review BA TẦNG** (Blind Hunter · Edge Case Hunter · Acceptance Auditor, song song, không tầng nào thấy tầng kia): 9 phát hiện thô ⇒ 1 quyết định · 2 vá · 1 nợ · 3 loại |
| 2026-08-16 | ✅ Ice duyệt **`trim()` cả hai vế phép so mốc** — chữ ký thứ **mười**, và nó **đọc rộng AC4** |
| 2026-08-16 | Ba bản vá đã áp; `cargo test --locked` **383/0** · vitest 133/133 · 11 cổng · build. Story chuyển sang `done` |

### Review Findings

**Lượt rà ba tầng 2026-08-16** — Blind Hunter · Edge Case Hunter · Acceptance Auditor, chạy song song, không tầng nào thấy kết luận của tầng kia. 9 phát hiện thô ⇒ 1 quyết định · 2 vá · 1 nợ · 3 loại (2 trùng nhau đã gộp).

- [x] [Review][Decision] ✅ **ICE CHỐT 2026-08-16 — `trim()` cả hai vế** *(chữ ký thứ mười, ghi ở `§Chữ ký của Ice`)*. Đã vá `segment.rs:1493` + doc-comment nêu lý do, cái giá, và vế chưa phủ + ca `a_stray_invisible_space_is_not_an_edit` **đỏ-được đã đo**. Vế Unicode NFC/NFD ghi nợ, chủ Ice. — **Phép so mốc ở nhánh ④ là so THÔ, trong khi nhánh ② ngay trên nó đã phải `trim()` cho cùng lớp hiểm hoạ** — `segment.rs:1493` viết `target_text != text_at_load`, so `String` byte-với-byte. `segment.rs:1471` thì viết `target_text.trim().is_empty()`, và doc-comment 🔵 của nó ghi lý do bằng chữ: *"một `U+00A0` do `contenteditable` để lại KHÔNG rỗng theo `str::is_empty()`"* — tức chính kho đã đo được rằng `contenteditable` để lại ký tự vô hình. Cùng nguồn hiểm hoạ, vá ở một nhánh, để hở ở nhánh kia. Đường đi: người dùng bấm vào một câu sẵn có, không sửa chữ nào, `contenteditable` để lại một `U+00A0` cuối dòng ⇒ flush ghi xuống đĩa ⇒ `target_text` lệch mốc đúng một ký tự vô hình ⇒ ghi `self` cho một câu người dùng chỉ duyệt. ⚠️ **Hôm nay hệ quả BẰNG KHÔNG và phải nói ra**: tập giá trị thật trên đĩa là `{'', 'self'}` *(bước 11 backfill `confirmed`→`self`, `insert_segments` ghi `''`, `confirm_segment` ghi `self`)*, mà `''` đi vào nhánh `is_empty()` ⇒ `self`, còn `'self'` thì hai nhánh cho cùng kết quả. ⇒ Nó **chỉ kích hoạt** khi Epic 4/6/7/8 sinh ra giá trị `other`/`bilingual_import` đầu tiên. Cần Ice chốt vì `trim()` **đổi nghĩa AC4** (*"so văn bản đích hiện tại với bản lúc nạp"* — chữ trong AC là so nguyên văn), và một khoảng trắng cuối do người dùng cố ý gõ thì đúng là một lượt sửa. Trục thứ hai chưa ai xét: chuẩn hoá Unicode NFC/NFD.

- [x] [Review][Patch] ✅ Doc-comment khai một lớp bảo vệ không tồn tại — `is_translation_origin` chưa từng được viết `[src-tauri/src/commands/segment.rs:1204]`. Đã sửa tại chỗ kèm 🔵 và ngày; khoảng hở thật *(`TRANSLATION_ORIGINS` chỉ một ca test đọc, đường đọc sản phẩm không đối chiếu danh mục)* ghi nợ, **chủ Ice** — đóng nó là một quyết định lược đồ *(gặp giá trị lạ thì từ chối mở? hạ về `''`? báo lỗi?)*, không một hàm.
- [x] [Review][Patch] ✅ Mốc ghim theo PHIÊN panel còn xuất xứ đọc SỐNG từ đĩa — vòng ký thứ hai trong cùng phiên không trả lại được xuất xứ gốc phi-`self` `[src/panels/editorPanelState.ts:795 + src-tauri/src/commands/segment.rs:1493]`. **Không sửa mã** — đường chưa tới được hôm nay *(đĩa chỉ mang `{'', 'self'}`)*, và câu phải trả lời trước là một câu ngữ nghĩa: *"xuất xứ lúc nạp"* là lúc nạp **phiên panel** hay lúc bắt đầu **vòng draft hiện tại**. Đã ghi nợ kèm đường đi đầy đủ, **chủ: Epic đầu tiên sinh ra một xuất xứ phi-`self`**.

- [x] [Review][Defer] Không cổng tự động nào canh hình dạng dây `textAtLoad` ↔ `text_at_load` — chỉ e2e chạy tay `[src-tauri/src/commands/segment.rs:2021 + src/config/segment.ts:538]` — deferred, pre-existing (đã có chủ trong `deferred-work.md`)

**Ba phát hiện bị loại, ghi ra thay vì im lặng bỏ:**

1. *"Ca `a_freshly_imported_chapter_starts_with_no_translation_origin` xanh cả khi bỏ `?6`"* — **đúng sự thật, và đã được khai bằng chữ ngay tại chỗ** (`segment_contract.rs:5158-5162`) kèm lý do nó vẫn đáng tồn tại (khoá mệnh đề mà Epic 6 sắp phá). Đây là khuôn *"ghi thẳng chỗ YẾU thay vì giấu"* của kho, không phải một điểm mù bị giấu.
2. *"Chỉ 4/10 ca Rust mới có bằng chứng đỏ-rồi-xanh"* — đọc nhầm phương pháp. Bốn phép tự kiểm đột biến **mã sản phẩm**, không đột biến từng test; hàng *"phép phân xử → `if true`"* cho **2 đỏ / 4 xanh** và hai ca đỏ đó **chính là** AC3 và AC5. Không phát hiện được ca rỗng cụ thể nào.
3. *"Nhánh `translation_origin.is_empty()` che mọi lỗi tương lai quên đặt xuất xứ"* — rủi ro có thật nhưng **đã ghi nguyên văn** trong `deferred-work.md` mục AD-47 ③: *"Quên vế xuất xứ ⇒ lượt xác nhận kế tiếp ghi tôi dịch cho chữ người dùng chưa gõ, và không cổng nào đỏ"*, kèm chủ cho từng cơ chế. Thêm: phương án thay thế *(để flush tự đặt `origin='self'`)* **phá AC5** — gõ rồi hoàn tác về nguyên trạng vẫn bị flush đóng dấu `self`. Nhánh hiện tại đứng được.

#### Phép tự kiểm ĐỎ-RỒI-XANH của lượt vá này

| Gỡ cái gì | Ca phải đỏ | Đo được |
|---|---|---|
| `.trim()` ở vế `target_text` của `segment.rs:1493` | `a_stray_invisible_space_is_not_an_edit` | **1 đỏ / 102 xanh** |

⚠️ **Một bẫy phương pháp gặp thật lúc trả lại bản vá, ghi ra vì nó cho một kết quả sai TRÔNG NHƯ
thật:** `mv tệp.bak tệp` **giữ nguyên mtime cũ**, nên cargo tưởng không có gì đổi và chạy lại
**binary đã đột biến** — ca vẫn đỏ sau khi bản vá đã được trả lại, và đọc thẳng thì nó nói *"bản
vá không có tác dụng"*. `touch` rồi chạy lại: **103/103**. ⇒ Khi phục hồi một tệp trong một lượt
tự kiểm, dùng `git checkout` hoặc `touch` sau `mv`.

**Nghiệm thu lượt vá (2026-08-16):** 11 cổng npm xanh · `npm run build` *(hai lượt `vue-tsc
--noEmit` + `vite build`)* xanh · vitest **133/133** · `cargo test --locked` **383 passed / 0
failed** *(baseline 382, +1 ca mới)* · `segment_contract` **103/103** *(baseline 102)*.
E2E **không chạy lại** — lượt vá không chạm hình dạng dây, và bộ e2e cố ý nằm ngoài `pre-push`.


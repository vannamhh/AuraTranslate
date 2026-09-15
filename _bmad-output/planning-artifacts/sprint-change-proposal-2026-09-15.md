# Sprint Change Proposal — 2026-09-15 · Bốn mục chờ quyết của retro Epic 6

**Người soạn:** `bmad-correct-course` (chế độ Incremental) · **Người duyệt:** Ice
**Baseline:** `1eb781b` (master, chưa đẩy — `origin` đi sau 3 commit) · cây có `sprint-status.yaml` sửa chưa commit
**Nguồn:** `_bmad-output/implementation-artifacts/epic-6-retro-2026-09-15.md` — F4, F5, F9, và câu hỏi mở 1
**Trạng thái:** chờ Ice duyệt lượt cuối. Chưa tệp nào được sửa.

---

## 1. Vấn đề

Retro Epic 6 (2026-09-15, phán quyết `rejected`) để lại bốn việc phải đi qua `correct-course` vì chúng đổi tài liệu quy hoạch, không phải đổi mã.

| # | Việc | Loại |
|---|---|---|
| ① | **6.6b, 6.7b** — hai năng lực bị tách khỏi Story 6.6 và 6.7, chỉ sống trong sổ nợ (AI-9 · F9) | Năng lực chưa dựng, thiếu mục quy hoạch |
| ② | **Story 6.18** — tạm ngưng trong Epic 6, hay dời đi (câu hỏi mở 1) | Xếp lại trình tự |
| ③ | **FR132 cho song ngữ** — AC cắt ngang của Story 6.10 không được đường song ngữ thừa kế (AI-5 · F4) | AC cắt ngang không ai đứng ở cả hai phía |
| ④ | **A13** — tỉ lệ dò đúng bảng mã chưa đo, 0 mẫu (AI-10 · F5) | Phép đo thiếu đầu vào |

Không mục nào là "hiểu sai yêu cầu ban đầu", và không mục nào đòi hoàn tác việc đã làm.

---

## 2. Phân tích tác động

### 2.1 Epic

**Epic 6 không đóng được như quy hoạch cũ.** Sau lượt này: **+3 story** (6.6b, 6.7b, 6.16b), **−1 story** (6.18 gộp vào 10.9). Danh sách FR của Epic 6 giữ nguyên — chính vì FR14, FR122 và FR132 chưa xong trọn nên ba story mới phải nằm ở Epic 6. NFR3 · NFR4 · NFR5 rời Epic 6 sang Epic 10.

**Epic 10** nhận thêm phép đo nghiệm thu ba ngưỡng. Story 10.9 vốn đã khai *"nghiệm thu cuối toàn bộ NFR1–NFR19"* (`epics.md:7001`), nên đây là gộp vào chỗ đã có, không phải mở việc mới.

**Không epic nào bị vô hiệu, không epic mới.** Thứ tự epic `1 → 2 → 3 → 5 → 6 → 4 → 7 → 8 → 9 → 10` không đổi.

### 2.2 Cái giá của việc dời 6.18, ghi thẳng

Epic 4, 7, 8, 9 sẽ dựng trên ba ngưỡng còn tạm, trong khi số **sơ bộ** của Story 5.14 đã vượt hai trong ba (`requirements.md:406-407`):

| Ngưỡng | Trần | Số sơ bộ 5.14 (fixture tổng hợp, `.app` 0 lớp từ điển) |
|---|---|---|
| NFR3 `[A6]` | p95 < 500 ms | 57,981 ms — dưới trần |
| NFR4 `[A7]` | < 3 s | cold median 2.581 ms, **max 4.365 ms** — vượt |
| NFR5 `[A8]` | < 300 MB | **894,570 MB** — vượt ~3 lần |

Nếu A8 đúng là vượt xa, phán quyết tầng PRD sẽ rơi vào lúc phát hành. Ice biết điều này khi chọn.

### 2.3 Tài liệu bị chạm

- **PRD:** hàng NFR3 (`:844`), `[A6]` (`:1105`), **Q4** (`:1150`) — thay đổi **thời điểm** đóng Q4, không thay đổi ngưỡng.
- **SPEC:** `SPEC.md:125` (A13, hai chỗ đã cũ), `SPEC.md:131` (Q4), `requirements.md:405`, `:411`. `risks.md` không đổi.
- **UX:** `EXPERIENCE.md:142` đang định nghĩa "cần xem" chỉ cho *"lần nhập nhiều link"* — hẹp hơn AC Story 6.10 từ trước lượt này.
- **Kiến trúc:** không `AD` mới. Dev kiểm lại AD-44 khi soạn spec 6.7b; nếu phải đổi một bất biến thì dừng và giao Winston.
- **Sổ nợ:** 11 mục mang `Chủ: Story 6.18`; hai mục nợ bảng mã, một trong đó ghi sai đường dẫn.
- **Mã nguồn:** không đổi tên gì. `story_6_18_*.rs` (3 tệp), `lib.rs`, feature `nfr-bench`, thư mục `6-18-ban-do/` giữ nguyên — tên là lịch sử, và là đầu vào của Story 10.9.

### 2.4 Hai chỗ cổng không canh được, đo ở lượt này

- **`check:debt-owner` không phát hiện chủ nợ trỏ vào story đã rời đi.** Nó chỉ hỏi có chuỗi `Chủ:` hay không (`check-debt-owner.mjs:196`). Cổng đang xanh (`0/503 mục mở thiếu Chủ:`) và vẫn xanh sau lượt này. ⇒ Đổi chủ 11 mục phải làm bằng tay.
- **`sprint_plan.py status` đề xuất story kế tiếp theo thứ tự ĐỌC của tệp**, không theo thứ tự thực thi: story `in-progress` đầu tiên, rồi `backlog` đầu tiên (`:565-576`). Hôm nay nó trỏ `1-3-ci-…`, và sẽ trỏ `4-2` trước `6-6b`. ⇒ Thứ tự thực thi phải ghi bằng chữ.

---

## 3. Hướng đi và năm quyết định của Ice (2026-09-15)

Hướng: **điều chỉnh trực tiếp**, cộng một thay đổi **thời điểm** ở tầng PRD. Không hoàn tác story nào, không cắt phạm vi MVP.

| # | Quyết định | Ice chọn |
|---|---|---|
| 1 | Story 6.18 | **Dời sang Epic 10, gộp vào Story 10.9** |
| 2 | FR132 cho song ngữ | **(A)** Story mới — **6.16b** |
| 3 | A13 bảng mã | **(C)** Giữ nợ có chủ, chỉ sửa đường dẫn sai và dòng A13 đã cũ; Ice chưa có tệp GBK/Big5 thật |
| 4 | Thứ tự ba story mới | **Trước Epic 4** |
| 5 | Lượt tách `commands/project.rs` (AI-6) | **Trước ba story**, dưới dạng **spec chỉnh lệ ngoài `epics.md`** (tiền lệ `spec-ca-wal-do-tren-windows.md`) |

**Thứ tự thi hành chốt:**

```
AI-4  →  AI-7  →  tách commands/project.rs (AI-6, spec chỉnh lệ)  →  6-6b  →  6-7b  →  6-16b  →  Epic 4
```

Ba story không phụ thuộc nhau; thứ tự giữa chúng xếp theo id và đổi được.

---

## 4. Đề xuất sửa chi tiết

### 4.1 Story 6.6b — Nhập nhiều tệp cùng lúc (`epics.md`)

**Ở Story 6.6**, AC7 (`:4724-4726`) được thay bằng ghi chú chuyển chỗ, theo tiền lệ cặp AC `⌥←/⌥→` của Story 6.10:

```
🔵 *(AC "người dùng chọn nhiều file cùng lúc — mỗi file thành một Chương hoặc được tách tiếp
theo mẫu" **chuyển sang Story 6.6b** ngày 2026-09-15 qua `correct-course`. Ice tách nó khỏi 6.6
ngày 2026-09-05 (`deferred-work.md:10076-10103`). AC không mất, nó đổi chỗ.)*
```

**Mục mới sau Story 6.6:** Covers FR14 *(vế "chọn nhiều file cùng lúc")*; thừa kế AC cắt ngang của Story 6.10a và 6.10. Năm AC: N tệp thành N Chương hoặc tách tiếp theo mẫu · mọi tệp có mặt trong màn xem trước · đây là "một lần nhập nhiều Chương" theo AC đầu của 6.10 nên hai con số và `⌥W` áp nguyên · chưa xác nhận thì không ghi gì xuống đĩa · Chương vào Library ở *Chưa bắt đầu*.

**Ghi chú cài đặt (đo 2026-09-15):** đường thả tệp đã nhận đủ N đường dẫn — Rust phát cả danh sách (`lib.rs:1334-1336`), TS lấy `paths[0]` (`libraryImport.ts:448`) và hiện `mode.library.drop_only_first` khi N > 1 (`:454`). Bẫy `MutexGuard` ở `deferred-work.md:10091-10094` chỉ áp nếu story thêm hộp thoại (AD-48). Hai câu hỏi để spec chốt: một bảng mã cho cả lượt hay mỗi tệp một; tệp hỏng giữ chỗ và khoá xác nhận như link hỏng hay khác. Chạy sau AI-4.

**Sổ nợ `:10100-10103`:** nối dòng 🔵 — story đã vào `epics.md`; vế "đứng TRƯỚC 6.7" hết đúng; tiền đề ② ③ chỉ áp nếu chọn hộp thoại.

### 4.2 Story 6.7b — Thêm Chương vào Tác phẩm có sẵn (`epics.md`)

**Ở Story 6.7**, AC2 (`:4748-4750`) giữ vế đầu, vế sau chuyển đi kèm 🔵.

**Mục mới sau Story 6.7:** Covers FR122 *(vế "thêm Chương vào Tác phẩm sẵn có")*; thừa kế AC cắt ngang của 6.7, 6.10a, 6.10, 5.4. Năm AC: chọn được đích là Tác phẩm mới hoặc Tác phẩm sẵn có · Chương mới vào đúng Tác phẩm đó và **Chương cũ không đổi một byte** · luật làm sạch tầng Tác phẩm áp lúc xem trước là luật của **Tác phẩm đích**, không phải Tác phẩm đang mở · Chương mới mặc định *Chưa bắt đầu*, trạng thái Tác phẩm suy lại theo Story 5.4, ghi đè thủ công giữ nguyên · chưa xác nhận thì không ghi gì xuống đĩa, kể cả xuống `.atproj` của Tác phẩm đích.

**Ghi chú cài đặt (đo 2026-09-15):** năng lực chưa có một dòng nào (grep `add_chapter|append_chapter|import_chapter|addChapter|appendChapter` trên `src-tauri/src` và `src` = 0 tệp); hai lệnh xác nhận đều gọi `create_work` (`project.rs:3392`, `:3747`), và `create_work` (`:354`) không nhận Tác phẩm đích; bề mặt đọc đã có (`library_list_works`, `commands/library.rs:682`); `store_for_tier` phân giải tầng Tác phẩm từ `OpenWorkState` (`deferred-work.md:10059-10064`) — đó là lý do AC thứ ba tồn tại; AD-44 phải được kiểm lúc soạn spec. Ba câu hỏi để spec chốt: vị trí Chương mới; ngôn ngữ nguồn (`work.source_lang` là một cột); đường nhập nào phải nghiệm thu. Story này **không** nhận mục `deferred-work.md:10056-10074` (bề mặt soạn luật tầng Tác phẩm, chủ Ice).

**Sổ nợ `:10382-10386`:** nối dòng 🔵 — story đã vào `epics.md`; ba tiền đề đo lại vẫn đúng, số dòng đã trôi.

### 4.3 Story 6.16b — "Cần xem" cho bản xem trước song ngữ

**Mục mới sau Story 6.17** (`epics.md`): Covers FR132 *(đường song ngữ)*; thừa kế toàn bộ AC Story 6.10 trừ hai nguyên nhân không áp được, cùng AC của 6.10a. Bảy AC: hai con số ở đầu màn · phân loại bằng **cùng** phép của 6.10 trên ba tín hiệu có nghĩa (độ dài · luật làm sạch xoá · số dòng bị nối), hai nguyên nhân *bóc ra ngắn bất thường* và *link hỏng* không áp · nêu rõ nguyên nhân · dấu hiệu không đo được thì không xếp *sạch*, và phân biệt được · `⌥W` lọc · cờ bảng mã tin cậy thấp cấp lượt nhập hiện ra (quyết định ① của 6.10) · không thao tác tay thì byte ghi xuống trùng đúng trước story.

**Ghi chú cài đặt (đo 2026-09-15):** hai tín hiệu bị **tính rồi vứt** — làm sạch từng ô giữ `cleaned.text` (`pipeline.rs:895`), chuẩn hoá từng ô giữ `.text` (`:949`, `:952`), mỗi `ImportedChapter` song ngữ mang `cleanup_report: None` và `joined_line_count: None` (`:1006-1012`); màn song ngữ **không có danh sách Chương** — chỉ một `chapter_count` theo ứng viên (`BilingualImportPreviewOverlay.vue:414`), còn `BilingualImportEncodingPreview` (`project.rs:3526-3541`) có sáu trường và không trường nào là Chương, trong khi màn đơn ngữ dựng `ChapterSplitPreviewWire` rồi gọi `review::classify` (`:2265-2278`); kiểu dữ liệu đã chở `confidence` (`:3527`) mà giao diện song ngữ nhắc 0 lần. Nếu spec đo thấy quá lớn thì tách theo TẦNG như 6.10a/6.10. Ba câu hỏi để spec chốt: đếm tín hiệu ở cột nào; hàng lệch số câu (6.17) có là nguyên nhân thứ sáu không; mockup `bilingual-import.html` nhắc "cần xem" 0 lần.

**Ghi chú ở Story 6.16** (`:5235`): 🔵 FR132 không được story này thừa kế, vế song ngữ ở 6.16b, lệch lộ ra ở retro F4.
**Bảng FR, hàng FR132** (`:782`): nối `· **vế song ngữ: Story 6.16b** 🔵`.
**`EXPERIENCE.md:142`:** "trong lần nhập nhiều link" → "trong một lần nhập nhiều Chương", kèm 🔵 nêu rằng *link hỏng* chỉ có nghĩa trên đường URL và *bóc ra ngắn bất thường* chỉ trên đường có bước bóc.

### 4.4 Story 6.18 gộp vào Story 10.9

**`epics.md`:**

| Chỗ | Sửa |
|---|---|
| `:794-796` (ba hàng NFR3/4/5) | `Epic 6 *(đóng)*` → `Epic 10 *(đóng)*`; "đóng ở Story 6.18" → "đóng ở Story 10.9"; kèm 🔵 |
| `:930` | nối 🔵 — 6.18 gộp vào 10.9; vế "hai story nằm liền kề nhau" hết đúng |
| `:941` | bỏ NFR3/4/5 khỏi dòng NFRs của Epic 6, thay bằng 🔵 trỏ sang Epic 10 |
| `:956` | nối 🔵 |
| `:4435`, `:4463` (Story 5.14) | nối 🔵 sau mỗi chỗ nhắc "Story 6.18" |
| `:1017` (NFRs Epic 10) | thêm NFR3 · NFR4 · NFR5 *(đóng Q4 — Story 10.9, gộp từ Story 6.18)* |
| `:5324-5358` (mục Story 6.18) | giữ tiêu đề; thân thay bằng khối 🔵 nêu việc gộp, và nêu phần đã làm (task 1–6, `6-18-ban-do/`, ba tệp test, feature `nfr-bench`) là đầu vào của 10.9, còn thiếu task 7 và task 8 |
| Story 10.9 | Covers thêm NFR3 · NFR4 · NFR5 *(đóng A6, A7, A8, Q4 — gộp từ Story 6.18)*; nối **nguyên văn** năm khối AC của 6.18, chỉ sửa "ở epic này" thành "ở Epic 6" |

AC *"đo trên **cả macOS lẫn Windows**"* giữ nguyên văn. Việc thu hẹp về macOS là quyết định trong **spec** 6.18 khi Windows còn đợi cuối dự án; tới 10.9 bảng nghiệm thu Windows (B7) đã mở, nên spec 10.9 quyết lại.

**PRD:** `:844` "sau Giai đoạn 3b" → "ở nghiệm thu cuối (Story 10.9, Giai đoạn 7)" · `:1105` nối 🔵 (A7, A8 ghi "Như A6" nên theo) · `:1150` Q4 nối 🔵: dời về nghiệm thu cuối, **điều kiện "sau Giai đoạn 3b" vẫn thoả — cái đổi là THỜI ĐIỂM**, "không chặn tiến độ" vẫn đúng, và hệ quả ở §2.2 được ghi thẳng vào ô ấy.

**SPEC:** `SPEC.md:131` → Story 10.9, và "Giai đoạn 3" → "nghiệm thu cuối" · `requirements.md:405` → "hiệu chỉnh ở nghiệm thu cuối (Story 10.9)" · `:411` → "Story 10.9 (gộp từ 6.18)".

**Sổ nợ:** 11 mục (`:8871`, `:8898`, `:8927`, `:9074`, `:9847`, `:9876`, `:10329`, `:10555`, `:10578`, `:10951`, `:11982-12002`) nối dòng `→ 🔵 SỬA 2026-09-15 … **Chủ: Story 10.9.**`. Mục Windows nhận thêm một câu: việc thu hẹp về macOS là quyết định của spec 6.18, spec 10.9 quyết lại.

🔴 **Không mục nào được đóng ở lượt này.** Đo 2026-09-15: **0/11** mục có dòng đóng mà cổng nhận ra — phiên 6.18 viết `→ 🔵 CẬP NHẬT … ĐÃ ĐO`, trong khi `continuationStatus` (`check-debt-owner.mjs:240`) chỉ đọc `✅`, `🟡`, `KHÔNG LÀM`; spec tự khai *"9 of 11 entries closed"*. Đóng chúng là một lượt riêng, và vài mục còn ghi *"owner của fix: MỚI, CHƯA ĐẶT TÊN (Ice đặt tên sau khi thấy verdict)"*.

**`spec-6-18-…md`:** thêm khối 🔵 dưới frontmatter, ngoài khối `frozen-after-approval`. Trạng thái để nguyên `in-progress` — không có giá trị nào nghĩa là "tạm gác", và ghi `done` thì sai. ⚠️ Hệ quả: `bmad-build` sẽ còn liệt kê nó là spec đang làm (`step-01-clarify-and-route.md:31`) cho tới khi Story 10.9 chạy.

**`epic-6-context.md:29`, `:63`:** nối 🔵.

### 4.5 A13 — mũi thăm dò bảng mã (phương án C)

- **`deferred-work.md:9415`** nối dòng 🔵: đường dẫn ở mục ấy **sai** — bàn đo đọc `_bmad-output/implementation-artifacts/6-1-ban-do/fixtures/encoding/` (`webimport_probe.rs:67-72`, `:262`), không phải `src-tauri/tests/fixtures/encoding/`; `fixtures/` bị `.gitignore` của `6-1-ban-do/` loại trừ nên tệp chỉ sống trên máy chạy đo; mục này trùng phép đo với mục `:9193`.
- **`SPEC.md:125`**: `chardetng` + `encoding_rs` **đã ghim 2026-09-03** (`ARCHITECTURE-SPINE.md:830`, `:1104`) chứ không còn "chưa ghim"; tỉ lệ dò đúng **chưa đo — 0 mẫu thật**; "Giai đoạn 3" → "Giai đoạn 3b".
- **`epics.md` không đổi một chữ.** Phép đo chưa làm là nợ có chủ, không phải chỗ lệch spec (`AGENTS.md:58`). Tiêu chí mũi thăm dò của Epic 6 vẫn là tiêu chí chưa đạt ở retro sau.

### 4.6 `sprint-status.yaml`

- Thêm `6-6b-nhập-nhiều-tệp-cùng-lúc: backlog` (sau `6-6`), `6-7b-thêm-chương-vào-tác-phẩm-có-sẵn: backlog` (sau `6-7`), `6-16b-bộ-lọc-cần-xem-cho-bản-xem-trước-song-ngữ: backlog` (sau `6-17`).
- Bỏ khoá `6-18-đo-lại-nfr3-nfr4-nfr5-trên-thư-viện-5-000-chương-thật`.
- Khối chú thích tại Epic 6: nêu lượt correct-course, việc gộp 6.18, việc retro đã chạy **trước** lượt thêm này, và **thứ tự thi hành** ở §3.
- Chú thích tại Epic 10, ngay trên `10-9`: gộp Story 6.18, spec 6.18 task 1–6 là đầu vào.
- Chú thích đầu tệp: nối 🔵 phép đo ở §2.4 về cách `sprint_plan.py status` chọn story.
- `epic-6` giữ `in-progress`; `epic-6-retrospective` giữ `done`.
- Action item: `epic-6-retro-item-63` (AI-9) → `done` · `epic-6-retro-item-66` (AI-5) → `done` · `epic-6-retro-item-69` (AI-10) → **giữ `open`** vì phép đo chưa làm. `epic-5-retro-item-55` không đổi.

Cả hai script đều nhận id có hậu tố chữ (`sprint_plan.py:57`, `sprint_status.py:29`), như tiền lệ `6-10a`, `3-4b`, `1-10c`.

### 4.7 Retro Epic 6 và các bản ghi khác

Tám ghi chú 🔵 tại chỗ trong `epic-6-retro-2026-09-15.md`: tiêu đề Nhóm A và câu hỏi mở 5 (**Epic 7 → Epic 4**, theo thứ tự chốt 2026-08-13) · hàng AI-9, AI-5, AI-10 · F5 vế NFR3/4/5 · câu hỏi mở 1 đã có câu trả lời · và §Acceptance verdict:

> Đường tới `accepted-with-open-items` **dài hơn**, không ngắn hơn: 6.18 đã rời Epic 6, nhưng cùng lượt ấy Epic 6 nhận ba story mới, nên `pending_stories` vẫn không rỗng.

---

## 5. Bàn giao

**Phân loại: Moderate** — sắp xếp lại backlog cộng một thay đổi thời điểm ở tầng PRD. Không cần replan.

| Ai | Việc | Điều kiện xong |
|---|---|---|
| Dev (lượt này) | Thi hành §4.1–§4.7 sau khi Ice duyệt | Tệp đã sửa; `check:debt-owner` xanh; `sprint_plan.py validate` không báo khoá lạ |
| Ice | Soạn spec chỉnh lệ cho lượt tách `commands/project.rs` (AI-6) hoặc giao Dev | Spec tồn tại, có chủ, đứng trước 6.6b |
| Dev | AI-4, AI-7 của retro (Nhóm A, mã) | Theo mô tả ở retro |
| Dev | Ba story mới, mỗi story một lượt `bmad-build` trong cửa sổ ngữ cảnh mới | AC đạt, spec có §Code Map |
| Ice | AI-1c và một lượt `schedule` xanh trên `origin` | Retro AI-1 |
| Ice | Cấp fixture GBK/Big5 thật khi có, rồi chạy `webimport_probe.rs:260` | A13 có số đo; hai mục nợ đóng cùng lượt |
| Story 10.9 (Epic 10) | Task 7 và task 8 của spec 6.18 | Q4 đóng; A6/A7/A8 mỗi ngưỡng một phán quyết |

---

## 6. Những gì lượt này **không** làm, ghi ra thay vì để người đọc đoán

- **Không đóng mục nợ nào**, kể cả 9 mục mà phiên 6.18 tự khai đã đóng (§4.4).
- **Không đổi tên mã** theo việc gộp: `story_6_18_*.rs`, `nfr-bench`, `6-18-ban-do/` giữ nguyên.
- **Không sửa `epics.md` cho A13**, và không sửa AC nào cho khớp mã đã viết.
- **Không quyết** ba câu hỏi của 6.6b, ba của 6.7b, ba của 6.16b — chúng thuộc bước soạn spec.
- **Không dựng cổng** trạng thái mà AI-1 đòi; đó là mục riêng, chủ Ice.
- **Không chạm** `risks.md`, `ARCHITECTURE-SPINE.md`, `DESIGN.md`, và không tạo `AD` mới.
- **Không đọc trọn** `epics.md` (7.043 dòng), PRD và spine; chỉ đọc các phần liên quan tới bốn mục.

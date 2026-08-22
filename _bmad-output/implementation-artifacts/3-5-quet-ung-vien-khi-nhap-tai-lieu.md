---
title: 'Story 3.5 — Quét ứng viên khi nhập tài liệu'
type: 'feature'
created: '2026-08-22'
status: 'done'
baseline_commit: '99dad1f8b3935a31b33102f9f37276a95e645f08'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-4b-danh-dau-thuat-ngu-o-cot-nguyen-van-cua-luoi.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Glossary khởi động từ con số không (rủi ro **R4**, `prd.md:1118`). Người dịch phải đọc hết truyện mới biết cần chốt gì. Bốn hàm `candidate_store` mà Story 3.2 dựng có **0 chỗ gọi sản phẩm** — bảng chờ tồn tại nhưng chưa ai từng ghi vào nó.

**Approach:** Khi nhập một Chương, chạy **nền** một lượt quét tìm chuỗi lặp ≥ ngưỡng và **không có trong từ điển nhúng**, ghi vào **bảng chờ** kèm số lần xuất hiện và một ví dụ ngữ cảnh. Ngưỡng sống trong `app_config` và đổi được ở một **lớp phủ thứ tư**, cùng khuôn `ShortcutsOverlay`. Bảng chờ được phơi qua một vỏ IPC chỉ-đọc để lượt quét nghiệm thu được bằng mắt.

## Boundaries & Constraints

**Always:**
- **Lọc theo TẦN SUẤT trước, tra từ điển sau.** Không có vị từ tra-có/không rẻ (`ports/dict_source.rs:47-129` — không `exists()`), và `lookup_grouped` lặp qua **mọi** lớp đang mở, không tắt sớm. Đảo thứ tự là biến một lượt quét thành hàng chục nghìn lượt tra.
- Ghi **chỉ** vào `glossary_candidate`. Không đường nào từ lượt quét chạm `glossary_entry` (AD-20).
- Lượt ghi hàng loạt đi qua **một** `Store::write`, `prepare_cached` một lần rồi lặp — khuôn `commands/segment.rs:98-149`.
- Ranh giới **câu** lấy từ segment đã tách lúc nhập (Story 2.1), không tự đoán lại. Đây là thứ định nghĩa "đứng đầu câu" cho nhánh tiếng Anh.
- Mọi số đếm **báo ra**, kể cả 0 và kể cả số bị bỏ qua. *Rỗng im lặng* là lớp lỗi trung tâm của dự án.
- Lớp phủ đi đúng khuôn `ShortcutsOverlay.vue` — `v-if` tự quản, mount ở `App.vue` cùng tầng, **không** `MODE_IDS` thứ tư.

**Ask First:**
- Bất kỳ phụ thuộc mới nào (NFR15) — kể cả bật một `feature` chưa bật của crate đã có.
- Nếu số đo lượt quét trên một Chương thật vượt **5 giây**: dừng, trình số, hỏi trước khi thêm cache/chỉ mục — đúng luật *"đo trước khi chốt kiến trúc"*.
- Nếu ngưỡng 5 cho ra dưới 3 hoặc trên 500 ứng viên trên Chương mẫu: trình số, đừng tự chỉnh ngưỡng mặc định.

**Never:**
- Không sửa `epics.md`/`prd.md`. Vế *"một loạt Chương"* KHÔNG có đường đi tới (một `INSERT INTO chapter` toàn kho, `commands/project.rs:181`, chủ nợ Epic 6) — ghi nợ, không sửa spec.
- Không dựng khung điều hướng 10 mục của `settings.html` — chín mục kia thuộc Epic 4/5/6/10.
- Không vẽ thanh chuyển phạm vi Toàn cục/Tác phẩm: ngưỡng là `AppConfig` ⇒ `GlobalOnly` (`kinds.rs:218`), `save_value` từ chối mọi loại khác. Một nút bấm được sẽ **trông như đã ghi mà không ghi**.
- Không component Vue cho bảng chờ (Story 3.8). Story này chỉ phơi dữ liệu.
- Không `ON CONFLICT DO UPDATE` chạm cột `resolution` — trigger `glossary_candidate_resolution_is_one_way` (`schema.rs:427-430`) canh, và hồi sinh một ứng viên đã bị bỏ là phá đúng AC cuối.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Zh, tên lặp | `萧炎` xuất hiện 40 lần, không có trong từ điển | Một hàng, `occurrence_count = 40`, `context_example` = câu chứa nó | N/A |
| Zh, n-gram lồng | `萧炎` 40 lần ⇒ `萧炎的` cũng ≥ ngưỡng | Giữ chuỗi **dài nhất chỉ khi** tần suất của nó **không** bằng tần suất chuỗi con; nếu bằng thì chuỗi con là chuỗi thật | N/A |
| Zh, có trong từ điển | `修炼` lặp 80 lần, có đầu mục | **0** hàng — bị loại ở bước tra | N/A |
| Zh, đoán tên người | Chuỗi 2–3 ký tự, ký tự đầu nằm trong bảng họ, tần suất = `ngưỡng − 1` | Một hàng — bảng họ hạ ngưỡng xuống `ngưỡng − 1` cho **riêng** hình dạng này. `candidate_origin` vẫn là `'import_scan'`, **0** cột mới | N/A |
| Zh, họ nhưng quá ngắn/dài | Ký tự đầu trong bảng họ, chuỗi 1 hoặc ≥ 4 ký tự | Đi đường thường (ngưỡng đầy đủ) — bảng họ không nới | N/A |
| En, cụm hoa giữa câu | `fire dragon` viết `Fire Dragon` 12 lần, không đứng đầu segment | Một hàng, `occurrence_count = 12` | N/A |
| En, hoa đầu câu | `The` mở đầu 300 segment | **0** hàng — vị trí đầu segment bị loại | N/A |
| Dưới ngưỡng | Chuỗi lặp 4 lần, ngưỡng 5 | **0** hàng | N/A |
| Đã có trong Glossary | `萧炎` đã là `glossary_entry` (Story 3.3) | **0** hàng — lọc trước khi ghi | N/A |
| Đã từng bị bỏ | Hàng `resolution = 'rejected'` cùng `source_term`, **cùng `project.db`** | **0** hàng mới; hàng cũ **không** đổi một cột nào | `ON CONFLICT DO NOTHING`, đếm số bị bỏ qua và báo ra |
| Chương rỗng / toàn khoảng trắng | `source_text` trắng | **0** ứng viên, sự kiện vẫn bắn với `count = 0` | N/A |
| Ngưỡng cấu hình sai | `config_value` chứa `"abc"` / `"0"` / `"-3"` | Rơi về mặc định **5** | Ghi chẩn đoán không dấu; **không** ném |
| Kho đóng giữa lượt quét | `Store` bị thả trong lúc luồng nền chạy | Luồng nền kết thúc lặng lẽ, **không** panic | `panic = "abort"` ⇒ mọi `unwrap` là lỗi thiết kế |

</frozen-after-approval>

## Code Map

**Lược đồ — thêm hai cột vào `project.db`**
- `src-tauri/src/core/store/schema.rs:405-408` — doc-comment **đã tiên liệu story này**: *"Không cột số lần xuất hiện/ví dụ ngữ cảnh (Story 3.5)"*.
- `schema.rs:409-426` `GLOSSARY_CANDIDATE_DDL` — 🔴 **không sửa tại chỗ**. Thêm hằng `ALTER TABLE` riêng, đúng tiền lệ `SEGMENT_TARGET_TEXT_DDL` (`schema.rs:656-657`).
- `schema.rs:1136-1213` `PROJECT_MIGRATIONS` — thêm `Migration { to_version: 14, .. }`. `GLOBAL_MIGRATIONS` **không** đụng: bảng chỉ ở `project.db` (`schema.rs:333-342`, `tests/pinned_contract.rs:187`).
- `schema.rs:427-430` — ⚠️ trigger một chiều trên `resolution`.
- **Test ghim 13 phải lên 14:** `src-tauri/tests/segment_contract.rs` dòng 540, 580, 787, 796, 1210. Cổng này cố ý (`schema.rs:437-441`) — đổi phiên bản là quyết định có ký.

**Bảng chờ — lõi Story 3.2, chưa ai gọi**
- `src-tauri/src/core/glossary/candidate.rs:114-124` `GlossaryCandidate` (5 trường) · `:129-131` `is_pending()` là vị từ DUY NHẤT định nghĩa "chờ duyệt".
- `candidate_store.rs:49-68` `insert_candidate` — **giữ nguyên**, không `ON CONFLICT`. Hàm ghi lô là hàm MỚI.
- `src-tauri/src/core/scope/store.rs:314-321` — khuôn `ON CONFLICT (…) DO UPDATE` duy nhất trong kho; story này dùng biến thể `DO NOTHING`.
- `src-tauri/src/commands/segment.rs:98-149` `insert_segments` — khuôn ghi lô trong một transaction, `prepare_cached` (đo: tiết kiệm ~57%).
- `deferred-work.md:5606-5617` — nợ có chủ **là story này**: lượt quét không được sinh ứng viên trùng `source_term` với `glossary_entry`, nếu không `approve_candidate` hỏng vĩnh viễn.
- `deferred-work.md:5630-5643` — bốn hàm `candidate_store` chưa vào `GLOSSARY_ONLY_SURFACE` (`tests/glossary_boundary.rs:78`); chủ là story dựng chỗ gọi đầu tiên.

**Thuật toán quét**
- `src-tauri/src/core/matching/mod.rs:374` `ngrams(text, lang, n)` — **sinh** n-gram, KHÔNG đếm. `:252` `tokenize` · `:333` `normalize` · `:470` `find_terms`. Lá phụ thuộc, không chạm DB. `:68-69` khai rõ xếp hạng/ngưỡng/chỉ mục ngược nằm **ngoài** phạm vi (Story 7.5/7.6).
- `matching/mod.rs:150` `static JIEBA: LazyLock<Jieba>`; `:254` `JIEBA.cut(text, HMM)`. ⚠️ Feature `tfidf`/`textrank` **không bật** (`Cargo.toml:40`) ⇒ `extract_keywords` chưa biên dịch vào — và không cần: AC đòi **tần suất thô**, không phải điểm TF-IDF.
- `ports/dict_source.rs:47-129` — `lookup`/`senses`/`han_viet`/`count_by_source`, **không** `exists()`. Rẻ nhất: `lookup()` nhánh `ExactBtree`.
- `core/dict/mod.rs:806-900` `lookup_grouped` — lặp qua `layers.layers()`, tối đa 4 tệp, **không** tắt sớm. `:71` — số đo pha 1 duy nhất trong kho: **p95 7,324 ms** (nhánh `char_idx`, một lớp, chưa đọc `dict_sense`).
- `src/config/segment.ts:66-120` / bảng `segment` — ranh giới câu đã có từ lúc nhập, KHÔNG tính lại lúc nạp.

**Cấu hình ngưỡng**
- `src-tauri/src/core/scope/kinds.rs:218` `AppConfig => "app_config" : GlobalOnly`.
- `core/scope/store.rs:49-53` `KEY_THEME`/`KEY_MODE` · `:123-193` getter — khuôn để chép. ⚠️ `:132-136` nói rõ **không** validate hai tầng.
- `core/store/schema.rs:98-105` `config_value(kind, key, value TEXT, updated_at)` — 🔴 `value` là TEXT **không `CHECK`** ⇒ `parse::<u32>` + mặc định + chặn ≤ 0 phải nằm trong getter.
- `src-tauri/src/commands/config.rs:56-93` `BootstrapConfig` · `:198-219` `put_config`/`delete_config` (đã có, chưa UI nào gọi cho khoá tự do). `tests/ipc_contract.rs` đóng băng danh sách trường.
- `src/config/bootstrap.ts:1-33` — bản sao kiểu phía TS, `snake_case`.

**Đường nhập + chạy nền**
- `src-tauri/src/commands/project.rs:138` `create_work` — tách segment **ngoài** closure ghi (`:170`), một transaction ghi `work` (`:175`) + `chapter` (`:181`) + `insert_segments` (`:196`), `meta.json` sau commit. Vỏ wire: `:429` `create_work_from_text` · `:445` `create_work_from_file`.
- ⚠️ `commands/chapter.rs:107-108` — *"không đường sản phẩm nào sinh Chương thứ hai; món nợ có chủ: Epic 6"*.
- `src-tauri/src/lib.rs:774` — tiền lệ `std::thread::spawn` duy nhất trong mã sản phẩm (luồng chờ flush lúc thoát, có trần thời gian). **0** `tokio::spawn`, **0** `tauri::async_runtime::spawn` toàn kho.
- `lib.rs:761` `EXIT_FLUSH_EVENT` — khuôn `app.emit`. **0 permission ACL** phải thêm (`lib.rs:631-646`).
- `src/panels/editorPanelState.ts:688` · `src/modes/libraryImport.ts:294,300` — hai tiền lệ `listen()` phía TS.
- `core/store/writer.rs:37-41,68,83-107` — mọi lượt ghi qua `Store::write` (mpsc, nhiều sender). ⚠️ `:47-54` cờ `ON_WRITER_THREAD` chỉ cắn khi lồng `write()` trong `write()`.

**Bề mặt IPC + lớp phủ**
- `src-tauri/src/commands/glossary.rs:291,309,347,385` — bốn vỏ `wire`, đăng ký `lib.rs:369-375`. 🔴 **không** `rename_all` (`:64`, `:201`) ⇒ về JS giữ `snake_case`.
- `src/config/glossary.ts:113` khuôn ba trạng thái · `:237` `isGlossaryMark` · `:263` `isGlossaryMarkArray` — khuôn type guard để chép.
- `src/ShortcutsOverlay.vue:5-26` — 🔴 khuôn lớp phủ, và nó **gọi tên sẵn cái bẫy thanh phạm vi**. `src/AttributionOverlay.vue` · `src/SegmentHistoryOverlay.vue` là hai tiền lệ còn lại; cả ba mount ở `App.vue:280,283,286`.
- `src/commands/index.ts:1521` `['shortcuts.open', 'openShortcuts', 'Mod+Comma']` — khuôn đăng ký lệnh. `:38` `MODE_IDS` là hằng **ba** phần tử. Đăng ký ở `main.ts`, KHÔNG ở `App.vue`.
- `src/GlossaryQuickAdd.vue:96-144` — khuôn `<label><span>{{t()}}</span><input v-model>` + `<fieldset :disabled>` lúc đang ghi. ⚠️ **Chưa có `input type="number"` nào trong `src/**`.**
- `src/i18n/vi.json:237` `glossary.quick_add.source_label` — khuôn khoá `<miền>.<mục>.<trường>`.

**Cổng — sàn phải nâng khi thêm tệp** *(giá trị hiện tại, đo 2026-08-22)*
- `check-tokens.mjs:91` `FILE_FLOOR=53` · `:93` `COMPONENT_FILE_FLOOR=50` · `check-layout.mjs:110` `FILE_FLOOR=50` · `check-panel-refs.mjs:517` `FILE_FLOOR=36` · `check-commands.mjs:211` `VUE_FLOOR=14` · `:233` `TS_FLOOR=36` · `check-i18n.mjs:288` `RS_FLOOR=42` · `:306` `VUE_FLOOR=14` · `check-debt-owner.mjs:497` `ITEM_FLOOR=397`.
- `check-commands.mjs:22-51` Kiểm A: `@click` phải là ĐÚNG MỘT `dispatch('<id>')`; `@input`/`@change`/`@submit` **không** thuộc luật này.
- `check-tokens.mjs:835` B (màu) · `:1012` B2 (cỡ/họ chữ) · `:1347` D (`opacity`) · `:1445` F (bóng/gradient, **không miễn trừ được**) · `:1488` H (`outline:none` chỉ trên gốc `tabindex="-1"`).
- `check-panel-refs.mjs:2-4` — ô nhớ cấp module trong `src/**/*.ts` **bắt buộc** có hàm reset.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` -- hai hằng `ALTER TABLE` mới (`occurrence_count INTEGER NOT NULL DEFAULT 0`, `context_example TEXT`) + bước `Migration { to_version: 14 }` vào `PROJECT_MIGRATIONS` -- sửa `GLOSSARY_CANDIDATE_DDL` tại chỗ sẽ làm kho cũ và kho mới lệch lược đồ trong im lặng.
- [x] `src-tauri/tests/segment_contract.rs` -- ghim 13 → 14 ở **8** dòng `assert_eq!(…schema_version(), 14)` (nhiều hơn năm dòng mà spec ước lượng ban đầu — `grep` thật ra 8, không 5) cộng danh sách `versions` của `the_project_migration_set_matches_the_declared_ladder_step_for_step` cộng fixture `STEP_FOURTEEN` → `STEP_FIFTEEN` (`[Migration; 13]` → `[Migration; 14]`, pin giả `14` → `15`); và `pinned_contract.rs` (2 chỗ: `PROJECT_MIGRATIONS.len()` 12→13, `schema_version()` 13→14) -- không nằm trong danh sách dòng của spec nhưng canh cùng con số, bỏ sót sẽ để lọt một cổng đỏ ở đúng tệp mà `PROJECT_MIGRATIONS` cũng được import.
- [x] `src-tauri/src/core/glossary/surnames.rs` (mới) -- mảng hằng họ phổ biến (Bách gia tính), kèm chú thích ghi rõ vì sao **0 cửa NFR15**: văn bản thế kỷ 11, và một danh sách họ là dữ kiện -- lấy qua `tools/dict-build` sẽ bắt dựng lại cả bốn `.db` + bốn SHA-256 + một release, đúng cho một mảng 100 chuỗi.
- [x] `src-tauri/src/core/glossary/scan.rs` (mới) -- hàm **thuần**: nhận `&[&str]` (segment đã tách), `MatchLang`, ngưỡng, bảng họ, và một vị từ `is_known: &mut dyn FnMut(&str) -> bool`; trả `Vec<ScanCandidate { source_term, occurrence_count, context_example }>` -- tiêm vị từ giữ `scan.rs` là lá không chạm DB, và là thứ vitest/`cargo test` kiểm tất định được. Lọc **tần suất trước, tra sau**.
- [x] `src-tauri/src/core/glossary/scan.rs` -- **bổ sung sau rà ba lớp** — `context_example` mang một TRẦN có tên (`CONTEXT_EXAMPLE_CHAR_LIMIT = 200`, cắt ở biên KÝ TỰ qua `truncated_context_example`, không byte) — bản đầu ghi thẳng `segments[first_segment].to_owned()` không trần nào, và một segment thiếu dấu kết câu (`split_source_text` vẫn phát một segment cho ca đó) có thể dài hàng nghìn ký tự. Hai ca test mới trong `glossary_scan_contract.rs`: đoạn dài hơn trần (cắt đúng 200 ký tự, đúng tiền tố, không panic trên chữ Hán nhiều byte) và đoạn ngắn hơn trần (giữ nguyên vẹn).
- [x] `src-tauri/src/core/glossary/candidate.rs` -- thêm `occurrence_count: i64` + `context_example: Option<String>` vào `GlossaryCandidate` -- `is_pending()` **không** đổi: nó vẫn chỉ đọc `resolution`.
- [x] `src-tauri/src/core/glossary/candidate_store.rs` -- hàm ghi lô mới `insert_import_scan_candidates`, một `Store::write`, `prepare_cached`, `ON CONFLICT (source_term) DO NOTHING`; trả `(đã chèn, đã bỏ qua)`; và một truy vấn lọc `glossary_entry` NGAY trong câu `INSERT` (`WHERE NOT EXISTS`) trước khi ghi -- trả cặp số thay vì `()` là thứ giữ *"0 ứng viên"* phân biệt được với *"quét chưa chạy"*.
- [x] `src-tauri/src/core/scope/store.rs` -- `KEY_GLOSSARY_SCAN_THRESHOLD` + `parse_glossary_scan_threshold` (hàm thuần, `Option<&str> -> u32`) · mặc định 5 (`DEFAULT_GLOSSARY_SCAN_THRESHOLD`) · chặn `<= 0` · getter `GlobalConfig::glossary_scan_threshold()` -- `config_value.value` là TEXT không `CHECK`, nên getter là chỗ DUY NHẤT biết một giá trị hỏng.
- [x] `src-tauri/src/commands/config.rs` -- thêm `glossary_scan_threshold: u32` vào `BootstrapConfig` -- frontend cần nó lúc khởi động để hiện giá trị đang có, không phải để quyết định gì.
- [x] `src-tauri/tests/ipc_contract.rs` -- cập nhật danh sách trường `BootstrapConfig` đã đóng băng (7 trường).
- [x] `src-tauri/src/commands/glossary.rs` -- vỏ `wire::glossary_pending_candidates()` trả `Vec<GlossaryCandidateWire>` (**không** `rename_all`, đúng hai anh em `:64`/`:201`) + hàm thuần `glossary_pending_candidates(open: Option<&OpenWork>)`; đăng ký ở `src-tauri/src/lib.rs` -- đây là chỗ gọi sản phẩm ĐẦU TIÊN của `core::glossary::pending_candidates` (Story 3.2 dựng, 0 chỗ gọi cho tới lượt này).
- [x] `src-tauri/src/commands/project.rs` -- sau khi transaction nhập **đã commit** (sau `replace_open_work`), vỏ `wire::create_work_from_text`/`wire::create_work_from_file` spawn một `std::thread` (`spawn_import_scan`) chạy lượt quét rồi `app.emit` một sự kiện (`GLOSSARY_IMPORT_SCAN_EVENT`) mang cặp số -- spawn trong hàm thuần sẽ làm `tests/**` không gọi được nó; spawn TRƯỚC commit là quét một Chương chưa tồn tại; hàm khoá `OpenWorkState` HAI LẦN NGẮN (đọc segment, ghi lô), không MỘT LẦN DÀI suốt lượt quét -- xem doc-comment tại chỗ + Spec Change Log.
- [x] `src-tauri/src/commands/project.rs` -- **bổ sung sau rà bảng I/O** — tách `guarded_open_store(open: Option<&OpenWork>, work_id: &str) -> Option<&Store>` khỏi `spawn_import_scan` thành một hàm THUẦN dùng ở CẢ HAI lần khoá; canh bằng ba test THẬT trong `#[cfg(test)] mod tests` của chính tệp (không cần `AppHandle`/webview) -- đóng hàng I/O Matrix "Kho đóng giữa lượt quét ⇒ kết thúc lặng lẽ, không panic" mà bản đầu KHÔNG có test nào canh. Xem Spec Change Log.
- [x] `src-tauri/tests/glossary_boundary.rs` -- **quyết định khác chữ, cùng tinh thần** -- xem Spec Change Log: thêm `insert_candidate` (không phải hàm ghi lô) vào `GLOSSARY_ONLY_SURFACE`, và thêm `pending_candidates` vào `QUICK_ADD_SURFACE` (không phải `glossary_pending_candidates`, tên đó là vỏ IPC sống ngoài `core/glossary/**`). Đóng nửa món nợ `deferred-work.md` (mục Story 3.2, "bốn hàm chưa vào GLOSSARY_ONLY_SURFACE").
- [x] `src-tauri/tests/glossary_scan_contract.rs` (mới) -- mọi hàng I/O Matrix ở tầng Rust: tên lặp, n-gram lồng (cả hai chiều — bằng tần suất bị loại, khác tần suất được giữ), có trong từ điển, họ 2-3 ký tự (cả ba biến thể), cụm hoa giữa câu, hoa đầu câu, dưới ngưỡng, Chương rỗng, ngưỡng hỏng (5 dạng) -- 16 ca, tên hàm là một CÂU khẳng định.
- [x] `src/config/glossary.ts` -- adapter `pendingGlossaryCandidates()` + kiểu `GlossaryCandidate` + type guard lúc chạy cho mảng (`isGlossaryCandidateArray`) -- chép khuôn `isGlossaryMarkArray`; `context_example` là `null` được, và type guard là chỗ duy nhất biết.
- [x] `src/config/bootstrap.ts` -- thêm trường `glossary_scan_threshold` vào bản sao kiểu, `snake_case`, cộng `bootstrapGlossaryScanThreshold` (ref chỉ đọc, đọc một lần lúc khởi động, cùng khuôn `bootstrapLayout`) -- chiều trả về giữ `snake_case`, chỉ tham số gửi đi mới `camelCase`.
- [x] `src/glossarySettingsState.ts` (mới) -- state lớp phủ + đọc/ghi ngưỡng qua `putConfig`; năm ô nhớ cấp module đi qua `EXEMPT` của `check:panel-refs` (không một hàm `reset*`) -- xem Spec Change Log, lượt rà ba lớp gỡ `resetGlossarySettings()` vì nó là MÃ CHẾT (0 chỗ gọi ngoài chính test của nó).
- [x] `src/glossarySettingsState.ts` -- **vá sau rà ba lớp** — `parsedGlossaryScanThreshold` khớp THẬT với `u32::from_str` phía Rust: nhận dấu `+` tuỳ chọn, chặn trần `u32::MAX = 4294967295`. Bản đầu từ chối `"+5"` (Rust nhận) VÀ chấp nhận `"5000000000"` (Rust từ chối, rơi về mặc định 5 lặng lẽ sau khi lớp phủ đã báo "đã lưu") -- bốn ca test mới: `"+5"`, `"4294967295"` (đúng trần), `"4294967296"`, `"5000000000"`.
- [x] `src/GlossarySettingsOverlay.vue` (mới) -- lớp phủ thứ tư, khuôn `ShortcutsOverlay.vue`/`AttributionOverlay.vue`; đúng một ô nhập số + nhãn qua `t()`; **0** thanh chuyển phạm vi -- ngưỡng là `AppConfig`/`GlobalOnly`, một nút *"Tác phẩm"* bấm được sẽ trông như đã ghi mà không ghi.
- [x] `src/commands/index.ts` + `src/main.ts` + `src/App.vue` -- đăng ký `glossary.settings.open`/`close`/`save` (hợp âm mặc định `Mod+Alt+T` cho `open`), mount lớp phủ cạnh ba cái đã có, thêm nút vào titlebar (`data-glossary-settings-open`) -- đăng ký port ở `main.ts` qua `installCommands()`, KHÔNG trong `App.vue` (HMR gọi lần hai và `register()` ném).
- [x] `src/i18n/vi.json` -- khoá mới dưới miền `glossary.settings.*` + ba khoá `command.glossary.settings.*` -- không nhân đôi nhãn đã có ở `glossary.quick_add.*`.
- [x] `tests/frontend/glossarySettings.test.ts` (mới) -- ô nhập từ chối giá trị ≤ 0/không phải số/thập phân; giá trị hợp lệ đi tới `put_config` đúng một lần; lượt lưu trượt hiện lỗi mà KHÔNG tự đóng lớp phủ; mở lại sau lưu nạp giá trị VỪA lưu, không giá trị bootstrap cũ -- 15 ca.
- [x] `scripts/check-*.mjs` -- nâng sàn quần thể theo số THẬT đo bằng chính các cổng sau khi thêm tệp, ghi kèm ngày tại chỗ -- `check-tokens.mjs` (`FILE_FLOOR` 53→55, `COMPONENT_FILE_FLOOR` 50→52), `check-layout.mjs` (`FILE_FLOOR` 50→52), `check-panel-refs.mjs` (`FILE_FLOOR` 36→37), `check-commands.mjs` (`VUE_FLOOR` 14→15, `TS_FLOOR` 36→37, `COMMAND_FLOOR` 44→47, `CLICK_FLOOR` 21→24, `DISPATCH_FLOOR` 30→34), `check-i18n.mjs` (`RS_FLOOR` 42→44, `VUE_FLOOR` 14→15), `glossary_boundary.rs` (`RS_FLOOR` 40→43), `check-debt-owner.mjs` (`ITEM_FLOOR` 397→444).
- [x] `scripts/check-debt-owner.mjs` + `src-tauri/tests/glossary_boundary.rs` -- **sửa sau rà ba lớp**: `ITEM_FLOOR` 444 (85,06 %, NHỈNH TRÊN dải 80–85 % do làm tròn lên) → 443 (84,87 %); `glossary_boundary.rs::RS_FLOOR` 43 (81 %) → 44 (83 %), khớp `check-i18n.mjs::RS_FLOOR` — cả hai quét CÙNG quần thể (53 tệp `.rs` dưới `src-tauri/src/**`) nên không có lý do chính đáng để hai con số lệch nhau.
- [x] `tests/frontend/bootstrap.test.ts` (mới) -- **bổ sung sau rà ba lớp** — `loadBootstrapConfig()` THẬT (mock `@tauri-apps/api/core`, không mock trọn `config/bootstrap`) chạy qua nhánh phân giải `glossary_scan_threshold` với sáu payload dây giả: thiếu trường · chuỗi `"5"` · `0` · `-1` · `3.5` · số nguyên hợp lệ (`12`) -- trước đó nhánh này có 0 test chạy QUA nó (`glossarySettings.test.ts` mock TRỌN `config/bootstrap`), lệch parity với `isGlossaryMarkArray`/`glossaryMarksRefresh.test.ts`.
- [x] `src-tauri/src/commands/project.rs` -- **vá sau rà ba lớp** — tách `guarded_dict_layers` (hàm thuần, cùng khuôn `guarded_open_store`): `DictLayers` CHƯA quản lý (`try_state` trả `None`) nay `eprintln!` rồi DỪNG lượt quét, thay vì âm thầm rơi về `DictLayers::empty()` — bản đầu gộp ca đó với ca "đã quản lý nhưng rỗng" (trạng thái BÌNH THƯỜNG, AD-25) thành một nhánh im lặng duy nhất, khiến `is_known` LUÔN `false` và bộ lọc từ điển vô hiệu HOÀN TOÀN mà không một dòng chẩn đoán — đúng ca bàn đo của story đã chạy phải (`DictLayers::empty()`, 969 ứng viên). Hai ca test mới trong `mod tests` của chính tệp.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- đóng mục Story 3.2 "va UNIQUE glossary_entry" (✅, mục ở dòng ~5606 lúc lập spec) và mục Story 3.2 "bốn hàm chưa vào GLOSSARY_ONLY_SURFACE" (🟡, mục ở dòng ~5630); mở mục mới `## Deferred from: 3-5-quet-ung-vien-khi-nhap-tai-lieu (2026-08-22)` với bốn mục có chủ: vế *"một loạt Chương"* (chủ **Epic 6**), bề mặt duyệt bảng chờ (chủ **Story 3.8**), AC "đo trên Chương thật + từ điển thật" chưa đóng trọn (chủ **Ice**, môi trường cài đặt không có `.db`/kho `.atproj` thật), AC NFR2 đóng bằng lập luận kiến trúc chứ chưa một phép đo khung hình thật (chủ **Ice**, cần webview thật).

**Acceptance Criteria:**
- Given một Chương tiếng Trung thật vừa nhập, when lượt quét chạy, then thời gian tường **và** số ứng viên sinh ra được ghi thẳng vào story kèm toolchain + ngày — **đo**, không suy luận; và nếu vượt 5 giây thì HALT theo §Ask First.
  ⚠️ **ĐÓNG MỘT PHẦN.** Đã đo (xem §Verification): `scan_candidates` 452 ms + ghi lô 19 ms = ~471 ms trên một văn bản TỔNG HỢP cùng quy mô Chương lớn nhất có thật (8.848 câu / 48.650 ký tự), CHẠY DƯỚI 5.000 ms một khoảng an toàn lớn — **không HALT**. Nhưng số đó **không gồm chi phí `is_known` trên từ điển thật** (môi trường cài đặt không có tệp `.db`) và **không phải Chương thật của Ice** — xem giới hạn đầy đủ + món nợ có chủ ở `deferred-work.md` §*Deferred from: 3-5-…*.
- Given cùng Chương đó, when quét trong lúc người dùng gõ, then **không frame nào vượt 50 ms** (NFR2) — lượt quét chạy ngoài luồng giao diện, và mệnh đề này chỉ đóng được bằng một phép đo, không bằng *"nó ở thread khác"*.
  ⚠️ **CHƯA ĐÓNG được bằng phép đo khung hình thật** — môi trường cài đặt (agent CLI) không dựng được một cửa sổ Tauri thật để đo `requestAnimationFrame`. Đã đóng được vế KIẾN TRÚC (spawn trên `std::thread` riêng, không giữ khoá `OpenWorkState` trong pha CPU nặng nhất) nhưng chính §Boundaries của story này cấm đúng kiểu đóng đó — ghi nợ có chủ cho Ice, `deferred-work.md`.
- Given ngưỡng đổi từ 5 sang một số khác qua lớp phủ, when nhập **cùng một văn bản** thành một Tác phẩm mới, then tập ứng viên đổi theo — đây là vế **cấu hình lại được** của FR52, và nó đóng ở tầng người dùng chứ không chỉ ở tầng khoá. Ngưỡng ở `global.db` nên nó sống qua ranh giới Tác phẩm; bảng chờ ở `project.db` nên mỗi lượt nhập bắt đầu từ một bảng rỗng — đó là thứ làm phép thử này sạch.
  ⚠️ **Đóng ở TẦNG ĐƠN VỊ/TÍCH HỢP** (`glossarySettings.test.ts` — ngưỡng ghi qua `putConfig` đúng khoá/giá trị; `parse_glossary_scan_threshold`/`GlobalConfig::glossary_scan_threshold` phía Rust; `spawn_import_scan` đọc ngưỡng qua đúng getter đó), CHƯA nghiệm thu bằng mắt qua giao diện thật (cần một webview thật + `.atproj` thật) — cùng nhóm giới hạn môi trường đã ghi ở hai AC trên.
- Given một `source_term` đã `rejected` trong **cùng một** `project.db`, when gọi lại hàm ghi lô với cùng chuỗi, then **0** hàng mới và hàng cũ **không** đổi một cột nào — đối chứng bằng `SELECT` trước/sau, không bằng số hàng. ⚠️ Nghiệm thu ở **tầng store**, không qua đường sản phẩm: `glossary_candidate` nằm trong `project.db` của từng Tác phẩm, và nhập lại cùng văn bản tạo một Tác phẩm MỚI với một bảng chờ rỗng — một phép thử qua giao diện sẽ xanh mà không chứng minh gì.
  ✅ **ĐÓNG** — `glossary_contract.rs::rescanning_a_rejected_source_term_via_the_batch_writer_changes_nothing_on_the_old_row` đối chứng đúng bằng `SELECT` trước/sau trên `resolution`/`occurrence_count`/`context_example`.
- Given `.githooks/pre-push`, when chạy, then mười một cổng + vitest + build + `cargo test --locked` xanh.
  ✅ **ĐÓNG** — chạy thật 2026-08-22, xanh trong 98s (xem §Verification cho log đầy đủ).
- Given lượt CI sau khi push, when đọc, then **cả** macOS lẫn Windows xanh — `pre-push` chạy trên macOS của Ice và không nói gì về nửa Windows.
  ⚠️ **CHƯA ĐÓNG** — story chưa được push; đây là bước Ice làm sau khi nhận bàn giao.
- Given bộ e2e, when chạy tay `npm run test:e2e`, then không hồi quy; và ghi rõ **bao nhiêu spec chạm bề mặt story này dựng** — *"e2e xanh"* không có nghĩa bề mặt MỚI đã được nghiệm thu.
  ⚠️ **CHƯA CHẠY trong lượt này** — môi trường cài đặt không có GUI để dựng cửa sổ Tauri thật cho bộ e2e (mỗi spec mở một cửa sổ thật, ~1,5 phút, sửa `global.db` thật của máy chạy). Đã đếm được phần tĩnh: `grep` trên `e2e/specs/**` cho `pending_candidates`/`glossary_scan_threshold`/`glossary.settings`/`import_scan` = **0** — **0/12** spec hiện có chạm bề mặt story này dựng. Chạy tay + đọc số hồi quy là việc của Ice.

## Spec Change Log

### 2026-08-22 — thực thi

**Quyết định thực thi không tường minh trong spec, ghi lại để lượt sau đọc được lý do:**

- **`GLOSSARY_ONLY_SURFACE` nhận `insert_candidate`, KHÔNG nhận tên hàm ghi lô mới.** Đọc kỹ lại
  §Code Map/Tasks: hàm ghi lô mới (`insert_import_scan_candidates`) PHẢI có một chỗ gọi sản
  phẩm NGOÀI `core/glossary/**` (chính là `commands::project::spawn_import_scan`) — nếu thêm
  chính tên hàm đó vào `GLOSSARY_ONLY_SURFACE`, cổng sẽ cấm đúng lời gọi mà story này cần dựng,
  một vòng luẩn quẩn giống hệt cái Story 3.1 đã gặp và giải bằng "sửa CHỮ KÝ, không nới cổng"
  (`glossary_boundary.rs:80-88`). Đọc lại `deferred-work.md` (mục Story 3.2, "bốn hàm chưa vào
  GLOSSARY_ONLY_SURFACE"): nó nói rõ quyết định phụ thuộc "hình dạng bề mặt IPC mà Story
  3.3/3.5/3.8 dựng", và hình dạng THẬT dựng ra là: `insert_candidate` (đơn lẻ) có **0** chỗ gọi
  sản phẩm còn lại (đo bằng `grep`) — nó khoá được, giống `insert_manual_entry`; hàm ghi lô mới
  và `pending_candidates` thì có chỗ gọi sản phẩm THẬT — chúng không khoá được. Kết quả:
  `GLOSSARY_ONLY_SURFACE` +1 (`insert_candidate`), `QUICK_ADD_SURFACE` +1 (`pending_candidates`).
  Xem `deferred-work.md` cho lý do đầy đủ, viết lại tại đúng mục đó.
- **Sự kiện `GLOSSARY_IMPORT_SCAN_EVENT` phát ra nhưng KHÔNG có người nghe phía frontend.**
  Task chỉ đòi "vỏ `wire` spawn một `std::thread` chạy lượt quét rồi `app.emit` một sự kiện
  mang cặp số" — không đòi một `listen()` phía TS. Story 3.6/3.8 là nơi tự nhiên để tiêu thụ sự
  kiện này (dải mọc "chờ chốt lần đầu gặp", màn duyệt hàng loạt) khi chúng thật sự cần biết
  "lượt quét vừa xong". Không dựng trước một người nghe cho một tính năng chưa tồn tại.
- **`OpenWorkState` khoá HAI LẦN NGẮN, không MỘT LẦN DÀI.** Không tường minh trong task, nhưng
  suy ra bắt buộc từ chính câu "quét chạy NỀN không chặn người dùng" của §Approach: giữ khoá
  suốt lượt quét (có thể ~500 ms – vài giây) sẽ chặn `read_open_chapter` mà frontend gọi NGAY
  SAU khi tạo Tác phẩm để mở Chương trong Editor — một hồi quy UX thật dù không AC nào của story
  gốc nói thẳng ra. Giải: khoá đọc segment (rồi nhả TRƯỚC pha CPU nặng), khoá lại để ghi lô (rồi
  nhả ngay) — đối chiếu `work_id` ở cả hai lần khoá để phát hiện ca Tác phẩm đổi giữa chừng.
- **Nhánh tiếng Anh lấy TRỌN cụm hoa liên tiếp làm MỘT ứng viên, không sinh mọi cụm con.**
  §Design Notes không nói rõ "Fire Dragon Sect" (giả sử) có sinh thêm "Fire Dragon"/"Dragon
  Sect" hay không. Quyết định: KHÔNG — chỉ dãy TOKEN liên tiếp tối đa (không cắt ngang dấu kết
  câu) là một ứng viên. Lý do: mọi ví dụ trong I/O Matrix chỉ có cụm 2 từ, và sinh cả cụm con
  đẩy độ phức tạp lên ngang với "n-gram lồng" của nhánh Zh mà không AC nào đòi — nếu Story 3.6+
  cần vế đó, đó là một quyết định thiết kế mới, không phải một lỗ hổng của lượt này.
- **N-gram Zh lọc bỏ mọi cửa sổ trượt chứa ký tự KHÔNG alphanumeric** (dấu câu/khoảng trắng/
  xuống dòng). `ngrams()` phía `core::matching` sinh MỌI cửa sổ trượt, kể cả những cửa sổ bắc
  qua dấu phẩy — một "thuật ngữ" mang dấu câu không phải một chuỗi lặp có nghĩa. Bộ lọc dùng
  `char::is_alphanumeric()` (đã có sẵn trong `core::matching::find_terms`, KHÔNG phải một định
  nghĩa "là chữ Hán" thứ hai — kho chỉ cho phép đúng MỘT định nghĩa `is_han`).
- **Bàn đo 5 giây chạy trên `DictLayers::empty()` + văn bản TỔNG HỢP, không phải Chương/từ điển
  thật của Ice.** Môi trường cài đặt (agent CLI) không mang tệp `.db` nào (AD-25, `.gitignore:
  *.db`) và không có quyền truy cập kho `.atproj` thật. Số đo đầy đủ (có `is_known` chạm dữ liệu
  thật) là một món nợ có chủ, ghi ở `deferred-work.md`. Không suy luận từ số đã có để tự kết
  luận "chắc chắn dưới 5 giây trên dữ liệu thật" — con số 969 ứng viên qua lọc trên văn bản tổng
  hợp (vượt trần 500 của §Ask First) tự nó là bằng chứng rằng văn bản tổng hợp KHÔNG đại diện đủ
  tốt để suy luận theo chiều đó.

### 2026-08-22 — rà bảng I/O phát hiện MỘT hàng chưa canh, vá trong cùng phiên

**Phát hiện:** hàng cuối §I/O & Edge-Case Matrix — *"Kho đóng giữa lượt quét ⇒ luồng nền kết
thúc lặng lẽ, không panic"* — KHÔNG có test nào canh ở lượt cài đặt đầu tiên. `grep` trên
`glossary_scan_contract.rs` + `glossary_contract.rs` cho 0 kết quả. Mã thì đúng
(`spawn_import_scan` bảo vệ ở cả hai lần khoá bằng `try_state`/`guard.as_ref()`/so `work_id`,
và khoá bằng `lock().unwrap_or_else(PoisonError::into_inner)`), nhưng **không nghiệm thu
được**: hàm nhận `tauri::AppHandle` nên `tests/**` không gọi tới nó — đúng kiểu đóng AC mà
chính §Boundaries của story này cấm ("không đóng được bằng 'nó ở thread khác'", áp cùng lý lẽ
cho "mã trông đúng").

**Vá — tách đúng luật hai lớp của `src-tauri/AGENTS.md`:** hàm mới
`guarded_open_store(open: Option<&OpenWork>, work_id: &str) -> Option<&Store>` — **hàm THUẦN**,
0 `AppHandle`, 0 `std::thread`. Đây là đơn vị QUYẾT ĐỊNH duy nhất mà `spawn_import_scan` gọi ở
CẢ HAI lần khoá (đọc segment, ghi lô) — không hai bản chép tay của cùng một điều kiện `if
open.meta.work_id != work_id`. Hành vi sản phẩm KHÔNG đổi: cùng chuỗi lệnh, cùng thứ tự khoá,
cùng cách xử lý lỗi; lượt này chỉ RÚT điều kiện ra khỏi vỏ để nó gọi được từ `#[cfg(test)]`.

**Ba ca canh — `src-tauri/src/commands/project.rs`, `mod tests` (chạy trong `cargo test --lib`,
không cần webview):**
1. `guarded_open_store_returns_none_when_no_work_is_open` — `open = None` ⇒ `None`.
2. `guarded_open_store_returns_none_and_blocks_every_write_when_the_work_id_has_changed_mid_scan`
   — dựng một `OpenWork` thật (`create_work_from_text`), gọi guard với một `work_id` KHÔNG khớp
   `opened.meta.work_id` (mô phỏng "Tác phẩm đổi giữa hai lần khoá"). Lái qua ĐÚNG hình dạng mà
   `spawn_import_scan` dùng (chỉ ghi khi guard trả `Some`) với một `ScanCandidate` GIẢ mang
   `occurrence_count = 99` — nếu vệ bảo vệ hỏng, hàng giả này sẽ lọt vào bảng chờ. Đối chứng
   BẰNG `SELECT` thật (`pending_candidates(&opened.store)` phải rỗng), không chỉ tin giá trị
   `None` trả về.
3. `guarded_open_store_returns_the_store_and_a_normal_scan_runs_to_completion_and_writes` — ca
   thường: `work_id` khớp ở cả hai lần gọi guard ⇒ đọc segment thật, `scan_candidates` sinh ứng
   viên, `insert_import_scan_candidates` ghi được ít nhất một hàng, đối chứng lại bằng
   `pending_candidates`. Đối chứng DƯƠNG của ca 2 — không có nó thì "0 ghi" ở ca 2 có thể xanh
   vì thuật toán tự nó hỏng, không phải vì vệ bảo vệ đúng.

**Đối chứng đỏ-xanh THẬT đã chạy:** gỡ tạm điều kiện `work_id` khỏi `guarded_open_store` (chỉ
còn `open?; Some(&open.store)`) ⇒ ĐÚNG MỘT ca đỏ (ca 2, hàng giả `occurrence_count = 99` lọt
vào bảng chờ thật), hai ca còn lại vẫn xanh; khôi phục ⇒ cả ba xanh lại.

**Một cạm bẫy ranh giới cây nguồn gặp phải, vá tại chỗ:** hai chuỗi fixture ban đầu dùng dấu
kết câu tiếng Trung (`萧炎登场。`) và va vào `segment_boundary.rs::
only_core_segment_may_name_the_sentence_terminators` (AC12 — bảng chữ cái kết câu chỉ được
mang trong `core/segment/**`, và `#[cfg(test)]` bên trong `src/commands/project.rs` vẫn nằm
dưới `src-tauri/src/**`, không được miễn trừ như `tests/**`). Bỏ dấu kết câu khỏi fixture
(`split_source_text` vẫn cho một segment hợp lệ dù không có dấu kết câu — "một đoạn không kết
thúc bằng dấu kết câu vẫn là một segment", đã có test riêng cho mệnh đề đó). Một cạm bẫy thứ
hai: thông báo `assert!` của ca 2 ban đầu gõ thẳng chuỗi `glossary_candidate` và va vào
`glossary_boundary.rs::only_glossary_and_schema_may_name_glossary_tables` (chỉ
`core/glossary/**`/`schema.rs` được mang tên bảng) — đổi câu chẩn đoán sang "bảng chờ ứng
viên", không đổi mệnh đề.

**Kết quả bộ nghiệm thu sau vá (chạy thật 2026-08-22):** `.githooks/pre-push` xanh — mười một
cổng + `check:lint` + `check:gates` + `check:debt-owner` + `npm run test` (339/339) + `npm run
build` + `cargo test --locked` (0 failed, nhóm unit test của `src/lib.rs` lên **12** ca, +3 so
với trước). 0 tệp mới ⇒ không sàn quần thể nào cần nâng lại.

### 2026-08-22 — rà ba lớp (blind-hunter · edge-case-hunter · verification-gap), sáu bản vá

Ice tự đối chứng từng phát hiện bằng lệnh trước khi giao — không lấy từ lời khai của
reviewer. Sáu bản vá, hai ưu tiên cao, hai ưu tiên vừa, hai ưu tiên thấp. `<frozen-after-
approval>` không bị chạm — không bản vá nào đòi đổi §I/O Matrix.

**[1] `src/glossarySettingsState.ts` — hai lớp phân giải ngưỡng LỆCH NHAU, cả hai chiều, im
lặng.** Chiều A (nhẹ): regex `/^[0-9]+$/` từ chối `"+5"` trong khi `u32::from_str` phía Rust
NHẬN nó. Chiều B (nặng — mất dữ liệu không tiếng động): TS nhận `"5000000000"`/
`"4294967296"` (đúng `Number.isInteger`, `> 0`, và JS biểu diễn chính xác — dưới
`Number.MAX_SAFE_INTEGER`), `putConfig` ghi chuỗi đó xuống đĩa, lớp phủ ĐÓNG như đã lưu
thành công; Rust đọc lại, `parse::<u32>()` trả `Err` (vượt `u32::MAX`), rơi về mặc định 5 —
người dùng gõ 5 tỷ, tưởng đã lưu 5 tỷ, ngưỡng THẬT lặng lẽ thành 5, không một câu báo.
Vá: `parsedGlossaryScanThreshold` nhận `+` tuỳ chọn, chặn trần `RUST_U32_MAX = 4294967295`.
Bốn ca test mới (`tests/frontend/glossarySettings.test.ts`): `"+5"` ⇒ 5, `"4294967295"` ⇒
đúng số đó, `"4294967296"` ⇒ `null`, `"5000000000"` ⇒ `null`. **Đối chứng đỏ-xanh:** gỡ tạm
cả hai vế (regex cũ + không trần) ⇒ đúng 3/4 ca mới đỏ (ca đúng-trần trùng hợp vẫn xanh vì
bản cũ không có trần nào để chặn nó); khôi phục ⇒ xanh lại.

**[2] `src-tauri/src/commands/project.rs` — nhánh `DictLayers` nuốt lỗi, LỆCH với chính anh
em của nó.** `layers_state.as_deref().unwrap_or(&empty_layers)` gộp HAI trạng thái khác hẳn
nhau thành một nhánh im lặng: ① đã quản lý nhưng RỖNG (0 lớp — bình thường, AD-25) và ②
CHƯA TỪNG được `app.manage(...)` (lỗi cấu hình, không nên xảy ra). Ca ② lẽ ra phải kêu như
nhánh `Store` thiếu ngay trên nó (`eprintln!` rồi dừng) nhưng lại im — hệ quả: `is_known`
LUÔN `false`, bộ lọc "không có trong từ điển nhúng" vô hiệu HOÀN TOÀN, bảng chờ ngập từ
điển mà không ai biết. Đúng ca bàn đo bàn giao của story đã chạy phải
(`DictLayers::empty()`, **969** ứng viên).
Vá: tách `guarded_dict_layers(Option<&DictLayers>) -> Option<&DictLayers>` — hàm THUẦN,
cùng khuôn `guarded_open_store`. `None` ⇒ `eprintln!` + lan `None` ra ngoài (gọi ở chỗ dùng
sẽ `return` — dừng lượt quét); `Some` (kể cả rỗng) ⇒ đi qua nguyên vẹn, lặng lẽ. Hai ca test
mới trong `mod tests` của chính tệp:
`guarded_dict_layers_returns_none_and_does_not_silently_fall_back_to_empty_when_not_managed`
· `guarded_dict_layers_passes_the_managed_layers_through_unchanged`. **Đối chứng đỏ-xanh:**
thay thân hàm bằng bản nuốt-tất-cả (luôn `Some` qua một `DictLayers::empty()` tĩnh) ⇒ đúng
2/2 ca đỏ; khôi phục ⇒ xanh lại.

**[3] `src-tauri/src/core/glossary/scan.rs` — `context_example` không trần.** Bản đầu ghi
thẳng `segments[first_segment].to_owned()`. `split_source_text` (Story 2.1) vẫn phát một
segment cho một đoạn KHÔNG kết thúc bằng dấu kết câu — một Chương thiếu dấu câu ở đâu đó có
thể sinh một "câu" dài hàng nghìn ký tự, đi nguyên vẹn vào `glossary_candidate.
context_example`. Vá: hằng có tên `CONTEXT_EXAMPLE_CHAR_LIMIT = 200` (KÝ TỰ, không byte) +
`truncated_context_example` cắt ở biên ký tự (`str::chars().take(N).collect()` — không bao
giờ cắt giữa một ký tự nhiều byte, tránh panic trên chữ Hán). Hai ca test mới trong
`glossary_scan_contract.rs`: đoạn dài 80×~19 ký tự Hán (cắt đúng 200, đúng tiền tố) và đoạn
ngắn hơn trần (giữ nguyên vẹn).

**[4] `src/config/bootstrap.ts` — nhánh phân giải `glossary_scan_threshold` KHÔNG có test
nào chạy QUA nó.** `glossarySettings.test.ts` mock TRỌN `config/bootstrap` (không
`importOriginal`), nên vệ `typeof === 'number' && Number.isInteger(...) && > 0` chưa từng
được thi hành thật trong bộ test — lệch parity với `isGlossaryMarkArray`/
`glossaryMarksRefresh.test.ts` (chạy qua bộ phân giải THẬT). Vá: `tests/frontend/
bootstrap.test.ts` (mới), mock `@tauri-apps/api/core` ở đúng biên IPC, gọi
`loadBootstrapConfig()` THẬT với sáu payload dây giả (thiếu trường · `"5"` · `0` · `-1` ·
`3.5` · `12`). **Đối chứng đỏ-xanh:** thay thân nhánh phân giải bằng một ép kiểu vô điều
kiện (`rawThreshold as number`) ⇒ 5/6 ca đỏ; khôi phục ⇒ xanh lại.

**[5] `src/glossarySettingsState.ts::resetGlossarySettings()` là MÃ CHẾT** — `grep` cho
thấy chỉ chính test của nó gọi, 0 chỗ gọi sản phẩm. Kết luận: KHÔNG nên nối vào đâu cả,
không hạ thành nợ — nêu lý do kèm neo. Ngưỡng quét là `AppConfig` ⇒ `GlobalOnly`
(`core/scope/kinds.rs:218`), tức nó KHÔNG thuộc về một Tác phẩm nào; "đổi Tác phẩm" (đường
teardown mà tiền lệ `resetGlossaryMarks` nối vào ở `editorPanelState.ts:647,2021`) không
phải một sự kiện của năm ô nhớ này. Đúng tiền lệ ĐÃ CÓ: `shortcutsState.ts::overlayOpen`
(cũng `AppConfig`/`GlobalOnly`) đi qua `EXEMPT` của `check:panel-refs`, không một hàm reset
nào. Vá: GỠ `resetGlossarySettings()` (và ca test của nó); thêm năm mục `EXEMPT` có tên +
lý do trong `scripts/check-panel-refs.mjs`, cùng khuôn `shortcutsState.ts`.

**[6] Sàn quần thể lệch nhau vô cớ.** `check-i18n.mjs::RS_FLOOR` (44/53 = 83,0%) và
`glossary_boundary.rs::RS_FLOOR` (43/53 = 81%) quét CÙNG một quần thể (53 tệp `.rs` dưới
`src-tauri/src/**`, đối chiếu bằng cách chạy cả hai cổng) mà không có lý do chính đáng để
lệch nhau — nâng `glossary_boundary.rs::RS_FLOOR` lên 44 cho khớp. `check-debt-owner.mjs::
ITEM_FLOOR` = 444/522 = 85,06% NHỈNH TRÊN dải 80–85% (làm tròn `0,85 × 522 = 443,7` LÊN thay
vì XUỐNG) — hạ về 443 (84,87%), đúng bên trong dải.

**Kết quả bộ nghiệm thu sau sáu bản vá (chạy thật 2026-08-22):** `.githooks/pre-push` xanh —
mười một cổng + `check:lint` + `check:gates` + `check:debt-owner` + `npm run test` (**30 tệp,
348 ca** — 339 trước lượt này, +9 ròng: +6 tệp mới `bootstrap.test.ts`, +4 ca ngưỡng `+`/trần
`u32::MAX` ở `glossarySettings.test.ts`, −1 ca `describe('resetGlossarySettings…')` đã gỡ) +
`npm run build` + `cargo test --locked` (0 failed; nhóm unit test của `src/lib.rs` lên **14**
ca, +2 so với vòng rà trước; `glossary_scan_contract.rs` lên **18** ca, +2). 0 tệp
`.ts`/`.vue`/`.rs` SẢN PHẨM mới (chỉ một tệp `tests/frontend/**` mới, không tính vào sàn
`src/**`) ⇒ không sàn quần thể `src/**`/`src-tauri/src/**` nào cần nâng lại — chỉ hai sàn ở
[6] đổi, và đó là để NHẤT QUÁN chứ không phải để phủ tệp mới.

## Design Notes

**Vì sao tần suất trước, từ điển sau.** Một Chương 48.640 ký tự sinh khoảng 146.000 n-gram độ dài 2–4. Lọc `≥ 5` trước cắt xuống hàng trăm chuỗi phân biệt; chỉ ngần đó đi tra. Đảo lại là 146.000 lượt `lookup_grouped`, mỗi lượt lặp tối đa 4 lớp, với số đo pha 1 duy nhất trong kho là **p95 7,324 ms** — tức hàng giờ. Đây là lý do kiến trúc, không phải tối ưu sớm.

**N-gram lồng.** `萧炎` 40 lần kéo theo `萧炎的`, `了萧炎` cũng ≥ ngưỡng. Quy tắc: nếu chuỗi con và chuỗi cha có **cùng** tần suất thì chuỗi cha là rác đuôi — giữ chuỗi con. Nếu tần suất khác nhau thì cả hai là chuỗi thật. Không có quy tắc này, một tên riêng sinh ra 6–8 hàng rác cho mỗi hàng thật, và Story 3.8 sẽ nhận một bảng chờ không dùng được.

**"Đứng đầu câu" lấy từ segment, không đoán lại.** Story 2.1 đã tách segment cấp câu **một lần lúc nhập** và lưu xuống; không đường mã nào tính lại lúc nạp. Nhánh tiếng Anh vì thế hỏi *"token này có phải token đầu của segment không"* — một phép so chỉ số, không một bộ tách câu thứ hai lệch với bộ thứ nhất.

**Bảng họ chỉ NỚI, không THÊM cột.** AC nói *"đối chiếu danh sách họ phổ biến để đoán tên người"*. Cách rẻ và đúng: một chuỗi 2–3 ký tự có ký tự đầu nằm trong bảng họ được nhận **dù dưới ngưỡng một bậc**. Nó không sinh cột mới, không sinh `candidate_origin` mới — `CHECK` hiện chỉ cho `'import_scan'`/`'review_harvest'` (`schema.rs:423`), và nới nó là một migration nữa cho 0 giá trị người dùng.

## Verification

**Commands — chạy thật 2026-08-22, kết quả dưới đây (macOS, Node 22, Rust 1.85/toolchain CI 1.97.1):**

- `cargo test --locked --manifest-path src-tauri/Cargo.toml` -- ✅ xanh, **0 failed** trên toàn bộ
  cây test (24 tệp `tests/**` + unit test của `src/lib.rs`). Các tệp trực tiếp của story:
  `glossary_scan_contract.rs` **16/16**, `glossary_contract.rs` **63/63** (bốn ca mới:
  `an_import_scan_candidate_colliding_with_an_existing_glossary_entry_is_never_inserted` ·
  `rescanning_a_rejected_source_term_via_the_batch_writer_changes_nothing_on_the_old_row` ·
  `a_batch_reports_inserted_and_skipped_counts_across_mixed_rows` ·
  `writing_an_empty_batch_returns_zero_inserted_and_zero_skipped`),
  `glossary_commands_contract.rs` **6/6** (hai ca mới cho `glossary_pending_candidates`),
  `glossary_boundary.rs` **11/11** (hai ca mới lộ ra sau khi sàn `RS_FLOOR` nâng — trước đó bị
  `< RS_FLOOR` giấu mất), `segment_contract.rs` **122/122**, `pinned_contract.rs` **10/10**.
- `npm run build` -- ✅ xanh (`vue-tsc --noEmit` × 2 + `vite build`, 0 lỗi kiểu — trường
  `glossary_scan_threshold` đi trọn qua `BootstrapConfig`/`GlossarySettingsOverlay.vue` mà
  không một lỗi TypeScript).
- `npm run test` (vitest) -- ✅ xanh, **29 tệp, 339 ca** (`glossarySettings.test.ts` mới: **15 ca**,
  gồm cả "lượt lưu TRƯỢT hiện lỗi mà KHÔNG tự đóng lớp phủ" và "mở lại sau lưu nạp giá trị VỪA
  lưu").
- `npm run check:gates` -- ✅ xanh — ba danh sách cổng khớp nhau (13 script · 16 lời gọi `ci.yml`
  · 11 cổng `pre-push`); story không thêm cổng mới.
- `npm run check:lint` -- 🔴 **ĐỎ MỘT LẦN THẬT trong lượt này**, hai lỗi ở `src/config/bootstrap.ts`
  (`eslint-disable-next-line` đặt sai dòng trong một biểu thức nhiều dòng — chỉ che dòng NGAY
  DƯỚI nó, không che cả khối). Vá bằng cách tách `rawThreshold` ra một biến cục bộ rồi đặt
  `eslint-disable-next-line` ngay trước dòng khai báo đó, đúng khuôn một-dòng mà
  `layout.value = …` (dòng liền trên) đã theo. Xanh lại sau vá.
- `npm run check:tokens` / `check:layout` / `check:commands` / `check:i18n` / `check:panel-refs`
  / `check:debt-owner` / `check:deps` -- ✅ tất cả xanh sau khi nâng sàn quần thể (xem Tasks).
  `check:tokens` in "66 tệp (63 component)"; `check:layout` in "63 tệp"; `check:commands` in
  "18 tệp `.vue` + 45 tệp `.ts` · 29 `@click` · 42 `dispatch()` · 57 command"; `check:i18n` in
  "53 tệp `.rs` + 18 tệp `.vue`… 260 khoá"; `check:panel-refs` in "45 tệp `.ts`, 115 ô nhớ cấp
  module, 26 miễn trừ có tên".
- `.githooks/pre-push` -- ✅ **xanh, mười một cổng + lint + gates + debt-owner + test + build +
  cargo test, 98 giây** ("Tất cả xanh trong 98s. Đẩy đi.").
- `npm run test:e2e` -- ⚠️ **CHƯA CHẠY** — môi trường cài đặt (agent CLI) không có GUI để dựng
  một cửa sổ Tauri thật. Đã đo được phần TĨNH: `grep -rl 'pending_candidates\|glossary_scan_
  threshold\|glossary\.settings\|import_scan' e2e/specs/` = **rỗng** ⇒ **0/12** spec hiện có
  chạm bề mặt story này dựng. Chạy tay + đọc kết quả là việc còn lại của Ice.
- CI GitHub (macOS/Windows) -- ⚠️ **CHƯA ĐỌC** — story chưa được push trong phiên cài đặt này.

**Bàn đo tạm (§Ask First), chạy 2026-08-22, XOÁ ngay sau khi lấy số (cùng tiền lệ
`zzz_scratch_bench_marks.rs` của Story 3.4/3.4b):**

Văn bản TỔNG HỢP ~48.650 ký tự / 8.848 câu (mô phỏng quy mô Chương lớn nhất có thật —
`commands/segment.rs:1111`, 9.850 câu/48.640 ký tự — KHÔNG phải chính văn bản đó; xem giới hạn
đầy đủ ở Spec Change Log và ở `deferred-work.md`):

| Bước | Thời gian | Ghi chú |
|---|---|---|
| `scan_candidates` (n-gram + dedup lồng, `is_known` trên `DictLayers::empty()`) | **452 ms** | 0 chi phí tra từ điển thật |
| `insert_import_scan_candidates` (ghi lô) | **19 ms** | 969 hàng |
| **Tổng** | **~471 ms** | dưới trần 5.000 ms một khoảng an toàn lớn — **KHÔNG HALT** |
| Ứng viên qua lọc | **969** | **VƯỢT** trần 500 của §Ask First — xem giới hạn ngay dưới |

⚠️ **Hai giới hạn phải đọc trước khi tin số trên:** (1) `is_known` chạy trên **0 lớp từ điển** —
môi trường không có `.db`; chi phí `lookup_grouped` trên dữ liệu thật CHƯA đo được ở đây, và số
đo p95 duy nhất có sẵn trong kho (7.324 ms) đo một nhánh khác (`CharIdx`, không phải `ExactBtree`
mà `is_known` dùng) nên không suy ra được. (2) Văn bản là "súp ký tự" tổng hợp, không mang cấu
trúc TỪ của ngôn ngữ thật — con số 969 ứng viên (vượt trần 500) nhiều khả năng là một GIẢ TẠO
của cách dựng văn bản (một lượt thử đầu, ít ký tự lấp đầy hơn, cho **1.490**), không phải một
tín hiệu thật về ngưỡng mặc định 5. **Ngưỡng mặc định KHÔNG bị tự đổi** — đúng luật *"đừng tự
chỉnh ngưỡng mặc định"* của §Ask First. Cả hai giới hạn đã ghi thành món nợ có chủ (Ice) ở
`deferred-work.md` §*Deferred from: 3-5-quet-ung-vien-khi-nhap-tai-lieu*.

**Manual checks (Ice, sau khi nhận bàn giao):**
- Nhập một Chương tiếng Trung thật, mở lớp phủ ngưỡng (`⌘⌥T` hoặc nút titlebar), đổi số, nhập
  lại **cùng văn bản** thành một Tác phẩm mới — đối chiếu tập ứng viên qua
  `glossary_pending_candidates`. Đây là đường duy nhất nghiệm thu vế *"cấu hình lại được"* ở
  tầng người dùng (FR52).
- Gõ liên tục trong Editor trong vài giây ngay sau khi import xong một Chương lớn — mở DevTools
  Performance, xác nhận không frame nào vượt 50 ms trong lúc lượt quét nền đang chạy (NFR2).
- Chạy `npm run test:e2e`, đọc số spec đỏ/xanh, ghi vào story.
- Đọc lượt CI trên GitHub, cả hai nền tảng, trước khi kết luận xanh.
- Lặp lại bàn đo 5 giây trên một máy có tệp `.db` từ điển thật và (lý tưởng) một Chương thật —
  ghi số thay cho số tổng hợp ở trên.

## Suggested Review Order

**Điểm vào — thuật toán quét**

- Hàm thuần, tiêm vị từ từ điển: lọc tần suất TRƯỚC, tra SAU.
  [`scan.rs:93`](../../src-tauri/src/core/glossary/scan.rs#L93)

- Trần độ dài ví dụ ngữ cảnh, cắt ở biên ký tự — vá từ vòng rà.
  [`scan.rs:47`](../../src-tauri/src/core/glossary/scan.rs#L47)

- Bảng họ nút cứng: 0 phụ thuộc mới, 0 cửa NFR15, 0 lượt dựng lại `.db`.
  [`surnames.rs:31`](../../src-tauri/src/core/glossary/surnames.rs#L31)

**Lược đồ và đường ghi**

- Bước di trú 14 — `ALTER TABLE` riêng, không sửa DDL gốc tại chỗ.
  [`schema.rs:1257`](../../src-tauri/src/core/store/schema.rs#L1257)

- Ghi lô một transaction; `DO NOTHING` để ứng viên đã bỏ không hồi sinh.
  [`candidate_store.rs:146`](../../src-tauri/src/core/glossary/candidate_store.rs#L146)

**Chạy nền và hai vệ bảo vệ** *(rủi ro cao nhất của lượt này)*

- Luồng nền sau commit; khoá `OpenWorkState` hai lần NGẮN, không một lần dài.
  [`project.rs:385`](../../src-tauri/src/commands/project.rs#L385)

- Vệ bảo vệ Tác phẩm đổi giữa chừng — hàm thuần, gọi ở CẢ HAI lần khoá.
  [`project.rs:327`](../../src-tauri/src/commands/project.rs#L327)

- Vệ bảo vệ `DictLayers` chưa quản: dừng và NÓI RA, không rơi im lặng.
  [`project.rs:350`](../../src-tauri/src/commands/project.rs#L350)

**Ngưỡng cấu hình được**

- Chỗ DUY NHẤT quyết một giá trị hợp lệ; `config_value` không có `CHECK`.
  [`store.rs:145`](../../src-tauri/src/core/scope/store.rs#L145)

- Lớp TS phải khớp Rust cả hai chiều: nhận `+`, chặn trần `u32::MAX`.
  [`glossarySettingsState.ts:83`](../../src/glossarySettingsState.ts#L83)

- Bộ phân giải lúc khởi động, có vệ kiểu lúc chạy.
  [`bootstrap.ts:86`](../../src/config/bootstrap.ts#L86)

**Bề mặt**

- Vỏ IPC chỉ-đọc — chỗ gọi sản phẩm ĐẦU TIÊN của lõi Story 3.2.
  [`glossary.rs:329`](../../src-tauri/src/commands/glossary.rs#L329)

- Adapter ba trạng thái + type guard lúc chạy cho mảng.
  [`glossary.ts:375`](../../src/config/glossary.ts#L375)

- Lớp phủ thứ tư; câu từ chối hiện NGAY, không đợi vòng IPC.
  [`GlossarySettingsOverlay.vue:134`](../../src/GlossarySettingsOverlay.vue#L134)

- Đăng ký lệnh + hợp âm `Mod+Alt+T` (đã đối chứng: duy nhất trong kho).
  [`index.ts:532`](../../src/commands/index.ts#L532)

- Lớp phủ mount cạnh ba cái đã có, cùng tầng.
  [`App.vue:52`](../../src/App.vue#L52)

**Ngoại vi — test và cổng**

- N-gram lồng: chuỗi cha cùng tần suất là rác đuôi, bị loại.
  [`glossary_scan_contract.rs:54`](../../src-tauri/tests/glossary_scan_contract.rs#L54)

- Ca đã tự chạy phép đỏ→xanh: gỡ vệ ⇒ ứng viên giả lọt vào bảng chờ.
  [`project.rs:606`](../../src-tauri/src/commands/project.rs#L606)

- Ứng viên đã `rejected` không đổi một cột nào — đối chứng bằng `SELECT`.
  [`glossary_contract.rs:1209`](../../src-tauri/tests/glossary_contract.rs#L1209)

- Bốn ca trần `u32` — ba ca đỏ nếu gỡ vá, đã tự chạy lại.
  [`glossarySettings.test.ts:81`](../../tests/frontend/glossarySettings.test.ts#L81)

- Chạy bộ phân giải THẬT, không mock trọn module.
  [`bootstrap.test.ts:18`](../../tests/frontend/bootstrap.test.ts#L18)

- Ghim phiên bản lược đồ 13 → 14; cổng cố ý, là chữ ký cho lượt đổi.
  [`segment_contract.rs:566`](../../src-tauri/tests/segment_contract.rs#L566)

- Năm miễn trừ CÓ TÊN, theo tiền lệ `shortcutsState.ts` đã có sẵn.
  [`check-panel-refs.mjs:284`](../../scripts/check-panel-refs.mjs#L284)

## Kết quả đóng vòng review 2026-08-22 — spec `3-5-fix-review-findings`

Vòng review ghi 19 mục; đối chứng rút còn **17 finding riêng** vì hai cặp frontend mô tả
cùng nguyên nhân và cùng hành động sửa. Finding *"writer đang bận"* cũng được thu hẹp theo
topology thật: chưa có job project khác xếp trước trong cùng mutex; lỗi có thật là
`OpenWorkState` bị giữ xuyên qua chính batch đã đo **19 ms / 969 hàng**. Bản vá vẫn tách
enqueue/reply thành một write-ticket package-private, và test kênh tất định (0 sleep) chứng
minh state được nhả trong lúc writer còn bị chặn.

Các finding còn lại đã đóng theo đúng biên spec:

- Lượt scan có outcome ba trạng thái; layer từ điển lỗi trả
  `dictionary_inconclusive`, không sinh ứng viên giả. Generation mới huỷ worker cũ trong
  pha đếm/trước lookup/trước enqueue; worker bị huỷ không phát event hoàn tất.
- Batch nạp Global + Work đúng một lần, `ScopeResolver::apply_override` một lần, lọc term
  đã tồn tại trước enqueue và cộng vào `skipped`; `WHERE NOT EXISTS` của Work vẫn giữ làm
  race guard. Không `ATTACH`, không giao dịch chéo database, không `Arc<Store>`.
- Cụm hoa được dựng lại từ token nên một/hai khoảng trắng và dấu phẩy cùng thành một key
  sạch; alias `蕭 → 萧` dùng lại bảng họ giản thể thay vì dựng bảng luật thứ hai.
- IPC lỗi lạ chỉ thành `null` ngoài Tauri; trong Tauri nó thành `err.unknown`, giữ modal mở.
  Save pending sở hữu vòng đời modal; modal mở chặn keymap toàn cục và đăng ký bề mặt chọn
  vai `display`.
- Năm khoảng hở kiểm chứng đã có chủ: predicate dưới/đủ ngưỡng, count/context qua command,
  lọc Global, writer/state bằng kênh, và đường IPC/event/modal trên webview thật.

**Bằng chứng chạy thật sau bản vá:** `npm run test` **30 tệp / 352 ca**; `npm run build`;
Rust unit **17/17**, `glossary_scan_contract` **24/24**, `glossary_contract` **63/63**,
`glossary_commands_contract` **6/6**, `glossary_boundary` **11/11**; `.githooks/pre-push`
xanh toàn bộ trong **132 giây**; targeted e2e `story-3-5-review.e2e.mjs` **2/2** trên
WKWebView thật. Ca e2e đăng ký listener trước import, dùng Work mới cho ngưỡng persisted
6 rồi 5 và quan sát command trả lúc event count còn 0. Tauri-service trên máy chạy không
lấy được active-window state cho pointer đầu; spec vẫn gọi `realClick`, rồi dùng fallback
DOM theo đúng thứ tự `mousedown → focus → mouseup → click` khi hiệu ứng chưa xảy ra. Đây là
giới hạn của bộ lái local, không đổi phán quyết event/ngưỡng/keymap/save đã đo trong webview.

### Bổ sung bằng chứng Matrix 2026-08-22

🔵 **Cập nhật 2026-08-22:** sau lượt audit Matrix, số unit trực tiếp tăng từ **17/17**
lên **19/19**. Ba khoảng phủ trước đó mới được chứng minh bằng các test rời nay đã được
nối qua đúng quyết định mà worker sản phẩm dùng:

- Một `GroupedLookup` có `SkippedLayer::OpenFailed` đi qua
  `dictionary_probe_from_grouped` thành `DictionaryProbe::Inconclusive`, rồi đi qua
  `import_scan_next_step` thành `EmitDictionaryInconclusive`; bảng `pending_glossary`
  của Work thật vẫn rỗng, nên đường này không thể enqueue batch giả.
- Một test duy nhất tạo generation A, phát generation B ngay trong callback đếm của
  `scan_candidates`, nhận `ScanOutcome::Cancelled`, rồi chứng minh quyết định worker là
  `Stop`: lookup không được gọi, bảng chờ rỗng và không có nhánh phát event hoàn tất.
- Seam spawn tối thiểu được tiêm `thread::Builder::spawn` trả `Err`; Work đã commit vẫn
  đọc được cả hàng dữ liệu, `project.db` và `meta.json`, đồng thời hàm trả bình thường —
  lỗi khởi động scan không panic và không đảo ngược import.

**Bằng chứng trực tiếp sau bổ sung:** Rust unit **19/19**,
`glossary_scan_contract` **24/24**, `glossary_commands_contract` **6/6** và
`glossary_boundary` **11/11**; `check:i18n` và `git diff --check` đều xanh. Full
`.githooks/pre-push` sau bổ sung xanh toàn bộ trong **123 giây**.

### Bổ sung vá review Step 4 ngày 2026-08-22

🔵 **Cập nhật 2026-08-22:** mapping lookup nay giữ đủ dữ liệu cắt trang theo precedence
đã duyệt: `skipped` thắng tuyệt đối; hit trong `groups`/`hidden_sources` là `Known`;
chỉ có `truncated_layers` là `Inconclusive`; còn lại mới là `Missing`. Nhánh
`dictionary_inconclusive` dùng một constructor payload đã có test serialization khóa
`outcome` và hai số đếm 0.

Pha đếm chỉ giữ chỉ số segment đầu tiên. Context được cắt sau threshold và chỉ khi probe
trả `Missing`; nhiều term cùng segment clone đúng một chuỗi cache. Scope filter vẫn đi
qua `ScopeResolver::apply_override`, nhưng hai query nay chỉ lấy `source_term` vào
`BTreeMap<String, ()>`; test chạy qua cả Global lẫn Work và SQL Work race guard không đổi.
Cancellation muộn đổi generation ngay sau scope filtering trả `None`, không tạo ticket và
bảng chờ thật vẫn rỗng.

Frontend có đủ bốn nhánh lỗi không-shaped mới cho bootstrap/delete trong và ngoài Tauri;
component test mount `GlossarySettingsOverlay` thật, tạo Range thật trong modal và chứng
minh vai `display` trả rỗng cho Auto-Lookup. E2E chờ `>= 1` rồi khóa đúng một event, ép
tiền đề Library trước khi thử `Mod+2`, và fallback save kiểm lại node trong cùng lượt DOM.

**Bằng chứng trực tiếp:** project worker **15/15**, Rust unit **23/23**,
`glossary_scan_contract` **25/25**, `glossary_contract` **63/63**,
`glossary_commands_contract` **6/6**, `glossary_boundary` **11/11**; Vitest đích danh
**2 tệp / 33 ca**, toàn bộ Vitest **30 tệp / 357 ca**; `npm run build` và
`check:lint` xanh; targeted WKWebView e2e **2/2** xanh. Cảnh báo active-window của
tauri-service vẫn là giới hạn local đã ghi, không đổi phán quyết.

Full `.githooks/pre-push` sau toàn bộ vá Step 4 xanh trong **105 giây**.

---
title: 'Story 5.1: Mô hình Library hai tầng'
type: 'chore'
created: '2026-08-27'
status: 'done'
baseline_commit: '7d1165f6f30f774e2eb4364d95bb230f346548fc'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-5-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Mô hình hai tầng `Work → Chapter` mà Story 5.1 mô tả **đã tồn tại gần trọn vẹn** trong mã — bảng `work`/`chapter` trong `project.db` (lược đồ v15), `work_id` là UUID v4, `chapter.id` là INTEGER cục bộ, `Tier::Work` là một trong hai tầng của `ScopeResolver`, không có thực thể tầng thứ ba. Thứ **không** tồn tại là **cổng canh**: `AGENTS.md:41` tự khai nguyên văn *"Không cổng nào canh luật này"* về quy ước đặt tên, `schema.rs:631-634` khai `source_lang` *"cưỡng chế ở tầng ứng dụng"* mà không test nào thử phá, và mệnh đề *"Glossary/TM gắn ở tầng Tác phẩm"* chưa có phép kiểm nào. Đồng thời phép đếm cho thấy 4 họ định danh đang đặt tên cho **khái niệm tầng Tác phẩm** bằng chính từ bị cấm — `ProjectError` (19), `ProjectNoWorkOpen` (9), `ProjectCreateFailed` (2), `ProjectMetaTooNew` (1) — nằm ngoài miễn trừ viết ra, vốn chỉ bao `StoreKind::Project` và `ProjectStore` vì hai cái đó đặt tên cho **kho**, không cho thực thể.

**Approach:** Đóng khoảng cách giữa *mô hình đã có* và *mô hình được cưỡng chế*: đổi tên 4 họ định danh tầng Tác phẩm sang tiền tố `Work`, rồi dựng `src-tauri/tests/naming_boundary.rs` theo đúng khuôn năm tệp `*_boundary.rs` đã có — quét tĩnh cây nguồn, kèm đối chứng dương trên chuỗi dựng tay để chứng minh phép quét không mù. Cùng lượt, bổ sung các ca hợp đồng cho ba mệnh đề còn trần trụi: không có thực thể tầng thứ ba, `source_lang` bất biến, Glossary/TM phân giải ở `Tier::Work`.

## Boundaries & Constraints

**Always:**
- Danh sách miễn trừ của cổng lấy **nguyên văn** từ `AGENTS.md:41` (`.atproj`, `project.db`, `StoreKind::Project`, `ProjectStore`, `PROJECT_MIGRATIONS`, `commands/project.rs`, `ports/project_store.rs`, `tests/project_contract.rs` — tất cả đặt tên cho KHO). Cổng không được tự nới thêm một mục nào.
- Mỗi vị từ quét phải có **đối chứng dương** (chuỗi vi phạm dựng tay bị bắt) **và đối chứng âm** (chuỗi sạch không bị bắt) — khuôn `matching_boundary.rs::the_warm_jieba_check_would_actually_flag_a_removed_call` và `ai_boundary.rs`.
- `document.*` của DOM và `document_dir()` của OS **không** là vi phạm — chúng là API nền tảng, không phải tên thực thể. Cổng phải phân biệt được, và phải có ca chứng minh nó phân biệt được.
- Đổi tên là đổi tên **thuần**: hai khoá i18n `err.project.create_failed`/`err.project.no_work_open` đổi cùng lượt ở `core/i18n/mod.rs` **và** `src/i18n/vi.json`, không để lại khoá mồ côi.

**Ask First:**
- Bất kỳ thay đổi nào lên `ARCHITECTURE-SPINE.md`. Tệp đó tự mâu thuẫn (dòng 779 cấm `Project`; các dòng 35/49/85/996/1030 đặt tên cổng chính thức là `ProjectStore`) — đó là hồ sơ của Winston, không phải của lượt build này.
- Nếu cổng bắt được vi phạm **ngoài** 4 họ định danh đã liệt kê: HALT, trình danh sách, không tự đổi tên.

**Never:**
- Không thêm trường `cover`/ảnh bìa. Hôm nay nó có **0 lần xuất hiện** trong toàn bộ `src-tauri/src` + `src`, và không đường sản phẩm nào ghi hay đọc nó — thêm cột + bump `META_SCHEMA_VERSION` 1→2 hôm nay là *"một khoá cho tính năng chưa tồn tại"*, đúng thứ Story 1.7 §Completion Notes #3 cấm và `scope_contract.rs` trích nguyên văn để cưỡng chế. Ghi nợ, chủ: **Story 5.6** (lưới Tác phẩm — nơi bìa lần đầu được nhìn thấy).
- Không làm `work.updated_at` sống. Hôm nay nó được ghi đúng một lần lúc `INSERT` (`commands/project.rs:177`) và **không câu `UPDATE` nào** chạm nó. Ghi nợ, chủ: **Story 5.2** (Indexer — nơi "sắp xếp theo ngày sửa" làm lời khai đó lộ ra).
- Không dựng đường mở lại một `.atproj` đã có, không dựng lưu trữ Translation Memory, không đổi lược đồ `project.db`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Vi phạm thật | Chuỗi nguồn dựng tay `pub enum ProjectError {` | Vị từ quét trả về "bắt được" | N/A |
| Miễn trừ có tên | Chuỗi `StoreKind::Project` và `dir.join("project.db")` | Không bị bắt | N/A |
| API nền tảng | `document.querySelector(...)` · `app.path().document_dir()` | Không bị bắt | N/A |
| Từ cấm còn lại | Chuỗi dựng tay chứa `struct Book` · `NovelMeta` · `DocumentStore` | Bắt được cả ba | N/A |
| Cây nguồn hôm nay | Toàn bộ `src-tauri/src` + `src` sau khi đổi tên | 0 vi phạm | Test đỏ kèm danh sách `file:line` |
| Bất biến ngôn ngữ nguồn | Quét mọi câu SQL trong `src-tauri/src` | Không câu `UPDATE` nào chạm cột `source_lang` | Test đỏ nêu câu vi phạm |

</frozen-after-approval>

## Code Map

- `AGENTS.md:41` -- nguồn nguyên văn của quy ước đặt tên **và** của lời tự khai *"Không cổng nào canh luật này"*. Sau story này, mệnh đề cuối đó phải được sửa lại cho đúng.
- `src-tauri/src/core/library/mod.rs:11` -- `pub enum ProjectError` + biến thể `CreateFailed`; 11 chỗ chạm trong tệp. Doc-comment của nó tự khai là *"thao tác ở tầng Tác phẩm"* ⇒ đây là tên thực thể, không phải tên kho ⇒ đổi thành `WorkError`.
- `src-tauri/src/core/library/atproj.rs` -- 6 chỗ chạm `ProjectError`.
- `src-tauri/src/core/i18n/mod.rs:156,159` -- `ProjectCreateFailed => "err.project.create_failed"`, `ProjectNoWorkOpen => "err.project.no_work_open"`; dòng 45 và dòng 271 mang chú thích liên quan.
- `src-tauri/src/commands/project.rs` -- 2 chỗ chạm; `:177` là câu `INSERT INTO work` duy nhất; `:183` là câu `INSERT INTO chapter` **duy nhất trong toàn kho** (`ord = 1` viết cứng); `:158` sinh `Uuid::new_v4()`; `:252` là chỗ gọi `ScopeResolver::with_work` duy nhất.
- `src-tauri/src/commands/chapter.rs` -- 1 chỗ chạm `ProjectNoWorkOpen`.
- `src/i18n/vi.json:14-15` -- hai khoá `err.project.*` phải đổi cùng lượt.
- `src/config/segment.ts:528,594,633` -- doc-comment trích tên khoá `err.project.no_work_open`; chỉ là văn bản, nhưng phải đổi để không nói dối.
- `src-tauri/src/core/store/schema.rs:636-645` -- DDL `work` (`CHECK (id = 1)`); `:671-680` DDL `chapter`; `:631-634` lời khai *"không có `UPDATE` nào chạm cột này"* mà story này biến thành test.
- `src-tauri/src/core/scope/mod.rs:81-85` -- `enum Tier { Global, Work }`; `kinds.rs:157-219` -- bảng chín `ScopeKind`, trong đó `Glossary` và `TranslationMemory` là hai cái phải khẳng định ở `Tier::Work`.
- `src-tauri/tests/ai_boundary.rs` -- **khuôn để chép**: quét tĩnh + đối chứng dương trên chuỗi dựng tay. Cùng họ: `scope_boundary.rs`, `matching_boundary.rs`, `glossary_boundary.rs`, `store_boundary.rs`, `segment_boundary.rs`.
- `src-tauri/tests/project_contract.rs:103-110` -- `work.id` UUID v4 **đã có test**; `:598-613` một Work đúng một Chapter **đã có test**; `:122` id Chương đã về hưu không tái dùng **đã có test**. Không dựng lại ba ca này.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/library/mod.rs` -- đổi `ProjectError` → `WorkError` (giữ nguyên biến thể `CreateFailed` và toàn bộ doc-comment giải thích vì sao nó không phải `StoreError`) -- tên phải nói đúng tầng nó phục vụ.
- [x] `src-tauri/src/core/library/atproj.rs`, `src-tauri/src/commands/project.rs`, `src-tauri/src/commands/chapter.rs` -- cập nhật chỗ gọi theo tên mới -- đổi tên thuần, không đổi hành vi.
- [x] `src-tauri/src/core/i18n/mod.rs` -- `ProjectCreateFailed` → `WorkCreateFailed` (`err.work.create_failed`), `ProjectNoWorkOpen` → `WorkNoneOpen` (`err.work.none_open`), `ProjectMetaTooNew` → `WorkMetaTooNew` -- khoá i18n cũng là tên thực thể lộ ra ngoài.
- [x] `src/i18n/vi.json` -- đổi hai khoá tương ứng, giữ nguyên văn bản tiếng Việt -- không để khoá mồ côi.
- [x] `src/config/segment.ts` -- cập nhật ba doc-comment trích tên khoá -- tài liệu không được nói dối sau đổi tên.
- [x] `src-tauri/tests/naming_boundary.rs` -- **tệp mới**: cổng quét tĩnh cưỡng chế `Work`/`Chapter` và cấm `Project`/`Book`/`Novel`/`Document`, với danh sách miễn trừ tường minh và trọn bộ đối chứng của I/O Matrix -- biến quy ước thành phép đo.
- [x] `src-tauri/tests/project_contract.rs` -- thêm ba ca: (a) `project.db` không có bảng thực thể nào ngoài `work` và `chapter`; (b) không câu SQL nào trong cây nguồn `UPDATE` cột `source_lang`; (c) `ScopeKind::Glossary` và `ScopeKind::TranslationMemory` đều phân giải ở `Tier::Work` -- ba mệnh đề của Story 5.1 hôm nay chỉ sống trong chú thích.
- [x] `AGENTS.md` -- sửa câu *"Không cổng nào canh luật này"* thành tên tệp cổng vừa dựng -- lời tự khai phải theo kịp thực tế.
- [x] `AGENTS.md` -- mở dòng luật để nó nêu **đủ tám** mục miễn trừ thay vì hai -- cổng khai "miễn trừ NGUYÊN VĂN" nhưng luật chỉ viết ra 2/8; sửa nguồn cho lời khai thành đúng, thay vì hạ giọng lời khai.
- [x] `src-tauri/tests/naming_boundary.rs` -- thêm ca thứ 13 đối chiếu dòng luật `AGENTS.md` với mảng `STORE_EXEMPT` -- câu "hai danh sách phải khớp" vừa viết ra cũng là một quy ước không cổng nào canh.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- ghi hai món nợ có chủ: `cover` (chủ: Story 5.6) và `work.updated_at` không bao giờ đổi (chủ: Story 5.2) -- nợ có chủ, không phải nợ trôi nổi.

**Acceptance Criteria:**
- Given cây nguồn sau khi đổi tên, when chạy `naming_boundary.rs`, then 0 vi phạm và cổng vẫn bắt được cả bốn chuỗi vi phạm dựng tay.
- Given một người sau này thêm `pub struct DocumentMeta` vào `src-tauri/src`, when chạy bộ test, then cổng đỏ và nêu đúng `file:line`.
- Given `AGENTS.md`, when đọc câu về quy ước đặt tên, then nó trỏ tới cổng có thật, không còn khai "không cổng nào canh".
- Given bộ test cũ (`project_contract.rs`, `scope_contract.rs`, `ipc_contract.rs`, bộ frontend), when chạy sau đổi tên, then xanh — đổi tên không đổi hành vi.

## Spec Change Log

- **Vòng rà 1 — phát hiện kích hoạt:** ba lớp rà độc lập chỉ ra `IpcError` vẫn dựng `code` bằng chuỗi `"project.create_failed"` / `"project.no_work_open"` trong khi biến thể `MessageKey` và khoá `vi.json` đã đổi sang `work.*` — một thế chia ba cho đúng khái niệm story này sinh ra để đổi tên.
  **Chỗ spec hở:** §Bounders "Đổi tên là đổi tên **thuần**" liệt kê `MessageKey` và `vi.json` nhưng **không nhắc chuỗi `code`**, và lệnh `grep -rn "err\.project\."` ở §Verification **không thể** bắt nó vì chuỗi đó không mang tiền tố `err.`. Agent thực thi làm đúng spec; spec thiếu.
  **Đã sửa:** 9 chỗ đổi sang `work.create_failed`/`work.none_open`; cổng mở rộng để quét chính đối số `code` của `IpcError::new(` ở dạng viết thường (cổng cũ khớp phân biệt hoa/thường nên mù với `"project."`); §Verification thêm lệnh grep dạng `"project.`.
  **Trạng thái xấu đã tránh:** một quy ước đặt tên được tuyên bố là đã cưỡng chế, trong khi định danh máy-đọc đi qua IPC vẫn mang đúng từ bị cấm — và cổng lẫn lệnh xác minh đều không thấy.
  **Xếp `patch` chứ không `bad_spec`, có chủ ý:** bản vá thuần cơ học, không mơ hồ, bán kính frontend bằng 0 (frontend chỉ kiểm `code` là chuỗi, không rẽ nhánh theo giá trị). Hoàn nguyên ~1.500 dòng đã xanh để dựng lại một kết quả gần y hệt là nghi thức, không phải tính mạch lạc. Ghi lại ở đây để spec thôi nói sai.
  **KEEP — phải sống sót mọi lần dựng lại:** (1) khuôn đối chứng dương/âm cho MỌI vị từ quét, gồm cả các vị từ mới; (2) `STORE_EXEMPT` đúng 8 mục, khớp hai chiều với dòng luật `AGENTS.md`; (3) `is_platform_document_type_reference` giữ vế chặt — `DocumentStore` ở vị trí kiểu vẫn phải bị bắt; (4) mọi giới hạn mà bộ quét chấp nhận phải được ghi thẳng vào doc-comment, không để ngầm.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test naming_boundary` -- expected: xanh, và mọi ca đối chứng dương/âm đều chạy.
- `cd src-tauri && cargo test --test project_contract --test scope_contract --test ipc_contract` -- expected: xanh, gồm ba ca mới.
- `cd src-tauri && cargo test` -- expected: toàn bộ xanh sau đổi tên.
- `cd src-tauri && cargo clippy --all-targets` -- expected: **22 cảnh báo có sẵn ở bảy tệp**, story này đẻ thêm **0**. ⚠️ Dòng này ban đầu ghi `-D warnings` kèm *"không cảnh báo"* — sai hai lần: kỳ vọng đó không đạt được kể cả ở baseline sạch, VÀ `-D warnings` dừng ở crate lib nên không bao giờ chạm target test (phơi 8/22). Chi tiết và cách phân biệt "story đẻ ra" với "có sẵn": mục `defer` trong `deferred-work.md`.
- `npm test` -- expected: xanh; bắt được nếu `vi.json` mất khoá hoặc thừa khoá mồ côi.
- `grep -rn "err\.project\." src src-tauri` -- expected: **0 dòng**.
- `grep -rn 'IpcError::new(' -A2 src-tauri/src | grep '"project\.'` -- expected: **0 dòng**. ⚠️ Lệnh grep phía trên KHÔNG thay được lệnh này: chuỗi `code` không mang tiền tố `err.` nên nó lọt qua — đó chính là chỗ hở vòng rà 1 bắt được.

## Completion Notes

**Đổi tên thuần, bốn họ định danh, đo trước khi khai đạt:**
- `ProjectError` → `WorkError` (`core/library/mod.rs`, 6 chỗ chạm cộng `atproj.rs` 6 chỗ, `commands/project.rs` 2 chỗ).
- `ProjectCreateFailed` → `WorkCreateFailed` (`err.work.create_failed`), `ProjectNoWorkOpen` → `WorkNoneOpen` (`err.work.none_open`) — cả hai đổi ở `core/i18n/mod.rs` (enum + mọi chú thích trích tên khoá) và `src/i18n/vi.json`, giữ nguyên văn bản tiếng Việt.
- `ProjectMetaTooNew` chỉ tồn tại trong MỘT dòng chú thích ở `core/i18n/mod.rs` (biến thể chưa từng được thêm thật — Ice chốt 2026-08-06 là chưa tới lúc, xem `core/library/mod.rs:41-50`); đổi chữ thuần, không thêm biến thể/khoá mới — đúng "Never" của story (không dựng đường mở lại `.atproj`).
- Ba tệp test dùng `ProjectNoWorkOpen` ngoài Code Map (`glossary_commands_contract.rs`, `segment_contract.rs` — 9 chỗ tổng cộng) cũng đổi cùng lượt để cây biên dịch.

**`src-tauri/tests/naming_boundary.rs` — tệp mới, 12 ca**, quét CẢ `src-tauri/src/**` (Rust, lọc `//`) LẪN `src/**` (`.ts`+`.vue`, bỏ comment `//`/`/* … */`/`<!-- … -->` thật vì `GridPanel.vue` mang 886 dòng JSDoc mà một bộ lọc chỉ-`//` sẽ đọc nhầm thành mã). Khớp CASE-SENSITIVE trên bốn từ hoa đầu chữ, khớp có neo biên TRƯỚC (bắt cả token đứng một mình lẫn tiền tố ghép như `NovelMeta`), miễn trừ nguyên văn tám chuỗi của `AGENTS.md:41`. Hai phát hiện ngoài dự tính của I/O Matrix gốc, cả hai đã vá và có ca riêng, không phải "vi phạm ngoài 4 họ" cần Ice quyết:
- `target: Document` (`src/panels/selectionContract.ts:275`) — kiểu DOM toàn cục, không phải thực thể ta đặt tên. Thêm `is_platform_document_type_reference` (hẹp đúng hình dạng: sau `": "`, không mang hậu tố PascalCase) cộng ca đối chứng âm chứng minh `DocumentStore` ở cùng vị trí vẫn bị bắt.
- `Project,` — điểm khai báo KHÔNG tiền tố của chính biến thể `StoreKind::Project` (`core/store/mod.rs:171`, bên trong `enum StoreKind`). Đây là cùng một miễn trừ đã đặt tên áp cho điểm khai thay vì điểm gọi, không phải một miễn trừ mới — khoanh chặt vào đúng thân `enum StoreKind { … }`, kèm ca đối chứng âm chứng minh một `Project` bên trong enum KHÁC vẫn bị bắt.

**`src-tauri/tests/project_contract.rs` — năm ca mới** (ba ca chính + hai ca đối chứng dương/âm cho vị từ SQL): bảng `project.db` ngoài `work`/`chapter` (miễn trừ tường minh sáu bảng chi tiết: `segment`/`segment_version`/`glossary_entry`/`glossary_candidate`/`schema_migration_log`/`sqlite_sequence`), quét toàn `src-tauri/src/**` không câu SQL nào `UPDATE` cột `source_lang` (bộ tách chuỗi literal Rust tự viết, nối `\`-continuation, chạy trên văn bản đã bỏ comment), và `ScopeKind::Glossary`/`TranslationMemory` phân giải thật ở `Tier::Work` qua `ScopeResolver::with_work` (không chỉ đọc bảng ngữ nghĩa tĩnh).

**Nghiệm thu:**
- `cargo test --test naming_boundary`: **12/12 xanh**.
- `cargo test --test project_contract --test scope_contract --test ipc_contract`: **5 + 33 + 24 = 62/62 xanh**.
- `cargo test` (toàn `src-tauri`): **xanh** (exit 0; suite đầy đủ, không ca nào đỏ).
- `npm test`: **43 tệp / 567 ca / 0 đỏ**.
- `node scripts/check-i18n.mjs`: Kiểm A–E đều đạt (387 khoá `vi.json`, không khoá mồ côi/thiếu).
- `grep -rn "err\.project\." src src-tauri`: **0 dòng**.

⚠️ **`cargo clippy --all-targets -- -D warnings` KHÔNG xanh, và đây KHÔNG phải nợ của story này** — đo bằng `git stash` về baseline gốc (trước mọi thay đổi của story): CÙNG 8 lỗi `-D warnings` (useless_conversion ở `commands/glossary.rs:1316,1393`; redundant_closure ở `commands/pinned.rs:116,164,197`; redundant_guards ở `core/glossary/exchange.rs:945`; type_complexity ở `core/scope/resolve.rs:198` và `core/scope/mod.rs:335`) đã có TRƯỚC khi story này chạm bất kỳ tệp nào — năm tệp đó nằm ngoài Code Map của Story 5.1. `cargo clippy --all-targets` (không `-D warnings`) trên riêng các tệp story này chạm tới (`naming_boundary.rs`, `project_contract.rs`, `commands/project.rs`, `commands/chapter.rs`, `core/library/mod.rs`, `core/library/atproj.rs`, `core/i18n/mod.rs`) — **0 cảnh báo**. Món nợ clippy có sẵn chưa được ghi vào `deferred-work.md` trước lượt này; không ghi thêm ở đây vì đó là một quyết định phạm vi (chủ sở hữu năm tệp không liên quan) ngoài "Ask First" của story — cờ lên cho Ice ở báo cáo bàn giao.

---

**Bổ sung ở lượt kiểm sau bàn giao (không phải việc của agent thực thi):**

🔴 **Cổng khai "miễn trừ NGUYÊN VĂN từ `AGENTS.md:41`" trong khi luật chỉ viết ra 2 trong 8 mục.** `AGENTS.md:41` nêu đích danh `StoreKind::Project` và `ProjectStore` (cộng `.atproj` là tên kho); năm mục còn lại (`project.db`, `PROJECT_MIGRATIONS`, `commands/project.rs`, `ports/project_store.rs`, `tests/project_contract.rs`) được suy từ **lý do** đã viết ("đặt tên cho KHO"), không từ **chữ**. Agent chép trung thành đúng câu §Boundaries của spec — mà câu đó là **lỗi của người soạn spec**, không phải của agent. Sửa theo hướng *sửa nguồn*: `AGENTS.md:41` nay nêu đủ tám mục, nên lời khai của cổng thành đúng.

🔴 **Ca thứ 13 — `the_written_rule_and_the_enforced_exemption_list_name_the_same_eight_things`.** Câu vừa thêm vào `AGENTS.md` (*"danh sách ở đây và mảng `STORE_EXEMPT` phải khớp nhau từng mục"*) tự nó lại là một quy ước không cổng nào canh — đúng hình dạng hỏng story này sinh ra để đóng. Ca mới đọc `AGENTS.md`, khẳng định mọi mục của `STORE_EXEMPT` có mặt trong dòng luật, và số mục trong backtick không ít hơn 8. **Đối chứng dương đã chạy thật:** gỡ `PROJECT_MIGRATIONS` khỏi dòng luật ⇒ ca đỏ với đúng thông điệp *"cổng đang rộng hơn luật đã viết"*; `AGENTS.md` đã khôi phục nguyên trạng sau phép thử.

🔵 **`cargo fmt` và `cargo clippy` KHÔNG phải cổng của kho này** — đo trực tiếp: không chuỗi `cargo fmt` hay `cargo clippy` nào xuất hiện trong `.github/workflows/ci.yml` (56KB), `package.json`, hay `scripts/`. Baseline lệch rustfmt 575 chỗ, tức mã định dạng tay là house style; `naming_boundary.rs` lệch 9 chỗ so với rustfmt và điều đó không vi phạm chuẩn nào kho đang giữ. Phép đo per-file phải dùng `rustfmt --edition 2024 --check <tệp>` — `cargo fmt --check -- <tệp>` **bỏ qua đối số** và trả cùng một con số ~590 cho mọi tệp, một cái bẫy đọc số.

⚠️ **Nợ chưa có chủ, cờ lên cho Ice:** 8 lỗi `clippy -D warnings` có sẵn ở năm tệp ngoài phạm vi story (đã liệt kê ở §Verification). Không ghi vào `deferred-work.md` ở lượt này vì gán chủ cho chúng là quyết định phạm vi của Ice, không phải của lượt build.

**Nghiệm thu lại, tự chạy, không nhận số của agent:**
- `cargo test --test naming_boundary`: **13/13 xanh** (12 của agent + 1 ca đối chiếu luật).
- `cargo test` toàn `src-tauri`: **xanh**, gồm `project_contract` 33/33 · `scope_contract` 24/24 · `ipc_contract` 5/5 · `segment_contract` 123/123.
- `npm test`: **43 tệp / 567 ca / 0 đỏ**. `node scripts/check-i18n.mjs`: A–E đạt.
- `grep -rn "err\.project\."`: **0 dòng**.

## Suggested Review Order

**Cổng — thứ story này thật sự thêm vào kho**

- Bắt đầu ở đây: bốn từ cấm và tám miễn trừ, cạnh nhau, là toàn bộ luật.
  [`naming_boundary.rs:82`](../../src-tauri/tests/naming_boundary.rs#L82)

- Nguồn của luật; cổng chỉ thi hành nó, và ca ở dòng 941 canh hai bên khớp nhau.
  [`AGENTS.md:41`](../../AGENTS.md#L41)

- Đối chứng nặng nhất: cây nguồn thật, 0 vi phạm — phần còn lại chứng minh phép quét không mù.
  [`naming_boundary.rs:765`](../../src-tauri/tests/naming_boundary.rs#L765)

- Che theo neo biên, không theo chuỗi con: `ProjectStoreView` vẫn bị bắt.
  [`naming_boundary.rs:305`](../../src-tauri/tests/naming_boundary.rs#L305)

- Vế viết thường, thêm ở vòng rà: cổng cũ mù với `code` của IPC.
  [`naming_boundary.rs:400`](../../src-tauri/tests/naming_boundary.rs#L400)

- Miễn trừ hẹp cho kiểu DOM; `DocumentStore` cùng vị trí vẫn phải đỏ.
  [`naming_boundary.rs:338`](../../src-tauri/tests/naming_boundary.rs#L338)

**Đổi tên — ba họ định danh thật, cộng chuỗi `code` bắt được ở vòng rà**

- Tên nay nói đúng tầng nó phục vụ, không phải tên kho.
  [`library/mod.rs:33`](../../src-tauri/src/core/library/mod.rs#L33)

- Chỗ hở vòng rà 1: `code` từng nói "project" trong khi hai lớp kia đã đổi.
  [`library/mod.rs:87`](../../src-tauri/src/core/library/mod.rs#L87)

- Cùng lỗi, chỗ thứ hai — thứ `grep err.project.` không thể bắt.
  [`chapter.rs:65`](../../src-tauri/src/commands/chapter.rs#L65)

- Khoá lộ ra ngoài cũng là tên thực thể, nên đổi cùng lượt.
  [`i18n/mod.rs:156`](../../src-tauri/src/core/i18n/mod.rs#L156)

- Đầu kia của khoá; `check-i18n.mjs` đỏ nếu lệch một bên.
  [`vi.json:14`](../../src/i18n/vi.json#L14)

**Ba mệnh đề trước nay chỉ sống trong chú thích**

- Không thực thể tầng ba — tách hàm thuần để gieo được vi phạm giả.
  [`project_contract.rs:959`](../../src-tauri/tests/project_contract.rs#L959)

- Bất biến `source_lang` thành phép đo, không còn là lời khai trong `schema.rs`.
  [`project_contract.rs:1217`](../../src-tauri/tests/project_contract.rs#L1217)

- Glossary/TM phân giải thật qua `ScopeResolver`, không chỉ đọc bảng tĩnh.
  [`project_contract.rs:1369`](../../src-tauri/tests/project_contract.rs#L1369)

**Phần đỡ**

- Bộ tách literal Rust: xử literal ký tự, raw string, và dòng nối bắt đầu bằng `//`.
  [`project_contract.rs:1081`](../../src-tauri/tests/project_contract.rs#L1081)

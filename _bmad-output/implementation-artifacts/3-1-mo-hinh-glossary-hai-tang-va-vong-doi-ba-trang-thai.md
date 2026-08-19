---
title: 'Story 3.1 — Mô hình Glossary hai tầng và vòng đời ba trạng thái'
type: 'feature'
created: '2026-08-19'
status: 'done'
baseline_commit: 'a83a1b756abae08cbc0c8f5a595c1be722f0c95c'
review_loop_iteration: 2
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Không tồn tại bảng Glossary nào. Epic 4 (`RagInjector`) và mọi story còn lại của Epic 3 đều dựng trên một mô hình chưa có, và tầng Tác phẩm của `ScopeResolver` — đã cắm từ Story 1.15 — vẫn là state chết: `OpenWork.scope` không được đọc ở đâu trong `src-tauri/src/**`.

**Approach:** Dựng bảng `glossary_entry` với **một** hằng DDL dùng chung cho **hai** thang di trú (`global.db` bước 4, `project.db` bước 12), một module `core/glossary/` sở hữu SQL và điều kiện chèn, và phân giải hai tầng đi qua `ScopeResolver::apply_override`. Vòng đời khoá bằng **cấu trúc** chứ không bằng kỷ luật: *chờ chốt* ⟺ `translation IS NULL`, nên ca "đã chốt mà bản dịch rỗng" — đúng ca AD-36 sinh ra để chặn — không biểu diễn được.

## Boundaries & Constraints

**Always:**
- `translation IS NULL` **là** trạng thái *chờ chốt*. Không cột `status` song song — hai dữ kiện nói cùng một chuyện thì chúng lệch được, và lệch trong im lặng.
- Một hằng `GLOSSARY_ENTRY_DDL` duy nhất, dùng cho cả hai thang. Hai tầng không được phép có hai hình dạng.
- **Lọc SAU khi phân giải, không trước.** Một mục *chờ chốt* ở tầng Tác phẩm che một mục *đã chốt* ở tầng Global ⇒ thuật ngữ đó **không** đủ điều kiện chèn. Lọc trước sẽ chèn bản dịch toàn cục cho đúng thuật ngữ người dùng vừa cố ý để ngỏ.
- Vòng đời **một chiều** cưỡng chế bằng trigger SQL, không bằng quy ước ở tầng gọi.
- Module `core/glossary/` sở hữu điều kiện chèn (AD-36). Đây là chỗ **cố ý lệch** khỏi tiền lệ `core/segment/` (logic thuần, SQL ở `commands/`); tiền lệ đúng là `core/scope/store.rs`.
- Mục Glossary có bảng riêng. Một hàng `kind = 'glossary'` trong `config_value` bị `save_value` từ chối và đó là cố ý (`core/scope/store.rs:283-291`).
- Chuỗi literal trong `src-tauri/src/**` viết **không dấu**; `impl Display` là chẩn đoán cho log.

**Ask First:**
- Bất kỳ đề xuất nào thêm cột `status`, gộp hai thang di trú, hoặc cho một cơ chế tự động ghi thẳng vào `glossary_entry` (AD-20).
- Nếu việc bỏ `Copy` khỏi `ScopeError` làm đỏ nhiều hơn `scope_contract.rs`.

**Never:**
- Bảng chờ ứng viên — đó là Story 3.2. Trạng thái *ứng viên* **không** nằm trong `glossary_entry`.
- Bề mặt IPC, `#[tauri::command]`, khoá `MessageKey`/`vi.json`, adapter `src/config/*.ts`, hay bất kỳ mã Vue nào. Không màn hình ⇒ không khoá chuỗi (`deferred-work.md:603`).
- Thêm `core/glossary` vào danh sách cho phép của `scope_boundary.rs`. Cổng giữ nguyên sức; chữ ký đổi thay vì cổng nới.
- Khớp thuật ngữ / stemming / đánh dấu ở lưới — Story 3.4.
- Sửa `epics.md` hay `prd.md` cho khớp mã.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Chỉ tầng Global, đã chốt | `global`: `慕容` → `Mộ Dung` | Trả về, **đủ điều kiện chèn** | — |
| Cả hai tầng, cả hai đã chốt | `global`: `慕容`→`Mộ Dung`; `work`: `慕容`→`Mộ Dong` | `Mộ Dong` — tầng Tác phẩm thắng theo từng thuật ngữ | — |
| Tầng Tác phẩm *chờ chốt* che tầng Global *đã chốt* | `global`: `慕容`→`Mộ Dung`; `work`: `慕容`→`NULL` | **KHÔNG** đủ điều kiện chèn | — |
| Chỉ tầng Global, chờ chốt | `global`: `青丘`→`NULL` | Có mặt khi liệt kê; **KHÔNG** đủ điều kiện chèn | — |
| Không mở Tác phẩm nào | `work` = `None` | Phân giải bằng nguyên tầng Global | — |
| Bản dịch rỗng | `INSERT translation = ''` hoặc `'   '` | Từ chối | `CHECK` ⇒ `StoreError::WriteFailed` |
| Lùi vòng đời | `UPDATE` đưa `translation` non-NULL về `NULL` | Từ chối | trigger `RAISE(ABORT)` |
| Gõ sai tên loại | `apply_override("glosary", …)` | Từ chối, không phân giải gì | `ScopeError::UnknownKind` — lỗi lập trình, không qua IPC |
| Gọi sai ngữ nghĩa | `apply_merge("glossary", …)` | Từ chối | `ScopeError::WrongSemantics` (đã có) |
| Kho phiên bản mới hơn | `global.db` ở `user_version = 5` | Từ chối mở, không chạm một byte | `StoreError::SchemaTooNew` (đã có) |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/schema.rs` — `GLOBAL_MIGRATIONS` `:179-192` (đang ở bước **3**, kế tiếp **4**) · `PROJECT_MIGRATIONS` `:849+` (`[1,2,3,5,6,7,8,9,10,11]`, kế tiếp **12**; số **4 đã bị đốt**, không tái dùng) · `migrate()` `:1012-1066` · `target_version()` `:917-919` · `validate_strictly_increasing` `:927-945`. Khuôn hằng DDL: `SEGMENT_DDL` `:344-355` (gộp `CREATE TABLE` + `CREATE INDEX` trong một bước — tiền lệ ghi ở `:431`). Vết sẹo "sửa hằng cũ tại chỗ là sai" ghi ở `:358-368`.
- `src-tauri/src/core/scope/mod.rs` — `apply_override` `:234-245` · `apply_merge` `:267-278` · `resolve_global_only` `:285-296` · `with_work` `:214` · `ScopeError` `:124-140` (`derive(… Copy …)` `:123`). Lý do đặt tên `apply_*` ≠ `resolve_*` ở `:249-263`.
- `src-tauri/src/core/scope/kinds.rs:162` — `Glossary => "glossary" : Override` **đã khai sẵn**. Không thêm loại mới. `from_wire` `:138`.
- `src-tauri/src/core/scope/store.rs` — **tiền lệ đúng** cho `core/<miền>/store.rs` có SQL: `load_global_config` `:213-224`, `save_value` `:283+` (khuôn `&str` + `from_wire`, chính là hình dạng story này áp cho `apply_*`).
- `src-tauri/tests/scope_boundary.rs` — `FORBIDDEN_OUTSIDE_SCOPE` `:69-74` (`"ScopeKind"`, `"Semantics"`, `"resolve_override"`, `"resolve_merge"`), `SCOPE_DIR` `:40`, test `only_core_scope_may_name_the_two_tier_vocabulary` `:165`. **Chỉ đọc — không sửa danh sách.**
- `src-tauri/src/commands/project.rs:50,250-252` — `OpenWork.scope` dựng ở `create_work`, **không được đọc ở đâu**. `OpenWorkState` `:297`.
- `src-tauri/src/core/glossary/mod.rs` — hôm nay bốn dòng doc-comment, 0 mã.
- `src-tauri/src/core/store/mod.rs` — `Store::write` `:610-618` (một job = một giao dịch) · `Store::read` `:634-637` · `SchemaTooNew` `:326`.
- `src-tauri/tests/store_contract.rs:881` — `a_fresh_database_migrates_up_to_target_and_logs_it` khẳng định `global.db` = **3**. Đỏ có chủ đích.
- `src-tauri/tests/segment_contract.rs:506` — `the_project_migration_set_matches_the_declared_ladder_step_for_step` khẳng định thang **nguyên văn**. Đỏ có chủ đích. `:474` (`…never_reuses_the_burned_number_four`) phải **giữ xanh**.
- `src-tauri/tests/scope_contract.rs` — ~10 chỗ gọi ba hàm đổi chữ ký (`:227,267,298,323,361,396,417,439,503,522,767`); `ScopeError` khớp mẫu ở `:450,464,477,490,514`.
- `_bmad-output/implementation-artifacts/deferred-work.md` — `:272` (cổng `ScopeKind`, chủ = story này) · `:601` (tầng Work chưa có consumer) · `:2465` (không có đường mở lại `.atproj`).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/scope/mod.rs` -- đổi `apply_override`/`apply_merge`/`resolve_global_only` sang nhận `kind: &str`, phân giải nội bộ bằng `ScopeKind::from_wire`; thêm `ScopeError::UnknownKind { wire: String }` và **bỏ `Copy`** (giữ `Clone, Debug, PartialEq, Eq`); `impl Display` cho biến thể mới, viết **không dấu** -- đóng nợ `deferred-work.md:272` bằng cách sửa nguồn thay vì nới cổng; module miền không bao giờ gõ tên kiểu `ScopeKind`.
- [x] `src-tauri/src/core/store/schema.rs` -- thêm `GLOSSARY_ENTRY_DDL` (bảng + chỉ mục UNIQUE trên `source_term` + trigger một chiều), nối làm **bước 4** của `GLOBAL_MIGRATIONS` và **bước 12** của `PROJECT_MIGRATIONS`, cùng **một** hằng -- một hằng là thứ làm cho "hai tầng cùng hình dạng" đúng theo định nghĩa thay vì nhờ hai chỗ tình cờ đồng ý.
- [x] `src-tauri/src/core/glossary/entry.rs` -- kiểu thuần: `GlossaryEntry`, `Category`, `TermOrigin`, vị từ `is_confirmed()` suy từ `translation.is_some()` -- không cột `status`, nên vị từ là chỗ DUY NHẤT định nghĩa "đã chốt".
- [x] `src-tauri/src/core/glossary/store.rs` -- SQL: `insert_entry`, `confirm_translation`, `load_tier` (→ `BTreeMap<String, GlossaryEntry>`), và **đúng một** hàm phơi ra `entries_eligible_for_injection(resolver, global, work)` phân giải rồi mới lọc -- điều kiện chèn ở nơi sở hữu dữ liệu (AD-36).
- [x] `src-tauri/src/core/glossary/mod.rs` -- khai hai submodule, giữ nguyên doc-comment cũ, thêm một mục **GIỚI HẠN THẬT** nêu rằng tầng Tác phẩm chưa đọc lại được sau khởi động lại.
- [x] `src-tauri/tests/scope_contract.rs` -- cập nhật ~10 chỗ gọi sang `&str`; thêm ca `UnknownKind` -- chữ ký đổi thì hợp đồng phải nói ra.
- [x] `src-tauri/tests/store_contract.rs` -- sửa `3` → `4` ở `:881` kèm 🔵 + ngày + lý do -- mệnh đề hết đúng thì sửa tại chỗ. *(Lượt sửa lộ thêm hai chỗ hardcode khác trong cùng tệp — `on_disk == 3` và bảng `glossary_entry` chưa được xác nhận có mặt — cả hai sửa cùng lượt.)*
- [x] `src-tauri/tests/segment_contract.rs` -- thêm `12` vào thang khai ở `:506` kèm 🔵 + ngày; **không** đụng `:474`. *(Nâng target kéo theo ~10 chỗ hardcode `schema_version() == 11` khác trong cùng tệp — mỗi `Store::open(StoreSpec::project(..))` không dùng fixture cắt đều đi tới target mới; sửa hết cùng lượt, kèm fixture "phiên bản mới hơn" nâng từ 12 lên 13.)*
- [x] `src-tauri/tests/glossary_contract.rs` -- mới: mọi hàng của I/O Matrix, mỗi tên hàm là một CÂU khẳng định -- ca "chờ chốt che đã chốt" là ca dễ cài ngược nhất.
- [x] `src-tauri/tests/glossary_boundary.rs` -- mới: không tệp nào ngoài `core/glossary/**` và `core/store/schema.rs` được mang chuỗi `glossary_entry` -- cưỡng chế "`ai/` không có đường nào khác chạm dữ liệu Glossary" trước khi Epic 4 tồn tại.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối `✅ ĐÃ ĐÓNG 2026-08-19 (Story 3.1)` vào `:272`; nối 🟡 vào `:601` (hàm tiêu thụ đã có, **chưa** có chỗ gọi sản phẩm — chủ chuyển sang Story 3.3); thêm mục mới **chủ = Epic 5** cho việc `project.db` chưa mở lại được -- không mục nào mồ côi.

*(Ngoài phạm vi khai ban đầu, cùng lượt vì cùng nguyên nhân: `src-tauri/tests/pinned_contract.rs` cũng hardcode `GLOBAL_MIGRATIONS.len() == 3` và `schema_version() == 3`/`11` — sửa theo cùng luật 🔵 + ngày.)*

**Acceptance Criteria:**
- Given một `global.db` mới tinh, when mở, then `user_version = 4` và `schema_migration_log` có đúng một hàng cho bước 4.
- Given một `project.db` mới tinh, when mở, then thang đã chạy là `[1,2,3,5,6,7,8,9,10,11,12]` và số 4 vẫn không xuất hiện.
- Given `core/glossary/**`, when `grep` toàn `src-tauri/src/**`, then không tệp nào ngoài `core/scope/**` mang token `ScopeKind`, và `scope_boundary.rs` xanh **mà danh sách cấm không đổi một dòng**.
- Given `core/glossary/`, when đếm bề mặt phơi ra cho module khác, then có **đúng một** hàm trả mục đủ điều kiện chèn.
- Given hai tầng cùng khai một thuật ngữ, when phân giải, then đi qua `ScopeResolver::apply_override` — không truy vấn `JOIN`/`UNION` nào tự cài lại luật "tầng Tác phẩm thắng".
- Given cả bộ, when chạy `.githooks/pre-push`, then mười một cổng + vitest + build + `cargo test --locked` đều xanh.

## Spec Change Log

### 2026-08-19 — vòng rà soát #1

**Phát hiện kích hoạt.** `trim()` của SQLite chỉ cắt **dấu cách ASCII**. Biểu thức
`CHECK (translation IS NULL OR trim(translation) <> '')` mà chính §Design Notes của spec này
viết sẵn **không** chặn được tab, xuống dòng, NBSP hay U+3000 — đo 2026-08-19 trên SQLite
3.53.4: `"   "` bị chặn, `"\t"` · `"\n"` · `" "` · `"　"` đều **lọt**.

**Đã sửa gì.** §Design Notes nay mang biểu thức đã đo, áp cho **cả** `translation` **và**
`source_term` (cột sau trước đây không có rào rỗng nào). Bảng ký tự khoảng trắng viết khai
triển tại chỗ, **không** đặt tên hằng — `Migration::sql` là `&'static str` và `concat!` chỉ
nhận literal.

**Trạng thái hỏng đã tránh.** Một bản dịch `"\t"` làm `is_confirmed()` trả `true` với nội
dung trắng ⇒ `RagInjector` (Epic 4) chèn một trường trống vào prompt, **đúng** ca AD-36 sinh
ra để chặn — và §Intent của spec này tuyên bố ca đó "không biểu diễn được". Tuyên bố ấy sai
cho tới lượt sửa này. Không cổng nào đỏ vì chuyện đó.

**KEEP — những thứ đã đúng, phải sống sót mọi lượt dựng lại:**
- **Một** hằng `GLOSSARY_ENTRY_DDL` cho **hai** thang di trú. Đừng tách thành hai hằng.
- Lọc **sau** khi phân giải trong `entries_eligible_for_injection`. Ca `a_pending_work_tier_entry_shadows_and_disqualifies_a_confirmed_global_entry` là ca dễ cài ngược nhất và nó đang đúng.
- `git diff` trên `scope_boundary.rs` **rỗng** — cổng được giữ nguyên sức bằng cách đổi chữ ký, không bằng cách nới danh sách.
- Sổ nợ chỉ **nối thêm**, không xoá; mục nửa đóng ghi 🟡 chứ không làm tròn lên ✅.
- Mọi neo số phiên bản đổi đều kèm 🔵 + ngày + lý do, và mệnh đề của từng ca test không đổi.

### 2026-08-19 — vòng rà soát #2

**Phát hiện kích hoạt.** Bảng ký tự của lượt rà soát #1 có **bảy** ký tự và dừng ở đó — nó
**thu hẹp** lỗ hổng chứ không đóng. Đo lại từng điểm một: bảng bảy ký tự vẫn để **17** điểm
mã `White_Space` khác đi lọt — U+0085 · U+1680 · U+2000‥U+200A (gồm U+2009 THIN SPACE) ·
U+2028 · U+2029 · U+202F · U+205F. Tức tuyên bố *"ca đã chốt mà bản dịch rỗng không biểu
diễn được"* của §Intent vẫn còn sai sau lượt #1, chỉ sai hẹp hơn.

**Đã sửa gì.** Cả hai `CHECK` của `GLOSSARY_ENTRY_DDL` nay liệt **trọn** 25 điểm mã
`White_Space` — đúng tập mà `str::trim()` của Rust cắt, nên hai lớp phòng thủ lần này thật
sự cùng một tập. `seven_blank_forms()` đổi tên thành `every_blank_form()` và nhận **cả 25**
điểm mã cộng một chuỗi trộn. Comment ở `insert_entry` sửa lại quan hệ hai lớp: bản trước
viết "cùng tập ký tự", nhưng lúc đó Rust là tập cha thực sự và chính lớp Rust — không phải
`CHECK` — mới đang đóng 17 điểm còn lại.

**Cửa sổ di trú đã dùng lần thứ hai, và không có lần thứ ba.** Vẫn chưa phát hành nên sửa
hằng tại chỗ còn hợp lệ. Ghi ra ở doc-comment của hằng: sau bản phát hành đầu tiên chạm bước
4/12, mọi lượt sửa bảng ký tự phải là một bước di trú MỚI.

**Khoảng trống nghiệm thu đã đóng cùng lượt** — năm hành vi trước đó xoá đi mà bộ test vẫn
xanh: `trim()` tầng Rust cho `source_term` và cho `confirm_translation`; nhánh `Err` của
`decode_category`/`decode_term_origin`; ba biến thể enum chưa từng ghi xuống đĩa
(`DomainTerm` · `ImportScan` · `ReviewHarvest`); năm cột chưa từng được khẳng định sau
round-trip (`note` · `category` · `term_origin` · `created_at` · `id`).

**KEEP — bổ sung cho danh sách của lượt #1:**
- Bảng ký tự phải khớp `char::is_whitespace` của Rust. Thêm ký tự vào một lớp thì thêm vào
  lớp kia CÙNG LƯỢT.
- `every_blank_form()` là bằng chứng CHẠY ĐƯỢC cho bảng đó. Đừng rút ngắn nó về một mẫu
  đại diện.

## Design Notes

Hình dạng bảng — hai vế cưỡng chế bằng SQL, không bằng kỷ luật:

```sql
CREATE TABLE glossary_entry (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  source_term  TEXT    NOT NULL,
  translation  TEXT,                     -- NULL == cho chot (FR114)
  note         TEXT    NOT NULL DEFAULT '',
  category     TEXT    NOT NULL,         -- person | place | domain_term | other
  term_origin  TEXT    NOT NULL,         -- manual | import_scan | review_harvest
  created_at   TEXT    NOT NULL,         -- ISO-8601 UTC
  CHECK (trim(source_term, WS) <> ''),
  CHECK (translation IS NULL OR trim(translation, WS) <> ''),
  CHECK (category    IN ('person','place','domain_term','other')),
  CHECK (term_origin IN ('manual','import_scan','review_harvest'))
);
-- WS = ' ' || char(9) || char(10) || char(13) || char(11) || char(12)
--        || char(160) || char(12288)      -- viet khai trien tai cho, khong dat ten
CREATE UNIQUE INDEX idx_glossary_entry_source_term ON glossary_entry (source_term);
CREATE TRIGGER glossary_entry_lifecycle_is_one_way
BEFORE UPDATE OF translation ON glossary_entry
WHEN OLD.translation IS NOT NULL AND NEW.translation IS NULL
BEGIN SELECT RAISE(ABORT, 'glossary lifecycle is one-way'); END;
```

🔴 **`term_origin`, không phải `origin` trần** — chữ *"xuất xứ"* chỉ **bốn** thực thể rời nhau trong dự án này (bản dịch · mục Glossary · tài liệu nguồn · trích dẫn từ điển), và `segment.translation_origin` đã lấy khuôn `<mô tả cái gì>_origin` (`schema.rs:691,749`).

⚠️ **`CHECK` bắt chuỗi rỗng là cố ý, và nó KHÁC `segment.target_text`.** Ở `segment`, *"chưa dịch"* là chuỗi **rỗng** `NOT NULL DEFAULT ''` (`schema.rs:749`) — vì mọi segment luôn tồn tại. Ở đây ngược lại: vắng mặt là một trạng thái **có nghĩa**, nên `NULL` mang nghĩa và chuỗi rỗng bị cấm. Hai bảng chọn ngược nhau có lý do, đừng "đồng bộ" chúng.

## Verification

**Commands:**
- `cd src-tauri && cargo test --locked` -- expected: xanh toàn bộ, gồm `glossary_contract`, `glossary_boundary`, `scope_boundary`, `scope_contract`, `store_contract`, `segment_contract`.
- `npm run check:i18n` -- expected: xanh; Kiểm A không thấy chữ có dấu ở vị trí mã trong `core/glossary/**`.
- `npm run check:gates` -- expected: xanh; story này **không** thêm cổng nào nên ba danh sách không đổi.
- `npm run check:deps` -- expected: xanh; không phụ thuộc mới (NFR15 không kích hoạt).

**Manual checks (if no CLI):**
- `git diff` trên `src-tauri/tests/scope_boundary.rs` phải **rỗng** — nếu có một dòng, cổng đã bị nới thay vì nguồn được sửa.
- `dict-manifest.toml` phải **không đổi**: bốn tệp `.db` từ điển không liên quan tới lược đồ `global.db`/`project.db`.

## Suggested Review Order

**Bất biến trung tâm — điều kiện chèn**

- Điểm vào: hàm DUY NHẤT module khác gọi được; lọc SAU khi phân giải.
  [`store.rs:280`](../../src-tauri/src/core/glossary/store.rs#L280)

- Vị từ DUY NHẤT định nghĩa "đã chốt" — suy từ `translation`, không từ cột `status`.
  [`entry.rs:132`](../../src-tauri/src/core/glossary/entry.rs#L132)

- Hai họ lỗi giữ phân biệt được: `ScopeError` không bao giờ qua IPC, `StoreError` thì có.
  [`store.rs:229`](../../src-tauri/src/core/glossary/store.rs#L229)

**Lược đồ — một hằng, hai thang**

- DDL cùng hai `CHECK` khoảng trắng đã đo; bảng ký tự viết khai triển tại chỗ.
  [`schema.rs:277`](../../src-tauri/src/core/store/schema.rs#L277)

- Bước 4 của `global.db`.
  [`schema.rs:327`](../../src-tauri/src/core/store/schema.rs#L327)

- Bước 12 của `project.db` — không phải 5; số 4 đã bị đốt.
  [`schema.rs:1060`](../../src-tauri/src/core/store/schema.rs#L1060)

**Phân giải hai tầng — sửa nguồn thay vì nới cổng**

- Chữ ký nhận `&str`, nên module miền không bao giờ gõ tên kiểu `ScopeKind`.
  [`mod.rs:281`](../../src-tauri/src/core/scope/mod.rs#L281)

- `ScopeError` mất `Copy` vì biến thể mới mang `String` — lý do ghi tại chỗ.
  [`mod.rs:124`](../../src-tauri/src/core/scope/mod.rs#L124)

**Cưỡng chế — hai cổng mới**

- Chỉ `core/glossary/**` và `schema.rs` được mang tên bảng.
  [`glossary_boundary.rs:153`](../../src-tauri/tests/glossary_boundary.rs#L153)

- Vế hai của AD-36: `ai/` không có đường nào khác chạm dữ liệu Glossary.
  [`glossary_boundary.rs:253`](../../src-tauri/tests/glossary_boundary.rs#L253)

**Test — hai ca đáng đọc nhất**

- Ca dễ cài ngược nhất của cả story: chờ chốt che đã chốt.
  [`glossary_contract.rs:155`](../../src-tauri/tests/glossary_contract.rs#L155)

- Ca từng xanh giả: tên khẳng định khoảng trắng, thân chỉ thử dấu cách.
  [`glossary_contract.rs:301`](../../src-tauri/tests/glossary_contract.rs#L301)

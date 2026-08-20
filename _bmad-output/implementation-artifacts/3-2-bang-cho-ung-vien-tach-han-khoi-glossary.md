---
title: 'Story 3.2 — Bảng chờ ứng viên tách hẳn khỏi Glossary'
type: 'feature'
created: '2026-08-20'
status: 'done'
baseline_commit: 'e6dee97da65368afe2c973cdfbba0e0a006a2711'
review_loop_iteration: 1
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-1-mo-hinh-glossary-hai-tang-va-vong-doi-ba-trang-thai.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR55 (*"không cơ chế nào được tự ghi vào Glossary"*) hôm nay chỉ là kỷ luật. `insert_entry` nhận `term_origin: TermOrigin` từ **nơi gọi**, nên một module quét (Story 3.5) hay thu hoạch (Epic 8) chỉ cần truyền `TermOrigin::ImportScan` là ghi thẳng vào `glossary_entry` — biên dịch sạch, qua cả mười một cổng. Bảng chờ ứng viên mà AD-20 đòi thì chưa tồn tại.

**Approach:** Dựng bảng `glossary_candidate` (bước **13** của `PROJECT_MIGRATIONS`) và **thu hẹp chữ ký** để vi phạm không biểu diễn được: `insert_entry` mất tham số `term_origin` và thành `insert_manual_entry` (luôn `manual`); `term_origin` khác `manual` chỉ sinh ra được bên trong `approve_candidate`, suy từ `candidate_origin` của chính hàng ứng viên. *Chờ duyệt* ⟺ `resolution IS NULL` — cùng khuôn cấu trúc mà Story 3.1 dùng cho `translation IS NULL`.

## Boundaries & Constraints

**Always:**
- `resolution IS NULL` **là** trạng thái *chờ duyệt*. Không cột `is_pending` song song.
- Hàng ứng viên **ở lại** sau khi duyệt/bỏ — đó là thứ làm "không quay lại ở lần quét sau" đúng theo cấu trúc. `UNIQUE(source_term)` là cơ chế chặn, không phải một phép kiểm ở tầng gọi.
- Không suy "đã duyệt" từ sự tồn tại của hàng trong `glossary_entry`: Story 3.9 cho xoá mục Glossary, và một suy diễn chéo bảng sẽ làm ứng viên đã duyệt sống lại trong im lặng.
- `approve_candidate` đặt `resolution` **và** chèn `glossary_entry` trong **một** giao dịch `store.write`.
- Bảng ký tự khoảng trắng của `CHECK` phải **trùng từng byte** với bảng trong `GLOSSARY_ENTRY_DDL` (25 điểm mã `White_Space`), viết khai triển tại chỗ — `Migration::sql` là `&'static str`.
- Vòng đời một chiều cưỡng chế bằng trigger SQL.
- Chuỗi literal trong `src-tauri/src/**` viết **không dấu**; không `FOREIGN KEY` (`schema.rs:444`).

**Ask First:**
- Bất kỳ đề xuất nào cho `approve_candidate` ghi sang `global.db` (giao dịch xuyên hai kho — không nguyên tử).
- Đổi khoá duy nhất khỏi `source_term` trần.
- Nếu việc bỏ tham số `term_origin` làm đỏ nhiều hơn `glossary_contract.rs` + `glossary_boundary.rs`.

**Never:**
- Cột `số lần xuất hiện`, `ví dụ ngữ cảnh` (Story 3.5) · `bản dịch đề xuất` (3.7) · `phân loại`, `con trỏ đang duyệt` (3.8) · `tỉ lệ nhất quán` (Epic 8). Bảng `segment` nhận sáu bước ALTER rải khắp Epic 2 — đó là tiền lệ, không phải thiếu sót.
- Bảng ứng viên ở `global.db`. Ứng viên sinh ra từ một Tác phẩm và AC khoá theo Tác phẩm.
- Bề mặt IPC, `#[tauri::command]`, khoá `MessageKey`/`vi.json`, adapter `src/config/*.ts`, mã Vue. Không màn hình ⇒ không khoá chuỗi.
- Sửa `epics.md`/`prd.md` cho khớp mã.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Quét sinh ứng viên mới | `insert_candidate("慕容", ImportScan)` | Hàng mới, `resolution` NULL, có trong `pending_candidates` | — |
| Quét lại chuỗi **đã bỏ** | `resolution = 'rejected'`, quét lại `insert_candidate("慕容", …)` | Từ chối — không quay lại bảng chờ | UNIQUE ⇒ `StoreError::WriteFailed` |
| Quét lại chuỗi **đã duyệt** | `resolution = 'approved'`, quét lại | Từ chối, cùng một đường | UNIQUE ⇒ `WriteFailed` |
| Duyệt một ứng viên | `approve_candidate(id, Some("Mộ Dung"), Person)` | `resolution='approved'` **và** `glossary_entry` mang `term_origin='import_scan'` | — |
| Duyệt ứng viên thu hoạch | ứng viên `review_harvest` | `glossary_entry.term_origin='review_harvest'` — cùng bảng chờ, không bảng thứ hai | — |
| Duyệt để ngỏ bản dịch | `approve_candidate(id, None, …)` | Mục Glossary ở *chờ chốt*; không đủ điều kiện chèn | — |
| Bỏ một ứng viên | `reject_candidate(id)` | Rời `pending_candidates`, hàng còn trên đĩa | — |
| Lùi vòng đời | `UPDATE` đưa `resolution` về NULL | Từ chối | trigger `RAISE(ABORT)` |
| Duyệt/bỏ `id` không có | `approve_candidate(999, …)` | Không ghi gì, **không** im lặng báo thành công | `StoreError::WriteFailed` |
| Ứng viên rỗng | `insert_candidate("\u{3000}", …)` | Từ chối | `CHECK` ⇒ `WriteFailed` |
| Ghi `term_origin` phi-manual từ ngoài | `insert_manual_entry(…)` | Không biểu diễn được — tham số đã biến mất | lỗi biên dịch |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/schema.rs` — `GLOSSARY_ENTRY_DDL` `:294-325` (bảng ký tự WS để chép); doc-comment `:292-293` mang mệnh đề *"Không bảng ứng viên — đó là Story 3.2"* → **hết đúng sau story này, sửa tại chỗ kèm 🔵 + ngày**. `:285-288` khẳng định `term_origin` không có giá trị thứ tư. `PROJECT_MIGRATIONS` `:1024-1093` (bước cuối **12**, số 4 đã đốt — vết sẹo `:993-1023`); khuôn một bước: `:1037-1041`. `GLOBAL_MIGRATIONS` `:341-360` — **không đụng**. `Migration` `:65-70` · `target_version` `:1099` · `migrate` `:1194`. `created_at` sinh ở tầng SQL bằng `strftime`, không truyền từ Rust (`:289-290`). Không `FOREIGN KEY` ở bất kỳ đâu (`:444`).
- `src-tauri/src/core/glossary/entry.rs` — `Category` `:20-29` + `as_str` `:34` + `from_wire` `:49` · `TermOrigin` `:71-78` (`Manual`/`ImportScan`/`ReviewHarvest`) `as_str` `:82` `from_wire` `:92` · `GlossaryEntry` `:112-128` · `is_confirmed` `:132-134`. Khuôn enum để chép cho `CandidateOrigin`/`Resolution`.
- `src-tauri/src/core/glossary/store.rs` — `insert_entry` `:66-114` (**đổi chữ ký**; `.trim()` ở `:99-100`, doc-comment 25 điểm mã `:74-98`) · `confirm_translation` `:142-154` · `load_tier` `:159-187` · `decode_category` `:202` / `decode_term_origin` `:215` (riêng tư, khuôn để chép) · `GlossaryError` `:239-270` · `entries_eligible_for_injection` `:290-312` · `GLOSSARY_SCOPE_KIND` `:57`.
- `src-tauri/src/core/glossary/mod.rs` — doc-comment `:1` **đã khai sẵn** *"+ bảng chờ ứng viên TÁCH RIÊNG"* · `pub mod` `:33-34` · re-export `:36-39` · §GIỚI HẠN THẬT `:21-31` (tầng Tác phẩm chưa đọc lại được sau khởi động lại — chủ Epic 5; áp thẳng lên bảng chờ).
- `src-tauri/src/core/store/mod.rs` — `Store::write` `:622-628` · `Store::read` `:642-647` · `StoreError` `:309-375` · `StoreSpec::project` `:288-295` · `ReadHandle` `:138`.
- `src-tauri/tests/glossary_boundary.rs` — `FORBIDDEN = "glossary_entry"` `:55` (**chuỗi cứng — không bắt được tên bảng mới**) · `GLOSSARY_ONLY_SURFACE` `:74` · `GLOSSARY_DIR`/`SCHEMA_FILE` `:29-36` · `RS_FLOOR = 38` `:45` · đối chứng dương `:199`, `:223` · test AD-36 `:253`.
- `src-tauri/tests/glossary_contract.rs` — helper `open_project` `:57`, **`every_blank_form()` `:296`** (27 biến thể, tái dùng — đừng viết lại) · `every_category_and_term_origin_variant_round_trips_through_the_store` `:825` (**sẽ đỏ**: chèn cả ba biến thể qua `insert_entry`).
- **Sẽ đỏ vì nâng target 12 → 13** *(đã liệt kê đủ, đừng để sót như lượt 3.1)*: `segment_contract.rs:516` (thang nguyên văn) · `:577,790,908,1201,1306,1508,5358` (**tám** chỗ `schema_version() == 12` — đo lại bằng `grep -n "^        12,$"`, danh sách bảy dòng này thiếu một) · `:1580-1608` `STEP_THIRTEEN` — **panic ngầm, không phải lỗi biên dịch**: mảng `[Migration; 12]` + bước giả `to_version: 13` nay trùng target thật ⇒ `Store::open` hết từ chối; phải lên `[Migration; 13]` + bước giả `14` + đổi tên. `pinned_contract.rs:187` (`len() == 11`) · `:194` (`== 12`).
- **Đối chứng phải GIỮ XANH** (không đụng `GLOBAL_MIGRATIONS`): `pinned_contract.rs:74,81,127` · `store_contract.rs:891,919` · `glossary_contract.rs:534` · `segment_contract.rs:474`.
- Không cổng `check:*` nào đọc thang di trú — vùng đỏ nằm trọn ở `cargo test --locked`.
- `_bmad-output/implementation-artifacts/deferred-work.md` — `:5328` (`translation` là trường `pub`, chủ Epic 4) · `:5358` (`GLOSSARY_ONLY_SURFACE` khớp chuỗi con tên hàm trần — **đổi tên hàm ở story này chạm đúng mục này**) · `:5365` (trigger không canh `INSERT OR REPLACE`, chủ 3.9) · `:5306` (chuẩn hoá `source_term` để lại cho 3.4).

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/store/schema.rs` -- thêm `GLOSSARY_CANDIDATE_DDL` (bảng + `UNIQUE INDEX` trên `source_term` + trigger một chiều trên `resolution`), nối làm **bước 13** của `PROJECT_MIGRATIONS`; sửa mệnh đề `:292-293` kèm 🔵 + ngày -- một mệnh đề hết đúng mà ở lại là đúng lớp nợ cổng sinh ra để chống.
- [ ] `src-tauri/src/core/glossary/candidate.rs` -- mới, kiểu thuần: `GlossaryCandidate`, `CandidateOrigin` (**hai** biến thể — `Manual` không biểu diễn được vì mục nhập tay không đi qua bảng chờ), `Resolution`, vị từ `is_pending()` suy từ `resolution.is_none()`, và `CandidateOrigin → TermOrigin` là hàm **toàn phần** -- ánh xạ toàn phần là thứ làm AC "mục vừa duyệt mang đúng xuất xứ" đúng theo kiểu chứ không theo kỷ luật.
- [ ] `src-tauri/src/core/glossary/candidate_store.rs` -- mới: `insert_candidate`, `pending_candidates`, `approve_candidate` (một giao dịch: đặt `resolution` + chèn `glossary_entry`), `reject_candidate`; cả hai hàm sau trả `WriteFailed` khi `id` không tồn tại -- `confirm_translation` đã có một ca "0 hàng vẫn `Ok`" (`store.rs:133-141`), đừng nhân bản nó.
- [ ] `src-tauri/src/core/glossary/candidate_store.rs` -- lượt `query_row` mở đầu của **cả hai** hàm đọc thêm cột `resolution`; đã có giá trị ⇒ trả lỗi **phân biệt được** với "id không tồn tại", không ghi gì -- lớp Rust cho một lỗi đọc được, lớp trigger cho bảo đảm; cùng khuôn hai lớp mà `trim()`/`CHECK` đã dùng ở Story 3.1.
- [ ] `src-tauri/src/core/glossary/store.rs` -- đổi `insert_entry` → `insert_manual_entry`, **bỏ** tham số `term_origin`, luôn ghi `manual`; tách phần chèn dùng chung thành helper riêng tư nhận `&Transaction` để `approve_candidate` gọi lại -- đây là vế cấu trúc của FR55: đường ghi phi-manual chỉ còn một cửa.
- [ ] `src-tauri/src/core/glossary/mod.rs` -- khai submodule mới, cập nhật re-export, và ghi vào §GIỚI HẠN THẬT rằng duyệt một ứng viên chỉ ghi được vào **tầng Tác phẩm** (đẩy lên Global là 3.9, vì hai kho không có giao dịch chung).
- [ ] `src-tauri/tests/glossary_boundary.rs` -- `FORBIDDEN` thành danh sách (`glossary_entry` + `glossary_candidate`); cập nhật `GLOSSARY_ONLY_SURFACE` theo tên hàm mới; thêm phép kiểm: token `ImportScan`/`ReviewHarvest` chỉ xuất hiện trong `core/glossary/**` -- cổng cũ so **chuỗi cứng** nên một bảng tên khác đi lọt; vá lỗ đó cùng lượt thay vì để nó chờ Epic 8.
- [ ] `src-tauri/tests/glossary_contract.rs` -- mọi hàng của I/O Matrix, tên hàm là một CÂU khẳng định; tái dùng `every_blank_form()` `:296`; viết lại `:825` cho hợp chữ ký mới (ba biến thể `TermOrigin` nay đi qua **hai** cửa) -- ca "quét lại chuỗi đã bỏ" là ca dễ cài ngược nhất.
- [ ] `src-tauri/tests/glossary_contract.rs` -- ba ca nữa mà lượt trước bỏ trống: `approve_candidate` hỏng vì `CHECK` bản dịch trắng (doc-comment nêu ca này nhưng chỉ ca đụng `UNIQUE` được kiểm) · `insert_candidate` với `source_term` có đệm va vào bản đã trim (Story 3.1 có ca song song cho `insert_manual_entry`) · mỗi biến thể `Resolution` đi vòng qua `decode_row` -- ba ca này xoá đi mà bộ test vẫn xanh, tức chúng chưa được ai canh.
- [ ] `src-tauri/tests/glossary_contract.rs` -- thêm phép kiểm chéo: bảng ký tự WS trong hai hằng DDL **trùng từng byte** -- cùng khuôn `han_ranges_are_verbatim_from_dict_build_char_idx`; hai bản chép không có cổng là hai bản chép sẽ lệch.
- [ ] `src-tauri/tests/segment_contract.rs` -- thang `:516`, **tám** chỗ `== 12`, và `STEP_THIRTEEN` `:1580-1608`; mỗi chỗ kèm 🔵 + ngày; **không** đụng `:474`. *(Tám chỗ `== 12`, không bảy — vết sẹo `PROJECT_MIGRATIONS` số 4 làm thang lệch một; xem Completion Notes.)*
- [ ] `src-tauri/tests/pinned_contract.rs` -- `:187` và `:194` kèm 🔵 + ngày.
- [ ] `_bmad-output/implementation-artifacts/deferred-work.md` -- thêm bốn mục nữa, mỗi mục MỘT chủ (`check:debt-owner` đỏ với mục mồ côi): **chủ = Story 3.9** — `DELETE` rồi `INSERT` lại cùng `source_term` đặt `resolution` về NULL, đúng lỗ mà mục `:5365` đã ghi cho trigger của `glossary_entry`; **chủ = Story 3.5** — một ứng viên trùng `source_term` với mục Glossary có sẵn thì không bao giờ duyệt được và nằm lại bảng chờ vĩnh viễn, chỗ chặn đúng là lượt quét (`epics.md:2984-2985`); **chủ = Story 3.8** — `ORDER BY source_term` là đối chiếu byte, vô nghĩa với chữ Hán và tiếng Việt, và `WHERE resolution IS NULL` chưa có chỉ mục; **chủ = story dựng chỗ gọi sản phẩm đầu tiên** — bốn hàm `candidate_store` chưa vào `GLOSSARY_ONLY_SURFACE` vì chưa có chỗ gọi, hôm nay chỉ một doc-comment nói ra.
- [ ] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối mục mới **chủ = Story 8.14**: khoá duy nhất theo **chuỗi** ở story này hẹp hơn luật "cùng cặp X→Y" mà `epics.md:6115-6117` đòi, nên một bản dịch thu hoạch khác cho chuỗi đã bị bỏ sẽ không đề xuất được; và mục **chủ = Story 3.8** cho `resolution` chưa mang thời điểm -- không mục nào mồ côi.

**Acceptance Criteria:**
- Given một `project.db` mới tinh, when mở, then thang đã chạy là `[1,2,3,5,6,7,8,9,10,11,12,13]`, số 4 vẫn vắng, và `global.db` vẫn dừng ở **4**.
- Given toàn `src-tauri/src/**`, when `grep`, then không tệp nào ngoài `core/glossary/**` và `core/store/schema.rs` mang `glossary_candidate`, và không tệp nào ngoài `core/glossary/**` mang `ImportScan`/`ReviewHarvest`.
- Given một nơi gọi bất kỳ ngoài `core/glossary/**`, when nó muốn ghi một mục Glossary, then chữ ký duy nhất nó gọi được không nhận `term_origin` — đường ghi phi-manual **không biên dịch được**.
- Given `approve_candidate` chạy nửa chừng thất bại, when kiểm, then cả `resolution` lẫn `glossary_entry` đều không đổi (một giao dịch).
- Given một ứng viên **đã bỏ**, when gọi `approve_candidate` trên đúng `id` đó, then bị từ chối và **không** hàng `glossary_entry` nào ra đời — đây là vế thứ hai của "ứng viên bị bỏ không quay lại", vế mà `UNIQUE(source_term)` **không** canh được vì nó chỉ chặn đường `insert_candidate`.
- Given một ứng viên **đã duyệt**, when gọi `reject_candidate` trên đúng `id` đó, then bị từ chối và mục Glossary đã sinh ra vẫn nguyên — hai bảng không bao giờ được nói ngược nhau.
- Given trigger `glossary_candidate_resolution_is_one_way`, when thử `UPDATE` một `resolution` non-NULL sang **bất kỳ** giá trị nào, then `RAISE(ABORT)` — kể cả sang chính giá trị cũ.
- Given cả bộ, when chạy `.githooks/pre-push`, then mười một cổng + vitest + build + `cargo test --locked` đều xanh.
- Given lượt CI sau khi push, when đọc, then cả nửa macOS lẫn nửa Windows xanh — `pre-push` chỉ chạy trên macOS của Ice.

## Spec Change Log

### 2026-08-20 — thực thi

**Phát hiện kích hoạt.** Code Map nói "bảy chỗ `schema_version() == 12`" trong
`segment_contract.rs`. Đo lại bằng `grep -n "^        12,$"`: **tám** chỗ, không bảy — chỗ
thứ tám ở ca `a_project_database_at_version_ten_backfills_the_origin_only_for_signed_rows`
(fixture dừng ở phiên bản 10, khẳng định đích sau di trú), không nằm trong danh sách bảy
dòng mà Code Map liệt (`:577,790,908,1201,1306,1508,5358`). Không sai kiến trúc — Code Map
chỉ đếm thiếu một dòng khi soạn. Sửa cả tám, cùng luật 🔵 + ngày.

**Đã sửa gì.** Cả tám chỗ `== 12` lên `== 13`; thang `:516` thêm `13`; `STEP_THIRTEEN`
(`[Migration; 12]`, đích giả `13`) đổi tên `STEP_FOURTEEN` (`[Migration; 13]`, đích giả
`14`).

### 2026-08-20 — vòng rà soát #1

**Phát hiện kích hoạt.** Ba lớp rà soát độc lập hội tụ vào cùng một chỗ. Trigger
`glossary_candidate_resolution_is_one_way` mà **chính §Design Notes của spec này viết sẵn**
chỉ bắn `WHEN OLD.resolution IS NOT NULL AND NEW.resolution IS NULL` — nó chặn chiều lùi về
*chờ duyệt* và **không** chặn chiều ngang. Đo trên mã đã dựng: `reject_candidate` rồi
`approve_candidate` trên cùng một `id` chạy sạch và sinh một hàng `glossary_entry` mới, tức
AC trung tâm *"ứng viên bị bỏ không quay lại"* chết — `UNIQUE(source_term)` chỉ canh đường
`insert_candidate`, không canh đường duyệt lại. Chiều ngược lại (`approve` rồi `reject`) để
lại `resolution='rejected'` cạnh một mục Glossary còn sống: hai bảng nói ngược nhau, không
lỗi nào ném ra.

**Đã sửa gì.** `WHEN` rút về `OLD.resolution IS NOT NULL` — đã quyết thì không quyết lại,
kể cả quyết lại y hệt. Thêm lớp Rust: `query_row` mở đầu của `approve_candidate` và
`reject_candidate` đọc thêm cột `resolution` và trả lỗi **phân biệt được** với *"id không
tồn tại"*. Ba AC mới nói ra cả ba chiều. Bốn món nợ mới có chủ và ba ca test còn trống được
gọi tên.

**Trạng thái hỏng đã tránh.** Một `id` ứng viên cũ đi qua lượt duyệt hàng loạt của Story 3.8
(hoặc một cú bấm đúp) đủ để phục sinh một thuật ngữ người dùng đã cố ý bỏ — và nó vào thẳng
đường ép AI qua `entries_eligible_for_injection`. Không cổng nào đỏ vì chuyện đó.

**KEEP — những thứ đã đúng ở lượt dựng đầu, phải sống sót lượt dựng lại:**
- Thu hẹp chữ ký `insert_entry` → `insert_manual_entry` (bỏ hẳn tham số `term_origin`) cộng
  helper `insert_entry_row` dùng chung. Đây là vế cấu trúc của FR55 và nó đã hoạt động.
- `CandidateOrigin` chỉ **hai** biến thể, `to_term_origin()` toàn phần.
- `FORBIDDEN` của `glossary_boundary.rs` thành danh sách hai tên bảng, cộng hai phép kiểm
  mới cho token `ImportScan`/`ReviewHarvest`, cộng hai đối chứng dương
  (`schema_rs_actually_declares_both_glossary_tables`,
  `core_glossary_actually_spells_both_non_manual_origin_tokens`).
- Phép kiểm chéo hai bảng ký tự WS trùng từng byte.
- `STEP_THIRTEEN` → `STEP_FOURTEEN` (`[Migration; 13]`, đích giả `14`) ở
  `segment_contract.rs:1580-1608` — đây là chỗ **panic ngầm**, không phải lỗi biên dịch.
- `git diff` trên `scope_boundary.rs` **rỗng**.
- Mười một tên hàm test của bảng chờ ở lượt đầu đều đúng khuôn một CÂU khẳng định; dựng lại
  thì giữ nguyên tên, đừng đặt lại.

**Một lỗi hình dạng phải sửa cùng lượt:** dòng assert ở `glossary_boundary.rs` trong
`only_glossary_and_schema_may_name_glossary_tables` thiếu dấu `\` nối dòng ở vế
``` `load_tier`/`insert_manual_entry`/`confirm_translation` cho `glossary_entry`; ``` nên
thông điệp panic nuốt một dòng mới cộng phần thụt đầu dòng.


### 2026-08-20 — vòng rà soát #2

**Không có mục nào đòi dựng lại.** Lượt sửa của vòng #1 đóng đúng chỗ hở: cả ba lớp đều
không tìm thấy `intent_gap` hay `bad_spec`. Bảy mục vá đã áp thẳng trên mã đang có.

**Một câu chữ trong §Tasks vẫn phải sửa cho khỏi bẫy người sau.** Task cổng biên viết
*"token `ImportScan`/`ReviewHarvest` chỉ xuất hiện trong `core/glossary/**`"*. Cài đúng chữ
đó cho một cổng so **chuỗi trần**, mà `CandidateOrigin` lại có hai biến thể TRÙNG TÊN với
`TermOrigin` — nên Story 3.5 gọi `insert_candidate(…, CandidateOrigin::ImportScan)` từ tầng
lệnh sẽ làm cổng **đỏ oan**. Đã tái hiện được, không phải suy luận. ⇒ Đọc task đó là
*"chỉ cách viết của `TermOrigin`"*: `NON_MANUAL_ORIGIN_TOKENS` nay giữ chuỗi **đủ định
danh** (`TermOrigin::ImportScan`), và vị từ so chuỗi tách thành hàm thuần dùng chung cho cả
cổng thật lẫn bài tự kiểm hai chiều.

**KEEP — bổ sung cho danh sách của vòng #1:**
- `NON_MANUAL_ORIGIN_TOKENS` phải giữ dạng **đủ định danh**. Rút về tên biến thể trần là
  dựng lại đúng cái bẫy vừa gỡ.
- Bài tự kiểm của cổng đó có **hai** vế: `TermOrigin::…` ⇒ bắt được; `CandidateOrigin::…` ⇒
  không bắt. Bỏ vế thứ hai là bỏ đúng phần chứng minh cổng không đỏ oan.
- `already_decided_error` nhận `col` qua **tham số**, không viết cứng — hai chỗ gọi có số
  cột khác nhau (2 và 0).
- Ba ca test của vòng này phải sống: nhiều hơn một hàng chờ · va chạm khác xuất xứ · ứng
  viên kẹt sau một mục nhập tay (ca cuối ghim hành vi ĐANG hở, có chủ ở sổ nợ).

## Design Notes

```sql
CREATE TABLE glossary_candidate (
  id                INTEGER PRIMARY KEY AUTOINCREMENT,
  source_term       TEXT NOT NULL,
  candidate_origin  TEXT NOT NULL,   -- import_scan | review_harvest
  resolution        TEXT,            -- NULL == cho duyet; approved | rejected
  created_at        TEXT NOT NULL,
  CHECK (trim(source_term, WS) <> ''),
  CHECK (candidate_origin IN ('import_scan','review_harvest')),
  CHECK (resolution IS NULL OR resolution IN ('approved','rejected'))
);
CREATE UNIQUE INDEX idx_glossary_candidate_source_term ON glossary_candidate (source_term);
CREATE TRIGGER glossary_candidate_resolution_is_one_way
BEFORE UPDATE OF resolution ON glossary_candidate
WHEN OLD.resolution IS NOT NULL          -- MOI giá trị, không riêng NULL
BEGIN SELECT RAISE(ABORT, 'glossary candidate resolution is one-way'); END;
-- WS = bảng 25 điểm mã, chép nguyên văn từ GLOSSARY_ENTRY_DDL
```

⚠️ **Hàng ứng viên KHÔNG bị xoá khi bỏ, và đó là ngược với chữ trong AC.** `epics.md:2854-2857` viết *"nó rời bảng chờ"* — "rời" ở đây là rời **danh sách chờ duyệt** (`resolution IS NULL`), không phải rời đĩa. Xoá hàng thật thì lần quét sau chèn lại được và AC kế tiếp (*"không quay lại"*) chết ngay trong cùng một câu.

🔴 **`resolution` không suy được từ `glossary_entry`.** Story 3.9 cho xoá một mục Glossary; nếu "đã duyệt" là một phép `EXISTS` chéo bảng thì lượt xoá đó làm ứng viên sống lại trong im lặng — đúng lớp lỗi *rỗng im lặng* mà `AGENTS.md` gọi là trung tâm của dự án. Một cột tường minh là bản ghi *"người dùng đã quyết"*, khác hẳn *"mục hiện có trên đĩa"*.

## Verification

**Commands:**
- `cd src-tauri && cargo test --locked` -- expected: xanh toàn bộ, gồm `glossary_contract`, `glossary_boundary`, `segment_contract`, `pinned_contract`, `store_contract`.
- `npm run check:i18n` -- expected: xanh; Kiểm A không thấy chữ có dấu ở vị trí mã trong `core/glossary/**`.
- `npm run check:gates` -- expected: xanh; story này không thêm cổng nên ba danh sách không đổi.
- `npm run check:deps` -- expected: xanh; không phụ thuộc mới (NFR15 không kích hoạt).

**Manual checks (if no CLI):**
- `git diff` trên `src-tauri/tests/scope_boundary.rs` phải **rỗng**.
- `dict-manifest.toml` phải **không đổi** — bốn tệp `.db` từ điển không liên quan lược đồ `project.db`.
- Đọc lượt CI trên GitHub sau khi push: `pre-push` xanh trên macOS **không** nói gì về nửa Windows.

## Suggested Review Order

**Bất biến trung tâm — FR55 thành cấu trúc, không còn là kỷ luật**

- Chữ ký đã mất `term_origin`: nơi gọi ngoài không diễn đạt được một mục phi-manual.
  [`store.rs:111`](../../src-tauri/src/core/glossary/store.rs#L111)

- Cửa DUY NHẤT còn sinh ra `term_origin` phi-manual — suy từ chính hàng ứng viên.
  [`candidate_store.rs:120`](../../src-tauri/src/core/glossary/candidate_store.rs#L120)

- Ánh xạ toàn phần: `CandidateOrigin` chỉ hai biến thể, `Manual` không biểu diễn được.
  [`candidate.rs:26`](../../src-tauri/src/core/glossary/candidate.rs#L26)

- Helper chèn dùng chung cho cả hai cửa — một hình dạng hàng, không hai.
  [`store.rs:78`](../../src-tauri/src/core/glossary/store.rs#L78)

**Vòng đời một chiều — chỗ vòng rà soát #1 đã sửa**

- `WHEN OLD.resolution IS NOT NULL`: đã quyết thì không quyết lại, kể cả quyết y hệt.
  [`schema.rs:427`](../../src-tauri/src/core/store/schema.rs#L427)

- Lớp Rust cho một lỗi ĐỌC ĐƯỢC, phân biệt với "id không tồn tại".
  [`candidate_store.rs:172`](../../src-tauri/src/core/glossary/candidate_store.rs#L172)

- *Chờ duyệt* ⟺ `resolution IS NULL` — không cột song song nào nói cùng chuyện.
  [`candidate.rs:129`](../../src-tauri/src/core/glossary/candidate.rs#L129)

**Lược đồ — một bảng mới, chỉ ở `project.db`**

- Bảng chờ, `UNIQUE(source_term)` là cơ chế "không quay lại", không phải một phép kiểm.
  [`schema.rs:409`](../../src-tauri/src/core/store/schema.rs#L409)

- Bước 13, và chỉ thang project — `global.db` vẫn dừng ở 4.
  [`schema.rs:1210`](../../src-tauri/src/core/store/schema.rs#L1210)

**Cưỡng chế — cổng biên, gồm chỗ suýt đỏ oan**

- Hai tên bảng, không một chuỗi cứng: cổng cũ để lọt mọi bảng đặt tên khác.
  [`glossary_boundary.rs:70`](../../src-tauri/tests/glossary_boundary.rs#L70)

- Chuỗi ĐỦ ĐỊNH DANH — dạng trần bắt nhầm `CandidateOrigin` của Story 3.5.
  [`glossary_boundary.rs:123`](../../src-tauri/tests/glossary_boundary.rs#L123)

- Tự kiểm hai chiều: bắt được `TermOrigin::`, không bắt `CandidateOrigin::`.
  [`glossary_boundary.rs:450`](../../src-tauri/tests/glossary_boundary.rs#L450)

**Test — bốn ca đáng đọc nhất**

- Ca giết story nếu cài ngược: bỏ rồi duyệt lại không được phục sinh thuật ngữ.
  [`glossary_contract.rs:1593`](../../src-tauri/tests/glossary_contract.rs#L1593)

- Chiều ngược: duyệt rồi bỏ — hai bảng không được nói ngược nhau.
  [`glossary_contract.rs:1631`](../../src-tauri/tests/glossary_contract.rs#L1631)

- Mọi bước ngang sau một quyết định, không riêng lượt lùi về NULL.
  [`glossary_contract.rs:1462`](../../src-tauri/tests/glossary_contract.rs#L1462)

- Ghim hành vi ĐANG hở, có chủ ở sổ nợ — không phải hành vi mong muốn.
  [`glossary_contract.rs:1680`](../../src-tauri/tests/glossary_contract.rs#L1680)

**Ngoại vi — neo phiên bản, chỗ panic ngầm**

- Bước giả nâng lên 14; để nguyên 13 thì ca này panic chứ không đỏ lúc biên dịch.
  [`segment_contract.rs:1600`](../../src-tauri/tests/segment_contract.rs#L1600)

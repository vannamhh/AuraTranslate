---
title: 'AI-7 — one named trim for the ChapterOrigin rule, and the second divergence the 2026-09-07 measurement missed'
type: 'bugfix'
created: '2026-09-15'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: 'c8e3894a78acd838602a51f57b08a1af8104fec2'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The rule *"a `ChapterOrigin` field holding only whitespace becomes `NULL`"* is written **three** times for those four columns — `commands/project.rs:3862` (`trimmed_or_none`, preview override), `commands/chapter.rs:582` (`trimmed_or_none`, edit after import), `core/webimport/origin.rs:57` (`present`, machine extraction) — all three `str::trim()`, while the preview half uses JS `.trim()` (`importPreviewState.ts:496-499`). Retro Epic 6 F7 measured one divergence in the live app: WKWebView trims `U+FEFF`, Rust does not, so a field holding only a BOM reads *"Không tìm thấy"* on the preview while the disk stores `U+FEFF` — and `LibraryMode` then shows a blank box with **no** label, against Story 6.15 AC4. The two cases guarding this rule (`chapter_origin_contract.rs:352`, `importPreviewChapterOrigin.test.ts:200`) both use `'   '`, a value both sides agree on, so neither guards the seam.

**Approach:** One named trim for the four `ChapterOrigin` columns, declared once and used by all three sites, with the JS-side rule as the reference. Then a contract case on each side carrying the exact codepoint the engines disagree on, verified RED before the patch.

## Decisions (Ice)

Round 1, 2026-09-15, both settled against a measurement rather than against the retro's wording:

- **D1 — the predicate is the JS set exactly: `White_Space ∖ {U+0085} ∪ {U+FEFF}`, 25 codepoints.** Measured the same day over every codepoint on both engines; V8 and JavaScriptCore returned identical sets, so the vitest half and the shipped app half measure one rule. Rejected: the 26-codepoint superset `trim_like_the_paste_box:4110` ships today, because it leaves the `U+0085` divergence standing in the direction `project.rs:4107` itself warns about; and the both-halves union, because it would put the character set in two places, Rust and TypeScript, which is the defect class this spec removes. **JS native `.trim()` stays the single definition of the set; Rust follows it.**
  - **Owned consequence, recorded not designed away:** a field holding *only* `U+0085` stops becoming `NULL` and is stored verbatim. Both halves agree on it, so no silent divergence — but it renders as a blank box with no "Không tìm thấy" label, the Story 6.15 AC4 symptom arriving through a different character. Frequency in the wild is **unmeasured**. Goes to `deferred-work.md` with an owner; it is not closed by this spec.
- **D2 — scope is the rule, not the retro's list.** The three `ChapterOrigin` sites share the predicate, and `trim_like_the_paste_box` is rebuilt on it keeping its `&str` shape. Chosen against a full census of the identical trim-or-`None` body — **six** copies, where the retro named three and the third of those was a different shape — and because `origin.rs:57::present`, which the retro missed, is the half that fills these four columns on every URL import. The three `title` sites stay out: different column, own JS counterpart, no measurement yet.
- **D3 — spec length.** 4 369 tokens when the decision was taken, 4 305 as approved (`tiktoken`, `cl100k_base`, counted not estimated) — above the 1 600 guideline, kept in full deliberately: the bulk is the Code Map and the two measurements that stop the implementing agent re-investigating. Same call as AI-4's D3. Mitigation is phase-splitting per `AGENTS.md`, not deletion.

## Boundaries & Constraints

**Always:**
- The predicate is declared in **exactly one place** and every shape (`&str` and `Option<String>`) is built on it. A second copy of the character set is the defect this spec exists to remove.
- Every new test case must be shown **RED before the patch and GREEN after**. The counter-check is a REMOVAL at the seam, not a commented line and not a case that calls the patched function directly (`AGENTS.md` §Known pitfalls).
- Both `&str` and `Option<String>` shapes keep their current return types; no call site changes signature.
- The shared symbol names the rule, not the mechanism, and never carries a bare `origin` identifier (`AGENTS.md` §Conventions — "Origin" names four disjoint things).

**Never:**
- Do **not** touch the 25-codepoint `White_Space` family: `GLOSSARY_ENTRY_DDL` (`store/schema.rs:305-323`), `core/cleanup/store.rs:140`, `core/segment/normalize.rs:88`, `glossary_contract.rs:287-325`. `src-tauri/AGENTS.md:37` pairs that class to `str::trim()` deliberately; this spec's predicate is a **different** class for a different entity and must be named so nobody "syncs" the two.
- Do **not** change the three `title` sites (`chapter.rs:542`, `pipeline.rs:1429`, `pipeline.rs:1508`) — different column, own JS counterpart, record as owned debt.
- No new `AD`. No change to `importPreviewState.ts` behaviour — native `.trim()` is the reference, not a thing to reimplement.
- No dependency added (NFR15 gate).

## I/O & Edge-Case Matrix

Applies to each of the four columns `origin_author` · `origin_site_name` · `origin_url` · `origin_published_at`, on both the preview path and the chapter-list edit path.

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Empty string | `""` | column `NULL`; preview renders the "Không tìm thấy" placeholder | N/A |
| ASCII spaces only | `"   "` | column `NULL`; preview placeholder | N/A |
| BOM only — the measured seam | `"\u{FEFF}"` | column `NULL`; preview placeholder. Both sides agree | N/A |
| BOM plus real text | `"\u{FEFF}Tấn Giang"` | column `"Tấn Giang"`; preview shows the same string | N/A |
| Real text with inner BOM | `"Tấn\u{FEFF}Giang"` | stored verbatim — inner zero-width is content, only the two ends are cut | N/A |
| Mixed whitespace both ends | `"\t\u{00A0}x\u{2028} "` | column `"x"` | N/A |
| NEL only — the reverse seam, D1 | `"\u{0085}"` | stored **verbatim**, column NOT `NULL`; preview shows it as present. Both sides agree. This is the owned consequence of D1, and a case must pin it so a future "sync with `str::trim()`" turns red | N/A |
| NEL plus real text | `"\u{0085}Tấn Giang"` | column `"\u{0085}Tấn Giang"` — the leading NEL is content under the JS set, not trimmed | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/project.rs:3860-3886` — `trimmed_or_none`, 1 call site at `:3885` inside the `pick` closure of `effective_origin_fields:3870`, which fans out to four fields at `:3892-3895`. The preview-override half.
- `src-tauri/src/commands/chapter.rs:573-599` — `update_chapter_origin`; nested `trimmed_or_none:582`, 4 call sites `:586-589`, bound into the `UPDATE chapter SET origin_*` at `:593-599`. Body byte-identical to the above apart from the parameter name. Rule stated on the enclosing fn at `:565-566`.
- `src-tauri/src/core/webimport/origin.rs:57` — `fn present(raw: &str) -> Option<String>`, private, 4 call sites `:69,:119,:180,:184`, same four columns. Third verbatim copy; doc'd as "khuôn dùng CHUNG cho mọi trường text". `core/webimport/mod.rs:63` already re-exports `ChapterOrigin`/`extract_origin`, so this module is reachable from both command files (`project.rs:40` already imports it; `chapter.rs` does not yet but already cross-imports `commands::project` at `:38`).
- `src-tauri/src/commands/project.rs:4088-4111` — `trim_like_the_paste_box`, 1 call site `:4119` in `fetch_url_import_items:4117`. Carries the 2026-09-07 measurement block; `:4107` is the line D1 revisits. Guarded by `webimport_contract.rs:860-881`.
- `src-tauri/tests/chapter_origin_contract.rs` — 15 test fns; the two whitespace cases are `:348` (via `create_work`, preview path) and `:577` (via `update_chapter_origin`, list path), both using `'   '`. Drives the real product fns directly, no test-only wrapper (header `:1-3`).
- `tests/frontend/importPreviewChapterOrigin.test.ts:195-202` — the JS-side whitespace case, `'   '`; loads `src/importPreviewState.ts` fresh per case via `freshState():41-51`, IPC mocked at `:27-39`.
- `src/importPreviewState.ts:487-499` — `importPreviewCurrentChapterOrigin`; native `.trim()` on the four fields. The reference implementation. Comment at `:490` pins the rule to the two Rust fns.
- `src/ChapterOrigin.vue:66-67` (repeated `:77-78`, `:88-89`, `:99-100`) — placeholder shows whenever the bound prop is `null`/`''`; single-label policy at `:15`, string at `src/i18n/vi.json:428`. This is where the blank-box-with-no-label symptom surfaces.
- `src/ImportPreviewOverlay.vue:1058-1066` — `v-if="importPreviewLastSubmittedFrom === 'urls'"` gates the whole four-field block; origin editing exists only on the URL import path.
- **Do not touch:** `src-tauri/src/core/store/schema.rs:305-323` · `core/cleanup/store.rs:140` · `core/segment/normalize.rs:88` · `src-tauri/tests/glossary_contract.rs:287-325` — the `str::trim()`-paired 25-codepoint family.

**Tests that move:** `src-tauri/tests/chapter_origin_contract.rs` · `tests/frontend/importPreviewChapterOrigin.test.ts` · `src-tauri/tests/webimport_contract.rs`.

## Tasks & Acceptance

**Execution:**
- [x] Record the RED counter-check FIRST — add the `U+FEFF`-only case to `src-tauri/tests/chapter_origin_contract.rs` and to `tests/frontend/importPreviewChapterOrigin.test.ts`, run both, and paste the failing output into §Implementation Notes before changing any product line. A case that is green on arrival means the seam was not hit.
- [x] `src-tauri/src/core/webimport/origin.rs` -- declare the shared predicate and the two shapes here, `pub(crate)`, with a doc-comment carrying the measurement table and an explicit "this is NOT the `GLOSSARY_ENTRY_DDL` class" sentence -- this module already owns `ChapterOrigin` and is reachable from both command files.
- [x] `src-tauri/src/commands/project.rs` -- delete `trimmed_or_none:3862`, call the shared fn -- removes copy 1 of 3.
- [x] `src-tauri/src/commands/chapter.rs` -- delete the nested `trimmed_or_none:582`, call the shared fn -- removes copy 2 of 3.
- [x] `src-tauri/src/core/webimport/origin.rs` -- fold `present:57` into the shared fn -- removes copy 3 of 3.
- [x] `src-tauri/src/commands/project.rs:4109` -- rebuild `trim_like_the_paste_box` on the shared predicate, keeping its `&str` shape. Its set therefore **loses** `U+0085` (D1); update the `:4107` conclusion in place with 🔵 and today's date rather than deleting it, because that line's own warning is what D1 acts on -- `AGENTS.md`: a claim that stops being true gets fixed in place.
- [x] `src-tauri/tests/chapter_origin_contract.rs` -- extend both whitespace cases to the matrix rows -- the existing `'   '` value cannot fail on either side.
- [x] `tests/frontend/importPreviewChapterOrigin.test.ts` -- same rows on the JS side -- proves the two halves now agree on the same inputs.
- [x] `src-tauri/tests/webimport_contract.rs` -- add a `U+0085`-only pasted line next to the existing `U+FEFF` cases at `:860-881` -- D1 changes this path too: today Rust drops such a line and `pastedUrlLines` keeps it, so the on-screen *N link* count and the number of Chapters created disagree by one. After the patch both keep it. Show this case RED before the patch.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- two owned entries: the three `title` sites, and the D1 consequence (a `U+0085`-only origin field now stored verbatim, rendering as an unlabelled blank box) -- `AGENTS.md`: never mark something passed by inference.

**Acceptance Criteria:**
- Given a `ChapterOrigin` field holding only `U+FEFF`, when the Work is created from a URL preview **and** when the field is later cleared from the chapter list, then the column is `NULL` on disk and the preview and `LibraryMode` both show the "Không tìm thấy" label — the same outcome on both paths.
- Given the patch is removed at the seam (the shared fn reverted to `str::trim()`), when the suite runs, then the new `U+FEFF` cases go RED on both sides — evidence pasted into §Implementation Notes.
- Given the whole tree, when the identical trim-or-`None` body is counted, then the `ChapterOrigin` rule accounts for exactly one declaration, and the remaining copies are named in `deferred-work.md` with an owner.
- Given `src-tauri/tests/glossary_contract.rs` and the `GLOSSARY_ENTRY_DDL` family, when the suite runs, then they are untouched and green — this spec's predicate did not leak into that class.
- Given a pasted URL list containing a line holding only `U+0085`, when the preview counts *N link · N Chương*, then Rust and `pastedUrlLines` produce the same count — the reverse-direction divergence D1 closes on this path as well.

## Implementation Notes

**Đối chứng đỏ TRƯỚC bản vá (§Tasks mục 1), chạy 2026-09-15 trên cây trước khi đụng bất kỳ
dòng sản phẩm nào.**

Thêm ca `an_override_holding_only_a_byte_order_mark_stores_null_the_measured_seam` vào
`chapter_origin_contract.rs` (ô override CHỈ mang `U+FEFF` phải ghi `NULL`) và một ca tương
ứng vào `tests/frontend/importPreviewChapterOrigin.test.ts` rồi chạy cả hai TRƯỚC khi sửa
`trimmed_or_none`/`present`:

```
$ npx vitest run tests/frontend/importPreviewChapterOrigin.test.ts
 Test Files  1 passed (1)
      Tests  11 passed (11)
```
JS đã XANH ngay từ đầu — đúng vai "tham chiếu" của D1: `.trim()` gốc của JS vốn đã cắt `U+FEFF`,
không cần sửa phía JS.

```
$ cd src-tauri && cargo test --locked --test chapter_origin_contract
test an_override_holding_only_a_byte_order_mark_stores_null_the_measured_seam ... FAILED

---- an_override_holding_only_a_byte_order_mark_stores_null_the_measured_seam stdout ----
thread '...' panicked at tests/chapter_origin_contract.rs:381:5:
assertion `left == right` failed: o chi mang U+FEFF phai ve NULL, khop luat cat cua JS .trim()
  left: Some("\u{feff}")
 right: None

test result: FAILED. 15 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s
```
Rust ĐỎ đúng như D1 dự đoán — `str::trim()` không cắt `U+FEFF` (không nằm trong `White_Space`
Unicode). Seam đã bị kích thật, không phải một ca xanh-sẵn nguỵ trang thành đối chứng.

Sau đó ca này (và ca đối xứng ở `clearing_a_field_to_empty_from_the_chapter_list_stores_null`,
gộp cùng nó) được mở rộng thành hai ca ma trận đầy đủ
(`an_override_cleared_to_whitespace_only_values_matches_the_io_matrix` ·
`clearing_a_field_to_whitespace_only_values_from_the_chapter_list_matches_the_io_matrix`) —
xem §Tasks mục 6.

**Đối chứng đỏ cho `webimport_contract.rs` (§Tasks mục 8, chiều NGƯỢC của D1).** Ca mới
`a_next_line_character_only_pasted_line_still_produces_exactly_one_item_matching_pasted_url_lines`
xác nhận GREEN sau bản vá, rồi bị đối chứng bằng phép GỠ THẬT (tạm khôi phục
`trim_like_the_paste_box` về `line.trim_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}')`
— hình dạng CŨ trước D1, gồm cả `U+0085` trong tập cắt):

```
$ cargo test --locked --test webimport_contract a_next_line_character_only_pasted_line
thread '...' panicked at tests/webimport_contract.rs:899:5:
assertion `left == right` failed: một dòng CHỈ có U+0085 phải sinh ĐÚNG một mục — ...
  left: 0
 right: 1
test result: FAILED. 0 passed; 1 failed; ...
```
Gỡ xong, khôi phục nguyên trạng bản vá (`webimport::chapter_origin_trim(line)`), chạy lại →
xanh. Bằng chứng seam thật ở CẢ HAI chiều (thêm `U+FEFF`, bớt `U+0085`), không chỉ một chiều.

**Tóm tắt thay đổi sản phẩm.** Một khai báo DUY NHẤT
(`core::webimport::origin::is_chapter_origin_trim_char` + hai hình dạng
`chapter_origin_trim`/`chapter_origin_trim_or_none`, `pub(crate)`) thay cho ba bản chép
(`commands::project::trimmed_or_none` · `commands::chapter::update_chapter_origin`'s nested
`trimmed_or_none` · `core::webimport::origin::present`) cộng một hình dạng thứ tư
(`commands::project::trim_like_the_paste_box`, giữ nguyên chữ ký `&str`, nay chỉ còn gọi
xuống hàm chung). Hai doc-comment cũ đã sai theo thời gian được sửa TẠI CHỖ với 🔵 và ngày
2026-09-15 (`chapter.rs` gần `update_chapter_origin`, `project.rs` gần `trim_like_the_paste_box`)
thay vì bị xoá.

**Đếm chỗ nối.** `grep -n "fn trimmed_or_none\|fn present" src-tauri/src/**/*.rs` sau bản vá
trả 0 dòng — cả ba bản chép `ChapterOrigin` đã biến mất, chỉ còn một khai báo ở `origin.rs`.
Ba bản chép của cột `title` (`chapter.rs:542`, `pipeline.rs:1429`, `pipeline.rs:1508`) vẫn
đứng nguyên, ngoài phạm vi D2 — ghi ở `deferred-work.md`.

**Đối chứng đỏ do NGƯỜI ĐIỀU PHỐI chạy lại, 2026-09-15, trên các ca CUỐI CÙNG.** Bằng chứng đỏ
ghi bên trên thuộc về một hàm test tên
`an_override_holding_only_a_byte_order_mark_stores_null_the_measured_seam` — hàm đó KHÔNG còn
tồn tại trong cây, nó đã bị thay bằng hai ca ma trận. Một đối chứng đỏ trên một hàm đã biến mất
không chứng minh gì về hàm đang đứng, nên AC số 2 được đo lại bằng hai phép GỠ THẬT tại chính
`is_chapter_origin_trim_char`, mỗi phép tách một chiều:

```
Gỡ 1 — phép cắt về `c.is_whitespace()` (đúng str::trim, tức "chưa có bản vá"):
  an_override_cleared_to_whitespace_only_values_matches_the_io_matrix ... FAILED
  clearing_a_field_to_whitespace_only_values_from_the_chapter_list_matches_the_io_matrix ... FAILED
    hang 'chi BOM -- seam do duoc 2026-09-07': input "\u{feff}"
      left: Some("\u{feff}")   right: None
  test result: FAILED. 13 passed; 2 failed

Gỡ 2 — phép cắt về `c.is_whitespace() || c == '\u{FEFF}'` (hình dạng CŨ 26 ký tự của
        trim_like_the_paste_box; hàng BOM xanh trở lại, chỉ còn chiều ngược hở):
  an_override_cleared_to_whitespace_only_values_matches_the_io_matrix ... FAILED
  clearing_a_field_to_whitespace_only_values_from_the_chapter_list_matches_the_io_matrix ... FAILED
    hang 'chi NEL -- seam NGUOC, D1: giu verbatim, KHONG con NULL': input "\u{85}"
      left: None   right: Some("\u{85}")
  a_next_line_character_only_pasted_line_still_produces_exactly_one_item... ... FAILED
    left: 0   right: 1
```

Phép gỡ 2 là phép cần thiết: dưới phép gỡ 1 cả hai ca ma trận chết ở hàng BOM (hàng đầu tiên
sai), nên hàng `U+0085` chưa được chứng minh có ai canh. Tách hai chiều mới thấy cả hai hàng
đều kích được bằng một khuyết tật thật.

Khôi phục từ BẢN SAO LƯU (`origin.rs.BACKUP`, md5 `05fd3c1d…` khớp trước và sau), không phải từ
`HEAD` — việc này chưa commit nên `git checkout` sẽ xoá luôn bản vá. Sau khôi phục:
`chapter_origin_contract` 15/15 · `webimport_contract` 34/34 (+3 ignored) ·
`glossary_contract` 72/72 · vitest 17/17.

**Vòng vá sau rà (R1·R2·R4·R5·R7·R8), và phép đo lại của người điều phối, 2026-09-15.**

R4 (ma trận assert trong vòng lặp) được kiểm bằng chính phép gỡ đã lộ ra nó. Trước bản vá, một
lượt gỡ chỉ báo hàng đỏ ĐẦU TIÊN. Sau bản vá, **cùng một** phép gỡ (`is_chapter_origin_trim_char`
→ `c.is_whitespace()`) báo đủ bốn hàng bị ảnh hưởng, trong một lượt, ở cả hai ca ma trận:

```
hang 'chi BOM -- seam do duoc 2026-09-07' (input "\u{feff}"): duoc Some("\u{feff}"), ky vong None
hang 'BOM cong chu that -- chi hai dau bi cat' (input "\u{feff}Tấn Giang"): duoc Some("\u{feff}Tấn Giang"), ky vong Some("Tấn Giang")
hang 'chi NEL -- seam NGUOC, D1: giu verbatim, KHONG con NULL' (input "\u{85}"): duoc None, ky vong Some("\u{85}")
hang 'NEL dau + chu that -- NEL la NOI DUNG duoi tap cua JS' (input "\u{85}Tấn Giang"): duoc Some("Tấn Giang"), ky vong Some("\u{85}Tấn Giang")
test result: FAILED. 13 passed; 2 failed
```

R1 (đường máy trích không ai canh) được kiểm bằng CÙNG phép gỡ đó — bốn ca mới đỏ, chứng minh
chúng canh đường `extract_origin` thật chứ không tự canh chính mình:

```
a_meta_tag_holding_only_a_byte_order_mark_is_read_as_absent ... FAILED
a_meta_tag_holding_only_a_next_line_character_is_kept_verbatim ... FAILED
a_json_ld_field_holding_only_a_byte_order_mark_is_read_as_absent_on_both_shapes ... FAILED
a_json_ld_field_holding_only_a_next_line_character_is_kept_verbatim_on_both_shapes ... FAILED
test result: FAILED. 9 passed; 4 failed
```

Khôi phục từ bản sao lưu thứ hai (md5 `b3f46d35…` khớp trước và sau). Lượt xanh cuối cùng:
`chapter_origin_contract` 15/15 · `webimport_contract` 34/34 (+3 ignored) · `glossary_contract`
72/72 · `--lib core::webimport::origin` 13/13 · vitest hai tệp 33/33.

R9 — `npm run test:story ai-7`, output đã trình thay vì khai:

```
Rust     (3/53) : chapter_origin_contract glossary_contract webimport_contract
Frontend (1/74) : importPreviewChapterOrigin.test.ts
── xong trong 10.9s · mã thoát 0
```

Không `THIẾU KHAI`. Lưu ý phạm vi: đây là vòng lặp dev, KHÔNG phải cổng — `pre-push` đầy đủ và
CI hai nền tảng vẫn là thứ phải đọc trước khi kết luận, và bộ e2e chạy hằng đêm nằm ngoài cả hai.

## Spec Change Log

## Review Triage Log

Pass 1, 2026-09-15 — three layers: `verification-gap` (1 finding), `blind-hunter` (9), `edge-case-hunter` (0, empty array with its trace). No `intent_gap`, no `bad_spec`, so no loopback: `review_loop_iteration` stays 0.

| # | Source | Verdict | Route | Evidence |
|---|---|---|---|---|
| R1 | verification-gap + blind-hunter (grouped, one root cause) | `medium` | patch | The extraction path calls the new predicate at `origin.rs:112,162,164,225,229`, but its only whitespace test, `whitespace_only_meta_content_counts_as_absent:297`, feeds `content="   "` — a value the old and new predicates agree on. Read the whole `#[cfg(test)] mod tests` in `origin.rs`: zero occurrences of `FEFF`/`0085`. A future edit that fixes only the override and list paths would leave extraction on the stale rule with nothing red. This is the half D2 named as the one the retro missed. |
| R2 | blind-hunter | `low` | patch | Verified in the diff: `chapter.rs` **deleted** the two-line stale claim and paraphrased it, while `project.rs` kept its original sentence verbatim and appended the dated 🔵 underneath. `AGENTS.md` §Conventions: "gets FIXED IN PLACE with 🔵 and a date; don't delete it". Two sites, two treatments, one rule. |
| R3 | blind-hunter | `false` | rejected | Claim: the measurement table in `origin.rs`'s doc-comment and in the spec's §Design Notes are two sources of truth that can drift. Refutation: a spec in this repo is a point-in-time record of a decision (no `spec-6-*.md` is edited after `done`); the living definition is the doc-comment. They are a record and a definition, not two definitions. |
| R4 | blind-hunter | `medium` | patch | Real, and demonstrated during this run: both matrix tests assert inside a `for` loop, so the first failing row aborts the rest. Under removal 1 both tests died on the BOM row and said nothing about the NEL row — the orchestrator needed a second, different removal to show the NEL row was guarded at all. A future regression on an early row hides every later row the same way. |
| R5 | blind-hunter | `low` | patch | The frozen matrix names the input `"\u{FEFF}Tấn Giang"`; the Rust and TS rows that satisfy it use `"Tan Giang"`. `src-tauri/AGENTS.md:16` grants `tests/**` a named exemption to keep diacritics, so the substitution was not forced. The literal the table commits to is not the literal that ran. |
| R6 | blind-hunter | `false` | rejected | Claim: `review_loop_iteration: 0` is wrong given the orchestrator's re-verification, and the triage log should already be filled. Refutation: the counter increments before a **loopback** (step-04 §4), and no loopback occurred; the re-verification was a verification pass, not a re-derivation. The triage log is written by this step — it was empty because this step had not run. |
| R7 | blind-hunter | `low` | patch | The `U+0085` frequency debt entry states the owner and what to do if the frequency turns out significant, but no method for measuring it — unlike its sibling entry, which spells one out. A debt with no method is a debt nobody can start. |
| R8 | blind-hunter | `medium` | patch | The AC asserts Rust and `pastedUrlLines` agree on a lone-NEL line. Verified true by reading `src/modes/libraryImport.ts:125-130` (`.map(line => line.trim())`, native JS, keeps NEL) — but measured on the Rust side only. `tests/frontend/importPreviewUrls.test.ts:172` covers ordinary URLs and a whitespace-only row, never NEL. The equality in the AC is half-measured. |
| R9 | blind-hunter | `low` | patch | `npm run test:story ai-7` is listed in §Verification with its result asserted but no output shown, unlike every other command in §Implementation Notes. Resolved by the orchestrator running it; output recorded in §Implementation Notes. |
| R10 | orchestrator | `low` | rejected | The frozen matrix row "BOM plus real text" adds *"preview shows the same string"*. `importPreviewState.ts:496` returns `draft.author` **untrimmed**, so the preview's in-memory value is `"\u{FEFF}Tấn Giang"` while the column holds `"Tấn Giang"`. Invisible on screen (zero-width) and it never reaches disk, so no functional defect. Rejected here only because the fix edits this build's spec (step-04 §Classify) — the root cause is one sentence inside `<frozen-after-approval>`, which only Ice may change. Raised to Ice at presentation. |

## Design Notes

**The measurement, run 2026-09-15 over every codepoint `0x0..=0x10FFFF` on both sides.** Rust via `rustc -O` calling `char::is_whitespace`; JS via node v26.8.2 (V8) and, because the app runs WKWebView rather than node, again via `osascript -l JavaScript` (JavaScriptCore). **V8 and JSC returned identical sets**, so the vitest half and the shipped app half are measuring the same rule.

```
Rust str::trim()  (25):  0009 000A 000B 000C 000D 0020 0085 00A0 1680 2000‥200A 2028 2029 202F 205F 3000
JS   .trim()      (25):  0009 000A 000B 000C 000D 0020      00A0 1680 2000‥200A 2028 2029 202F 205F 3000 FEFF
Difference: JS-only {FEFF}   ·   Rust-only {0085}
```

JSC spot-check, verbatim: `85=keep FEFF=TRIM 9=TRIM 20=TRIM A0=TRIM 200B=keep 180E=keep 2028=TRIM`.

Note what this does **not** say: `U+200B` (zero-width space) is cut by neither side, so it is content on both — a separate rule already owns it (`core/glossary/exchange.rs:503 ZERO_WIDTH_CHARS`), and this spec does not touch it.

**Why the retro's "three functions" is the wrong unit.** Counting by the criterion being cut rather than by the names the retro supplied gives six identical bodies; of the three the retro listed, two are duplicates of each other and the third is a different shape entirely, while the one that matters most — the machine-extraction half writing the same four columns — was not listed. The lesson `AGENTS.md` already records applies to the retro's own row: count the population with the criterion you are about to cut, before agreeing a scope.

## Verification

**Commands:**
- `npm run build` -- expected: `dist/` present. Required before `cargo test`, which otherwise fails at compile time, not at an assert.
- `npx vitest run tests/frontend/importPreviewChapterOrigin.test.ts` -- expected: RED on the new `U+FEFF` rows before the patch, all green after.
- `cd src-tauri && cargo test --locked --test chapter_origin_contract` -- expected: RED on the new rows before the patch, 15+ green after.
- `cd src-tauri && cargo test --locked --test webimport_contract` -- expected: RED on the new `U+0085` pasted-line case before the patch, green after; the existing `U+FEFF` cases at `:860-881` stay green throughout.
- `cd src-tauri && cargo test --locked --test glossary_contract` -- expected: green, untouched — the predicate did not leak into the `str::trim()` family.
- `npm run test:story ai-7 -- --list` then `npm run test:story ai-7` -- expected: the listed set matches §Tests that move, no `THIẾU KHAI`.
- Full `pre-push` scope before pushing; then read the CI run, both platforms, before writing any status.

**Manual checks (if no CLI):**
- The live-app path the retro measured is reachable only through a real URL import (network) — not run here. If a manual pass happens, the shape to reproduce is retro §Behavior verification Ca 2, which printed `{"jsTrimOfFeffIsEmpty":true,"authorCodepoints":[65279]}` and must print an empty/absent author after the patch.

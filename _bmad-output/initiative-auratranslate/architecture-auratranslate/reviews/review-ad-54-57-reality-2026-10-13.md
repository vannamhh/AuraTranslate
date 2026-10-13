# Review thực tế (reality-check) AD-54 · AD-55 · AD-56 · AD-57 — 2026-10-13

Phạm vi: spine dòng 996–1114. Đối chiếu với mã thật trong `src-tauri/`, `src/`, và chạy SQL thật trên SQLite đóng gói. Không sửa spine hay mã.

## Phán quyết

Bốn AD khả thi về SQLite (mọi DDL và trigger chạy đúng như AD khai, cả 3.53.2 đóng gói lẫn 3.43.2 cục bộ), nhưng có **2 chỗ AD nói sai về mã hôm nay** (đường ghi `translation` của Glossary không chỉ có `confirm_translation`/`update_manual_term`; đường xuất TMX đọc `tm_unit` ngoài ba hàm đã nêu) và **1 tham chiếu tới một cổng không tồn tại**. Nên sửa trước khi giao Story 12.14/12.15/12.19/12.20.

## Công cụ đo

- SQLite đóng gói: `rusqlite =0.40.1` (`src-tauri/Cargo.toml:37`, feature `bundled`) → `libsqlite3-sys 0.38.1` (`Cargo.lock:2656-2657`) → `SQLITE_VERSION "3.53.2"` (`~/.cargo/registry/src/*/libsqlite3-sys-0.38.1/sqlite3/sqlite3.c:470`). Lock còn mang `libsqlite3-sys-0.35.0` trong registry nhưng không phải bản được link (Cargo.lock chỉ có 0.38.1).
- `sqlite3` cục bộ: 3.43.2 (2023-10-10). Chênh phiên bản 3.43 → 3.53.
- Cách chạy: biên dịch chính `sqlite3.c` 3.53.2 của crate với một harness C nhỏ (`scratchpad/run.c`) chạy từng khối SQL riêng, không dừng ở lỗi đầu; chạy lại cùng script bằng `sqlite3` 3.43.2. DDL gốc trích nguyên văn từ `schema.rs` (`GLOSSARY_ENTRY_DDL`, `GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL`, `GLOSSARY_ENTRY_OCCURRENCE_COUNT_AND_ZERO_WIDTH_GUARD_DDL`, `PROOFREAD_MEMORY_DDL`, `SEGMENT_STATUS_AND_VERSION_DDL`, `WORK_DDL`).
- Kết quả: **cùng tập lỗi trên cả hai phiên bản** (9 abort `glossary lifecycle is one-way`, 1 CHECK glossary, 2 abort + 2 CHECK của segment_version, 2 CHECK của AD-56, 2 CHECK/NOT NULL của AD-57). Không có khác biệt hành vi giữa 3.43.2 và 3.53.2.
- Giới hạn đo: đây là SQL thuần trong bộ nhớ, không phải chạy qua `Store::write` của app; không chạy `cargo test`. Không đo phép di trú qua `PROJECT_MIGRATIONS`/`GLOBAL_MIGRATIONS` thật.

## Phát hiện

### F1 — CAO — AD-54 mục 3 bỏ sót hai đường ghi `glossary_entry.translation`; trigger mới sẽ ABORT chúng khi mục có `pending_confirmed_at`

AD-54 mục 3 chỉ kể: `confirm_translation` đặt cột; "sửa thủ công, thu hoạch review" đặt `NULL`. Mã thật có **năm** câu ghi `translation`:

| Chỗ | Câu | Hệ quả với trigger mới |
|---|---|---|
| `core/glossary/store.rs:266` `confirm_translation` | `SET translation = ?1` | AD đã tính |
| `core/glossary/store.rs:1045` `update_manual_term` | `SET translation = ?1, note = ?2, category = ?3` (luôn gán `translation`, kể cả khi chỉ sửa ghi chú) | cùng chữ ⇒ qua (đã đo T7b); chữ khác trên mục có ts ⇒ ABORT trừ khi câu lệnh cũng đặt ts `NULL` |
| `core/glossary/candidate_store.rs:376` (nhận ứng viên `review_harvest` đè mục có sẵn) | `SET translation = ?1, term_origin = ?2` | AD có nhắc "thu hoạch review"; đo T9: ABORT nếu ts còn |
| `core/glossary/store.rs:2028` (nhập Glossary, `ConflictDecision::TakeTheirs`) | `SET translation = ?1 WHERE id = ?2 AND translation IS ?3` | **AD không nhắc.** Đo T8: ABORT khi mục có ts ⇒ một lượt nhập hợp lệ vỡ bằng `store.write_failed` thay vì xử lý va chạm |
| `INSERT` (`store.rs:106,120` `insert_entry_row`) | không đặt cột | AD đã tính (mục 3 cuối) |

Thêm: doc của `commands/glossary.rs:~505` nói `confirm_translation` dùng cho **cả hai chiều** (chốt lần đầu, và sửa mục ĐÃ chốt sang chữ khác). Với trigger mới, chiều thứ hai trên mục có ts sẽ ABORT nếu câu lệnh vẫn là `SET translation = ?1`; câu lệnh phải thành kiểu `SET translation = ?1, pending_confirmed_at = CASE WHEN translation IS NULL THEN ?now ELSE NULL END`. AD mục 3 nói "chỉ `confirm_translation` đặt nó" nhưng không nói nó phải tự xoá khi sửa mục đã chốt.
Đề nghị: liệt kê đủ bốn writer trong mục 3 (và đưa `TakeTheirs` của nhập Glossary vào), ghi rõ `confirm_translation` rẽ nhánh theo `translation IS NULL`.

### F2 — CAO — AD-57 mục 3 và mục 6: đường xuất TMX không đi qua ba hàm đã nêu

AD-57 khẳng định "các hàm đọc của `core/tm` (`read_tier_rows`, `pairs_for_source`, `merge_tagged_tiers`)" là chỗ lọc duy nhất, và `tm_export_tier` "lọc bằng cùng hàm ở mục 3". Mã thật:

- `commands/tm.rs:570` `tm_export_tier` → `tm_render_tier` → `tm_read_tier_pairs` (`commands/tm.rs:513`) → `core/tm/tmx.rs:664` `distinct_tier_pairs` → `load_all_pair_rows` + `merge_tiers` trực tiếp. **Không** qua `read_tier_rows` hay `pairs_for_source`.
- `merge_tagged_tiers` là hàm **private** (`core/tm/mod.rs:243`, `fn`, không `pub`); không bên tiêu thụ nào gọi được. Nó được gọi từ `merge_tiers` (private) và `fuzzy_pairs_in_candidates` (pub). Cặp hàm công khai thực sự là `pairs_for_source` (`mod.rs:217`) và `read_tier_rows` (`mod.rs:328`); `merge_tagged_tiers` chỉ là nơi xếp hạng.
- Bên tiêu thụ thật của hai hàm công khai: `chapter_read.rs:543` (`fill_exact_tm_matches`), `tm_match.rs:105,130`, `commands/tm.rs:215` (danh sách quản lý), `core/ai/rag.rs:281` (`read_tier_rows`, qua `gather_tm_context`/`fuzzy_pairs_in_candidates`), `fuzzy_pairs_for_source` (`mod.rs:342`). `commands/aitranslate.rs` **không** gọi `core::tm` trực tiếp (đi qua `rag`); `commands/aiprompt.rs:42` chỉ nhập `PairSide, TmTier` để dựng dây.
- Còn các hàm công khai khác đọc `tm_unit` ngoài tập đã nêu: `pair_by_id` (`mod.rs:457`), `live_copies`/`update_copies_target`/`delete_copies`/`push_copies_to_global` (đều là quản lý, hợp lệ để `MineFirst`), `tmx.rs:575 existing_pair_keys` (xem trước nhập). Không phải đường đề xuất nên không vi phạm AD-57, nhưng câu "mọi chỗ đọc" cần nói rõ phạm vi *đường đề xuất*.

Hệ quả: nếu Story 12.14 làm theo chữ AD (đặt tham số chính sách ở `read_tier_rows` + `pairs_for_source`), bộ lọc xuất TMX phải được thêm riêng vào `distinct_tier_pairs`, và "tham số bắt buộc, không mặc định" không ép được nó vì hàm đó gọi `load_all_pair_rows` trực tiếp. Đề nghị: AD nêu `distinct_tier_pairs` là đường thứ ba, hoặc đặt hàm lọc ở mức `load_all_pair_rows`/`query_pair_rows`.
Ngoài ra: concordance (`rank_concordance`) và danh sách quản lý cũng đi qua `read_tier_rows` ⇒ cần truyền `MineFirst` tường minh ở đó; AD mục 4 đã nói đúng, chỉ cần nhắc rằng đây là hai chỗ gọi `read_tier_rows` phải chọn `MineFirst`.

### F3 — TRUNG BÌNH — AD-56 mục 5 và AD-53 mục 7: "ngoại lệ có tên *`commands/proofread.rs` không ghi `project.db`*" không tồn tại trong test

- `grep` toàn `src-tauri/tests`, `src`, `scripts`: không có phép kiểm nào mang ý đó. `tests/proofread_write_boundary.rs` chỉ canh `write_non_user_target(` (`WRITE_CALL`, dòng 490) và `ACCEPT_FILE = commands/segment/proofread_accept.rs` (dòng 488), không canh ghi `project.db`. `tests/ai_boundary.rs:533-` chỉ đóng băng đường dẫn `crate::core::ai::` mà `commands/proofread.rs` được nêu.
- Chữ "không ghi `project.db`" chỉ còn ở doc-comment đầu `commands/proofread.rs:1-9` (đã sửa thành "write only the not-an-error memory and the scan count") và ở spine (`architecture-auratranslate.md` dòng 981, "9.6 sở hữu … ngoại lệ có tên cho phép thử"). 9.6 đã ship (`commands/proofread.rs:402` `INSERT OR IGNORE INTO proof_ignore`, `:485` `INSERT INTO proof_scan`) mà không dựng cổng đó.
- Hệ quả: AD-56 "nhận thêm đúng hai lượt ghi `proof_decision`" nói về một ngoại lệ chưa hề được viết thành cổng; không có gì canh để hai lượt ghi đó là *đúng hai*.
Đề nghị: sửa câu thành "không có cổng nào hôm nay; story 12.15 thêm cổng đếm lượt ghi `INSERT INTO proof_decision` trong `commands/proofread.rs` và `commands/segment/proofread_accept.rs`" hoặc xoá tham chiếu.

### F4 — TRUNG BÌNH — AD-56 mục 2: `ai_proofread_accept` và `ai_proofread_revert` dùng chung một hàm ghi; chưa có chỗ trả `decision_id` trên dây

- `commands/segment/wire.rs:471` (`ai_proofread_accept`) và `:517` (`ai_proofread_revert`) đều gọi `super::replace_proofread_range(...)` với cùng chữ ký 7 tham số, trả `PromoteAiTranslationOutcome`. Không có tham số hay trường `decision_id`.
- AD-56 đòi: accept chèn `proof_decision` và trả `decision_id`; revert nhận `decision_id`, xoá đúng hàng, xoá 0 hàng ⇒ `ai_proofread.decision_missing`. Hai nhánh này khác nhau về bản chất, nên `replace_proofread_range` phải biết đang là accept hay revert (hiện chỉ phân biệt qua tham số `expected_phrase`/`replacement` đảo chiều). Đây không phải sai sót của AD mà là chi tiết dựng cần ghi: `PromoteAiTranslationOutcome` được dùng chung với `promote_ai_translation` (AI → Editor) nên thêm trường vào đó làm lệch DTO kia; cần một kiểu kết quả riêng cho accept, hoặc `ipc_contract` ghim hai hình dạng.
- Giữ nguyên: lượt chèn nằm "sau phép kiểm AD-53 mục 4 và lượt ghi văn bản" khớp luồng `proofread_accept.rs` (`ACCEPT_FILE` có đúng một lời gọi `write_non_user_target`).

### F5 — TRUNG BÌNH — AD-54 mục 2 / AD-57 mục 1: "cùng khuôn `status_override`" không đúng ở điểm có `CHECK`/`NOT NULL`

- `WORK_STATUS_OVERRIDE_DDL = "ALTER TABLE work ADD COLUMN status_override TEXT;"` (`schema.rs:887`): **không** `CHECK`, **không** `NOT NULL`, **không** `DEFAULT`. Doc-comment dòng 876-879 nói thẳng "không bằng `CHECK … IN (…)`".
- `tm_only_mine INTEGER NOT NULL DEFAULT 0 CHECK (tm_only_mine IN (0, 1))` hợp lệ trên bảng có hàng (đo: hàng có sẵn nhận `0`, `UPDATE =2` bị CHECK, `=NULL` bị NOT NULL) nhưng **đây là khuôn khác**: khuôn `NOT NULL DEFAULT` đã có sẵn ở `segment.status`/`segment.translation_origin` (`schema.rs:1655,1934`) và khuôn bậc `ALTER` riêng ở `status_override`. AD nên nói "bậc `ALTER` riêng như `status_override`, cột như `segment.status`".
- Ngoài ra: lược đồ kết hợp `ADD COLUMN … CHECK` chưa có tiền lệ trong `schema.rs` (grep `ADD COLUMN[^"]*CHECK` không có kết quả). Đã đo chạy được trên cả hai phiên bản SQLite, nên chỉ là ghi chú: đây là lần đầu repo dùng nó; test `the_project_migration_set_…` và các cổng so lược đồ cần bao nó.

### F6 — THẤP — AD-55 mục 1: phép kiểm tập lúc mở kho "phủ thêm cột này" cần thêm một nhánh, và tập khác nhau

- Phép kiểm thật: `commands/project/mod.rs:3126` `reject_unknown_translation_origin`, truy vấn `UNION` hai cột của `segment`, `value NOT IN (TRANSLATION_ORIGINS…)`, `TRANSLATION_ORIGINS: [&str; 4]` gồm `''` (`commands/segment/mod.rs:107`). Nó chạy **chỉ** cho `project.db`.
- Thêm `segment_version.translation_origin` vào cùng `UNION` sẽ nhận `''` (vì dùng chung tập 4 giá trị) trong khi cột này `CHECK` chỉ cho 3 giá trị. Có hai hệ quả nhỏ: (a) `CHECK` đã chặn `''` nên nhánh kiểm không bắt được gì mà CHECK chưa bắt; (b) `NULL NOT IN (…)` ra `NULL` nên hàng cũ `NULL` không bị bắt (đã đo, rỗng) — đúng ý AD. Giá trị thực của phép kiểm trên cột này chỉ là bắt tệp bị sửa tay bằng `PRAGMA ignore_check_constraints`; AD nên nói rõ lý do giữ hoặc bỏ nhánh này.
- Không phải lỗi, nhưng lưu ý: `CHECK` trong `ADD COLUMN` không đổi được nữa (SQLite không có `ALTER … CHECK`); thêm giá trị thứ tư sau này phải dựng lại bảng `segment_version` (có `AUTOINCREMENT` + index `idx_segment_version_segment_created` phải dựng lại). AD-55 Prevents 3 đã nói cần AD mới và bước di trú, khớp.

### F7 — THẤP — Tên gọi

- `PROOF_IGNORE` (AD-53 dòng 919, AD-56 mục 1): là tên *thực thể*; bảng thật là `proof_ignore`, DDL ở hằng `PROOFREAD_MEMORY_DDL` (`schema.rs:988`, "Step 34"). AD-56 dòng 1087 đã trỏ đúng hằng; dòng 1070 "cấu trúc như `PROOF_IGNORE`" nên đọc là bảng `proof_ignore`.
- `OpenWorkState` là kiểu `Mutex<Option<OpenWork>>` (`commands/project`), tồn tại; AD-57 mục 2 dùng đúng.
- `restoreFocusAndSelection` tồn tại **hai** bản private: `src/glossaryConfirmStripState.ts:315` (không export) và `src/glossaryQuickAddState.ts:285`. AD-54 Prevents 5 trỏ đúng tệp; chỉ lưu ý nó không export và chú thích dòng 313 của chính tệp nói "đúng khuôn `glossaryQuickAddState.ts::restoreFocusAndSelection`" (bản sao, không dùng chung).

## Bảng xác nhận tên

| Tên AD nêu | Có thật? | Vị trí | Ghi chú |
|---|---|---|---|
| `confirm_translation` | có | `core/glossary/store.rs:258` | cả chốt lần đầu và sửa mục đã chốt (F1) |
| `glossary_confirm_pending_translation` | có | `commands/glossary.rs:519` (pure) · `:1270` (wire) | |
| `update_manual_term` | có | `core/glossary/store.rs:1024` | `translation: Option<&str>`; luôn `SET translation` |
| `is_confirmed()` | có | `core/glossary/entry.rs:233` | `translation.is_some()` |
| `Arbitrated::Unsigned` | có | `core/segment/translation_origin.rs:14` | trả khi `target` rỗng sau trim+NFC |
| `diff_spans` | có | `core/matching/mod.rs:490` | `(old, new, lang) -> Vec<DiffSpan>` |
| `MatchLang::En` | có | `core/matching/mod.rs:173` | biến thể `Zh`, `En` |
| `ReviewSpans.vue` | có | `src/panels/ReviewSpans.vue` | |
| `SegmentHistoryOverlay.vue` | có | `src/SegmentHistoryOverlay.vue` | |
| `SegmentVersionRow` | có | `commands/segment/history.rs:19` | 4 trường; doc-comment dòng 11-17 nói rõ "KHÔNG cột xuất xứ"; `ipc_contract.rs` chưa ghim hình dạng này (grep 0 kết quả cho tên) |
| `ipc_contract` | có | `src-tauri/tests/ipc_contract.rs` (+ `ipc_argument_contract.rs`) | `baseline_*` được nhắc ở `:2554` |
| `ai_proofread_accept` / `ai_proofread_revert` | có | `commands/segment/wire.rs:471,517` | cùng gọi `replace_proofread_range` (F4) |
| `proofread_ignore_add` | có | `commands/proofread.rs:372` | `INSERT OR IGNORE INTO proof_ignore` ở `:402` |
| `PROOF_IGNORE` | tên thực thể | bảng `proof_ignore`, `schema.rs:989` | F7 |
| `proof_scan.reported` | có | `schema.rs:1000-`; ghi ở `commands/proofread.rs:485` | |
| `read_tier_rows` | có | `core/tm/mod.rs:328` | |
| `pairs_for_source` | có | `core/tm/mod.rs:217` | |
| `merge_tagged_tiers` | có, **private** | `core/tm/mod.rs:243` | F2 |
| `fill_exact_tm_matches` | có | `commands/segment/chapter_read.rs:516` | private fn |
| `tm_export_tier` | có | `commands/tm.rs:570` | đi qua `distinct_tier_pairs` (F2) |
| `status_override` | có | `schema.rs:887`, `work` | không CHECK (F5) |
| `OpenWorkState` | có | `commands/project` (dùng ở `lib.rs:681,1648`) | |
| `scope_kinds!` | có | `core/scope/kinds.rs:96` | 9 loại ⇒ "loại thứ 10" đúng |
| `scripts/measure-proofread-false-alarm.mjs`, `scripts/lib/falseAlarm.mjs`, `src/proofreadState.ts` | có | | |
| ngoại lệ "`commands/proofread.rs` không ghi `project.db`" | **không có cổng** | | F3 |
| phép kiểm tập lúc mở kho (AD-50 mục 1) | có | `commands/project/mod.rs:3126` | F6 |
| `GLOSSARY_ENTRY_DDL` + trigger `glossary_entry_lifecycle_is_one_way` | có | `schema.rs:300-331`; dựng lại ở `:521-564` (GLOBAL ở `:773,779`; PROJECT `:2243,2266`) | "hai bậc dựng lại trigger" đúng |
| trigger zero-width UPDATE | có | `schema.rs:625` `BEFORE UPDATE OF translation, note` | không bị đụng bởi AD-54 (đo T10) |

## Kiểm các giả định hành vi (mục 3 của yêu cầu)

1. **Đường xác nhận không bao giờ sinh `''`**: đúng. `confirm_segment` (`confirm.rs:176-290`): `target_text.trim().is_empty()` ⇒ `NothingToConfirm` (dòng 237); `arbitrate` trả `Unsigned` khi rỗng sau `trim`+NFC ⇒ cũng `NothingToConfirm` (dòng 261-264); nhánh còn lại luôn `PairOrigin` (3 giá trị). Chèn `segment_version` ở `:278-282` nằm sau hai cổng đó và là `INSERT INTO segment_version` duy nhất trong `src/` (đã grep). Nhưng câu `INSERT` hiện **không** có cột `translation_origin` ⇒ khi trigger `BEFORE INSERT … RAISE` được thêm, câu này ABORT (đã đo) cho đến khi story sửa nó ở **cùng bậc** — AD-55 mục 2 đã đòi đúng như vậy; cần đảm bảo cùng commit.
2. **Khôi phục không động tới `translation_origin`**: đúng. `write_non_user_target(tx, id, text, TRANSLATION_ORIGIN_OTHER, None)` (`history.rs:~372`; hàm ở `targets.rs:234`) dùng `COALESCE(?3, translation_origin)` với `?3 = None`. Hôm nay nó đặt `baseline_translation_origin = 'other'` cứng — đúng chỗ AD-55 mục 3 muốn đổi thành xuất xứ của phiên bản (nhánh `NULL` ⇒ `other` giữ nguyên hành vi). Nó cũng đặt `status = draft` và `baseline_target_text = target_text`.
3. **Lưu của người dùng không đổi `status`**: đúng cho lượt lưu (`targets.rs:150-153` chỉ `SET target_text, updated_at`). Hạ `confirmed → draft` là lệnh riêng `unconfirm_edited_segments` (`confirm.rs:570-`, `:655`), không phải lượt lưu.
4. **Mọi bên dùng TM đều qua ba hàm**: không (F2).
5. **Hàng `segment_version` ghi ở nơi khác**: chỉ `confirm.rs:279`. Gộp/tách/hoàn nhập không sao chép `segment_version` (grep `segment_version` ngoài `history.rs`/`schema.rs`: không có câu `INSERT/UPDATE/DELETE`). Nên AD-55 "lượt chèn duy nhất" đúng.

## Đo SQLite: kết quả chính xác

Script: `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/f2d5ec34-38f5-4006-9afe-b7e29779f21d/scratchpad/{ad54,ad55,ad56_57}.sql`; harness `run.c`; đầu ra đầy đủ `out_bundled.txt` (3.53.2) và `out_local.txt` (3.43.2). Dưới đây là đầu ra 3.53.2.

Trigger thử (thân đúng như AD-54 mục 4, đọc theo chữ):

```sql
ALTER TABLE glossary_entry ADD COLUMN pending_confirmed_at TEXT NULL
  CHECK (pending_confirmed_at IS NULL OR translation IS NOT NULL);
DROP TRIGGER glossary_entry_lifecycle_is_one_way;
CREATE TRIGGER glossary_entry_lifecycle_is_one_way
BEFORE UPDATE OF translation, pending_confirmed_at ON glossary_entry
WHEN
  (OLD.translation IS NOT NULL AND NEW.translation IS NULL
     AND NOT (OLD.pending_confirmed_at IS NOT NULL AND NEW.pending_confirmed_at IS NULL))
  OR (OLD.translation IS NOT NULL AND NEW.translation IS NOT NULL AND NEW.translation <> OLD.translation
     AND NEW.pending_confirmed_at IS NOT NULL)
  OR (OLD.pending_confirmed_at IS NULL AND NEW.pending_confirmed_at IS NOT NULL AND OLD.translation IS NOT NULL)
BEGIN SELECT RAISE(ABORT, 'glossary lifecycle is one-way'); END;
```

| Ca | Kỳ vọng | Kết quả 3.53.2 | Kết quả 3.43.2 |
|---|---|---|---|
| `ALTER ADD COLUMN … CHECK` trên bảng 3 hàng (1 chờ chốt, 2 đã chốt) | chạy | chạy; 3 hàng ts `NULL` | giống |
| `DROP TRIGGER` + `CREATE TRIGGER` | chạy; còn 3 trigger (lifecycle + 2 zero-width) | đúng | giống |
| T1 chốt `NULL→'A'` + ts | qua | `1|A|2026-10-13` | giống |
| T2 `UPDATE translation=NULL` trên mục có ts | ABORT | `rc=19 glossary lifecycle is one-way` | giống |
| T2b `UPDATE translation=NULL` trên mục đã chốt cũ (ts NULL) | ABORT | ABORT | giống |
| T3 gỡ chốt: `translation=NULL, ts=NULL` | qua | `1|NULL|NULL` | giống |
| T3b sửa chữ đã chốt, giữ ts | ABORT | ABORT | giống |
| T4 sửa chữ và đặt ts `NULL` cùng câu | qua | `1|A2|NULL` | giống |
| T5 đặt ts trên mục đã chốt cũ, chữ không đổi | ABORT | ABORT | giống |
| T5b chốt lại mục đã chốt cũ: chữ mới + ts | ABORT (bullet 3 và 2) | ABORT | giống |
| T6 `INSERT` mục `translation NULL` + ts có giá trị | CHECK fail | `CHECK constraint failed: pending_confirmed_at IS NULL OR translation IS NOT NULL` | giống |
| T7b `SET translation='D', note='n2'` (chữ không đổi) trên mục có ts | qua (dạng `update_manual_term`) | `d|D|n2|t` — ts giữ nguyên | giống |
| T8 dạng `TakeTheirs` (đổi chữ, không xoá ts) | — | **ABORT** | giống |
| T9 dạng nhận ứng viên `review_harvest` (đổi chữ + `term_origin`) | — | **ABORT** | giống |
| T10 trigger zero-width UPDATE vẫn bắn (mục không ts) | ABORT | `glossary entry text is zero-width only` | (không chạy riêng; script T10 gốc bị lifecycle che — đã chạy lại sạch bằng `t10full.sql` trên 3.53.2) |

Ghi chú: một ca gỡ chốt hợp lệ kèm xoá ts được; một `UPDATE translation=NULL` sơ ý bị chặn như hôm nay; cột mới chỉ có giá trị qua `NULL→text` + ts đúng như AD-54 mục 4 cuối. Hai trigger zero-width (`BEFORE INSERT` và `BEFORE UPDATE OF translation, note`) còn nguyên sau `DROP/CREATE` riêng trigger một chiều. **Không cần dựng lại bảng** (khác tiền lệ `GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL`, vì đó là đổi `CHECK` trong `CREATE TABLE`; ở đây chỉ `ADD COLUMN` + `DROP/CREATE TRIGGER`). Cả hai thang (global/project) dùng cùng hằng nên `sqlite_sequence`/index không bị đụng.

AD-55:

| Ca | Kết quả 3.53.2 |
|---|---|
| `ALTER TABLE segment_version ADD COLUMN translation_origin TEXT NULL CHECK(IS NULL OR IN ('self','other','bilingual_import'))` trên 2 hàng | chạy; hàng cũ `NULL` |
| `CREATE TRIGGER … BEFORE INSERT … WHEN NEW.translation_origin IS NULL RAISE(ABORT)` | chạy |
| `INSERT … translation_origin NULL` | ABORT `segment_version needs translation_origin` |
| `INSERT` **bỏ cột** (đúng câu `confirm.rs:279` hôm nay) | ABORT cùng thông điệp (⇒ phải sửa câu này cùng bậc) |
| `INSERT` với `'self'`, `'other'`, `'bilingual_import'` | qua |
| `INSERT` với `''` hoặc `'ai'` | `CHECK constraint failed` |
| `UPDATE` cột khác trên hàng cũ `NULL` | qua (trigger chỉ `BEFORE INSERT`) |
| Truy vấn kiểu `value NOT IN (…)` trên hàng `NULL` | rỗng (không bắt) |
| `current_version_id` = `max(id)` các hàng có `target_text` bằng | `1` trong mẫu thử |

AD-56 (DDL thử nghiệm `proof_decision` theo mục 1): `outcome` ngoài ba giá trị và `kind` ngoài năm giá trị đều bị `CHECK`; `DELETE … WHERE id=999 AND outcome='accepted'` ⇒ `changes()=0` (cơ sở cho `decision_missing`); `INSERT OR IGNORE INTO proof_ignore` lần hai ⇒ `changes()=0` còn hàng `proof_decision` phải chèn riêng (đúng ý "kể cả khi chữ ký đã có").

AD-57: `ALTER TABLE work ADD COLUMN tm_only_mine INTEGER NOT NULL DEFAULT 0 CHECK (tm_only_mine IN (0, 1))` trên bảng có 1 hàng: hàng nhận `0`; `=2` ⇒ CHECK fail; `=NULL` ⇒ `NOT NULL constraint failed: work.tm_only_mine`.

## Đối chiếu bổ sung

- AD-54 Prevents 1: câu "`update_manual_term` nhận `Option<String>`" thực tế là `Option<&str>` (`store.rs:1030`) — chi tiết, không đổi lập luận.
- AD-54 mục 5: `glossary_unconfirm_translation` chưa có trong mã (grep 0 kết quả) — đúng, đây là hạng mục dựng mới.
- AD-55 mục 4: `diff_spans` (`core/matching/mod.rs:490`) với `MatchLang::En` dùng `similar::TextDiff::from_words` sau `trim`+NFC (`mod.rs:497-500`), tức tách theo từ (khoảng trắng) kiểu Unicode, không dùng bộ tách `is_ascii_alphanumeric` của `tokenize` (dòng 252, dành cho chấm điểm tương tự). Chữ Việt có dấu không bị cắt. Khớp AD-55 "tách theo từ". Chưa chạy thử trên một câu tiếng Việt thật; chỉ đọc mã.

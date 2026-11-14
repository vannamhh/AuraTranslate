# Handoff — plan-tra-tu-nhanh-trong-bang-mod-k

Ghi chú làm việc giữa các phase. Plan là nguồn sự thật duy nhất; tệp này chỉ để phase sau biết phase trước để lại gì. Không chép vào spec.

Các phase: 1 AD + correct-course PRD · 2 Rust + dict-build · 3 Webview · 4 Test cùng đối chứng đỏ.

## Phase 1 — AD + PRD

Xong, chưa commit. Cây có hai tệp spine/PRD sửa, hai `.memlog.md`, và thư mục `change-tra-tu-nhanh-trong-bang-mod-k/`. Không viết code.

**Đọc đúng chỗ, đừng đọc cả spine (344 KB):** `grep -n '^### AD-69' architecture-auratranslate.md` rồi đọc từ đó tới `## Consistency Conventions`. Đọc FR161 ở PRD §6.3 sau FR41.

**AD-69 chốt, phase 2 theo đúng:**
- Wire trả nhóm với định danh `pinyin` · `vi_zh` · `en_vi` · `zh_vi`; mỗi nhóm: hàng (20) + `has_more` + một trong bốn trạng thái (có hàng · rỗng thật · `OutdatedDictionary` · lỗi). Route `Zh` chỉ ra `zh_vi`.
- Schema v4: `dict_entry.reading_key` + `idx_entry_reading_key`; `dict_source.gloss_lang` (`vi`/`en`/NULL). `SCHEMA_VERSION` và `SUPPORTED_SCHEMA_VERSION` lên 4, `MINIMUM_SCHEMA_VERSION` GIỮ 3. Test `tests/dict_sources.rs::the_supported_schema_version_matches_dict_build` đọc hai hằng, sửa cùng lúc.
- `gloss_lang` đo mẫu trên v3: `vi` = cvdict, viwiktionary, viwiktionary-en, thieu-chuu, vietphrase, tran-van-chanh; `en` = cc-cedict, en-wiktionary, unihan; NULL = en-wiktionary-vi (không có `dict_sense`).
- Việt→Trung lọc CẢ `e.lang='zh'` VÀ `s.gloss_lang='vi'`. Một mình `gloss_lang` lấy cả viwiktionary-en (đầu mục `lang='en'`, 190.543 nghĩa Việt).
- `pinyin_key`: cắt từ `(` đầu tiên, hạ chữ thường, bảng đóng dấu thanh (không thêm crate vào dict-build, NFR15; src-tauri đã có `unicode-normalization` nhưng hai bản phải cùng bảng), `ü ǖ ǘ ǚ ǜ u:` → `v`, bỏ số thanh (kể cả chữ mũ) và dấu câu, ký tự lạ ⇒ khoá NULL. Hàng có `reading` mà khoá NULL phải đếm ra 0 hoặc kê tên.
- Khoảng pinyin: `reading_key >= ?1 AND reading_key < ?1 || '{'`. Hạng Việt→Trung ①② lấy bằng truy vấn neo đầu `gloss : ^"…"` (đã chạy thử SQLite 3.43.2), hạng ③ khớp thường có `LIMIT`. Mỗi nhóm lấy 21, trả 20.
- Phân loại dấu dùng `classify_query` + `fold_diacritics_case_preserving` (`core/library/indexer.rs:1415`, `:1239`): dời xuống mô-đun trung lập, không chép, không để `dict` import `library`.

**Số đo 2026-11-14 (dict-core.db v3, sqlite3 3.43.2, tải 4,2):** `^"ăn"` 474 hàng (194 sau lọc), `"người"` 17.629 (6.865 sau lọc) trong 0,03 s, `^"người"` 2.655 sau lọc. CHỈ `dict-core.db`; `dict-vietphrase.db` (679.302 nghĩa Việt) chưa đo ⇒ nguy cơ lớn nhất cho p95 < 100 ms ở câu như `người`. Đỏ ⇒ quay lại AD-69, không nới ngưỡng.

**Bẫy cho phase 2:**
- Số đếm trong AD (14.219 hàng ngoặc, 14.946 chữ mũ, 2.404 `u:`, hai hàng chữ sau `)`: `靠背兒椅子`, `筷兒筒`; 47 hàng `(` không đóng) là trên bản v3. Chạy lại trên bản v4 dựng mới; số khác ⇒ sửa AD, không sửa số cho khớp.
- Kích thước bốn tệp v4 so NFR6 (trần 500.000.000, macOS 432.363.050 ngày 2026-10-21) CHƯA đo; AD-69 mục 9 bắt story ghi số.
- `quick_lookup` phải đọc phiên bản của TỪNG tệp trước khi chạm cột v4; `list_source_attributions` vẫn chạy trên v3.
- `tools/dict-build/out/` đang là dict-v3 (ngày 2026-08-10). Dựng lại bằng `--layer all`, rồi chép sang `target/debug/dict` (xem memory `tauri-dev-doc-tu-dien-o-target-debug-dict`). Không commit `.db`; không tự đưa lên Release.

**Việc phase 1 CHƯA làm, có chủ ý:**
- Reviewer Gate cho AD-69 (rubric/reality/adversary như AD-68). Ice quyết có chạy hay không; spine ghi "chờ Ice ký".
- Ticket và epic file (Sofia), `EXPERIENCE.md`/`DESIGN.md` (Sally), nợ `Chủ: Ice` cho release `dict-v4` và lượt thử app thật (ghi khi story đóng, theo AGENTS.md).
- Năm điểm Winston tự đề xuất thêm ngoài quyết định của Ice (liệt kê ở mục 3 của change proposal): Ice chưa ký riêng.

**Đã kiểm:** `npm run check:debt-owner` xanh. Không có cổng nào khác đọc spine/PRD; không chạy vitest hay cargo (chỉ tài liệu).

- Đo thêm (điều phối, 2026-11-14, `dict-vietphrase.db` v3, sqlite3 3.43.2, cache nóng, máy tải 6,5–8): `sense_fts MATCH '"người"'` cho 23.293 hàng trong 10 ms; có `ORDER BY` nghĩa trùng khít và `LIMIT 21` thì 34 ms. `sense_fts_nd MATCH '"nguoi"'` cho 33.882 hàng trong 8 ms. Mới chỉ là SQL thô, chưa phải p95 qua Rust.

## Phase 2 — Rust + dict-build

Xong, chưa commit. Không có webview. Bốn `.db` v4 dựng lại ở `tools/dict-build/out/` (không vào git), manifest đã sửa.

**Wire `quick_lookup` (lệnh Tauri `quick_lookup`, một đối số `query: String`; webview gửi chuỗi đã cắt khoảng trắng):**

```
QuickLookupResponse {
  route: "zh" | "en",
  query_truncated: bool,          // câu > 200 ký tự bị cắt
  layers_loaded: bool,            // false = chưa nạp lớp nào: phải nói tên, đừng coi là "không có kết quả"
  skipped: string[],              // mã máy của lớp hỏng/không nạp (như panel Lookup)
  groups: QuickGroup[]
}
QuickGroup {
  id: "pinyin" | "vi_zh" | "en_vi" | "zh_vi",
  status: "rows" | "empty" | "outdated_dictionary" | "error",
  rows: QuickRow[],               // tối đa 20
  has_more: bool,
  incomplete: bool,               // có hàng, nhưng ít nhất một lớp không trả lời (v3 hay lỗi)
  error: IpcError | null          // chỉ khi status = "error"; hiện qua tError()
}
QuickRow {
  headword: string, headword_simp: string|null, reading: string|null (nguyên văn, còn dấu thanh/số thanh),
  gloss: string|null,             // nghĩa đầu của dòng, đúng nguồn của dòng
  source_code: string, source_name: string,
  source_count: number            // số nguồn có đầu mục này
}
```
- Chữ Hán (`route: "zh"`) ⇒ đúng một nhóm `zh_vi`. Còn lại (`"en"`) ⇒ ba nhóm, đúng thứ tự `pinyin`, `vi_zh`, `en_vi`; nhóm `empty` ẩn, trừ khi cả ba `empty` ⇒ "không có kết quả" có tên.
- `zh_vi` và `en_vi` là CÙNG đường với panel Lookup (`commands::dict::lookup`, kể cả lùi sang chuỗi con ≤ 4 ký tự), nên xem trước bằng `lookupDictionary(headword)` ra đúng dòng này.
- Mỗi dòng là một đầu mục (`headword`), không phải một nghĩa. Dòng ưu tiên nguồn có nghĩa tiếng Việt (v4); tệp v3 không có thông tin đó nên theo thứ tự nguồn của panel.
- Không có hàng đủ-bản-ghi: khung xem trước phải gọi `lookupDictionary`. Cột pinyin của dòng lấy từ `reading` bằng `toneMarkedReading`.
- Nhóm lỗi (`error`) và `outdated_dictionary` KHÔNG được ẩn; `outdated_dictionary` nghĩa là "cần cập nhật bộ từ điển" (tệp v3 hoặc chưa cập nhật bản v4).
- Thứ tự dòng: pinyin = khớp hẳn khoá trước, tiền tố sau, trong hạng thì đầu mục ngắn trước rồi điểm mã; `vi_zh` = nghĩa trùng khít, nghĩa bắt đầu bằng câu tra, còn lại.

**Đã đổi (đường dẫn):** `tools/dict-build/src/{schema,sources_meta,insert,pinyin,lib}.rs`, `tests/{pinyin_key,reading_key_real_data,layers,schema}.rs`, `tests/fixtures/pinyin_key.tsv`; `dict-manifest.toml`; `src-tauri/src/core/{diacritics.rs (mới), dict/{pinyin,quick}.rs (mới), dict/{layer,mod}.rs, library/indexer.rs, mod.rs}`, `commands/dict.rs`, `lib.rs`; tests `quick_lookup.rs` (mới), `dict_boundary.rs` (sàn DICT_FLOOR 6, SRC_ONLY_RS_FLOOR 144), `dict_lookup.rs`/`dict_sources.rs` (bản chép DDL v4).

**Quyết định / bẫy cho phase sau:**
- `classify_query` + `SearchMode` + `fold_diacritics_case_preserving` dời sang `core/diacritics.rs`; `library::indexer` còn `pub use` hai tên đầu nên đường dẫn cũ vẫn chạy.
- `DictLayer` giữ `schema_version` và `gloss_lang` theo nguồn (chỉ v4). Câu SQL chạm cột v4 chỉ chạy khi `schema_version() >= 4`.
- `vi_zh`: ba lượt mỗi tệp — anchored `^"…"` có cửa sổ độ dài (trùng khít), anchored thường, rồi khớp thường; KHÔNG `ORDER BY` (FTS tự trả rowid tăng và dừng ở `LIMIT`; sắp theo độ dài nghĩa trên 6.865 hàng làm một tệp chậm 30 ms). Thứ tự cuối do Rust xếp (hạng, rồi nghĩa gốc-chưa-gấp-dấu khớp, độ dài nghĩa, đầu mục, id).
- Số đo AD đã kiểm trên v4 dựng mới: 14.219 hàng ngoặc, 14.946 chữ mũ, 2.404 `u:` KHỚP; hai hàng chữ sau `)` KHỚP; 47 hàng `(` không đóng KHÔNG khớp: đo ra 0 hàng (so số `(` với số `)` từng hàng). AD-69 mục 3 ghi "47", cần Winston sửa (hoặc kê nguồn con số 47).
- 70 hàng của `dict-core.db` có `reading` mà `reading_key` NULL (phương ngữ/Quảng với `ȵ ɡ å ů`, và dấu kết hợp NFD như `ê̄`, `m̄`). AD đòi "0 hoặc kê tên": đang kê bằng số 70 ở `tests/reading_key_real_data.rs` (không kê từng hàng). Ba tệp còn lại 0.
- Kích thước bốn `.db` v4: 208.252.928 + 5.894.144 + 167.567.360 + 11.960.320 = 393.674.752 byte (v3 cũ: 195.813.376 + 5.795.840 + 160.788.480 + 10.842.112 = 373.239.808; +20,4 MB). Bản cài so NFR6 (trần 500.000.000, macOS 432.363.050) CHƯA đo.
- SHA-256 mới (một lượt dựng 2026-11-14, chưa dựng lượt hai; test fixture "hai lượt cùng sha" xanh): core 53d99afc…f731, thieu-chuu c001e2b4…436f, vietphrase e0ef84b0…6e44, tran-van-chanh 361c31ba…d1d7. URL đã trỏ `dict-v4`; Release CHƯA có ⇒ `scripts/fetch-dicts.mjs`/`release.yml` không tải được cho tới khi Ice tải lên (nợ `Chủ: Ice`, ghi khi story đóng).
- `tools/dict-build/out/` v3 cũ được sao ra scratchpad trước khi dựng đè. `target/debug/dict` CHƯA chép (webview/app thật cần `cp tools/dict-build/out/*.db src-tauri/target/debug/dict/` hoặc `npm run predev`).

**Đo NFR1 (`cargo test --test quick_lookup -- --ignored --nocapture bench_quick_lookup_on_the_real_dictionaries`, `AURA_QUICK_BREAKDOWN=1` in thêm từng nhóm):** 480 lượt (160 câu tra khác nhau × 3, bốn hình dạng, toàn từ phổ biến: `người`, `một`, `a`, `the`, `con`), dict-core v4 thật cùng ba tệp kia, sqlite3 bundled, cache nóng.
- Bản debug, tải máy 5–8: p50 13,8 ms, p95 59,5 ms, max 89,5 ms (`a`). Câu chậm nhất là nhóm `vi_zh` của từ Latin thường gặp (`a`, `nguoi`, `the`, `ban`).
- Bản release (lượt đầu, TRƯỚC khi bỏ `ORDER BY`): p50 8,6 ms, p95 129,9 ms ⇒ đỏ, là chỗ dẫn tới bản sửa trên. Chưa đo lại release sau bản sửa (dựng release mất ~12 phút).

**Test (chỉ phần phase 2):** `cargo test` trong `tools/dict-build` xanh (cả `reading_key_real_data -- --ignored` xanh); `cargo test --test quick_lookup --test dict_boundary --test dict_lookup --test dict_sources` xanh. Đối chứng gỡ thật đã chạy (đỏ đúng ca): gỡ vế `lang = 'zh'`; gỡ vế `gloss_lang`; gỡ phép so phiên bản; lấy 20 thay vì 21; cho `ăn` qua `sense_fts_nd`; gỡ nhánh `ü → v` ở MỖI crate; bỏ khai `gloss_lang` một nguồn; đổi bảng thanh `ā` ⇒ hàng không có khoá; Hán cũng chạy nhóm pinyin. Chưa làm đối chứng (k) gỡ `dict-vietphrase.db` và (c) `pick_route` một lần (đường tra panel đã gọi nó đúng một lần và `quick_lookup` không gọi lại).

**Chạm dây chung ⇒ đã chạy đủ `cargo test --no-fail-fast` trong `src-tauri` một lần (debug, máy tải cao):** xanh TRỪ `ipc_argument_contract::every_registered_commands_business_parameters_match_camel_case_across_the_wire`, đỏ vì `quick_lookup` đã đăng ký ở `lib.rs` mà chưa có `invoke('quick_lookup')` nào trong `src/**/*.ts`. Phase 3 thêm adapter `quickLookup` ở `src/config/dict.ts` (đối số `{ query }`) thì cổng này xanh; chạy lại `cargo test --test ipc_argument_contract` sau đó. Để xanh phần còn lại, đã nâng 24 sàn `*RS_FLOOR*` của `tests/*_boundary.rs` và vài tệp khác từ 133/135 lên 144 (thêm ba tệp `.rs` dưới `src/`), `tests/config_invariants.rs` (`COMMAND_FILE_CENSUS` của `commands/dict.rs` 4→5, tổng 139→140; lệnh khai plain, cùng khuôn `lookup_dictionary`).

**Phase 4 vẫn nợ:** vitest cho chế độ/IME/phản hồi cũ/xem trước/Enter; đối chứng (i) (j); kiểm Ice ở app thật.

## Phase 3 — Webview

Xong, chưa commit. Không đụng Rust.

**Đã đổi:** `src/config/dict.ts` (`quickLookup` + kiểu wire + kiểm hình dạng lúc chạy; hình sai ⇒ `ipc.unknown`, không rỗng), `src/paletteLookupState.ts` (mới), `src/paletteState.ts`, `src/PaletteOverlay.vue`, `src/panels/LookupRecord.vue` (prop `noPin`), `src/commands/index.ts` (`palette.lookup_mode`, `palette: true`, dep `enterPaletteLookupMode`), `src/main.ts` (`paletteCommandDeps({ runLookup })`), `src/i18n/vi.json` (`palette.lookup.*`, `command.palette.lookup_mode`), `tests/frontend/paletteLookup.test.ts` (mới, 13 ca), sàn `DISPATCH_FLOOR` 445→475 và `HANDLER_ATTR_FLOOR` 193→206 (`check-commands.mjs`), `RS_FLOOR` 134→145 (`check-i18n.mjs`, trôi từ phase 2).

**Quyết định / bẫy cho phase 4:**
- Chế độ tra nằm ở `paletteLookupState` (`paletteLookupActive`); `paletteState` gọi sang, không ngược lại. Hai danh sách Tác phẩm/Lệnh trả `[]` khi chế độ bật nên `paletteItems` rỗng.
- Ô gõ dùng chung `paletteQuery`; `/` đầu ô vào chế độ và KHÔNG giữ lại. ⚠️ `query` có thể không đổi (`''` → `''`) nên Vue không vẽ lại ô: `onInput` gán lại `el.value` tay. Gỡ dòng đó thì ô còn chữ `/`.
- IME: `setPaletteQuery(value, composing)`; `compositionend` gọi lại không-composing. Guard `sequence` ở `queryPaletteLookup`, riêng `previewSequence` cho xem trước.
- Dòng cũ giữ trên màn hình khi câu mới đang bay, nhưng Enter bị chặn (`selectedPaletteLookupHeadword` trả null khi `pending`) để không mở dòng của câu cũ.
- Xem trước gọi thẳng `lookupDictionary(headword)` (không qua `runLookup`), nên không ghi lịch sử; nút ghim bị ẩn (`noPin`) vì không có mục tiêu ghim ở bảng.
- Lệnh "Tra từ" ở hàng danh sách KHÔNG đóng bảng (`runSelectedPaletteItem` rẽ riêng cho id này); nút `data-palette-lookup-button` trong ô gõ là điểm chuột của lệnh (AD-62). Lệnh chạy được cả khi bảng đóng (mở bảng rồi vào chế độ).
- Enter ở chế độ tra: đóng bảng, `nextTick`, rồi `runLookup(headword)` (port tiêm từ `main.ts`). `runLookup` chỉ điền panel Lookup, không tự mở/hiện panel nếu nó đang ẩn: chưa kiểm ở app thật.
- Thiếu port `runLookup` ⇒ log lỗi và bảng giữ nguyên (không đóng im lặng).
- `quickLookup` ngoài Tauri trả `{null,null}` ⇒ trạng thái `unavailable` có tên; xem trước null/null sau khi xong cũng báo `palette.lookup.unavailable`, không treo "đang tải".
- Nhóm `empty` ẩn; `outdated_dictionary`/`error` hiện tên; `incomplete`, `has_more`, `query_truncated`, `skipped` đều có dòng chữ riêng.

**Đã kiểm:** `vue-tsc` sạch, `check:lint` · `check:commands` · `check:tokens` xanh; vitest `palette.test.ts` + `paletteLookup.test.ts` 31/31 và 9 tệp liên quan (titlebar, shortcuts, commands, lookup) 126/126; `cargo test --test ipc_argument_contract` xanh (gate phase 2 để lại nay xanh). Đối chứng gỡ thật, đỏ đúng ca: bỏ guard `sequence` ở `queryPaletteLookup`; bỏ `if (!composing)`; bỏ chặn Enter khi `pending`.
- 🔴 `check:i18n` ĐỎ 11 ca Kiểm A, toàn bộ ở `src-tauri/src/core/dict/pinyin.rs` (bảng thanh `('ā', 'a')…` bị coi là chuỗi tiếng Việt ở vị trí mã). Là việc của phase 2 (Rust), phase 3 không được sửa Rust. Cần `aura-allow-text` có lý do hoặc đổi cách viết bảng; phần `.vue` (Kiểm A2) đạt.
- Chưa làm: đối chứng cho "xem trước không ghi lịch sử" (không có đường nào để gỡ vì xem trước không gọi `recordLookup`; ca chỉ khẳng định kết quả), hình học khung/xem trước ở app thật, cuộn của `.pal-body--lookup`, chạy đủ vitest.

## Phase 4 — Test

Xong phần test, chưa commit. Chi tiết số đo và đối chứng nằm ở `## Implementation Notes` của plan.

**Đã làm:** sửa `check:i18n` (bảng thanh `pinyin.rs` dùng `\u{..}`); thêm ca vitest "xem trước chỉ nhận phản hồi mới nhất" (`paletteLookup.test.ts`, 14 ca); đối chứng (c) `pick_route` và (k) thiếu tệp; đo lại NFR1 ở bản release (p95 35,3 ms); ghi hai nợ `Chủ: Ice`; chạy đủ bộ một lần.

**Còn đỏ, không do plan này, chưa sửa (sửa tệp cổng bị chặn quyền):**
- `check:layout`: regex `GLOBAL_MEMBER_RE` trong `scripts/check-layout.mjs` đọc `@mousedown.self.prevent` thành `self.prevent`; gợi ý lookbehind `(?<![.\w$])`.
- `check:panel-refs`: `FILE_FLOOR` 140 → 151 (`scripts/check-panel-refs.mjs`).
- `check:fr77` (`ai_boundary`): `src/commands/chapter.rs:1468` gọi `commands::aiprompt`, thiếu khi gỡ `core::ai`; tệp không đổi ở cây này.

**Việc chờ người:** AD-69 mục 3 sửa 47 → 0 (Winston); Ice tải `dict-v4` lên Release, thử app thật; ca Enter→lịch sử và dòng `el.value` chưa có test.

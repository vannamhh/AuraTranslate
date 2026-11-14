---
title: 'Tra nhanh một từ Trung↔Việt trong bảng Mod+K, gõ được pinyin'
type: 'feature'
ticket: ''
created: '2026-11-14'
status: 'built'
baseline_revision: '98a73f758f439c48527258d613d73d2a1f56fd09'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/tools/dict-build/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Hiện chỉ tra được một từ trong panel Lookup, phải gõ chữ Hán, và không có chiều Việt→Trung. Ice muốn mở Mod+K, gõ một từ rồi thấy nghĩa ngay trong khung nổi. Từ đó có thể là chữ Hán, pinyin, tiếng Việt hoặc tiếng Anh.

**Approach:** Bảng Mod+K có thêm chế độ "Tra từ". Rust có một lệnh tra nhanh trả về các nhóm theo hình dạng câu tra. Pinyin dò qua một cột khoá trong schema v4, còn Việt→Trung dò FTS trên nghĩa tiếng Việt. Khung hiện danh sách, và dòng đang chọn được xem trước ngay trong khung.

**Quyết định của Ice (2026-11-14):**
- Làm đủ ba phần: tra chữ Hán, gõ pinyin, Việt→Trung.
- Index pinyin là P1: cột `reading_key` + index trong schema v4. Dựng lại cả 4 .db và phát hành dict-v4.
- Vào chế độ tra riêng bằng tiền tố `/` ở đầu ô. Có thêm lệnh "Tra từ" để dùng bằng chuột.
- Kết quả là danh sách kèm khung xem trước bản ghi đủ của dòng đang chọn. Xem trước không ghi lịch sử, Enter mở sang panel Lookup và có ghi lịch sử.
- Câu tra bằng chữ Latin hiện cả ba nhóm: Hán theo pinyin, Việt→Trung, Anh→Việt.

## Boundaries & Constraints

**Always:**
- Phải có hai thứ trước dòng code đầu tiên: một AD mới do Winston viết (định tuyến chế độ tra của Mod+K, đổi AD-44; schema v4) và lượt correct-course cho PRD §3.2 kèm một FR mới.
- Luật nghiệp vụ nằm ở Rust (AD-1). Nguồn không bao giờ gộp, mỗi nghĩa mang nguồn của nó (AD-19).
- Câu tra có dấu dò `sense_fts`, không dấu dò `sense_fts_nd` (AD-27, AD-61).
- Trạng thái đang tải, lỗi, không có kết quả và bộ từ điển cũ đều có tên riêng, không bao giờ rỗng im lặng.
- p95 của lệnh tra nhanh dưới 100 ms (NFR1).

**Never:**
- Không dùng `LIKE` hay quét toàn bảng trên đường nóng (AD-26).
- Không ghi lịch sử khi xem trước.
- Không mount `LookupCharPopup.vue`.
- Không nâng `MINIMUM_SCHEMA_VERSION`: tệp v3 vẫn tra Trung→Việt và Anh→Việt như cũ.
- Không tự phát hành GitHub Release: Ice làm.
- Không commit `.db` (AD-25).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Vào chế độ | `/` gõ ở ô trống | hiện nhãn "Tra từ", danh sách Lệnh ẩn đi | — |
| Thoát chế độ | Backspace khi ô trống ở chế độ tra | trở về Lệnh | — |
| Chữ Hán | `你好` | nhóm Trung→Việt, giống panel | — |
| Pinyin | `nihao`, `ni hao`, `ni3hao3`, `nǐhǎo` | cùng khoá `nihao`; 你好 xếp đầu (khoá khớp hẳn trước, khớp tiền tố sau) | — |
| Việt có dấu | `xin chào` | Việt→Trung có 你好 / 您好…; nghĩa trùng khít đứng đầu | — |
| Latin không dấu | `an` | ba nhóm pinyin, Việt→Trung (bỏ dấu), Anh→Việt; nhóm nào 0 mục thì ẩn | cả ba đều 0 ⇒ trạng thái "không có kết quả" có tên |
| Tệp v3 | core.db v3 | pinyin và Việt→Trung báo "cần cập nhật bộ từ điển"; Trung/Anh vẫn chạy | trạng thái có tên |
| Đang gõ IME | composition đang mở | không gửi truy vấn | — |
| Phản hồi cũ | gõ nhanh hai phím | chỉ hiện phản hồi của câu mới nhất | guard `sequence` |

</frozen-after-approval>

## Code Map

- `src/PaletteOverlay.vue`, `src/paletteState.ts` -- bảng Mod+K.
  - `paletteItems = [...works, ...commands]` có một con trỏ phẳng. Template lệch chỉ số theo `paletteWorkItems.length`.
  - Lọc qua `foldVietnamese` + `includes`. IME chỉ chặn bằng `event.isComposing`.
  - `setPaletteQuery` đặt con trỏ về 0. Esc không `.stop`.
  - Chạy: đóng bảng, chờ `nextTick`, rồi `dispatch`.
- `src/commands/index.ts:4446-4494` -- các lệnh `palette.*`, deps ở `:1019-1029`, wiring ở `main.ts:1042`. AD-62: một dòng của bảng không phải điểm chuột của lệnh, nên lệnh mới cần một nút trong khung.
- `src/commands/index.ts:3177` `lookup.lookup_selection` -- khuôn cho Enter: port `deps.runLookup(text)`.
- `src/panels/lookupCharPopupState.ts` -- khuôn state không ghi lịch sử: `lookupDictionary` cộng guard `sequence`.
- `src/panels/LookupRecord.vue`, `lookupView.ts` (`lookupBodyOf`), `src/panels/pinyin.ts` (`toneMarkedReading`) -- dùng lại để vẽ khung xem trước và cột pinyin của dòng.
- `src/config/dict.ts` -- adapter IPC, không bao giờ ném. Thêm `quickLookup` theo khuôn `lookupDictionary`.
- `src-tauri/src/commands/dict.rs:244,384`, `lib.rs:1268` -- lệnh tra và chỗ đăng ký.
- `src-tauri/src/core/dict/mod.rs:146,309` -- `pick_route`/`pick_branch`.
- `src-tauri/src/core/dict/query.rs` -- SQL nằm đây.
- `src-tauri/src/core/dict/layer.rs:73,91` -- hằng số phiên bản schema.
- `src-tauri/src/ports/dict_source.rs:49` -- port `DictionarySource` (AD-2, không thêm port).
- `tools/dict-build/src/schema.rs` (`SCHEMA_VERSION=3` :34, `dict_entry` :94, FTS :171-189).
  - `reading` trộn `ni3 hao3` / `nǐhǎo`.
  - dict-build không dùng chung crate với `src-tauri`.
- `dict-manifest.toml` -- sha256 cho từng tệp và lịch sử dựng.
- Gate phải nâng sàn khi thêm tệp:
  - `check-commands.mjs` VUE/TS_FLOOR, COMMAND_FLOOR
  - `check-i18n.mjs` VUE_FLOOR
  - `check-tokens.mjs` COMPONENT_FILE_FLOOR
  - Chuỗi mới vào `src/i18n/vi.json` với khoá `palette.lookup.*` và `command.palette.lookup_mode`.

## Tasks & Acceptance

**Execution:**
- [ ] Spine -- Winston viết AD mới (số kế tiếp sau AD-68, quét cả `ad-brief-*`) -- định tuyến chế độ tra, schema v4, khoá pinyin, `gloss_lang`.
- [ ] PRD -- correct-course: §3.2 nhận Việt→Trung dò ngược trên dữ liệu Trung→Việt, thêm FR161 -- quyết định của Ice phải có chỗ trong quy hoạch.
- [ ] `tools/dict-build/src/` -- schema v4:
  - thêm `dict_entry.reading_key` cùng index, và `dict_source.gloss_lang` khai theo nguồn;
  - viết hàm `pinyin_key`: hạ chữ thường, bỏ dấu thanh và số thanh, `ü`/`u:` thành `v`, bỏ khoảng trắng và dấu câu.
- [ ] `tools/dict-build/tests/fixtures/pinyin_key.tsv` -- bảng đối chiếu dùng chung cho cả hai crate, vì hai crate có hai bản `pinyin_key`.
- [ ] Dựng lại 4 tệp `.db`, cập nhật sha256 trong `dict-manifest.toml` -- AD-25.
- [ ] `src-tauri/src/core/dict/` -- `pinyin.rs` chứa `pinyin_key`.
  - Viết `quick_lookup(query)`. Chữ Hán đi đường zh có sẵn. Chữ Latin chạy ba nhóm:
    - pinyin: range btree trên `reading_key`, khớp hẳn trước;
    - Việt→Trung: FTS, chỉ lấy nguồn có `gloss_lang='vi'`, xếp nghĩa trùng khít → bắt đầu bằng → còn lại;
    - Anh: đường en có sẵn.
  - Mỗi nhóm `LIMIT 20`. `SUPPORTED_SCHEMA_VERSION=4`. Tệp v3 trả cờ bộ từ điển cũ.
- [ ] `src-tauri/src/commands/dict.rs`, `lib.rs` -- thêm wire `quick_lookup`.
- [ ] `src/config/dict.ts` -- thêm adapter `quickLookup` và kiểu wire snake_case.
- [ ] `src/paletteLookupState.ts` -- state của chế độ tra:
  - chạy truy vấn với guard `sequence`, bỏ qua khi đang composition;
  - giữ danh sách phẳng qua các nhóm;
  - xem trước bằng `lookupDictionary(headword)`, không gọi `recordLookup`.
- [ ] `src/paletteState.ts`, `src/PaletteOverlay.vue`:
  - tiền tố `/` và Backspace khi ô trống;
  - nhãn và nút "Tra từ" (`dispatch('palette.lookup_mode')`);
  - nhóm, dòng (chữ Hán · pinyin dấu thanh · nghĩa đầu), khung xem trước `LookupRecord`;
  - Enter: đóng bảng, rồi `runLookup(headword)`.
- [ ] `src/commands/index.ts`, `main.ts`, `src/i18n/vi.json`, các gate -- lệnh `palette.lookup_mode` (`palette: true`), port `runLookup` cho bảng, chuỗi và sàn tệp.
- [ ] Test:
  - Rust: `pinyin_key` theo fixture; mỗi hàng của Matrix; xếp hạng; tệp v3; p95 trên core.db thật.
  - dict-build: schema v4.
  - vitest: chế độ, IME, phản hồi cũ, xem trước không ghi lịch sử, Enter.
  - Đối chứng mỗi guard bằng phép gỡ thật.

**Acceptance Criteria:**
- Given Ice mở Mod+K và gõ `/nihao`, when phản hồi về, then dòng đầu nhóm pinyin là 你好 và khung xem trước hiện nghĩa cvdict cùng nguồn.
- Given đang xem trước một dòng, when Ice mở panel Lookup, then lịch sử tra không có mục mới. When Ice nhấn Enter, then panel hiện từ đó và lịch sử có thêm một mục.
- Given bảng ở chế độ Lệnh, when Ice bấm lệnh "Tra từ" bằng chuột, then bảng vào chế độ tra và tiêu điểm ở ô gõ.
- Given core.db v3, when tra `an`, then nhóm Anh→Việt vẫn có kết quả, còn pinyin và Việt→Trung hiện thông báo cần cập nhật.
- Given core.db v4, when đo 200 câu tra trộn bốn hình dạng, then p95 của `quick_lookup` dưới 100 ms (ghi rõ bản build và tải máy).

## Design Notes

- Tone gõ trong câu tra (`ni3`) bị bỏ qua khi dò. Cột pinyin của dòng vẫn hiện dấu thanh, nên người dùng phân biệt bằng mắt.
- Câu tra pinyin thì luôn dò, không cần tách âm tiết. Range btree dưới 1 ms, và nhóm 0 mục thì ẩn.
- Phát hành dict-v4: agent dựng tệp, cập nhật manifest và đo. Tải lên GitHub Release là việc của Ice, ghi thành nợ `Chủ: Ice` nếu chưa làm trước `done`.

## Verification

**Commands:**
- `cargo test --test <tên test tra nhanh>` trong `src-tauri/` (chạy `npm run build` trước) -- xanh.
- `cargo test` trong `tools/dict-build/` -- xanh.
- `npx vitest run tests/frontend/palette.test.ts tests/frontend/paletteLookup.test.ts` -- xanh.
- `npm run check:commands && npm run check:i18n && npm run check:tokens` -- xanh.
- Chạm `Cargo.lock`, đăng ký lệnh trong `lib.rs` ⇒ chạy đủ bộ một lần trước khi push.

**Manual checks (if no CLI):**
- Ice thử ở app thật: `/` rồi gõ pinyin, tiếng Việt, chữ Hán. Ghi nợ `Chủ: Ice` nếu chưa thử.

## Số đo lúc lập plan

Đo trên `dict-core.db` dict-v3 ở `tools/dict-build/out`, sqlite3 3.43.2, cache nóng, máy tải 6,5–8.

- Có 247.354 mục mang pinyin trong cvdict + cc-cedict, gom thành 90.160 khoá bỏ thanh khác nhau.
- Quét toàn bảng tìm pinyin mất 0,37 s, nên cần index.
- Dò `sense_fts` cho `"ăn"` trên cvdict ra 1.100 mục trong 25 ms; `sense_fts_nd` cho `"an"` ra 3.129 mục trong 50 ms.

## Implementation Notes

- Hồ sơ quy hoạch: AD-69 nằm trong spine, FR161 trong PRD, và correct-course ở `change-tra-tu-nhanh-trong-bang-mod-k/`; cả ba đang chờ Ice ký. Winston tự đề xuất thêm năm điểm, liệt kê ở mục 3 của change proposal: lọc theo cả `lang` và `gloss_lang`, `gloss_lang` khai trên `SourceMeta`, `pinyin_key` cắt ở `(`, câu có dấu không tự nới sang bản bỏ dấu, lấy 21 trả 20.
- Schema v4: thêm `dict_entry.reading_key` cùng index, và `dict_source.gloss_lang`. `MINIMUM_SCHEMA_VERSION` giữ 3. Mỗi `DictLayer` mang phiên bản của chính tệp, nên tệp v3 trả `outdated_dictionary` thay cho lỗi SQL.
- Hai bản `pinyin_key` (dict-build và src-tauri) bị khoá với nhau bằng `tools/dict-build/tests/fixtures/pinyin_key.tsv`. Bảng thanh điệu viết bằng `\u{..}` để qua Kiểm A của `check:i18n`.
- Có 70 mục trong `dict-core.db` mang `reading` mà `reading_key` NULL: âm đọc phương ngữ (`ȵ`, `ɡ`…) và dấu kết hợp NFD. Test `reading_key_real_data.rs` khai đúng con số 70 này. AD-69 §3 còn ghi 47 dòng có `(` không đóng, trong khi bản v4 đếm ra 0; Winston cần sửa.
- Việt→Trung không dùng `ORDER BY`: FTS đã trả rowid tăng dần rồi dừng ở `LIMIT`. Câu sắp xếp phép join làm bản release vượt NFR1 (129,9 ms). Sau khi gỡ, p95 bản release là 35,3 ms, p50 9,0 ms, max 53,4 ms: 480 lượt trên 160 câu, từ điển v4 thật, máy tải khoảng 10.
- `classify_query`/`SearchMode` và các hàm gấp dấu chuyển sang `core/diacritics.rs`, dùng chung cho Library (AD-61) và tra ngược. `library::indexer` re-export hai cái đầu.
- Webview: `paletteLookupState.ts` có hai guard `sequence` riêng, một cho câu tra và một cho xem trước. Enter bị chặn khi câu mới hơn đang chạy. Bản xem trước của `LookupRecord` ẩn nút ghim (`noPin`), vì bảng không có đích ghim.
- `check:panel-refs` đứng im vì sàn cũ (140 < 151): cổng dừng trước khi quét. Nâng sàn xong lộ 21 ô nhớ. State của bảng Mod+K được miễn trừ có tên. `lookupCharPopupState` có thêm `resetLookupCharPopup()`, gọi từ `resetLookupPanel()`. `check:layout` đỏ oan vì đọc `.self.prevent` thành biến `self`; đã thêm lookbehind và một ca tự kiểm âm. Hai lỗi cổng này có sẵn từ `847b5b3c` và `98a73f75`.
- Đủ bộ, chạy một lượt trên máy tải cao: 14 cổng; vitest 284 tệp, 4569/4569, không có dòng Errors; `cargo test --locked` xanh. Hai cổng layout và panel-refs sửa sau lượt đó và đã chạy lại riêng, xanh. `check:fr77` đỏ ở `ai_boundary`: `commands/chapter.rs:1468` gọi thẳng `commands::aiprompt`, có từ `8b8508c3`, không do plan này.
- Chưa phủ: gỡ dòng đồng bộ `el.value` ở `PaletteOverlay.vue` vitest vẫn xanh (chưa đo WKWebView). Lịch sử có thêm một mục khi Enter chưa được test, vì `runLookup` là bản giả. Phát hành dict-v4 và lượt thử app thật là hai mục nợ `Chủ: Ice` trong `deferred-work.md`.


## Plan Change Log

## Review Triage Log

Lượt 1 (quick), 2026-11-14 — high 0 · medium 3 · low 5 · false 0 · maybe-false 0; bị loại 3 (một `low` không đáng sửa, một phát hiện mà cách sửa là sửa plan này, một phát hiện về trạng thái ký).

- medium · patch — Khi câu mới đang chạy, trạng thái vẫn `ready`: `paletteLookupStatus` chỉ ra `loading` khi `response === null`, còn `selectedPaletteLookupHeadword()` trả `null` lúc `pending`, nên Enter và cú bấm không làm gì mà cũng không báo. Gửi agent webview: báo `loading` mỗi khi `pending`.
- medium · patch — `pal-body--lookup` là vùng cuộn duy nhất, chứa tới 60 dòng, và không có `scrollIntoView` nên dòng đang chọn và khung xem trước trôi khỏi tầm nhìn. Gửi agent webview: danh sách cuộn riêng, xem trước ghim bên dưới, cuộn theo con trỏ.
- medium · patch — Nửa sau của AC2 (Enter thì lịch sử có thêm một mục) chưa có test, vì `runLookup` là bản giả. Gửi agent webview: thêm ca dùng `runLookup` thật, chỉ giả IPC.
- low · patch — `aria-expanded` đọc `paletteItems`, vốn rỗng trong chế độ tra. Gửi agent webview.
- low · patch — `/` gõ trước chữ đã có cũng vào chế độ, trái hàng Matrix "`/` gõ ở ô trống". Gửi agent webview, kèm một ca test.
- low · patch — Doc comment của `list_dict_sources` nằm nhầm trên `quick_lookup` (`commands/dict.rs`). Đã dời về chỗ cũ.
- low · patch — Doc comment của `gloss_sql` chứa số đo, AGENTS.md cấm. Đã bỏ vế số đo.
- low · patch — AD-69 §3 ghi 47 dòng có `(` không đóng, bản v4 đếm ra 0. Đã thêm dòng 🔵 2026-11-14 trong spine, nêu cả 70 dòng không ra khoá.
- low · bị loại — `pinyin_group` cộng `source_count` hai lần khi cùng một nguồn khớp ở cả tầng khớp hẳn lẫn tầng tiền tố. Có thật, nhưng webview không hiện `source_count`; muốn sửa phải đổi SQL sang trả mã nguồn, không chỉ là sửa thẳng một dòng.
- bị loại — Implementation Notes có hai khối trùng nhau, khoảng 20 bullet: cách sửa là sửa plan này. Khối thứ hai do agent phase 4 ghi trước khi bị dừng; đã xoá như việc dọn phần của agent.
- bị loại — "AD và correct-course chưa được ký": mục Always chỉ đòi hai hồ sơ đó có mặt trước dòng code, và cả hai đã viết. Chờ Ice ký là trạng thái bình thường. Dòng đồng bộ `el.value` không thể đỏ dưới happy-dom, đã ở trong nợ thử app thật.

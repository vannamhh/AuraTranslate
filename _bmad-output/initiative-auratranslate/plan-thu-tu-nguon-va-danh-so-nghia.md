---
title: 'Panel Lookup: thứ tự nguồn do người dùng đặt, nghĩa đánh số thay vạch đứng'
type: 'feature'
ticket: ''
created: '2026-10-29'
status: 'done'
baseline_revision: 'e06f4a4f3bb1ec68c3fe15bf6d50a271101e535f'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Hiện nay nguồn trong Panel Lookup hiện theo lớp rồi theo `source.code` xếp chữ cái. Người dùng không đưa được nguồn mình hay đọc lên đầu. Ngoài ra, mỗi nghĩa có một vạch trái 2px, nên một nguồn nhiều nghĩa thành một dãy vạch khó nhìn: en-wiktionary có 4 457 đầu mục từ 5 nghĩa trở lên, nhiều nhất 138 nghĩa.

**Approach:** Ice quyết (2026-10-29):
- Thứ tự nguồn do người dùng đặt bằng nút ↑↓ trên mỗi hàng của menu Nguồn, cộng một phím tắt cho hàng đang focus. Thứ tự lưu bền trong global.db và áp cho cả nhóm kết quả lẫn hàng của menu.
- Nghĩa đánh số `1. 2. 3.` màu nhạt, thụt treo, thay cho vạch trái. Cả Panel Lookup lẫn popup tra ở Chế độ đọc đều dùng `LookupRecord.vue`.
- Sắp ở Rust: `commands/dict.rs` sắp nhóm trong `lookup()`, `list_dict_sources` sắp danh sách menu, cả hai đọc thứ tự từ Store. Webview không chứa luật sắp; nó chỉ đổi chỗ hai hàng ngay khi nhấn, rồi ghi và tra lại như đường tắt nguồn.
- Phím tắt `Alt+Shift+ArrowUp`/`Alt+Shift+ArrowDown` (`Alt+ArrowUp/Down` đã thuộc `review.diff_prev`/`next`). Phím chỉ có tác dụng khi focus ở một hàng của menu Nguồn.
- Giữ một plan cho cả hai việc dù dài quá trần token; triển khai chia pha.

## Boundaries & Constraints

**Always:**
- Khi chưa đặt gì, thứ tự mặc định giống hệt hiện nay. Nguồn chưa có trong thứ tự đã lưu (tệp `.db` mới thả vào) đứng sau mọi nguồn đã xếp, theo thứ tự mặc định. Một `code` đã lưu mà không còn tệp nào thì bị bỏ qua, không báo lỗi.
- Sắp xếp chỉ đổi thứ tự: số nhóm, đầu mục và nghĩa vẫn giữ nguyên.
- Khoá config mới `dict_sources_order`, cùng cách mã hoá `,` của `dict_sources_disabled` và ghi qua `put_config` có sẵn. Ghi lỗi thì UI quay về thứ tự trước, giống đường tắt nguồn.
- Đánh số theo từng cụm đầu mục, bắt đầu từ 1. Cụm chỉ có 1 nghĩa thì không hiện số. Hai dòng trở đi của một nghĩa thẳng hàng với chữ, không thẳng hàng với số. Chữ của mọi nghĩa trong cụm thẳng một cột, kể cả khi số có 3 chữ số.
- Nút ↑↓ là `@click` gọi đúng một `dispatch('<id>')` (Kiểm A của `check:commands`). Nút ↑ của hàng đầu và nút ↓ của hàng cuối bị vô hiệu.

**Never:**
- Không đổi `priority_order`, tức cách chọn âm Hán Việt.
- Không hợp nhất nguồn (AD-19). Không thêm thứ hạng nào do ứng dụng tự tính.
- Không bỏ vạch trái `primary` của trích dẫn. Không thêm token chữ hay màu mới.
- Không thêm Tauri command mới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mặc định | `dict_sources_order` vắng | thứ tự nhóm và menu như baseline | — |
| Đã xếp | order `thieu-chuu,cvdict` | Thiều Chửu, CVDICT, rồi các nguồn còn lại theo mặc định | — |
| Code mồ côi | order chứa `khong-co` | bị bỏ qua, các nguồn khác vẫn theo order | không lỗi |
| Nguồn tắt | nguồn bị tắt có vị trí trong order | không có nhóm kết quả; menu vẫn giữ hàng ở đúng vị trí | — |
| Ghi lỗi | `put_config` từ chối | menu quay về thứ tự cũ | như rollback của tắt nguồn |
| 1 nghĩa | cụm Trần Văn Chánh (`① ② …` nằm trong chữ) | không có số | — |
| Nhiều nghĩa | cụm 138 nghĩa | số 1–138, chữ thẳng một cột | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/scope/store.rs:87,101,284-292` -- `KEY_DICT_DISABLED`, `parse_disabled_sources`, getter trên `GlobalConfig`. Thêm khoá và getter thứ tự theo đúng mẫu này. Không cần migration, vì `save_value` không có allow-list.
- `src-tauri/src/commands/config.rs:92,164` -- field `dict_sources_disabled` của `BootstrapConfig`. Thêm field thứ tự. Phải sửa thêm: `tests/ipc_contract.rs:165-215` (struct literal và danh sách khoá đã đóng băng), `src/config/bootstrap.ts:78,195`, `main.ts:1222-1223`, cùng các fixture BootstrapConfig trong `tests/frontend/{settingsUpdateSection,bootstrap,aiUsageVisibleState,panelHeaders}.test.ts`.
- `src-tauri/src/commands/dict.rs:46,219-249,322,336` -- `disabled_sources(store)` được đọc ở mỗi lượt tra. `lookup()` gọi `lookup_grouped` hai lần (Exact, rồi Substring). `list_dict_sources` không đọc Store. `core/dict/**` cấm đọc Store (`tests/dict_boundary.rs`).
- `src-tauri/src/core/dict/mod.rs:840,910,979` -- vòng lặp theo lớp, rồi `in_layer.sort_by(code)`, rồi `list_source_attributions`. Các chỗ khác gọi `lookup_grouped` (`commands/glossary.rs:1290,1341`, `commands/project/mod.rs:756`, `glossary_estimate.rs:59`) không phụ thuộc thứ tự nhóm, nên giữ nguyên.
- `src/panels/dictSourcesState.ts:35-38,133-160,178,215,341,367` -- các ref `sources`/`disabled`, mã hoá, `loadDictSources`, `writeQueue` với `toggleSequence` và rollback, `aimDictSourceFrom`/`toggleFocusedDictSource` (nhắm theo `aimedCode ?? activeElement.dataset.sourceCode`). Đường thứ tự dùng lại khuôn này.
- `src/panels/lookupView.ts:21` -- `lookupGroups`, nguồn chung của `LookupPanel.vue:575` và `ReadingLookupPopup.vue`.
- `src/panels/LookupSourcesMenu.vue:32,57-64` -- `rows` lấy từ `dictSources`. ⚠️ `onKeydown` bắt ArrowUp/ArrowDown mà không xét `altKey`/`defaultPrevented`, nên phím tắt thứ tự sẽ dời luôn focus. Phải chặn.
- `src/commands/index.ts:717,3032` -- port `toggleDictSource` và vòng đăng ký `lookup.toggle_source`. ⚠️ `Alt+ArrowUp`/`Alt+ArrowDown` đã thuộc `review.diff_prev`/`review.diff_next` (`index.ts:3990,3999`), và `createKeymap` ném lỗi khi trùng hợp âm. Lệnh mới cần: port, `main.ts` (~:931), khoá `command.<id>` trong `src/i18n/vi.json`, `COMMAND_FLOOR` (`scripts/check-commands.mjs:238`, hiện là 267), và bảng Kiểm K (`check-commands.mjs:2599`) nếu `onKeydown` dispatch.
- `src/panels/LookupRecord.vue` (`.lookup-sense`, vòng `v-for` theo `cluster.senses`) -- bỏ `border-left`/`padding-left`. Số nghĩa dùng `--color-on-surface-variant` và token `lookup-gloss` (`check-tokens.mjs:240`).
- Test: `src-tauri/tests/dict_sources.rs` (khuôn `off()` ở :3913, bản bọc ở :62), `tests/frontend/lookupSourcesPopup.test.ts`, `lookupPanelBlocks.test.ts`, `readingLookupPopup.test.ts`, `tests/frontend/support/lookupFixtures.ts`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/scope/store.rs` -- khoá `dict_sources_order` và getter trả `Vec<String>` (cắt theo `,`, trim, bỏ rỗng, giữ thứ tự, bỏ trùng) -- thứ tự khác tập hợp: dùng `BTreeSet` thì mất thứ tự.
- [ ] `src-tauri/src/commands/dict.rs` (hàm thuần có thể đặt ở `core/dict`, miễn là không đọc Store) -- `order_by_user(items, key, &order)` sắp ổn định: hàng có trong order theo chỉ số, hàng không có xếp sau theo thứ tự cũ. Áp cho nhóm của `lookup()` (sau khi đã chọn Exact hay Substring) và cho `list_dict_sources`.
- [ ] `src-tauri/src/commands/config.rs` cùng `ipc_contract.rs`, `bootstrap.ts`, `main.ts` và các fixture -- field bootstrap mới.
- [ ] `src/panels/dictSourcesState.ts` -- ref thứ tự, `moveDictSource(code, ±1)` cập nhật ngay, ghi qua `writeQueue` (lần nhấn sau cùng thắng, ghi lỗi thì rollback), ghi xong thì gọi lại `listDictSources` và `runLookup`, cùng hai hàm "focused" nhắm như `toggleFocusedDictSource`.
- [ ] `src/commands/index.ts`, `main.ts`, `vi.json`, `check-commands.mjs` -- `lookup.move_source_up`/`lookup.move_source_down` với `Alt+Shift+ArrowUp`/`Alt+Shift+ArrowDown`; nâng `COMMAND_FLOOR` thêm 2.
- [ ] `src/panels/LookupSourcesMenu.vue` -- nút ↑↓ dạng `ui-icon-btn` mang `aria-label`. Focus ở lại trên hàng vừa dời. `onKeydown` bỏ qua phím có modifier.
- [ ] `src/panels/LookupRecord.vue` -- danh sách đánh số cho cụm có ≥2 nghĩa, bỏ vạch trái.
- [ ] Test: Rust cho phép sắp (mặc định, đã xếp, mồ côi, trùng, nguồn tắt). Vitest cho ↑↓ (đổi thứ tự menu và nhóm, nút ở biên bị vô hiệu, rollback, phím không dời focus) và cho đánh số (1 nghĩa không số, 2 nghĩa có số "1." và "2.").

**Acceptance Criteria:**
- Given Ice đưa Thiều Chửu lên đầu, when tra `打` ở Panel Lookup rồi ở popup của Chế độ đọc, then nhóm Thiều Chửu đứng đầu ở cả hai nơi, và thứ tự vẫn giữ sau khi khởi động lại ứng dụng.
- Given focus đang ở một hàng của menu Nguồn, when nhấn hợp âm ↑ một lần, then hàng đổi chỗ với hàng trên, focus vẫn ở hàng đó và không nhảy sang hàng khác.
- Given tra `打`, when nhìn nhóm CVDICT, then thấy 18 nghĩa đánh số 1–18 và không còn vạch trái nào trên nghĩa; nhóm Trần Văn Chánh không có số.

## Implementation Notes

- Chia hai pha (Rust, rồi webview), chuyển giao qua `handoff-thu-tu-nguon-va-danh-so-nghia-phases.md`.
- Bỏ field bootstrap `dict_sources_order`: Rust đã trả `list_dict_sources` theo đúng thứ tự, nên không chỗ nào ở webview đọc field đó. Webview chỉ ghi khoá này qua `KEY_DICT_ORDER`.
- Webview ghi toàn bộ danh sách code của menu ở mỗi lần nhấn, không chỉ cặp vừa đổi chỗ.
- `<ol>` khai token `lookup-gloss`, để `::marker` và đơn vị `ch` của phần thụt treo đo theo font của nghĩa.
- Đối chứng đỏ cho `the_dict_source_order_survives_a_write_and_a_reopen`: cho getter `dict_sources_order` luôn trả `""` thì test ra `[]` thay vì `["thieu-chuu","cvdict"]`. Bên webview, gỡ `row.focus()` thì ca giữ focus đỏ.
- Lượt vitest đầy đủ duy nhất (pha 2, load average khoảng 200) đỏ 4 ca trong `settingsOverlayAiConfigTierRender`/`promptLibraryOverlayRender`. Chạy riêng thì cả hai xanh. Chưa chạy lại lượt đầy đủ trên máy rảnh; `pre-push` sẽ chạy.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 9 · false 3 · maybe-false 0.
- medium · patch · AC1 "giữ sau khi khởi động lại" chưa có test nào đi qua Store. Đã thêm `scope_contract::the_dict_source_order_survives_a_write_and_a_reopen`, đối chứng đỏ đúng lý do.
- low · patch · field bootstrap `dict_sources_order` không ai đọc. Đã gỡ khỏi `BootstrapConfig` (Rust và TS), `ipc_contract` và 4 fixture.
- low · patch · tên test `…including_disabled_positions` nói về nguồn tắt nhưng không tắt nguồn nào (`list_sources` không nhận tập tắt). Đã đổi thành `user_order_applies_to_the_source_list_too`.
- low · patch · doc của `moveDictSource` dài 4 dòng, trái AGENTS.md (1–2 dòng). Đã rút còn 2.
- low · reject · hai lượt ghi thứ tự liên tiếp cùng lỗi thì lần đổi chỗ thứ nhất còn hiện trên màn hình. Chỉ xảy ra khi `put_config` lỗi hai lần liền, và sửa thì phải thêm một nhánh tải lại.
- low · reject · một lớp `.db` không mở được (`skipped`) sẽ mất vị trí đã lưu khi người dùng dời một nguồn khác. Ca hiếm; sửa thì webview phải giữ thứ tự đã lưu để gộp.
- low · reject · nút ↑↓ là `<button>` trong `role="menu"`, `tabindex=-1`, ngoài vòng mũi tên. Đường bàn phím là `Alt+Shift+↑/↓` (Ice chọn); muốn sửa phải dựng lại cấu trúc menu.
- low · reject · số nằm ngang dòng từ loại khi nghĩa có `pos`. Plan không đòi số phải nằm trên dòng nghĩa; dòng gói vẫn thẳng với chữ.
- low · reject · chưa có ca vitest đặt tên cho CVDICT 18 nghĩa hay Trần Văn Chánh. Cùng luật ≥2 / 1 nghĩa đã được canh bằng fixture `DA` và ca 138 nghĩa.
- low · reject · ca phím có modifier chỉ thử Alt+Shift, chưa thử Ctrl/Meta riêng. Cùng một điều kiện `if`.
- false · độ thụt `ch` "đoán": marker native nằm ngoài hộp nội dung, nên chữ mọi `li` luôn bắt đầu cùng một `padding-inline-start`, tức thẳng cột theo cấu trúc. Chỉ việc marker có tràn ra ngoài hay không là cần nhìn tận mắt (nợ cho Ice).
- false · `COMMAND_FLOOR` 269: số lệnh tăng từ 314 lên 316, và ⌈0,85 × 316⌉ = 269 khớp công thức; `check:commands` xanh.
- false · `code` chứa `,`: code nguồn là slug kebab, đúng quy ước mã hoá `dict_sources_disabled` đã dùng sẵn; không có code nào như vậy.

## Design Notes

Thụt treo không dùng `subgrid` (WebKit chậm, xem `research-go-cham-luoi-subgrid-webkit`). Dùng `<ol>` với `::marker` native: marker canh phải sát mép chữ nên chữ thẳng cột; `padding-inline-start` theo số chữ số của nghĩa lớn nhất, đặt bằng một biến CSS. `::marker` chỉ nhận `color`/`font`, đủ cho màu nhạt và `tabular-nums`.

## Verification

**Commands:**
- `npm run test:story -- --list` rồi chạy đúng: `cargo test --test dict_sources`, `cargo test --test ipc_contract`, vitest `lookupSourcesPopup`/`lookupPanelBlocks`/`readingLookupPopup` -- xanh
- `npm run check:commands && npm run check:tokens && npm run check:i18n` -- xanh

**Manual checks (if no CLI):**
- Ice dùng thật một lượt (nợ `Chủ: Ice`): thứ tự còn giữ sau khi khởi động lại; menu không đóng khi nhấn ↑↓.

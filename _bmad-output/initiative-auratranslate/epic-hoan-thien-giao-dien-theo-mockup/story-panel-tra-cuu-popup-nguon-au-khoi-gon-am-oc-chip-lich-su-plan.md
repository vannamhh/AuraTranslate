---
title: 'Panel Tra cứu: popup nguồn, đầu khối gọn, âm đọc, chip lịch sử'
type: 'feature'
ticket: '8'
created: '2026-10-14'
status: 'built'
baseline_revision: '23b2981b896d2dba8405fe8edbde063dc563c29a'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Panel Tra cứu lệch mockup ở sáu chỗ: chọn nguồn bằng dải chip thay vì popup (D2, D12, change 6.3); đầu khối nguồn to, không số nghĩa, không thu gọn (D5, b:63, B7); còn dòng dẫn "Có nhiều hơn một nguồn…" (D6); Ghim nằm trên từng nhóm mục thay vì ở đầu panel (b:51); không hiện âm đọc dù `dict_entry.reading` đã nạp đủ cho CC-CEDICT và CVDICT (FR136, B4, B5); lịch sử không lọc được theo Chương (D13a, change 6.4). Kèm theo: tiêu đề chưa hiện phím thật (B3), thanh nhịp bị cắt câm khi nhiều nguồn (b:53), hai trạng thái rỗng khác mockup (a:163, a:164).

**Approach:** Lệnh Rust tra cứu trả thêm `reading` của từng đầu mục; webview đổi dải chip thành nút `Nguồn N/M ▾` mở popup, đầu khối thành một dòng gọn có nút thu gọn, nhóm nghĩa trong khối theo âm, Ghim lên đầu panel, và lịch sử ghi Chương của mỗi lượt tra để chip `Chương này` lọc được.

## Decisions (Ice, 2026-10-14)

1. Ghim: nút ở đầu panel tác động khi kết quả có đúng một mục (đường `lookup.toggle_pin` sẵn có); mỗi nhóm mục giữ một nút ghim nhỏ cho ca nhiều mục. Lệch mockup b:51 một phần, có chủ đích.
2. Âm đọc chỉ trong khối: mỗi nhóm mục mở bằng `âm · N nghĩa`; cạnh đầu mục ở đầu panel chỉ hiện âm khi mọi mục có âm đều cùng một khoá so (đổi sang dấu, bỏ đuôi ngoặc `(shan¹)`, bỏ khoảng trắng, chữ thường), và hiện chính khoá đó (`zǒuláng`). Trong khối, âm hiện theo dạng của nguồn sau khi đổi sang dấu. Không dải âm, không cộng số giữa nguồn, không lọc theo âm.
3. Khối đang mở hiện mọi nghĩa; `⌄` thu cả khối về dòng đầu. Trạng thái thu gọn nhớ theo `source_code` trong phiên.
4. Thanh nhịp chỉ còn "N nguồn · M nghĩa"; bỏ chip từng nguồn, nên không gì bị cắt. Đóng nợ §1-17 "Chip thanh nhịp" của `deferred-work.md`.
5. Không kết quả: "Không nguồn nào có {q}", "Đã tra N nguồn đang bật", nút "Tra riêng" cho từng chữ của truy vấn nhiều chữ, và giữ dòng trỏ Concordance sẵn có.
6. Một story; nhận kế hoạch quá 1.600 token như 12.7 (Ice chưa trả lời câu này; đổi được ở bước duyệt).

## Boundaries & Constraints

**Always:**
- Mỗi nguồn vẫn là một khối riêng (AD-19); không hợp nhất nghĩa của hai nguồn.
- Popup: `↑↓` chọn dòng, `Space` bật/tắt qua `lookup.toggle_source`, `Esc` đóng và trả tiêu điểm về nút (UX-DR17); không phím riêng cho từng nguồn (AD-34).
- Mọi `@click` mới là đúng một `dispatch()` (Kiểm A); lệnh mới không gán phím mặc định.
- Số chưa biết không hiện thành `0`: nguồn tắt, nguồn bị LIMIT cắt (`hidden_sources`), lớp lỗi, hay chưa tra thì không có số mục.
- Âm đọc chỉ hiện khi nguồn có dữ liệu (`reading` khác NULL); pinyin số (`da3`, `lu:4`, `Zhong1`) hiện bằng dấu thanh (`dǎ`, `lǜ`, `Zhōng`); dạng khác hiện nguyên văn.
- Phím hiển thị qua `formatChord` theo phím đang gán; chữ mới qua `vi.json`.
- Lịch sử vẫn chỉ sống trong phiên (FR41); bộ ghim không bị chip lọc.

**Never:**
- Không đổi schema từ điển, không dựng lại `.db`, không thêm dependency.
- Không chip "Cả Tác phẩm", không `⌘⌫` xoá lịch sử (AD-49), không chip loại từ (`pos` NULL ở mọi nguồn Trung).
- Không vế "Hoặc F3 để gõ tay" của a:164: panel không có ô gõ, và F3 là `tm.concordance`.
- Không câu giải thích hai âm (`.split-note`, không có nguồn dữ liệu); không Hán Việt theo từng âm (`đả · tá`: không cột nào có).
- Không đổi `ReadingMode.vue`, `AttributionOverlay.vue` (12.21); giữ `[data-attribution-open]` làm đích trả tiêu điểm.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tra 打 | 5 nguồn bật | Khối xếp chồng, không dòng dẫn; mỗi khối mở bằng tên · N nghĩa · `⌄`; CVDICT hai nhóm `dá · 1 nghĩa`, `dǎ · 17 nghĩa` | — |
| Popup | Mở, `↓↓ Space` | Nguồn thứ hai tắt, lượt tra chạy lại, N/M giảm; `Esc` ⇒ tiêu điểm ở nút | Ghi cấu hình lỗi ⇒ rollback như `toggleDictSource` |
| Số mục trong popup | Nguồn tắt / bị LIMIT cắt / lớp lỗi | Không số; nhãn "tắt" cho nguồn tắt | — |
| Nguồn không âm đọc | Vietphrase, Thiều Chửu | Không dòng âm, nghĩa không nhóm | — |
| Chip `Chương này` | Tra A ở Chương 1, B ở Chương 2, đang mở Chương 2 | Chỉ B; tra A lại ở Chương 2 ⇒ A có ở cả hai Chương | Không Chương nào mở ⇒ chip bị vô hiệu, lọc về `Phiên này` |
| Chưa tra | Panel mới | Câu hướng dẫn kèm phím thật của `lookup.lookup_selection` | — |
| Không kết quả | Tra 鐵鏽 | "Không nguồn nào có 鐵鏽" · "Đã tra N nguồn đang bật" · nút "Tra riêng: 鐵 · 鏽" (bấm 鐵 ⇒ tra 鐵) · dòng Concordance khi đếm > 0 | Mọi nguồn tắt ⇒ giữ câu `all_sources_off` hiện có; truy vấn một chữ ⇒ không có "Tra riêng" |
| Ghim | Tra 走廊 (một mục) / 打 (7 mục) | Nút đầu panel ghim mục duy nhất / bấm nút đầu panel ⇒ câu `pin_no_target`, nút trên từng nhóm vẫn ghim đúng mục | — |
| Âm ở đầu panel | 走廊: CVDICT, CC-CEDICT cùng `zou3 lang2` / 打: `da2`, `da3`, `dǎ` | `zǒu láng` cạnh chữ / không âm ở đầu (hai âm) | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/dict/mod.rs:250` `EntryHit` + `query.rs:50` `COLUMNS` -- thêm `reading: Option<String>` (`e.reading`); TS `EntryHit` ở `src/config/dict.ts:122`. Dữ liệu đo trên `resources/dict`: cc-cedict 124 758/124 758 và cvdict 122 596/122 596 có `reading` dạng số (`da3`, `zou3 lang2`); unihan `dǎ`; en-wiktionary `dǎ (da³)`; Vietphrase, Thiều Chửu, Trần Văn Chánh NULL. Repo chưa có hàm đổi pinyin số → dấu (`tools/dict-build` giữ nguyên văn); viết hàm thuần trong webview, test riêng.
- `src/panels/LookupPanel.vue` (1411 dòng, đọc theo offset) -- dải chip `.lookup-sources` :383-437 (nút `[data-attribution-open]` :429-436 phải sống tiếp); tablist :454-494; `.lookup-head` 76px :511-556 với `.lookup-headword` :513, `.lookup-spine` :514-540 (bỏ chip từng nguồn; chiều cao 76px giữ); dòng trỏ Concordance khi tra trượt đã có: `probeTotal` :109, watch `probeConcordanceCount` :111-117, nút :629-633; dòng dẫn `.lookup-disagree` :636-637 và CSS :1146-1160; tab Lịch sử :706-805 (chú thích :756-760 ghi Quyết định #5 cũ bỏ chip — xoá cùng lúc).
- `src/panels/LookupRecord.vue` -- `clusters` theo `entry_id` :56, `showEntryHeadwords` :81, đầu khối `.lookup-source-head` :86-90, nút ghim từng nhóm :111-117.
- `src/panels/lookupPanelState.ts` -- `sourcesDisagree` :35 (xoá cùng khoá `panel.lookup.sources_disagree`), `computeSpine` :83, `groupedLookup` :272, `recordLookup` gọi ở :325.
- `src/panels/dictSourcesState.ts` -- `enabledSourceCount` :109 (N), `dictSources` :42 (M), `toggleFocusedDictSource` :355 đọc `activeElement.dataset.sourceCode` — dòng popup mang `data-source-code` để `Space` dùng lại nó.
- `src/panels/lookupHistoryState.ts` -- `LookupHistoryEntry` :81 thêm danh sách Chương; `recordLookup` :313 (gộp theo query), `resetLookupHistory` :361; `resolvePinTarget` (mục được chạm › mục duy nhất › `null` + `pin_no_target`) giữ nguyên: nút đầu panel không nằm trong `[data-entry-key]` nên rơi đúng vào nhánh mục duy nhất. Chương lấy từ `editorChapterId` (`src/panels/editorPanelState.ts:161`).
- Mẫu popup: `src/TitlebarToolsMenu.vue` (`focusOnPointerDown` :27, `onKeydown` :32-48, đóng khi mousedown ngoài :50, focusout :62) + `src/titlebarToolsState.ts`. Repo chưa có `menuitemcheckbox`.
- Tiêu đề: `src/panels/PanelTab.vue:80` chỉ in `titleKey`; thêm phím của `lookup.lookup_selection` cho tab Tra cứu theo mẫu `StatusBar.vue:232` (`shortcutRows`). Nút `Nguồn N/M ▾` và link "Nguồn dữ liệu" đứng cùng hàng tablist.
- Lệnh: `src/commands/index.ts` (`lookup.toggle_source` :2815, `lookup.toggle_pin` :2895, `lookup.clear_history` :2905); lệnh mới không phím cho mở popup, thu gọn khối (nhắm `source_code` qua `@mousedown`), "Tra riêng" (nhắm chữ qua `@mousedown`, chạy `runLookup` với chữ đó), hai chip lịch sử, theo đúng mẫu đó; nhãn `command.lookup.*` trong `vi.json`; khối `panel.lookup.*` `vi.json:1102-1158`, rỗng ở khoá `empty`/`no_result` quanh :997-998.
- Gates: `scripts/check-commands.mjs` (danh sách miễn trừ handler không dispatch :2503-2505, `LookupPanel.vue` :1756/:1865); `scripts/check-panel-refs.mjs` (ref cấp module :287-313 — trạng thái popup, thu gọn, chip phải đăng ký và có reset).
- Tests that move: `tests/frontend/{lookupSourcesPopup (was lookupSourceChipToggle),lookupPanelBlocks,lookupHistoryChapter,panelTabChord,pinyin,lookupSensesFailed,statusBarWorkspace,tmConcordance,editorChapterSwitch,attributionOverlayActions,editorAutoLookup,titlebarToolsMenu}.test.ts`; `e2e/specs/{attribution-focus,story-12-8-lookup-sources-popup}.e2e.mjs`; Rust `src-tauri/tests/{dict_lookup,ipc_contract,dict_sources}.rs`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/dict/{mod,query}.rs`, `src/config/dict.ts` -- `EntryHit.reading`; test Rust: 打 ở CVDICT trả hai mục `da2`, `da3`; Vietphrase trả `None` -- FR136.
- [x] `src/panels/pinyin.ts` (mới) -- đổi pinyin số → dấu và khoá so của Quyết định 2; test bảng: `da3`, `lu:4`, `Zhong1 guo2`, `r5`, `shān (shan¹)`, `zǒuláng` giữ nguyên; khoá so của `zou3 lang2` bằng khoá của `zǒuláng`, của `Shan1` bằng của `shān (shan¹)`.
- [x] `src/panels/LookupRecord.vue` -- đầu khối gọn tên · N nghĩa · nút `⌄`; nhóm mục mở bằng âm · N nghĩa; nút ghim nhỏ trên từng nhóm giữ lại -- D5, B4, B5, B7, Quyết định 1–3.
- [x] `src/panels/LookupPanel.vue`, `lookupPanelState.ts`, `vi.json` -- bỏ dải chip và dòng dẫn; nút `Nguồn N/M ▾` + popup; Ghim ở đầu panel; thanh nhịp chỉ còn số; âm cạnh đầu mục; hai trạng thái rỗng và nút "Tra riêng" -- D2, D6, D12, b:51, b:53, a:163, a:164, Quyết định 1, 2, 4, 5.
- [x] `src/panels/lookupHistoryState.ts`, tab Lịch sử -- ghi Chương mỗi lượt tra; chip `Phiên này · Chương này` -- D13a.
- [x] `src/panels/PanelTab.vue` -- phím thật cạnh "Tra cứu" -- B3.
- [x] `src/commands/index.ts`, gates -- đăng ký lệnh mới, miễn trừ có tên.
- [x] `deferred-work.md` -- nối dòng `→ ✅ ĐÃ ĐÓNG` cho mục §1-17 "Chip thanh nhịp" (grep, không đọc cả tệp) -- Quyết định 4.
- [x] Tests -- sửa các tệp ở Code Map; e2e mới theo Verify của 12.8; đối chứng đỏ: gỡ thật `Esc` → `trigger.focus()` của popup, và gỡ ghi Chương trong `recordLookup`.

**Acceptance Criteria:**
- Given tra 打, when đọc panel, then không có dòng dẫn và mỗi khối nguồn mở bằng tên · số nghĩa · `⌄`.
- Given popup mở, when `↑↓` rồi `Space`, then nguồn đang chọn tắt và lượt tra chạy lại; when `Esc`, then tiêu điểm ở nút `Nguồn N/M ▾`.
- Given hai lượt tra ở hai Chương khác nhau, when chọn chip `Chương này`, then chỉ còn mục của Chương đang mở.
- Given đổi phím `lookup.lookup_selection` trong Cài đặt › Phím tắt, when nhìn tab Tra cứu, then phím mới hiện cạnh tiêu đề.

## Implementation Notes

- Âm đọc ở ba dạng (đo `resources/dict`): CC-CEDICT/CVDICT pinyin số tách âm tiết (`zou3 lang2`), Wiktionary liền có dấu (`zǒuláng`), 14.063/145.136 mục en-wiktionary có đuôi `(shan¹)`; 山 có cả `Shan1` và `shan1`. Đầu panel hiện khoá so (`zǒuláng`) theo Quyết định 2; hàng I/O ghi `zǒu láng` là chỗ lệch trong khối frozen, chưa sửa.
- Popup nguồn lúc đầu cao 375 px với 9 nguồn thật, đáy lọt dưới tab "Đề xuất AI" (e2e bắt, happy-dom không có bố cục): khi mở, `max-height` lấy theo chỗ còn lại tới đáy `.lookup-body`.
- `lookupHistoryState.ts` không import được `editorPanelState.ts` (vòng import), nên Chương đang mở được đẩy sang bằng `watch(chapterId, setLookupChapter, { flush: 'sync' })`.
- `dict_source` không có cột tên ngắn, popup chỉ hiện `display_name`; ghi nợ `Chủ: Epic 12`.
- Câu "chưa tra" rời `PanelFrame` (`:show-status="false"`) vào thân panel để mang phím thật của `lookup.lookup_selection`.
- `computeSpine` chỉ còn trả số; `SpineSource`/`SpineHiddenSource` bỏ cùng chip từng nguồn.
- Đối chứng đỏ: gỡ `trigger.value?.focus()` khỏi `Esc` ⇒ đỏ `expected <body> to be <button>`; cho `mergeChapter` trả `known` ⇒ đỏ `expected [] to deeply equal ['B']`.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens quick): high 0 · medium 0 · low 4 · false 3 · maybe-false 0 · loại vì sửa kế hoạch 1.

- low · reject — mục tiêu nhắm từ `@mousedown` không có `click` sau còn sót tới lần kích bằng phím kế tiếp (`toggleAimedSourceCollapse`, `lookupAimedChar`): cần kéo chuột ra rồi dùng phím; khuôn có từ trước ở `dictSourcesState::toggleFocusedDictSource`, sửa là thêm nhánh canh.
- false — Xoá khi chip `Chương này` bật xoá cả hàng ẩn: nhãn nút là "Xoá lịch sử phiên" (`vi.json:1161`), đúng điều nó làm.
- low · patch — `.tab-chord` chen giữa chú thích Kiểm B2 và `.tab.current` (`PanelTab.vue`); dời khối lên trên chú thích.
- reject (sửa kế hoạch) — bảng I/O ghi `zǒu láng`, Quyết định 2 ghi khoá `zǒuláng`; mã theo Quyết định 2, mâu thuẫn trong khối frozen báo Ice.
- false — e2e không chạy lượt tra: spec e2e trong Verification là để bắt vòng import làm app không khởi động, nó làm được; khối nguồn trong app thật là nợ kiểm tay `Chủ: Epic 12`.
- false — `status-key` thành dây chết: `statusKey` là prop bắt buộc của `PanelFrame.vue:49`, `showStatus` là cửa tắt có sẵn.
- low · reject — biểu thức phím hiển thị lặp ở `LookupPanel.vue` và `PanelTab.vue`: đã lặp sẵn ở `StatusBar.vue:232`, `TitlebarToolsMenu.vue`; gom là thêm hàm dùng chung.
- low · reject — `listMaxHeight` không đo lại khi đổi kích thước lúc popup đang mở: hiếm, sửa cần `ResizeObserver`.

## Design Notes

Mỗi mục CC-CEDICT/CVDICT là một âm (打: `da2`, `da3` là hai `entry_id`), nên "nghĩa nhóm theo âm" trong một khối chính là `clusters` theo `entry_id` sẵn có, chỉ thêm âm và số nghĩa vào đầu nhóm. Số mục trong popup đếm trên `groupedLookup` của lượt tra hiện tại; nguồn không có nhóm nào chỉ được ghi `0` khi nó bật, không bị LIMIT cắt và lớp không lỗi.

## Verification

**Commands:**
- `npm run build && cargo test --test dict_lookup` -- expected: xanh.
- `npm run test:story 12-8` -- expected: test panel Tra cứu, lịch sử, pinyin xanh.
- Một spec e2e của panel trước push (vòng import, bộ nhớ `vitest-khong-bat-tdz`).

**Manual checks (if no CLI):**
- Bản đóng gói macOS: popup nguồn dùng trọn bằng bàn phím, VoiceOver đọc trạng thái bật/tắt (nợ `Chủ: Epic 12`).

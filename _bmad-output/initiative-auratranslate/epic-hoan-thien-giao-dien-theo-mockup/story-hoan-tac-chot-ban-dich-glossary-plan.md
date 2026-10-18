---
title: 'Hoàn tác chốt bản dịch Glossary'
type: 'feature'
ticket: '20'
created: '2026-10-18'
status: 'built'
baseline_revision: 'c2fd3f4a95cc2a0c1776f34f477929f397beefbd'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/glossary-confirm-inline.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Chốt sai một bản dịch Glossary ở dải chốt (FR114) là ngõ cụt: vòng đời một chiều, mục đã chốt không về được chờ chốt (FR147).

**Approach:** Dựng đúng AD-54: cột `pending_confirmed_at` và trigger mới ở cả hai kho, lượt chốt ghi và trả cột, lệnh gỡ chốt kiểm ba điều kiện trong một giao dịch; dải hiện thông báo *Đã chốt* theo mockup với nút chạy lệnh mới không phím mặc định.

## Boundaries & Constraints

**Always:** AD-54 mục 1–8 là hợp đồng, plan không chép lại. Mọi `INSERT` để cột `NULL`; không người ghi `translation` nào khác (`update_manual_term`, `TakeTheirs`, `approve_candidate`, `promote_to_global`) được sửa để biết cột — trigger `AFTER` lo. Nút chỉ là vẽ; quyền gỡ chốt do trạng thái hàng trên đĩa quyết.

**Never:** bỏ trigger một chiều hay xoá-rồi-chèn; giữ lượt chốt chờ trong webview; gắn `Mod+Z` hay in cứng `⌘Z`; đưa mục gỡ chốt về bảng chờ hay đổi `glossary_candidate.resolution`; ánh xạ lỗi ABORT của trigger sang lỗi có kiểu (AD-54 mục 4).

## Decisions (Ice, 2026-10-18)

1. Không có hàng ⇒ mã có sẵn `glossary.entry_missing` (`MessageKey::GlossaryEntryMissing`), không thêm `glossary.not_found`; lệch chữ AD-54 mục 5 đúng một tên.
2. Phép kiểm lược đồ AD-54 mục 8 chạy trong `Store::open` cho Global và Project ngay sau `migrate`: thiếu cột hoặc một trong hai trigger ⇒ kho từ chối mở với lỗi có kiểu. Thời gian thêm của phép kiểm được đo và ghi số (build, tải máy).
3. Dải tự chuyển sang thuật ngữ chờ chốt kế ngay sau lượt chốt không làm nút chết; "dải chuyển sang mục khác" của AD-54 mục 6 là khi người dùng chốt hoặc Để sau mục đó, hay rời câu.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Chốt từ dải | mục chờ chốt, tầng Global hoặc Work | `translation` + `pending_confirmed_at` ghi cùng câu; lệnh trả cột | — |
| Chốt lại mục đã chốt (đua) | `translation` khác `NULL` | câu sửa riêng, cột không đặt, trả `null` | — |
| Gỡ chốt ngay | đúng chữ, đúng mốc | `translation` và cột về `NULL`, cột khác không đổi; mục rời prompt lượt lắp kế; vào *Để sau* | — |
| Sửa rồi gỡ | bản dịch đổi bằng bất kỳ đường nào | không ghi | `glossary.translation_changed` |
| Sửa về đúng chữ cũ rồi gỡ | chữ khớp, cột đã bị trigger xoá | không ghi | `glossary.not_unconfirmable` |
| Hàng có trước bậc mới | cột `NULL` | không ghi | `glossary.not_unconfirmable` |
| Đã đẩy lên Global / đã xoá | không còn hàng ở tầng gửi | không ghi | `glossary.entry_missing` |
| Sửa mỗi `note`/`category` | `translation` không đổi | cột giữ nguyên, gỡ chốt vẫn được | — |
| `UPDATE translation = NULL` trần | hàng đã chốt | ABORT như hôm nay | lỗi kho thô |

</frozen-after-approval>

## Code Map

Đường dẫn tính từ gốc repo.

- `src-tauri/src/core/store/schema.rs` -- `GLOSSARY_ENTRY_DDL` :300-332 (trigger :328) và bậc dựng lại `GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL` :521 (trigger :561) là lịch sử, không sửa. Mẫu một hằng cho hai thang: `GLOSSARY_ENTRY_OCCURRENCE_COUNT_AND_ZERO_WIDTH_GUARD_DDL` :567. Bậc mới: Global 14 (cuối hiện 13 `TRANSLATOR_NAME_DDL` :820), Project 40 (cuối hiện 39 `SEGMENT_VERSION_ORIGIN_DDL` :2429); `ADD COLUMN … CHECK`, `DROP TRIGGER` cũ, dựng hai trigger. Trigger zero-width :625 không đổi (recheck AD-54 đã kiểm không va).
- `src-tauri/src/core/store/mod.rs:640` `Store::open`, `migrate` :702 -- chỗ móc phép kiểm lược đồ (Quyết định 2); bỏ qua `StoreKind` LibraryIndex (:211-212).
- `src-tauri/src/core/glossary/store.rs` -- `confirm_translation` :258 hiện một câu `UPDATE … WHERE id` :266, tách hai nhánh và trả `Option<String>`; `confirm_pending_translation` :1297 chọn kho theo tầng (mẫu cho gỡ chốt); `GlossaryError` :446, `From<GlossaryError> for IpcError` :665 (`glossary.entry_missing` đã có); `MessageKey` ở `core/i18n/mod.rs:349`, chuỗi `src/i18n/vi.json:293`.
- `src-tauri/src/commands/glossary.rs` -- hàm thuần :525 + wire :1330 (`OpenWorkState`); lệnh mới đặt cạnh, đăng ký `src-tauri/src/lib.rs:1324`; `tests/ipc_argument_contract.rs:556` `REGISTERED_COMMAND_FLOOR` và đòi một `invoke()` TS khoá camelCase.
- `src/config/glossary.ts:552-580` -- wrapper chốt trả `GlossaryWriteResult<true>`, chưa guard; thêm guard theo mẫu `isGlossaryTmSuggestions` :592, wrapper gỡ chốt, và một bộ đếm lượt ghi Glossary (chưa có bus chung) mà mọi wrapper ghi trong tệp này tăng.
- `src/glossaryConfirmStripState.ts` -- chốt :465-501, `sequence` :120 (tăng khi đổi mục :280, Để sau :425, reset :510), tập *Để sau* theo `sourceTerm` :107, reset :521 (gọi từ `resetEditorPanel` `src/panels/editorPanelState.ts:1117` khi đổi Chương/Tác phẩm và `applyRegroup` :2793), làm mới dấu `refreshGlossaryMarks` :499.
- `src/GlossaryConfirmStrip.vue:161` -- `<form v-if>` gỡ khi dấu nạp lại nên thông báo phải nằm ngoài nó. Nối lệnh như `src/main.ts:974-979`.
- `src/commands/index.ts:3716-3750` -- lệnh `glossary.confirm.*`; mẫu không phím `ai.proofread.hide` :5037; hiện phím thật như `src/ProofreaderStrip.vue:42-53`. `scripts/check-commands.mjs`: Kiểm A (`@click` = `dispatch('<id>')`), E (`labelKey` = `command.<id>`), floors :247.
- Mockup `glossary-confirm-inline.html:195-198`: thanh `.done` "X → Y đã chốt. …" + "⌘Z hoàn tác" — in cứng `⌘Z` là lệch có chủ đích theo AD-54 mục 6; vế "ngày chốt ghi hôm nay" bỏ vì không cột nào là ngày chốt.
- Test phải đổi theo bậc mới (Global 13→14, Project 39→40): `pinned_contract.rs:91,99-100,162,261-272`; `store_contract.rs:1174,1268,1290`; `glossary_contract.rs:411,3341-3490,3441`; `segment_contract.rs:937,1017,1186,1267,1402,1727,1839,2053,2200-2253,6423`; `chapter_origin_contract.rs:1152`; `segment_role_contract.rs:870`; `review_harvest_contract.rs:688,701`; `tm_contract.rs:778,786,817,823,1220,1249`; `proofread_decision_contract.rs:278` (hạ bậc phải gỡ cả bậc 40). Vitest: `tests/frontend/glossaryConfirmStrip*.test.ts`, `glossaryIpcBridge.test.ts:113`. e2e mẫu: `e2e/specs/story-12-10-glossary-tm-suggestions.e2e.mjs`. Thêm: `glossary_boundary.rs`, `glossaryConfirmStrip{,Suggestion,Template,TmOptions}.test.ts`, `glossaryIpcBridge.test.ts`, `glossaryConfirmStripUndo.test.ts`, `stripAnchorSegment.test.ts`, `proofreaderStripSlots.test.ts`.

## Tasks & Acceptance

**Execution (ba pha mã, mỗi pha một agent mới, bàn giao qua `handoff-hoan-tac-chot-ban-dich-glossary-phases/`):**
- [x] `schema.rs` `core/store/mod.rs` -- Pha 2 Rust: một hằng DDL, bậc Global 14 + Project 40; phép kiểm cột và hai trigger lúc mở kho, lỗi có kiểu + `MessageKey`; đo thời gian thêm.
- [x] `core/glossary/store.rs`, `core/i18n/mod.rs`, `commands/glossary.rs`, `lib.rs` -- Pha 2: tách `confirm_translation`; lượt chốt trả mốc; `glossary_unconfirm_translation` với ba lỗi theo thứ tự AD-54 mục 5; `MessageKey` + chuỗi `vi.json`.
- [x] `src/config/glossary.ts`, `glossaryConfirmStripState.ts`, `GlossaryConfirmStrip.vue`, `commands/index.ts`, `main.ts`, `i18n/vi.json` -- Pha 3 Webview: thông báo *Đã chốt* (thuật ngữ → bản dịch) với nút lệnh `glossary.confirm.undo` không phím, gợi ý phím thật khi có; nút chết theo AD-54 mục 6 (đổi mục, đóng, reset, bộ đếm lượt ghi tăng); thành công ⇒ vào *Để sau*, làm mới dấu bằng đường sau lượt chốt; lỗi hiện bằng `tError`.
- [x] Pha 4 Tests: ma trận I/O ở `glossary_unconfirm_contract.rs` mới, cả hai tầng; kho dựng ở bậc 13/39 rồi lên bậc mới cho ca hàng cũ; mỗi người ghi khác (`update_manual_term`, `TakeTheirs`, `approve_candidate`) chạy thật trên hàng có cột (recheck AD-54 H1); prompt lắp sau gỡ chốt không còn thuật ngữ; không `segment.target_text` nào đổi. Vitest: nút chết theo từng sự kiện, gợi ý theo phím gán lại, *Để sau*. e2e `story-12-20-glossary-unconfirm`. Đối chứng đỏ bằng phép gỡ thật: bỏ trigger `AFTER`; bỏ đặt cột ở nhánh chốt; bỏ tăng bộ đếm ở một wrapper ghi.
- [x] `deferred-work.md` -- nợ lượt dùng thật `Chủ: Epic 12` nếu e2e không chạy được.

**Acceptance Criteria:**
- Given một thuật ngữ chờ chốt, when chốt ở dải rồi bấm nút trên thông báo *Đã chốt*, then mục về chờ chốt, không vào bảng chờ, dải không mọc lại trên thuật ngữ đó trong Chương, và lượt lắp prompt kế không chèn nó.
- Given thông báo *Đã chốt* đang hiện, when sang Chương khác hoặc một lượt ghi Glossary khác xảy ra, then nút không còn.
- Given lệnh gỡ chốt được gán phím qua FR22, when thông báo hiện, then nó in đúng phím đó; chưa gán thì không in phím.
- Given hai kho mở mới, when chạy hết di trú, then phép kiểm thấy cột và cả hai trigger; một bậc dựng lại bảng bỏ quên chúng làm phép kiểm đỏ.

## Implementation Notes

- Bậc Project 40 và Global 14; `PROJECT_MIGRATIONS` không có số 4 nên cắt thang cho kho bậc N là `[..N-1]`, còn `GLOBAL_MIGRATIONS[..13]` cho bậc 13.
- Bậc mới `DROP TRIGGER` trigger cũ không `IF EXISTS`: fixture "đĩa trôi" của `glossary_contract` phải mang trigger một chiều giả, và mọi ca hạ bậc thủ công (`tm_contract`, `proofread_decision_contract`) phải gỡ cột rồi dựng lại trigger cũ trước khi mở lại.
- `approve_candidate` thu hoạch review là đường ghi `translation` thật duy nhất trên hàng có sẵn; ứng viên chỉ tồn tại ở `project.db`, nên ca đó chỉ chạy ở tầng Work.
- Thông báo *Đã chốt* từng vô hình trong app thật: `inlineStripAnchorSegmentId` chỉ cấp track cho dải khi `confirmStripIsOpen`, mà thông báo sống sau khi mục cuối được chốt. Vitest (happy-dom, không bố cục) không thấy; e2e đo `getBoundingClientRect` 0x0 mới lộ. Sửa bằng `confirmStripNoticeShown` đưa vào `eligibleInlineStrips`.
- Chiều ghi `Enter` trong ô nhập của dải qua `browser.keys` không gửi form ở e2e; spec bấm nút Lưu bằng `realClick`.
- Phép đối chứng: bỏ trigger `AFTER` (kèm tên khỏi danh sách phép kiểm lúc mở kho) đỏ 4 ca; bỏ đặt cột ở nhánh chốt đỏ 11/14; bỏ bộ đếm ở `rejectGlossaryCandidate` đỏ 3 ca vitest.
- Lượt dùng thật trên bản đóng gói còn là nợ `Chủ: Epic 12` trong `deferred-work.md`.
- Con trỏ câu thành `null` ngay trong lúc lượt chốt đang bay (e2e đo: trước Lưu `caret=1`, 300 ms sau `null`), nên thông báo không bao giờ được tạo; `null` không phải rời câu (`setEditorCaret`): thông báo nhớ câu của nó, chỉ chết khi con trỏ sang câu khác khác `null`, và track neo vào câu đó (`confirmStripNoticeSegmentId`). Vitest không thấy vì happy-dom không làm mất tiêu điểm.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick + ba phát hiện của người điều phối): medium 2 · low 4 · false 3 · intent 1 (Ice đáp, Quyết định 3).
- R1 medium · patch: không ca nào mở kho thiếu cột/trigger để thấy `store.glossary_lifecycle_missing` (grep `src-tauri/tests` không ra); AC4 chưa canh.
- R2 medium · patch: `bumpGlossaryWriteCount()` của chính lượt gỡ chốt làm `confirmStripUndo` về `null` lúc bắt đầu, nên "Đang hoàn tác…" và `:disabled` không bao giờ hiện, lỗi làm thông báo nháy lại.
- R3 low · patch: doc-comment của `confirmStripIsOpen` bị tách, nằm trên `confirmStripUndo`.
- R4 false: mục nợ có trong `deferred-work.md` với `Chủ: Epic 12`; diff gửi reviewer loại `_bmad-output`.
- R5 false: `check:panel-refs` khai miễn trừ có tên kèm lý do là đường hợp lệ của chính cổng; bộ đếm không phải dữ liệu Tác phẩm.
- R6 low · reject: số đo phép kiểm (0,6–3,5 ms) chỉ ở handoff; sửa là sửa plan, số đi vào commit.
- R7 false: e2e bấm Hoàn tác qua lệnh thật rồi kiểm `translation` về `null`; đảo thứ tự hai `&str` sẽ ra `translation_changed` và làm e2e đỏ.
- O1 low · reject: `trim()` của JS và Rust khác nhau ở vài ký tự hiếm (U+FEFF, U+0085), chữ gửi lại có thể lệch chữ đã lưu; mọi lựa chọn đã `trim()` ở webview, sửa cần thêm trường dây.
- O2 low · patch: doc-comment mới của `undoGlossaryConfirmStrip` bằng tiếng Việt.
- O3 intent: dải tự chuyển sang thuật ngữ kế sau lượt chốt giữ nút; Ice chọn giữ (Quyết định 3), mã không đổi.
- V1 high · patch (lượt Verification sau vá): e2e đỏ hai lượt liền ở "thông báo không hiện"; dò module trong webview thấy con trỏ `null` giữa lượt chốt làm `stillCurrentTarget` sai và luật rời câu đọc `null` là rời. Vá ba chỗ ở `glossaryConfirmStripState.ts`/`inlineStripEligibility.ts`, bốn ca vitest mới, mỗi chỗ đối chứng đỏ bằng phép gỡ thật; e2e xanh hai lượt.

## Verification

**Commands:**
- `npm run build && npm run test:story 12-20` -- expected: mọi ca chọn ra xanh.
- `cargo test` toàn bộ và `npx vitest run` toàn bộ một lần -- expected: xanh; bắt buộc vì story chạm migration và đăng ký lệnh trong `lib.rs`.
- `npm run test:e2e -- --spec e2e/specs/story-12-20-glossary-unconfirm.e2e.mjs` (ngoài sandbox) -- expected: xanh, hoặc ghi nợ nếu webview không mount.

**Manual checks:**
- Lượt dùng thật trên bản đóng gói: chốt, gỡ chốt, thấy dấu thuật ngữ trở lại nét đứt.


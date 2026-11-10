---
title: 'Quét lại sau flush và quét gợi ý ở panel AI Translation'
type: 'feature'
ticket: '4'
created: '2026-11-10'
status: 'built'
baseline_revision: '9d61b71a409a38662e2bbe467b5335386a15aa6b'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-67 mục 4, 5, 11, 12b'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md §Quy ước chữ Việt — FR159 (State Patterns hàng "Câu đang gõ" và "Gợi ý AI đang chảy", Interaction Primitives)'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Sau 16.1–16.3 dấu quy ước chỉ sinh ra ở một lượt quét khi mở Chương. Người dùng gõ xong một câu thì dấu của câu đó chết và **không bao giờ trở lại** cho tới lần mở Chương sau, và gợi ý ở panel AI Translation chưa bao giờ được quét — hai vế còn lại của AD-67 mục 4.

**Approach:** Sau mỗi lượt flush thành công (AD-35), webview xin quét lại **đúng** các câu vừa flush; Rust đọc văn bản từ đĩa, không nhận chữ từ webview (AD-1, AD-67 mục 4). Kết quả **gộp** vào tập kết quả đang có thay vì thay cả tập. Gợi ý AI khi đã trọn được quét bằng một lệnh chỉ-đọc riêng và vẽ dấu khai báo trên panel, không dải, không lệnh ghi. 0 lời gọi IPC mới trên đường gõ.

**Decision (2026-11-10, Ice):** quét cả gợi ý một câu **và** các hàng của lượt dịch mẻ (`aiTranslateBatchRows`), không quét bản nháp chờ nhận — nó sắp thành `target_text` và sẽ được quét lại ngay sau khi nhận. Một lượt mẻ đi trong **một** lời gọi nhận nhiều chuỗi, không N lời gọi.

**Decision (2026-11-10, Ice):** lệnh quét gợi ý nhận `segment_id` của từng chuỗi và **chịu phép kiểm Glossary** (R12) như đường quét Chương, để dấu trên gợi ý và dấu trong ô sau khi nhận giống nhau, và để 16.5 lọc được ghi nhớ "không phải lỗi" trên gợi ý bằng cùng đường. Giá đã nhận: một lượt nạp `GlossarySnapshot` mỗi lượt quét.

**Decision (2026-11-10, Ice):** giữ nguyên cả hai nửa của vé 16.4 dù bản kế hoạch vượt trần 1600 token của BMad; AGENTS.md đã chẻ story thành ~4 phase với một agent MỚI mỗi phase.

## Boundaries & Constraints

**Always:**
- AD-67 mục 4 (ba nơi quét, thế hệ quét), mục 5 (Glossary chỉ kiểm câu có chỗ khớp), mục 6 (bốn số đếm không im lặng), mục 11 (ngân sách), mục 12b.
- Lượt quét lại chạy **sau** khi `flushEditorNow` đã trả `'saved'`, ngoài đường chờ của nó: một lượt quét trượt hay chậm không được làm chậm hay hỏng lượt lưu kế tiếp.
- Chỉ các câu vừa flush được quét lại; kết quả của các câu khác trong `conventionResults` giữ nguyên từng chỗ.
- Thế hệ quét cũ hơn lượt đổi Chương, đổi Tác phẩm hay đổi danh sách luật bị bỏ (AD-67 mục 4), kể cả ở đường flush và đường gợi ý.
- Dấu trên gợi ý dựng từ mô hình khoảng của Rust bằng interpolation của Vue; AD-16 không đổi chữ.
- Mọi khoá i18n mới đi qua `check:i18n` và `check:commands`.

**Never:**
- Gửi chữ đang gõ cho một lệnh quét; thêm một lời gọi IPC nào trên đường gõ (`reportEdit`, `noteEditorEdit`, bộ hẹn idle).
- `editorPanelState.ts` import `conventionState.ts` (chiều phụ thuộc hôm nay là ngược lại; vòng import + mã cấp module không bị vitest bắt).
- Lệnh ghi nào nhận khoảng của một gợi ý; dải, nút **Nhận thay** hay **Thay cả Chương** trên panel AI Translation.
- Quét gợi ý đang chảy; quét câu đã cắt hay đã về hưu.
- "Không phải lỗi" / `CONVENTION_IGNORE` (16.5); xuất nhập (16.6). `ignored` vẫn là 0.
- Đổi chữ ký ghi nhớ, lượt ghi hay hành vi dải của Proofreader; đổi hình `convention_scan_chapter` mà 16.1–16.3 đang gọi.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Gõ rồi ngừng | luật `đảm bảo→bảo đảm`, gõ `đảm bảo` vào ô trống | dấu của câu đó biến ở phím đầu; sau lượt flush dấu hiện lại; 0 lời gọi IPC trong lúc gõ | — |
| Chỉ câu vừa flush | hai câu có dấu, sửa câu A | kết quả câu B không bị chạm; chỉ A được quét lại | — |
| Flush trượt | `flushEditorNow` trả `'failed'` | không lượt quét nào; dấu cũ không hiện lại | tập chờ giữ nguyên |
| Quét lại trượt | lệnh quét lại ném | dấu cũ của câu đó vẫn vắng, không dấu mới, `convention.status.scan_failed` | lượt lưu kế tiếp không bị ảnh hưởng |
| Đổi Chương giữa lượt | flush rồi đổi Chương trước khi kết quả về | kết quả bị bỏ, không ghi vào tập của Chương mới | — |
| Câu thành rỗng | sửa câu có dấu thành trống | không dấu; câu không vào tập quét | — |
| Câu bị cắt sau khi quét | `is_omitted` bật rồi flush | không dấu, đếm vào `not_scanned` | — |
| F8 khi đang gõ | con trỏ ở câu đang gõ, không còn chỗ nào phía sau | `convention.status.sentence_pending` thay cho `no_next`/`no_marks` | — |
| Gợi ý đã trọn | gợi ý chứa `Đảm bảo`, trạng thái `done` | dấu chấm trên gợi ý + lời chú khi rê chuột; không dải, không nút thay | — |
| Gợi ý đang chảy | trạng thái `generating` | không dấu | — |
| Hàng của lượt mẻ | lượt mẻ xong (`aiTranslateBatchState` = `'done'`) | **một** lời gọi quét cho mọi hàng `'received'`; dấu theo từng hàng | — |
| Hàng mẻ chưa nhận | hàng `'queued'`/`'streaming'`/`'missing'`/`'not_sent'` | không dấu — `row.text` lúc đó là **câu nguồn**, không phải bản dịch | — |
| Glossary che trên gợi ý | chỗ khớp nằm trọn trong bản dịch Glossary đã chốt của câu đó | không dấu trên gợi ý, `glossary_suppressed` tăng | — |
| Gợi ý không có Tác phẩm/luật | 0 luật hiệu lực, hay chưa mở Tác phẩm | không dấu, không báo lỗi | — |

</frozen-after-approval>

## Code Map

Rust (`src-tauri/src/`):
- `commands/convention.rs:274` `read_scan_input(open)` — lọc `is_omitted` và `target_text` rỗng, đếm `not_scanned`. Thêm tham số lọc theo tập id (`Option<&[i64]>`); `match_sentences` :319, `load_glossary` :301 và `ConventionScanWire` :241 dùng lại **nguyên hình**.
- `commands/convention.rs:411` `convention_scan_chapter_with_progress` — khuôn "khoá để đọc, nhả khoá rồi khớp" cần chép cho lệnh mới; `cache.matcher(global, open)` giữ bộ khớp theo Tác phẩm.
- `commands/convention.rs:602` `pub mod wire` — nơi thêm `convention_scan_segments(generation, segment_ids)` và `convention_scan_text(generation, text)`; `state_missing` :454.
- `core/convention/matcher.rs:30` `Matcher::build`, `:65` `Matcher::scan(&self, text)` — nhận văn bản rời nên dùng được cho gợi ý mà không thêm hàm thuần nào.
- `lib.rs:1121` danh sách handler; `core/i18n/mod.rs:100` `message_keys!`.
- Cổng: `tests/config_invariants.rs:1449` `blocking_wire_cases` (khai hai lệnh mới là blocking hay không); `tests/ipc_argument_contract.rs:630` `NO_FRONTEND_CALLER`.

Webview (`src/`):
- `panels/editorPanelState.ts:891` `flushEditorNow` — `snapshot` :909 và `edits` :910 là các biến **cục bộ** mang đúng tập id vừa ghi; đường thành công là :933-936 (`flush.onFlushed` → `lastSavedAt.value`). Chỗ duy nhất mọi đường flush đi qua (bộ hẹn :795, rời câu :329, thoát app :1244, `flushEditorBeforeDiscreteWrite` :999).
- `panels/editorFlush.ts:95` `pending()`, `:106` `isDirty()` — `isDirty` là toàn cục, chưa có phép hỏi theo từng câu.
- `conventionState.ts:99` `scanConventionNow` (bơm `++generation` :100 và **thay cả** `results`), `:49` `results`, `:131` `clearConventionFor`, `:148` `liveEntry`, `:69` `say`, `:496` `stepMarks` (hai chỗ `say`: `no_marks` :499 và `no_next`/`no_prev` :527), `setEntry` (dùng ở :355, :426) — hàm riêng đã làm đúng việc ghi một câu.
- `config/convention.ts:181` `scanConventionChapter`, `:71-80` hằng tên lệnh, `:120` `isConventionScanWire`.
- `panels/GridPanel.vue:1448-1450` lượt quét khi đổi Chương; `:1763` `reportEdit` (chỗ duy nhất đọc DOM vào tập chờ, gọi `clearConventionFor` :1771); `:1269` `repaintProofread`, `:1275` `conventionMarkEntries`.
- `panels/proofreadHighlight.ts:152` phép so `own.scannedText === text` — lý do kết quả phải mang `scanned_text` của chính lượt đọc đĩa.
- `aiTranslateState.ts:70` `aiTranslateStateValue` (`'done'` là đã trọn), `:71` `aiTranslateAccumulatedText`, watch :127.
- `aiTranslateBatchState.ts:14` `AiTranslateBatchRowStatus` — ⚠️ "đã nhận" là `'received'`, **không** `'done'`; `:16` `AiTranslateBatchRow { segmentId, status, text, … }`, `:26` `AiTranslateBatchState` (`'done'` là cả mẻ xong), `:39` `aiTranslateBatchRows`, `:82` `aiTranslateBatchTextForSegment`.
- `panels/AiTranslationPanel.vue:624-625` `<p data-ai-translate-text>{{ aiTranslateAccumulatedText }}</p>` và `:799-801` `<span data-ai-translate-batch-row-text>{{ batchRowTexts[index] }}</span>` (computed :355) — đều là một text node kèm chú thích `aura-allow-text`. ⚠️ `batchRowTexts` trả **câu nguồn** khi hàng chưa có bản dịch, nên chỉ hàng `'received'` được vẽ dấu.
- `i18n/vi.json:911-912` `convention.mark.title_*` đã có; `:932-936` các khoá `convention.status.*`.

Tests: `src-tauri/tests/convention_contract.rs`; `tests/frontend/conventionState.test.ts`, `tests/frontend/conventionRescanAfterFlush.test.ts`, `tests/frontend/conventionSuggestionMarks.test.ts`, `tests/frontend/aiTranslationPanelConventionMarks.test.ts`, `tests/frontend/aiTranslate.test.ts`, `tests/frontend/aiTranslateBatch.test.ts`, `tests/frontend/aiBatchPanelRows.test.ts` (mock `config/convention`), `proofreadHighlight.test.ts`; `e2e/specs/story-16-1-convention-mark-accept.e2e.mjs` (khuôn), `e2e/support/flushWait.mjs` (`markFlushBaseline`, `waitForFlushAfter`).

## Tasks & Acceptance

**Execution:**
- [ ] `commands/convention.rs` — `read_scan_input` nhận bộ lọc id; `convention_scan_segments(generation, segment_ids)` async theo khuôn :411 (một lượt đọc, nhả khoá, khớp, Glossary chỉ trên câu có chỗ khớp), trả đúng `ConventionScanWire`; id lạ hay ngoài Chương đang mở ⇒ bỏ qua, không lỗi.
- [ ] `commands/convention.rs` — `convention_scan_suggestions(generation, items: [{segment_id, text}])`: khớp từng chuỗi bằng bộ khớp đã cache, một `GlossarySnapshot` cho cả lượt, câu nguồn đọc từ đĩa theo `segment_id` và **chỉ** trên chuỗi có chỗ khớp (AD-67 mục 5); trả khoảng theo từng mục + `glossary_suppressed`/`overlapped`; không ghi hàng `segment` nào, không nhận lệnh ghi khoảng.
- [ ] `lib.rs`, `core/i18n/mod.rs`, `tests/config_invariants.rs` — đăng ký hai lệnh, khai `blocking_wire_cases`, khoá lỗi mới nếu có.
- [ ] `panels/editorPanelState.ts` — xuất `onEditorFlushed(cb)` (danh sách người theo, gọi **sau** `lastSavedAt.value` với tập id của `snapshot`, bọc `try`/`catch`, không `await` trong đường flush) và `editorSegmentIsPending(id)` đọc `flush.pending()`. Không import `conventionState`.
- [ ] `conventionState.ts` — `scanConventionSegments(ids)` **gộp** kết quả vào `results` (chỉ các câu được quét), cộng dồn bốn số đếm, bỏ kết quả lệch thế hệ; đăng ký người theo flush một lần ở module này; `stepMarks` báo `sentence_pending` khi câu của con trỏ đang chờ flush, ở cả hai chỗ `say`.
- [ ] `config/convention.ts` — hai adapter mới theo khuôn :181, không bao giờ ném.
- [ ] `conventionSuggestionMarks.ts` (mới) — quét khi `aiTranslateStateValue` sang `'done'` (một mục) và khi `aiTranslateBatchState` sang `'done'` (mọi hàng `'received'`, một lời gọi); giữ khoảng theo thế hệ và theo `segmentId`; xoá khi một lượt mới bắt đầu chảy, khi đổi câu, đổi Chương hay đổi danh sách luật.
- [ ] `panels/AiTranslationPanel.vue` — dựng danh sách phần văn bản từ khoảng cho gợi ý một câu và cho từng hàng mẻ `'received'`, render bằng `v-for` + interpolation (`convention-mark` cho phần khớp, `title` từ `convention.mark.title_*`); không `v-html`, không thao tác DOM, giữ chú thích `aura-allow-text` trên mỗi chỗ nội suy dữ liệu.
- [ ] `i18n/vi.json` — `convention.status.sentence_pending` (nguyên văn EXPERIENCE dòng 553).
- [ ] `src-tauri/tests/convention_contract.rs` — các hàng ma trận thuộc Rust: quét theo tập id không chạm câu khác, câu rỗng/bị cắt, Glossary trên đường quét lại **và** trên đường gợi ý, `convention_scan_suggestions` không ghi hàng nào, hai đường quét cho cùng tập chỗ khớp trên cùng một câu; mỗi guard mới đối chứng bằng một phép **gỡ** thật.
- [ ] `tests/frontend/conventionState.test.ts` + spec mới cho gợi ý — ca bất đối xứng: gõ ⇒ **0** lời gọi adapter quét; flush thành công ⇒ **đúng một** lời gọi với **đúng** tập id; flush trượt ⇒ 0; thế hệ cũ bị bỏ; `sentence_pending`; gợi ý `generating` ⇒ 0 dấu, `done` ⇒ có dấu; mẻ 50 hàng ⇒ **một** lời gọi và hàng chưa `'received'` không có dấu.
- [ ] `e2e/specs/story-16-4-convention-rescan-after-flush.e2e.mjs` — `verify` của vé trên WKWebView.

**Acceptance Criteria:**
- Given một luật `đảm bảo → bảo đảm` và một ô trống, when người dùng gõ `đảm bảo` rồi ngừng, then dấu biến ở phím đầu tiên và hiện lại sau lượt flush, và không lời gọi IPC quét nào xảy ra trong lúc gõ.
- Given hai câu đang có dấu, when chỉ một câu được sửa rồi flush, then câu còn lại giữ nguyên dấu và `scanned_text` của nó không bị ghi lại.
- Given một lượt quét lại trượt, when lượt flush kế tiếp chạy, then lượt lưu vẫn thành công và mốc `editorLastSavedAt` vẫn tăng.
- Given con trỏ ở một câu đang gõ và không còn chỗ đánh dấu nào phía sau, when người dùng bấm F8, then thanh trạng thái báo *Câu đang gõ chưa quét lại; dấu hiện sau lượt lưu kế tiếp.*
- Given một gợi ý đã trọn chứa `Đảm bảo`, when panel AI Translation vẽ nó, then chỗ đó có dấu và lời chú khi rê chuột, và panel không có dải cũng không nút thay.
- Given một lượt dịch mẻ 50 câu, when lượt mẻ xong, then đúng **một** lời gọi quét chạy cho cả mẻ và chỉ các hàng `'received'` có dấu.
- Given cùng một câu, when nó được quét qua `convention_scan_chapter` và qua `convention_scan_segments`, then hai lượt cho cùng tập chỗ khớp.

## Implementation Notes

- ⚠️ `__TAURI_INTERNALS__.invoke` trên WKWebView là non-writable và non-configurable: phép bọc nó từ spec e2e im lặng không làm gì, và ca "0 lời gọi IPC khi gõ" xanh RỖNG ở lượt chạy đầu. Spec bọc `window.fetch` (lệnh ra khỏi webview dưới dạng `ipc://localhost/<cmd>`) và tự kiểm bằng một lời gọi adapter thật trước khi tin vào sổ ghi.
- `convention_scan_segments` chỉ trả về những câu CÒN dấu, nên "vắng mặt" nghĩa là "đã sạch", không phải "không biết". Phép gộp xoá entry của mọi id đã xin quét trước khi đặt lại cái trả về; chỉ gộp cái có là để dấu cũ sống sót trên câu vừa sửa sạch.
- Hàng mẻ chưa `'received'` hiển thị CÂU NGUỒN (`batchRowTexts` lùi về `source_text`), nên điều kiện `status === 'received'` ở tầng vẽ là thứ duy nhất chặn dấu quy ước đậu lên câu nguồn. Gỡ nó ra thì ca âm ở `aiTranslationPanelConventionMarks.test.ts` đỏ.
- `conventionSuggestionMarks.ts` nổ adapter thật ngay khi một lượt dịch chạm `'done'`, nên ba tệp test AI không liên quan phải mock `config/convention` ở biên. `failure()` của adapter chạy TRONG `catch`, nên một mock khuyết `isIpcError` làm nó ném ra ngoài hợp đồng "không bao giờ ném" — trong sản phẩm đường đó không tồn tại.
- `select_segments_by_id` đọc từng id bằng một `SELECT` có chỉ mục thay vì đọc cả Chương, để đường flush không trả giá theo độ dài Chương.
- 🔴 `check:panel-refs` đã ĐỎ 15 lỗi ở `conventionManageState.ts` và `conventionReplaceState.ts` từ 16-2/16-3 (ba commit còn chưa push), đo trên worktree sạch của `9d61b71a`. Cổng này nằm trong `pre-push` và `ci.yml`, nên nó chặn lượt push của 16.4 dù 16.4 không chạm hai tệp đó.

## Plan Change Log

- 🔵 2026-11-10 — §Tasks nói đường quét lại "cộng dồn bốn số đếm"; điều đó bị **bỏ**. `counts` là tổng cấp Chương, một lượt quét vài câu không dựng lại được nó, nên đường quét lại không chạm vào nó nữa. Ca `leaves the Chapter-wide counts alone` canh điều này.
- 🔵 2026-11-10 — §Boundaries chỉ nói bỏ thế hệ cũ khi đổi Chương, đổi Tác phẩm hay đổi danh sách luật, nên nó không phủ lượt quét đầy và lượt quét lại cùng bay. `conventionState.ts` nay giữ một tập `mergedSinceScan`: lượt quét đầy KHÔNG ghi đè những câu mà một lượt quét lại đã viết trong lúc nó bay, kể cả khi lượt quét lại đã xoá entry. Đọc đĩa của lượt quét lại muộn hơn nên nó thắng.

- 🔵 2026-11-10 — Ice chọn **cấp module**, đúng như §Tasks viết, nên bản đăng ký muộn (`followFlushOnce` trong `scanConventionNow`) đã bị gỡ cùng cờ `followsFlush` và miễn trừ cổng của nó: `check:panel-refs` nay còn 58 miễn trừ, ít hơn một. Giá đã trả, đo thật: **11 factory mock ở 10 tệp** phải khai thêm `onEditorFlushed`, và **6 factory** phải khai thêm `scanConventionSegments` — một lượt flush thật trong các tệp ấy làm người theo gọi adapter và `void` biến lỗi mock thành unhandled rejection, thoát 1 dù mọi ca xanh. Đối chứng: gỡ phép đăng ký ⇒ 10 ca đỏ.
- Dấu gợi ý KHÔNG bị xoá khi đổi câu như Tasks viết; thay vào đó giữ theo `segmentId` và chỉ vẽ khi `scannedText` khớp đúng chữ đang hiển thị (`conventionSuggestionMarks.ts:112`, cùng hình với `proofreadHighlight.ts:152`). Phép so này đóng chính cái bẫy mà "đổi câu" nhắm, và đóng cả khi lượt xoá không chạy.
- `conventionGeneration` bơm ở mọi `scanConventionNow`, nên lượt thay cả Chương và lượt nhận cũng xoá dấu gợi ý tới lượt sinh kế tiếp.
- `scripts/check-panel-refs.mjs` nhận hai miễn trừ có tên (`editorPanelState.ts::flushedListeners`, `conventionState.ts::followsFlush`); tệp này không có trong Code Map. Câu lý do đầu khai sai "đăng ký lúc nạp module" và đã sửa thành "ở lượt quét Chương đầu tiên".

## Review Triage Log

- 🔵 2026-11-10 — Ice chọn **vá tại chỗ** cả hai mục dưới thay vì hoàn nguyên mã rồi sửa §Boundaries trước. Hai phép vá đã vào `conventionState.ts` và đều có ca canh gỡ được: gỡ vòng giữ ⇒ 2 đỏ, trả lại phép cộng dồn ⇒ 3 đỏ.
- `low` · patch · ✅ đã vá — Bốn số đếm cộng dồn ở `scanConventionSegments` nên một câu bị Glossary che hay đã cắt được đếm lại sau MỖI lượt flush. `conventionCounts` hôm nay không có nơi đọc nào ngoài test, nên chưa ai gặp; phép sửa nhỏ nhất là XOÁ lượt cập nhật đếm ở đường quét lại, vì một lượt quét vài câu không thể dựng tổng cấp Chương.
- `medium` · intent_gap → vá tại chỗ theo lệnh Ice · ✅ đã vá — Một lượt flush xong GIỮA lúc `scanConventionNow` đang bay mang cùng thế hệ, nên phép gộp lọt qua rồi bị bản đầy (đọc đĩa TRƯỚC lượt ghi) thay cả tập; dấu của câu vừa flush mất và không gì gọi lại lượt quét cho nó. Đúng cái hỏng mà vé này tồn tại để chữa. Đường tới được: gõ dở rồi bật/tắt một luật ở lớp phủ quản lý, bộ hẹn 2 s nổ trong lúc quét. §Boundaries chỉ nói bỏ thế hệ cũ khi đổi Chương, đổi Tác phẩm hay đổi danh sách luật — không nói hai đường quét phân xử nhau thế nào.
- `false` — "Lượt quét lại trượt vô hình với F8": hàng ma trận 4 khai đúng hành vi này (dấu cũ vẫn vắng, không dấu mới, `scan_failed`), nên không có kết cục xấu ở chỗ bị trỏ.
- `low` · rejected — F8 trong khe giữa lúc flush xong và lượt quét lại về báo `no_marks` thay vì `sentence_pending`. Khe đúng một vòng IPC; phép sửa phải thêm một trạng thái "đang quét lại", tức hơn một phép sửa trực tiếp.
- `low` · rejected — Người theo cũng nổ ở lượt flush do `flushEditorBeforeDiscreteWrite`, nên một lượt quét lại có thể hạ cánh SAU `setEntry` của accept và để lại entry cũ. Không sinh ra lượt ghi sai chỗ: `acceptConventionSpan` chịu phép kiểm `expectedText`/`expectedPhrase` ở Rust; hậu quả là nút accept/revert từ chối trên câu đó tới lượt quét sau. Phép sửa cần trạng thái mới.
- `low` · rejected — Hàng `'received'` của một mẻ kết thúc bằng lỗi hay bị huỷ không có dấu. Hàng ma trận 11 gắn lượt quét vào `aiTranslateBatchState === 'done'`; đây là trạng thái lỗi, và phép sửa thêm một nhánh.
- `low` · rejected — `watch(conventionGeneration, …)` xoá dấu gợi ý ở mọi lượt quét đầy, kể cả sau một lượt đổi luật. Đã ghi ở Plan Change Log; dấu vắng không khẳng định câu đạt quy ước.
- `low` · rejected — `convention_scan_suggestions` bỏ im lặng mục có `segment_id` không thuộc Chương đang mở. Hàng của lượt mẻ luôn thuộc Chương đang mở nên đường này gần như không tới được; phép sửa thêm một số đếm vào dây, tức thêm mặt công khai.
- `maybe-false` · rejected — Đường quét gợi ý không đọc `is_omitted` nên gợi ý của một câu đã cắt vẫn được đánh dấu. Cần bằng chứng rằng một câu `is_omitted` gửi được đi dịch AI mới kết luận; nếu đúng thì cũng chỉ `low`.
- `false` — "Hai miễn trừ `check:panel-refs` là nới danh sách loại trừ": cổng này có sẵn cơ chế miễn trừ CÓ TÊN kèm lý do, và có tiền lệ cùng hình (`onChapterOpened`). Một câu lý do khai sai đã được sửa trong lượt này.
- `false` — "Không có bằng chứng đối chứng đỏ cho các ca Rust": các phép gỡ thật đã chạy (bộ lọc id, Glossary đường gợi ý, `notifyFlushed`, bộ lọc `'received'`, `next.delete`, phép gộp). Bằng chứng thuộc tệp handoff và thông điệp commit, không thuộc mã.
- ✅ đã chốt — chỗ đăng ký người theo flush: Ice chọn cấp module, mã đã chuyển, xem Plan Change Log. Không phải khuyết tật.

## Design Notes

- **Chiều phụ thuộc.** `conventionState` → `editorPanelState` là chiều đang có; một `import` ngược lại dựng vòng import mới, và vòng import + mã cấp module làm app không khởi động mà `pre-push` vẫn xanh. Nên đường quét lại là một **người theo** do `conventionState` tự đăng ký, không một lời gọi từ `flushEditorNow`.
- **Gộp, không thay.** `scanConventionNow` thay cả `results` vì nó quét cả Chương. Đường flush chỉ biết vài câu, nên nó phải gộp; thay cả tập ở đây là xoá dấu của mọi câu khác — đúng hình "im lặng rỗng".
- **Tập id lấy từ `snapshot`, không từ `dirty`.** `flush.onFlushed` chỉ xoá entry khi `dirty.get(id) === text`, nên tập "vừa ghi" là `snapshot`; người dùng gõ tiếp trong lúc lô đang bay thì câu đó còn bẩn và lượt flush sau quét lại nó.
- **Quét ngoài đường chờ của flush.** `flushEditorNow` là đường lưu dữ liệu; `await` một lượt quét trong đó đẩy trần `EDITOR_HARD_CAP_MS` và biến một lượt quét trượt thành một lượt lưu trượt.
- **Dấu trên gợi ý là khai báo.** Khoảng từ Rust là mô hình có cấu trúc; render bằng `v-for` + `{{ }}` nên AD-16 được giữ mà không cần `proofreadHighlight` (vốn gắn với ô contenteditable của lưới).
- **`sentence_pending` thay câu báo, không chặn đường đi.** Còn chỗ phía sau thì F8 vẫn tới đó; chỉ khi không còn gì để tới thì câu báo trung thực thay cho `no_next`/`no_marks` — EXPERIENCE: *"Dấu vắng không phải lời khẳng định câu đạt quy ước"*.
- **Phép đếm IPC trong e2e** bọc `__TAURI_INTERNALS__.invoke` **từ spec**, 0 dòng mã sản phẩm mới, nên nó không mở cửa mà `flushWait.mjs` đã đóng; ca đỏ in nguyên tên các lệnh thấy được trong lúc gõ.
- **"Đã nhận" của hàng mẻ là `'received'`.** `AiTranslateBatchState` có `'done'` nhưng đó là trạng thái của **cả mẻ**; trạng thái hàng không có `'done'`. Và `batchRowTexts` trả câu **nguồn** khi hàng chưa có bản dịch, nên vẽ dấu theo một trạng thái sai là đánh dấu quy ước lên câu nguồn — một lỗi im lặng và trông như thật.
- **Một lời gọi cho cả mẻ.** Nhịp quét bám trạng thái của cả mẻ, không bám từng hàng: 50 hàng `'received'` lẻ tẻ sẽ là 50 lời gọi IPC, đúng cái hình mà AD-67 Prevents 5 đóng.
- **Chưa cần số đo mới:** 16.1 đã đo một câu p95 0,005 ms (bench-release, 2.000 luật), dưới trần 1 ms của AD-67 mục 11; đường flush quét đúng một câu nên trần đó đã có bằng chứng.
- **Phases** (AGENTS.md): Rust · Webview · Tests that move (vitest + e2e), một agent mới mỗi phase, giao nhau qua tệp handoff cạnh plan này.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --manifest-path src-tauri/Cargo.toml --test convention_contract --test convention_boundary` — expected: xanh.
- `npm run test:story 16-4` — expected: xanh.
- full `cargo test` + vitest một lượt (`lib.rs` và `config_invariants.rs` đổi) — expected: xanh, đọc cả mã thoát và dòng `Errors`.
- `npm run check:commands` và `npm run check:i18n` — expected: xanh, `POINTER_DEBT` rỗng.
- `npm run test:e2e -- --spec e2e/specs/story-16-4-convention-rescan-after-flush.e2e.mjs` (ngoài sandbox, máy đã mở khoá) — expected: xanh trên WKWebView.
- Đọc lượt CI (nửa Windows và lượt UTC) và lượt nightly e2e gần nhất — expected: xanh, đỏ thì ghi lý do.

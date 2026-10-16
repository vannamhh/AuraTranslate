---
title: 'Cài đặt: bố cục, phím tắt, thử kết nối, khung Cập nhật'
type: 'feature'
ticket: '18'
created: '2026-10-16'
status: 'built'
baseline_revision: 'e5ac8e64cf1d2eab5396481a51d9076989e4083c'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/settings.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Cài đặt chỉ có bảy mục (`settingsState.ts:20`); Bố cục, Chế độ đọc, Dữ liệu, Cập nhật không có thân; mục AI dùng ô văn bản cho nhà cung cấp, nút `Lưu` từng trường, radio tầng nằm trong mục, không có Thử kết nối (FR143); màn Phím tắt không tìm, không tra ngược, không xuất/nhập, chỉ trả mặc định từng hàng (FR144). Dòng a:197, a:206, a:208; A21–A26 (A24 chỉ khung).

**Approach:** Thêm bốn mục nav theo thứ tự mockup (…Phím tắt · Bố cục · Chế độ đọc · Dữ liệu · Cập nhật, giữ `privacy` cuối). Mục AI theo `settings.html` khung 1: thanh Phạm vi ở đầu cửa sổ, chuỗi nút nhà cung cấp, lưu khi rời ô, Thử kết nối là lệnh Rust mới qua `TranslationProvider` (AD-15 điểm 1, AD-13, AD-29). Màn Phím tắt theo khung 2: một ô vừa tìm theo tên vừa tra ngược theo tổ hợp, bộ đếm, Xuất/Nhập qua hộp thoại Rust (AD-48), Trả về mặc định. Ba pha Rust · Webview · Tests, mỗi pha một agent mới, bàn giao qua `handoff-cai-at-bo-cuc-phim-tat-thu-ket-noi-khung-cap-nhat-phases.md`.

**Decisions (agent, từ nguồn; Ice duyệt cùng plan):**
- Thanh Phạm vi (a:197) thay radio "Tầng đang sửa"; ở mục không có tầng Tác phẩm (Phím tắt, Bố cục, Chế độ đọc, Dữ liệu, Cập nhật) thanh khoá ở Toàn cục kèm câu lý do của mockup. "N mục đang ghi đè" đếm trường AI có `tier === 'work'`; lệnh mới `settings.ai_config.clear_all_overrides` (`Mod+Shift+R`, đang trống) và nút "Trả toàn bộ mục về kế thừa" gọi cùng lệnh.
- Chọn nhà cung cấp (a:206) ghi đúng trường `provider` (`anthropic|openai|ollama|lmstudio|custom`); giá trị cũ ngoài danh sách hiện là "Endpoint khác". Không tự ghi `endpoint`: địa chỉ mặc định của nhà cung cấp chỉ là placeholder (một lượt ghi chỉ chạm cột người dùng đồng ý đổi).
- Thử kết nối dùng cấu hình ĐÃ LƯU của tầng đang xem (rời ô đã lưu trước khi bấm). 401/403 → khoá bị từ chối; 404 → endpoint hoặc mô hình không có; lỗi mạng → không tới được máy chủ; chưa cấu hình → lời mời, không phải lỗi (UX-DR27). Mỗi lỗi là message key riêng, có thể huỷ.
- Tệp phím tắt là JSON có phiên bản, chỉ chứa các ghi đè (`{ "format": "auratranslate-shortcuts", "version": 1, "bindings": { id: [chord…] } }`, `[]` = bỏ gán). Nhập THAY TOÀN BỘ tập ghi đè; id lạ bị bỏ và đếm; chord sai hoặc va nhau sau khi áp ⇒ từ chối cả tệp, không ghi gì. Trả về mặc định hỏi một bước rồi xoá mọi hàng `shortcut`.
- Ô tra ngược: phím in được không kèm Mod/Ctrl/Alt là chữ để tìm tên; tổ hợp có Mod/Ctrl/Alt hoặc phím không in được (trừ Tab, Esc, Backspace, Enter) là chord, lọc ra lệnh đang gán chord đó qua `resolveChord`.
- Khung Cập nhật (A24): câu "chỉ kiểm tra và báo…" và ghi chú "Vì sao không có nút Cập nhật ngay" của mockup; không công tắc tự kiểm khi khởi động (AD-15), không nút chưa có lệnh, không dòng phiên bản (`capabilities/main.json` không có quyền đọc phiên bản; dòng trạng thái là việc của 10.7).
- Dữ liệu (A23) chỉ dùng lệnh đã có: thư mục Library (`chooseLibraryRoot`), dựng lại chỉ mục kèm thời điểm (`rebuildLibraryIndex`, `readLibraryRebuiltAt`), mở báo cáo quét lại đã có.
- Công tắc token (A25) là một khoá `app_config` Toàn cục; mockup vẽ "kế thừa" nhưng không dựng tầng Tác phẩm cho nó (cần `ScopeKind` mới, AD-18).

**Decisions (Ice, 2026-10-16):**
- Thử kết nối gọi `GET {endpoint}/models` (0 token) rồi kiểm mô hình đã lưu có trong danh sách; thiếu ⇒ lỗi "mô hình không có ở endpoint này". Endpoint không có `/models` báo lỗi là chấp nhận.
- Không đòi khoá khi `provider` là `ollama` hoặc `lmstudio` (Rust quyết trong `call_ready`/`read_api_key`); câu chưa cấu hình của panel AI theo mockup; đóng nợ dòng 13206.
- Khoá API giữ nút `Lưu khoá`/`Xoá khoá` riêng, lệch có chủ đích với a:208; mọi trường khác lưu khi rời ô.
- Dựng công tắc "Hiện token và ước tính chi phí", mặc định bật, chỉ ẩn/hiện dòng token ở panel Đề xuất AI.
- Bố cục: thêm ("Lưu bố cục hiện tại thành preset" có tên), đổi tên, xoá, áp bằng bấm preset người dùng qua `ScopeKind::LayoutPreset`; preset người dùng không có phím (lệch `⌥3`).
- Chế độ đọc: bốn tuỳ chọn đã có (mức, song ngữ, cỡ chữ, giãn dòng), dùng chung state và lịch ghi của `readingState.ts`.
- `⌘T` (`Mod+T`, đang trống) là lệnh mới `settings.ai_config.open`, mở Cài đặt ở mục AI và mô hình.
- Giữ một story, ba pha.

## Boundaries & Constraints

**Always:** cấu hình AI ở Rust quyết (AD-1), webview không chép logic `call_ready`; khoá không qua IPC (AD-29); mọi lượt ghi qua `putConfig`/`deleteConfig`; `@click` là đúng một `dispatch`; chữ hiển thị chỉ từ `vi.json`; đóng Cài đặt trả tiêu điểm về `Công cụ ▾` (UX-DR17).

**Never:** không nội dung Cập nhật của Story 10.7 (lệnh kiểm bản mới, checksum, mở trang phát hành); không công tắc kiểm khi khởi động; không xuất sao lưu (FR102); không mục "Từ điển và nguồn" (loại 3, D2/D12); không cột "Ghi chú" ở bảng phím (đã gỡ cố ý); không quyền mới trong `capabilities/main.json`, không nới CSP; không import `shortcutsState` hay `vue` vào `src/commands/index.ts`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error Handling |
|---|---|---|---|
| Tra ngược | gõ `⌘M` vào ô tìm | chỉ hàng `editor.merge_segments` | chord không gán ⇒ "không lệnh nào dùng ⌘M" |
| Xuất–đổi–nhập | xuất, gán lại một lệnh, nhập tệp vừa xuất | phím cũ trở về, ghi đè khác giữ đúng tệp | huỷ hộp thoại ⇒ không ghi gì |
| Nhập tệp hỏng | JSON sai, `format` khác, chord sai hoặc va nhau | không hàng `shortcut` nào đổi | lỗi tường minh nêu lý do |
| Nhập id lạ | tệp có id không còn trong registry | phần còn lại áp | "bỏ qua N lệnh không còn" |
| Thử kết nối khoá sai | provider trả 401 | lỗi "khoá bị từ chối" | không retry tự động |
| Thử kết nối thiếu cấu hình | endpoint hoặc mô hình trống | lời mời cấu hình | không gọi mạng |
| Mạng chết | endpoint không tới được | "không tới được máy chủ" | huỷ được giữa chừng |

</frozen-after-approval>

## Code Map

- `src/SettingsOverlay.vue` (819 dòng) -- nav `v-for SETTINGS_SECTIONS` 255–272; chuỗi `v-if` thân mục 275–541; mục AI 277–444 (radio tầng ~288–310, `v-for AI_CONFIG_FIELDS` ~312, nút `Lưu` ~356, khoá ~374–440). Thân mục mới tách thành `Settings<Tên>Section.vue` theo khuôn `SettingsTmSection.vue`.
- `src/settingsState.ts` -- `SettingsSection`, `SETTINGS_SECTIONS` 20–30, `settingsSectionLabelKey` 33; ⚠️ có byte NUL ở dòng 99, `grep -a`.
- `src/aiConfigState.ts` -- `AI_CONFIG_FIELDS`, `aiConfigFieldWire`, `aiConfigReadiness`, `refreshAiConfigReadiness` 229; `src/config/aiconfig.ts` -- adapter `aiConfigGet` 92, `SaveField` 120, `ClearOverride` 140, `SaveKey` 164; adapter Thử kết nối mới đặt ở đây.
- `src-tauri/src/commands/aiconfig.rs` -- hàm thuần 169–268, `mod wire` 289; lệnh Thử kết nối mới theo hai lớp (hàm thuần nhận `Option<&Store>` + `#[tauri::command]` trong `wire`). `commands/aitranslate.rs` -- `resolve_call_config` 246, `read_api_key` ~320, `prepare_translate_call` 352, ánh xạ lỗi 118–170 (mọi non-2xx hiện là `provider_refused`).
- `src-tauri/src/ports/translation_provider.rs:138` -- trait; thêm phương thức kiểm kết nối. `src-tauri/src/core/ai/client.rs` -- `OpenAiChatClient` (reqwest 0.13.4, Bearer 339–390), `OpenAiClientError::NonSuccessStatus` 97. Chỉ `core/ai` gọi mạng (`tests/ai_boundary.rs`).
- `src-tauri/src/lib.rs:1118–1448` -- `generate_handler!`; `tests/ipc_contract.rs` (vd `the_aiconfig_key_wires_are_registered` 2224) liệt tên wire; `core/i18n/mod.rs:100` `message_keys!` (vd `provider_unreachable` 852).
- `src-tauri/src/commands/tm.rs:1232` `blocking_save_file`, `:1284` `blocking_pick_file` -- khuôn hộp thoại cho Xuất/Nhập bộ phím.
- `src/commands/index.ts` -- `registerAll()` 1337; `editor.merge_segments` = `Mod+M` 3584; `settings.open` 4553; `shortcuts.*` 5170–5173; API `effectiveBindings`, `defaultChordsFor`, `currentOverrides`, `conflictFor`, `applyBindings` 5213–5390. `src/commands/keys.ts` -- `chordFromEvent` 257, `formatChord` 328, `resolveChord` 368, `createKeymap` 461 (ném khi chord trùng).
- `src/config/shortcutsState.ts` -- `shortcutRows` 131, `captureShortcut` 293, `resetShortcut` 329, `handleCaptureKey` 352; `src/SettingsShortcutsSection.vue` (374 dòng). Lưu: `SCOPE_SHORTCUT` (`src/config/bootstrap.ts:196`), `putConfig` 335, `deleteConfig` 377; hàng thiếu = mặc định, `''` = bỏ gán.
- `src/layout/workspaceLayout.ts` -- `LAYOUT_PRESETS` 158, `presetById` 171; `layoutPresetState.ts`; `WorkspaceDock.vue:292` áp preset; `BootstrapConfig.layout_presets` (`bootstrap.ts:53`) là preset có tên, chưa có UI.
- `src/modes/readingState.ts` -- `StoredReadingPreferences` ~744, lịch ghi `readingPreferencesSchedule`, `putConfig(SCOPE_APP_CONFIG, KEY_READING_PREFERENCES)` 795.
- `src/config/library.ts` -- `chooseLibraryRoot` 458, `rebuildLibraryIndex` 468, `readLibraryRebuiltAt` 473.
- `src/i18n/vi.json` -- `settings.nav.*` 1861–1869, `settings.ai_config.*` ~1429–1442.
- `scripts/check-commands.mjs` -- `COMMAND_FLOOR=250`, `CLICK_FLOOR=228`, `DISPATCH_FLOOR=322`, Check K `HANDLER_TABLE` cần hàng cho mỗi `@keydown` mới; `check:panel-refs` mỗi ref cấp module phải xoá trong `reset*()`.
- Test sẽ đổi: `tests/frontend/settingsFrame.test.ts:63` ghim `SETTINGS_SECTIONS`; `settingsOverlayAiConfig*Render.test.ts`; e2e `shortcuts-focus.e2e.mjs`, `shortcuts-capture-mouse.e2e.mjs`. Rust: `aiconfig_contract.rs`, `ai_boundary.rs`, `ipc_contract.rs`.
- Tests that move: `src-tauri/tests/connection_check_contract.rs` · `src-tauri/tests/aiconfig_contract.rs` · `src-tauri/tests/config_invariants.rs` · `src-tauri/tests/ipc_contract.rs` · `tests/frontend/aiBatchPanelRows.test.ts` · `tests/frontend/aiConfigState.test.ts` · `tests/frontend/aiTranslate.test.ts` · `tests/frontend/importPreviewTwoColumn.test.ts` · `tests/frontend/settingsOverlayAiConfigTierRender.test.ts` · `tests/frontend/settingsState.test.ts` · `tests/frontend/settingsFrame.test.ts` · `tests/frontend/aiConfigConnectionTest.test.ts` · `tests/frontend/aiUsageVisibleState.test.ts` · `tests/frontend/layoutUserPresets.test.ts` · `tests/frontend/settingsReadingDataBodies.test.ts` · `tests/frontend/shortcutsSearchExchange.test.ts` · `e2e/specs/story-12-18-settings.e2e.mjs`.

## Tasks & Acceptance

**Execution:**

Phase Rust:
- [x] `src-tauri/src/ports/translation_provider.rs`, `src-tauri/src/core/ai/client.rs` -- phương thức kiểm kết nối `GET /models` + kiểm mô hình, huỷ được, lỗi phân 401/403 · 404 · mạng · khác -- FR143 đòi lỗi tường minh.
- [x] `src-tauri/src/commands/aiconfig.rs`, `lib.rs`, `core/i18n/mod.rs` -- lệnh `ai_config_test_connection` (hàm thuần + wire), message key mới, đăng ký -- AD-15 điểm 1, AD-29.
- [x] `src-tauri/src/commands/` (tệp phím tắt mới hoặc `config.rs`) -- `shortcuts_export(content)` qua hộp thoại lưu, `shortcuts_import()` trả nội dung tệp qua hộp thoại mở -- AD-48.
- [x] `src-tauri/src/commands/aitranslate.rs` -- `call_ready`/`read_api_key` không đòi khoá cho `ollama`/`lmstudio`; `aiconfig_contract.rs::ready_is_true_false_or_none_by_endpoint_model_and_key` thêm hàng tương ứng.

Phase Webview:
- [x] `src/settingsState.ts`, `src/SettingsOverlay.vue`, `src/i18n/vi.json` -- bốn mục nav mới, thanh Phạm vi đầu cửa sổ, chuỗi nút nhà cung cấp, lưu khi rời ô, Thử kết nối, công tắc token, `settings.ai_config.clear_all_overrides` -- a:197, a:206, a:208, A25.
- [x] `src/panels/AiTranslationPanel.vue`, `vi.json` -- dòng token theo công tắc; câu chưa cấu hình cho `ollama`/`lmstudio` không còn đòi khoá.
- [x] `src/config/aiconfig.ts`, `src/config/shortcuts*.ts` -- adapter cho ba lệnh Rust mới, không ném.
- [x] `src/SettingsShortcutsSection.vue`, `src/config/shortcutsState.ts` -- ô tìm/tra ngược, bộ đếm, Xuất, Nhập (kiểm tệp, thay toàn bộ ghi đè), Trả về mặc định hỏi một bước -- FR144.
- [x] `src/SettingsLayoutSection.vue`, `SettingsReadingSection.vue`, `SettingsDataSection.vue`, `SettingsUpdateSection.vue` -- thân mục theo Decisions.
- [x] `src/commands/index.ts`, `src/main.ts`, `scripts/check-commands.mjs` -- đăng ký `settings.ai_config.clear_all_overrides` (`Mod+Shift+R`) và `settings.ai_config.open` (`Mod+T`), nâng sàn đúng số lệnh/`@click` thêm.

Phase Tests:
- [x] Rust: `aiconfig_contract.rs` (mỗi hàng lỗi của ma trận qua máy chủ HTTP giả cục bộ), `ipc_contract.rs` (tên wire mới).
- [x] Vitest: tra ngược `Mod+M` → `editor.merge_segments`; xuất–đổi–nhập trả phím cũ; tệp hỏng không ghi gì; id lạ bị đếm; `settingsFrame.test.ts` theo danh sách mục mới.
- [x] e2e `e2e/specs/story-12-18-settings.e2e.mjs`: mở Cài đặt từ `Công cụ ▾`, đi qua bốn mục mới, tra ngược một chord; sửa hai spec e2e phím tắt nếu bộ chọn đổi.
- [x] `deferred-work.md` -- đóng nợ dòng 13206 (`→ ✅ ĐÃ ĐÓNG`); mục `Chủ: Epic 12`: Ice bấm Thử kết nối với khoá sai và khoá đúng trên nhà cung cấp thật.

**Acceptance Criteria:**
- Given Cài đặt mở, when bấm lần lượt Bố cục, Chế độ đọc, Dữ liệu, Cập nhật, then mỗi mục có thân theo Decisions và các câu trả lời, mục Cập nhật không có nút nào.
- Given đang xem tầng Tác phẩm với 2 trường ghi đè, when bấm `⌘⇧R`, then thanh Phạm vi ghi 0 mục đang ghi đè và hai trường hiện "kế thừa".
- Given đã sửa một trường AI, when rời ô, then giá trị đã lưu mà không có nút Lưu nào trong mục.
- Given `provider` là `ollama`, endpoint và mô hình đã lưu, không có khoá, when mở panel Đề xuất AI, then panel ở trạng thái sẵn sàng.
- Given đang ở Workspace, when bấm `⌘T`, then Cài đặt mở ở mục AI và mô hình; đóng thì tiêu điểm về chỗ cũ.
- Given tắt công tắc token, when một lượt dịch AI xong, then panel không hiện dòng token; bật lại thì hiện.
- Given đã lưu một preset người dùng, when đổi tên, áp, rồi xoá nó, then danh sách Bố cục phản ánh từng bước và còn sau khi mở lại app.
- Given chọn "Ollama" khi endpoint trống, then trường `provider` đổi, endpoint vẫn trống và hiện placeholder `http://localhost:11434/v1`.

## Implementation Notes

- Thử kết nối là trait `ConnectionCheck` riêng, không thêm phương thức vào `TranslationProvider`: sáu fake của trait cũ sẽ phải sửa theo. Chỉ `core/ai` gọi mạng; `ai_boundary` xanh.
- Thử kết nối có `ConnectionCheckGeneration` và `ai_config_test_connection_cancel` riêng. Thời hạn 10 s và trần 4 MiB của danh sách mô hình là số ban đầu, chưa đo.
- Một hàm dựng URL (`endpoint_base` trong `core/ai/client.rs`) dùng chung cho dịch, lô, gửi lại, soát và `/models` (xem Plan Change Log).
- Công tắc token cần đọc lúc khởi động nên `BootstrapConfig` thêm `ai_usage_visible`; pha Webview sửa Rust ngoài pha Rust, `ipc_contract` cập nhật.
- Khi Cài đặt mở, `KeymapGate.isBlocked` nhận sự kiện và chỉ cho qua lệnh Cài đặt sở hữu (`⌘T`, `⌘⇧R`), không cho qua trong ô tìm phím.
- Nhập và Trả về mặc định xoá mọi hàng `shortcut` có trên đĩa lúc khởi động (`bootstrapShortcutRowIds`), nên một bộ phím đĩa bị từ chối cũng được dọn.
- Áp hoặc lưu preset bố cục cần dock Workspace đang sống và bị khoá trong Review Mode; JSON sai hình báo lỗi, bố cục giữ nguyên.
- Mục Dữ liệu mở báo cáo quét lại qua lệnh mới `settings.data.open_report` (đóng Cài đặt, sang Library).
- Click WebDriver không làm ô nhập trong WKWebView nhận tiêu điểm (ô Endpoint cũng vậy), nên e2e gọi `.focus()`; bấm chuột thật nằm trong mục nợ `Chủ: Epic 12`.

## Plan Change Log

- 2026-10-16, sau Phase Rust: lượt dịch POST thẳng vào endpoint đã lưu, còn mockup và placeholder dùng địa chỉ gốc `…/v1`. Ice chọn nhận cả địa chỉ gốc: một hàm dựng URL dùng chung cho dịch và Thử kết nối, nối `/chat/completions` khi thiếu; endpoint dạng đầy đủ đã lưu chạy như cũ. Cùng lượt: Thử kết nối có bộ đếm và lệnh huỷ riêng, vì dùng chung `AiTranslateGeneration` làm bấm Thử kết nối huỷ lượt dịch đang chạy.

## Review Triage Log

Lượt 1 (quick, 2026-10-16): high 0 · medium 5 · low 3 · false 2 · maybe-false 0; bị bác 3.
- medium · patch — `⌘⇧R` xoá mọi ghi đè Tác phẩm từ bất kỳ mục Cài đặt nào (`settingsOwnsChord` cho qua, `clearAllAiConfigOverrides` không xét mục/tầng xem); chỉ chạy ở `ai_and_model` + tầng `work`.
- medium · patch — xoá trắng một trường AI rồi rời ô: không lưu, không báo, ô trống mà giá trị đã lưu còn nguyên (`commitAiConfigField` trả sớm khi rỗng); trả nháp về giá trị đã lưu.
- medium · patch — đóng Cài đặt khi nháp trường AI còn bẩn dựa vào `blur`, không chắc bắn khi ô bị gỡ; xả nháp lúc đóng.
- medium · patch — `writeOverrideDiff` chỉ xoá id trong `currentOverrides()`; bộ phím đĩa bị từ chối lúc khởi động (`installCommands` rơi về mặc định) sống qua Trả về mặc định và Nhập, nên lần mở sau vẫn bị từ chối; xoá mọi hàng `shortcut` đã biết trên đĩa.
- medium · patch — mục Dữ liệu thiếu lối mở báo cáo quét lại mà Decisions ghi; chỉ có câu trỏ sang Thư viện.
- low · patch — Huỷ bấm trong lúc chờ `inFlightSave` tăng bộ đếm Rust trước khi lệnh chạy nên lượt kiểm vẫn chạy.
- low · patch — doc comment của `openSettingsToPrivacy` nằm trên `openSettingsToAiConfig` (`settingsState.ts`).
- low · patch — chú thích trái AGENTS.md trong dòng mới (banner, ngày và người quyết, mã FR/A, tiếng Việt kể lại dòng kế) và thụt lề lệch ở `WorkspaceDock.vue` `onActivated`.
- low · bác — `settings.update.lead` "chỉ kiểm tra và báo có bản mới" khi chưa có lệnh kiểm: câu nằm trong Decisions đã duyệt của khung Cập nhật; nêu lại cho Ice ở lúc trình.
- low · bác — Nhập gọi `commitBindings` trước khi ghi, ngược Design Notes: ghi trượt giữa chừng để đĩa lẫn cũ/mới ở cả hai thứ tự, đảo thứ tự không chữa được hậu quả nêu ra.
- false — trait `ConnectionCheck` riêng thay vì phương thức trên `TranslationProvider`: AD-13/AD-15 vẫn giữ (chỉ `core/ai` gọi mạng, `ai_boundary` xanh), không tác hại nêu được.
- false — "e2e 4/4 xanh trong sổ nợ là suy diễn": pha Tests chạy thật ngoài sandbox và chạy lại sau khi đổi về `⌘M`; sẽ chạy lại một lần sau vá.
- bác — kế hoạch chưa đánh dấu task và Implementation Notes: cách sửa là sửa chính kế hoạch.

## Design Notes

Ô tra ngược đặt `chordFromEvent` trước khi trình duyệt chèn ký tự; khi sự kiện là chord, `preventDefault` và lọc theo `resolveChord(chord)` trên `effectiveBindings()`, không qua `handleCaptureKey` (hàm đó ghi). Nhập bộ phím dựng keymap thử bằng `createKeymap` trên tập hợp nhất trước mọi `putConfig`; ném ⇒ từ chối cả tệp. Ghi: xoá các hàng `shortcut` không còn trong tệp, `putConfig` các hàng trong tệp, rồi `applyBindings`.

## Verification

**Commands:**
- `npm run test:story 12-18` -- expected: mọi test chọn được xanh.
- `cargo test --test aiconfig_contract` và `--test ipc_contract` (sau `npm run build`) -- expected: xanh.
- `npm run check:commands` và `npm run check:i18n` -- expected: xanh với sàn đã nâng.

**Manual checks:**
- Ice: Thử kết nối với khoá sai và khoá đúng trên một nhà cung cấp thật (e2e không giả được provider) -- nợ `Chủ: Epic 12`.

# Handoff: Cài đặt: bố cục, phím tắt, thử kết nối, khung Cập nhật

## Pha 1 (Rust)

Trạng thái: Rust xong, chưa commit. Một test cố ý còn đỏ cho pha Webview (xem cuối).

### Wire (Rust -> webview)

Bốn lệnh mới, đã đăng ký ở `lib.rs`:

- `ai_config_test_connection(tier)`: `tier` = `"global" | "work"` (cùng kiểu `AiConfigTier` của `ai_config_save_field`). Async. Dùng cấu hình ĐÃ LƯU của tầng đó (Global đọc `global.db`; Work phân giải hai tầng, cần Tác phẩm đang mở). Trả (tag `kind`, snake_case):
  - `{ kind: "connected", model_count: number }`
  - `{ kind: "not_configured" }`: endpoint hoặc mô hình trống, hoặc nhà cung cấp cần khoá mà chưa có khoá. Lời mời cấu hình, KHÔNG phải lỗi, không gọi mạng.
  - `{ kind: "cancelled" }`
- `shortcuts_export(content: string)`: mở hộp thoại LƯU (tên gợi ý `auratranslate-shortcuts.json`, lọc `json`), ghi `content` nguyên văn qua tệp tạm + rename. Trả `string | null`: đường dẫn đã ghi, `null` khi huỷ hộp thoại (không ghi gì).
- `shortcuts_import()`: mở hộp thoại MỞ (lọc `json`), trả `string | null`: nội dung UTF-8 của tệp, `null` khi huỷ. Rust KHÔNG kiểm JSON/`format`/chord: webview kiểm hết trước mọi `putConfig`.

- `ai_config_test_connection_cancel()`: không tham số, không trả gì. Huỷ CHỈ lượt Thử kết nối đang chạy (bộ đếm riêng `ConnectionCheckGeneration`); lượt bị huỷ trả `{ kind: "cancelled" }`. Bắt đầu hoặc huỷ Thử kết nối KHÔNG đụng lượt dịch / lô đang chạy (`ai_translate_cancel` vẫn là của dịch). Huỷ được quan sát ở ranh giới các pha của lệnh, lượt còn bị chặn trần 10 giây. Webview dùng lệnh này, không dùng `ai_translate_cancel`, cho nút huỷ Thử kết nối.

Wire IPC (camelCase theo `ipc_argument_contract`): đối số `tier` và `content` không đổi tên. Không lệnh nào còn tham số khác.

### Lỗi (IpcError, `message_key` đã vào `vi.json` dưới `err.*`)

Thử kết nối (webview hiện câu lấy qua `message_key`, không tự dựng):

| Tình huống | code | message_key | tham số | retryable |
|---|---|---|---|---|
| 401 / 403 | `ai_config.test_key_rejected` | `ai_config.test_key_rejected` | | false |
| 404 | `ai_config.test_endpoint_not_found` | `ai_config.test_endpoint_not_found` | | false |
| mô hình không có trong danh sách | `ai_config.test_model_missing` | `ai_config.test_model_missing` | `model` | false |
| lỗi mạng / hết 10 giây | `ai_config.test_unreachable` | `ai_config.test_unreachable` | | true |
| non-2xx khác | `ai_config.test_refused` | `ai_config.test_refused` | `status` | false |
| 2xx nhưng không phải danh sách mô hình | `ai_config.test_reply_unreadable` | `ai_config.test_reply_unreadable` | | false |
| dựng client / khoá không vào header | họ `ai_translate.*` sẵn có | `ai_translate.client_build_failed`, `ai_translate.api_key_header_invalid` | | false |
| Tầng Tác phẩm khi chưa mở Tác phẩm | `ai_config.work_tier_unavailable` | sẵn có | | |
| Keychain không trả lời | sẵn có | `ai_config.keychain_unavailable` | | true |

Tệp phím tắt: `shortcuts.file_too_large` (quá 1 MiB), `shortcuts.file_unreadable` (không mở được / không UTF-8), `shortcuts.file_write_failed` (retryable). Mỗi khoá có `err.shortcuts.*` trong `vi.json`.

### Hành vi Rust cần biết khi dựng UI

- Endpoint chấp nhận URL gốc (Ice chốt). Một bộ dựng URL dùng chung (`core::ai::client::{chat_completions_url, models_url}`, cùng một `endpoint_base`): lượt dịch / lô / gửi lại / Proofread POST tới `<gốc>/chat/completions`, Thử kết nối GET `<gốc>/models`. Endpoint đã lưu đuôi `/chat/completions` (kể cả có `/` cuối) giữ nguyên, URL gốc có hoặc không `/` cuối được nối thêm đuôi. Webview không dựng URL. Mô hình khớp khi id bằng đúng mô hình đã lưu hoặc `<mô hình>:latest`. Khoá chỉ gửi cho nhà cung cấp cần khoá.
- `ready` của `ai_config_get` nay tính theo nhà cung cấp: `provider` là `ollama` hoặc `lmstudio` (so không phân biệt hoa thường) ⇒ `ready = true` khi endpoint và mô hình đã có, bất kể khoá và bất kể keychain. `key_configured` không đổi nghĩa. Mọi nhà cung cấp khác (kể cả `custom`, `openai`, `anthropic`, trống) vẫn cần khoá. Lượt dịch, lô, gửi lại và Proofread cũng không đòi khoá cho hai nhà cung cấp cục bộ và không gửi header `Authorization`. Webview KHÔNG chép logic này: dùng `ready`. Panel AI chỉ cần bỏ câu đòi khoá khi `ready === true` (câu `panel.ai_translation.unconfigured_local` theo Quyết định Ice).
- `provider` không bị Rust giới hạn danh sách (chuỗi tự do, `validate_non_empty`). Danh sách `anthropic|openai|ollama|lmstudio|custom` và "Endpoint khác" cho giá trị lạ là việc của webview.

### Chữ ký Rust thay đổi

- `call_ready(provider, endpoint, model, key_configured)`, `read_api_key(provider)`, `ResolvedCallConfig.provider` (đều `pub(crate)`). `proofread.rs` đã theo.
- `ConnectionCheckGeneration` (state mới, `app.manage` ở `lib.rs`) cạnh `AiTranslateGeneration`; `AiTranslateGeneration::is_current` nay `pub`.
- Cổng mới `ports::translation_provider::ConnectionCheck` (trait riêng, cài đặt bởi `OpenAiChatClient`), KHÔNG thêm phương thức vào `TranslationProvider` để sáu bản giả trong `tests/` không phải sửa. Seam test: `commands::aitranslate::run_connection_check(provider, &PreparedConnectionCheck, should_cancel)`, `core::ai::client::{models_url, check_model_list}`.
- `commands/shortcuts.rs` mới: `shortcuts_write_file`, `shortcuts_read_file` (hàm thuần) + `wire::{shortcuts_export, shortcuts_import}`.

### Đo và test

- Chạy xanh: `cargo clippy --all-targets -- -D warnings`, `cargo fmt`, `cargo test --test aiconfig_contract` (24, thêm các hàng Ollama / LM Studio / Anthropic / custom vào `ready_is_true_false_or_none_by_endpoint_model_and_key`), `--test connection_check_contract` (7 ca mới: URL chat + models cho gốc có / không `/` cuối và URL đầy đủ, bộ đếm riêng không đụng bộ đếm dịch khi bắt đầu / huỷ Thử kết nối, danh sách mô hình, sáu họ lỗi có khoá riêng, connected / cancelled là kết quả, vòng tròn tệp phím tắt + tệp quá lớn / không UTF-8 / thiếu).
- Cũng xanh: `config_invariants` (bảng `COMMAND_FILE_CENSUS`: aiconfig.rs 2->3 plain, hàng mới `shortcuts.rs` 0/2/2 cùng hai hàng `blocking_wire_cases`, tổng 101/63 -> 103/65, aiconfig.rs plain 2->4), `ipc_contract`, `naming_boundary`, mọi `*_boundary` + `boundary_scan_contract`, `ai_boundary`, `ai_translate_contract`, `ai_proofread_contract`, lib unit; `check:i18n`, `check:commands`, `check:doc-refs`, `cargo clippy -D warnings`, `cargo fmt --check`.
- CHƯA đo: máy chủ HTTP thật hoặc giả (ma trận 401 / 404 / mạng chết / huỷ giữa chừng qua máy chủ cục bộ là việc của pha Tests), nhà cung cấp thật, hộp thoại Xuất / Nhập (hộp thoại gốc, không test được ở đây).
- Trần 10 giây (`CONNECTION_CHECK_TIMEOUT`) và trần danh sách mô hình 4 MiB là số ban đầu, chưa đo.

### Còn ĐỎ cho pha Webview

- `ipc_argument_contract::every_registered_commands_business_parameters_match_camel_case_across_the_wire` đỏ với `ai_config_test_connection`, `ai_config_test_connection_cancel`, `shortcuts_export`, `shortcuts_import` cho tới khi `src/**/*.ts` có `invoke('<tên>', ...)` của từng lệnh (xem dưới). Xanh khi đủ ba adapter.

## Pha 2 (Webview)

Trạng thái: Webview xong, chưa commit. `vue-tsc`, `npm run build`, `check:lint/commands/i18n/panel-refs/layout/tokens/doc-refs/gates` xanh. Vitest CHƯA sửa (xem "Test đang đỏ").

### Đã đổi

- Một thay đổi Rust nhỏ ngoài pha Rust (công tắc token cần đường đọc lúc khởi động, `BootstrapConfig` là cửa duy nhất): `core/scope/store.rs` (`KEY_AI_USAGE_VISIBLE` + `ai_usage_visible()`), `commands/config.rs` (trường `ai_usage_visible: String`, `"off"` = ẩn, rỗng = bật), `tests/ipc_contract.rs` (struct literal + danh sách khoá đóng băng). Đã chạy xanh: `ipc_contract`, `ipc_argument_contract` (hết đỏ nhờ ba adapter), `scope_contract`, `config_invariants`, `aiconfig_contract`, `connection_check_contract`. Quét của `ipc_argument_contract` coi mọi `CMD_X,` là lời gọi `invoke`: đừng truyền hằng `CMD_*` làm đối số đầu của hàm khác.
- `src/settingsState.ts`: 11 mục (`ai_and_model prompt glossary tm export shortcuts layout reading data update privacy`), `settingsSectionHasWorkTier`, `settingsSectionLocksScope`, `openSettingsToAiConfig`, `settingsOwnsChord`.
- `src/SettingsOverlay.vue`: thanh Phạm vi `[data-settings-scope]` đầu cửa sổ (radio tầng cũ bị gỡ), mục AI mới (chuỗi nút nhà cung cấp, lưu khi rời ô/Enter, khoá API giữ `Lưu khoá`/`Xoá khoá`, công tắc token, Thử kết nối + Huỷ, "Trả toàn bộ mục về kế thừa"), bốn thân mục mới.
- Mới: `SettingsLayoutSection.vue`, `SettingsReadingSection.vue`, `SettingsDataSection.vue`, `SettingsUpdateSection.vue`, `src/config/shortcuts.ts` (adapter `shortcutsExport`/`shortcutsImport`), `src/aiUsageVisibleState.ts`.
- `src/config/aiconfig.ts`: `aiConfigTestConnection(tier)` -> `{ outcome, error }`, `aiConfigTestConnectionCancel()`. `src/aiConfigState.ts`: `commitAiConfigField`, `selectAiConfigProvider`, `clearAllAiConfigOverrides`, `aiConfigOverrideCount`, `runAiConfigTest`/`cancelAiConfigTest`, `aiConfigTestStatus` (`idle|running|connected|not_configured|cancelled|error`), `AI_PROVIDERS`, `aiConfigEndpointPlaceholder`.
- `src/config/shortcutsState.ts`: `visibleShortcutRows`, `handleSearchKey`, `parseShortcutsFile`, `buildShortcutsFile`, `exportShortcuts`, `importShortcuts` (thay toàn bộ ghi đè; thử bằng `applyBindings` trước mọi `putConfig`), `askResetAllShortcuts`/`confirmResetAllShortcuts`, `shortcutDisplayFor`. `SettingsShortcutsSection.vue`: ô tìm, bộ đếm, Xuất/Nhập/Trả về mặc định.
- Lệnh mới `settings.ai_config.open` (`Mod+T`) và `settings.ai_config.clear_all_overrides` (`Mod+Shift+R`); `main.ts` nối dep. `KeymapGate.isBlocked` nay nhận `event` (`keys.ts`); khi Cài đặt mở, chỉ hai lệnh trên còn chạy (`settingsOwnsChord`), trừ khi tiêu điểm ở ô tìm phím tắt.
- Bố cục: `layoutPresetState.ts` (preset người dùng qua `layout_preset`: lưu/đổi tên/xoá/áp, `initUserLayoutPresets` gọi ở `main.ts`), `dockController.ts` + `WorkspaceDock.vue` thêm `currentLayoutJson()`/`applyLayoutJson()`. Áp/lưu cần dock đang sống (Workspace đang mở); ngoài đó nút khoá và có câu nói rõ.
- Panel AI: dòng token (lượt đơn, tổng lô, từng hàng) theo `aiUsageVisible`; câu `unconfigured_local` đã bỏ ý "vẫn cần khoá".
- `vi.json`: ~90 khoá mới; gỡ `settings.ai_config.tier_label`, `settings.ai_config.save`.

### Selector ổn định (cho Tests)

- Nav: `[data-settings-section="layout|reading|data|update|..."]`.
- Phạm vi: `[data-settings-scope]`, `[data-settings-scope-global]`, `[data-settings-scope-work]`, `[data-settings-scope-msg]`, `[data-settings-scope-overrides]`, `[data-settings-scope-shortcut-count]`.
- AI: `[data-ai-field="provider|endpoint|model|temperature|max_tokens"]`, `[data-ai-provider="anthropic|openai|ollama|lmstudio|custom"]` (`aria-pressed`), `[data-ai-usage-toggle]`, `[data-ai-test-connection]`, `[data-ai-test-cancel]`, `[data-ai-test-result="running|connected|not_configured|cancelled|error"]`, `[data-ai-clear-all]`, `.ai-key-field`, `.ai-field-input`.
- Phím tắt: `[data-shortcut-search]`, `[data-shortcut-search-chord]`, `[data-shortcut-count]`, `[data-shortcut-chord-empty]`, `[data-shortcut-export]`, `[data-shortcut-import]`, `[data-shortcut-reset-all]`, `[data-shortcut-reset-all-ask]`, `[data-shortcut-reset-all-confirm]`, `[data-shortcut-reset-all-cancel]`, `[data-shortcut-file-notice]`; hàng giữ `[data-command-id]`, ô phím giữ `[data-key-cell]`.
- Bố cục: `[data-settings-layout]`, `[data-layout-builtin="layout.preset_grid"]`, `[data-layout-user="<tên>"]`, `[data-layout-new-name]`, `[data-layout-save]`, `[data-layout-apply]`, `[data-layout-rename-input]`, `[data-layout-rename]`, `[data-layout-delete]`, `[data-layout-error]`, `[data-layout-no-workspace]`.
- Chế độ đọc: `[data-settings-reading-level="lg|md|sm"]`, `[data-settings-reading-bilingual]`, `[data-settings-reading-font-size]`, `[data-settings-reading-line-height]`. Dữ liệu: `[data-settings-data-root|choose|rescan|rebuild|rebuilt-at|error]`. Cập nhật: `[data-settings-update]` (0 nút).

### Sàn cổng đã nâng

`scripts/check-commands.mjs`: `COMMAND_FLOOR` 250 -> 267 (live 313), `TS_FLOOR` 113 -> 121 (live 142), `CLICK_FLOOR` 228 -> 236 (live 277), `DISPATCH_FLOOR` 322 -> 333 (live 391), `HANDLER_ATTR_FLOOR` 131 -> 146 (live 171); `scripts/check-panel-refs.mjs`: `FILE_FLOOR` 113 -> 121 (+ miễn trừ có tên `bootstrap.ts::aiUsageVisible`). Thêm 18 hàng `HANDLER_TABLE` và `R_FILTER_LOCAL`.

### Test đang đỏ (do hành vi đổi có chủ ý, Tests sửa)

- `settingsOverlayAiConfigTierRender.test.ts` (radio tầng -> `[data-settings-scope-*]`), `settingsFrame.test.ts:63` (danh sách mục, 11), `settingsOverlayAiConfigKeyRender.test.ts`, `settingsState.test.ts`, `settingsFrameDom.test.ts`: các ca timeout 5 giây ở ca đầu tiên của tệp vì đồ thị import của `SettingsOverlay.vue` lớn hơn (chạy riêng `settingsState.test.ts` xanh, 5,7 giây); nên import trước ở `beforeAll` hoặc tăng `testTimeout` của tệp, không hạ ngưỡng.
- e2e `shortcuts-focus.e2e.mjs`, `shortcuts-capture-mouse.e2e.mjs`: chưa đọc lại; bảng phím nay lọc theo `visibleShortcutRows` nhưng không đổi `[data-command-id]`/`[data-key-cell]`.

### Còn CHƯA test / rủi ro

- Chưa chạy bất kỳ tương tác thật nào: Thử kết nối với nhà cung cấp thật, hộp thoại Xuất/Nhập, áp preset người dùng lên dock thật, `Mod+T`/`Mod+Shift+R` trong app (cần e2e hoặc Ice).
- Ca Vitest cần có: tra ngược `Mod+M` -> `editor.merge_segments`; Xuất–đổi–nhập trả phím cũ; tệp hỏng/chord sai/va nhau không `putConfig` hàng nào; id lạ bị đếm (`shortcuts.import_done_skipped`); `settingsOwnsChord`; `commitAiConfigField` bỏ qua giá trị không đổi/không hợp lệ; `clearAllAiConfigOverrides`; vòng đời preset người dùng; `aiUsageVisible` ẩn/hiện 3 dòng token.
- Chưa có mục "mở báo cáo quét lại" trong Dữ liệu: câu `settings.data.report_note` chỉ trỏ tới màn Thư viện (không có lệnh mở thẳng báo cáo).
- 🔵 Đã sửa sau pha: `applyLayoutJson` (WorkspaceDock) dùng chính `fromJSON` + kiểm "có panel" làm bộ kiểm hình dạng duy nhất; hỏng thì dựng lại bố cục trước đó và trả `false`, Layout hiện `settings.layout.error_preset_broken` (`[data-layout-error]`). Review Mode mở: mọi nút áp/lưu khoá và hiện `[data-layout-review-open]` (`settings.layout.review_open`); preset tích hợp trả `false` cũng hiện `error_no_workspace`. Ca Vitest cần thêm: preset sai hình dạng không đổi bố cục; Review Mode khoá nút.

- 🔵 Sửa sau review: `⌘⇧R` chỉ chạy khi Cài đặt mở ở mục AI với tầng Tác phẩm (`clearAiOverridesFromSettings` trong `settingsState.ts`); blur ô rỗng trả ô về giá trị đã lưu; `closeSettings` gọi `flushAiConfigDrafts()`; Huỷ trong lúc chờ lượt lưu blur chỉ ghi nhận ở webview rồi bỏ invoke (trạng thái `cancelled`); Nhập và Trả về mặc định xoá mọi hàng `shortcut` trên đĩa lúc khởi động (`bootstrapShortcutRowIds`) cộng ghi đè đang sống nằm ngoài tập mới; Dữ liệu có lệnh `settings.data.open_report` (`[data-settings-data-report]`, đóng Cài đặt và mở Library); bỏ banner/ghi chú vi phạm quy tắc comment; sửa thụt lề `setDockController`. Sàn: `CLICK_FLOOR` 237, `DISPATCH_FLOOR` 334, `COMMAND_FLOOR` 267 (live 314). Thêm 3 ca vitest (`settingsState`, `aiConfigConnectionTest`, `shortcutsSearchExchange`).

## Pha 3 (Tests)

Trạng thái: Tests xong, chưa commit. Không sửa mã sản phẩm. Một chỗ sửa test ngoài dự kiến (xem "Test cũ đã sửa").

### Ma trận I/O -> test

| Hàng ma trận | Test chạy thật |
|---|---|
| Tra ngược `⌘M` / chord không gán | `shortcutsSearchExchange.test.ts` (`handleSearchKey` -> `editor.merge_segments`; `⌘⌥⇧U` -> rỗng; thân mục: `[data-shortcut-chord-empty]`, bộ đếm); e2e `story-12-18-settings` (chạy xanh với `⌘M` -> chỉ `editor.merge_segments`) |
| Xuất–đổi–nhập | `shortcutsSearchExchange.test.ts` (xuất chỉ ghi đè; đổi rồi nhập tệp xuất ⇒ phím cũ về, ghi đè khác giữ; nhập thay toàn bộ xoá hàng thừa; huỷ hộp thoại không ghi gì) |
| Nhập tệp hỏng | cùng tệp, `it.each` 9 ca: không JSON · không phải đối tượng · format khác · version khác · bindings là mảng · giá trị không phải mảng chuỗi · chord sai · hai lệnh va nhau · va phím mặc định của lệnh khác; mỗi ca khẳng định 0 `putConfig`/`deleteConfig` và keymap không đổi |
| Nhập id lạ | cùng tệp (`shortcuts.import_done_skipped` với `skipped: '2'`, phần còn lại áp); ghi trượt ⇒ `save_failed` |
| Thử kết nối khoá sai (401) | `connection_check_contract.rs::http_wrong_key_is_401_...` (máy chủ HTTP giả cục bộ: header `Authorization: Bearer`, đúng 1 request, không retry), `http_403_...`; vitest `aiConfigConnectionTest.test.ts` (adapter · state · nút) |
| Thử kết nối thiếu cấu hình | `aiconfig_contract.rs::test_connection_without_endpoint_or_model_or_key_...` (`prepare_connection_check` qua endpoint/mô hình/khoá/Ollama + wire `ai_config_test_connection` ⇒ `NotConfigured` và `accept()` của cổng thật không nhận kết nối nào); vitest: `[data-ai-test-result="not_configured"]` role `status` |
| Mạng chết / huỷ giữa chừng | `http_dead_network_is_unreachable_and_retryable` (cổng đóng), `http_cancel_before_sending_reaches_no_server`, `http_cancel_between_reply_chunks_gives_cancelled_not_connected`; thêm 404 · 500 · HTML · mô hình thiếu · mô hình có · URL đầy đủ đã lưu. Không đo trần 10 giây |
| Wire mới | `ipc_contract.rs::the_connection_check_and_shortcut_file_wires_are_registered` (+ `ipc_argument_contract` đã xanh nhờ adapter) |
| AC bốn mục nav / Cập nhật không nút | `settingsFrame.test.ts` (11 mục đúng thứ tự, mỗi mục một thân, Cập nhật 0 button/input/a/form, thanh Phạm vi khoá Toàn cục kèm câu lý do) |
| AC `⌘⇧R`, lưu khi rời ô, Ollama placeholder | `settingsOverlayAiConfigTierRender.test.ts` (viết lại cho thanh Phạm vi; `⌘⇧R` đi qua nút -> `dispatch` -> `clearAllAiConfigOverrides`), `aiConfigState.test.ts` |
| AC Ollama không khoá ⇒ sẵn sàng | Rust `ready_is_true_false_or_none_...` (Pha 1) + `aiBatchPanelRows.test.ts` (panel tin `ready`); câu chưa cấu hình đổi |
| AC `⌘T` | `settingsState.test.ts` (`settingsOwnsChord`, và `attachKeyboard` với cổng như `main.ts`: Cài đặt mở thì `⌘T` chạy, `⌘M` bị chặn); trả tiêu điểm chỉ có ở e2e hiện có `shortcuts-focus` |
| AC công tắc token | `aiTranslate.test.ts`, `aiBatchPanelRows.test.ts` (3 dòng token), `aiUsageVisibleState.test.ts` (mặc định bật, chỉ `off` ẩn, khoá `app_config`) |
| AC preset bố cục | `layoutUserPresets.test.ts` (vòng đời, còn sau mở lại qua `initUserLayoutPresets`, tên rỗng/trùng/dài, ghi trượt, preset hỏng; thân mục; dock THẬT: lưu -> đổi -> áp -> hình dạng về; JSON sai hình ⇒ `false` và bố cục giữ nguyên; Review Mode khoá) |
| Chế độ đọc / Dữ liệu | `settingsReadingDataBodies.test.ts` (đúng lệnh sẵn có; thanh trượt dùng chung state và lịch ghi `reading_preferences`; mốc dựng chỉ mục lỗi thì hiện lỗi) |

### Đối chứng bằng phép GỠ thật (từng cái đo đỏ đúng lý do, rồi khôi phục bản sao, `cmp` khớp)

- Rust: gỡ header Bearer trong `check_connection` ⇒ `http_wrong_key_...` đỏ; gỡ kiểm huỷ giữa các chunk ⇒ `http_cancel_between_reply_chunks_...` đỏ; gỡ `endpoint_and_model_set` ở `prepare_connection_check` ⇒ ca `test_connection_...` đỏ. LƯỢT ĐẦU CA NÀY XANH (chưa canh gì): khoá vắng đã che vế endpoint/mô hình; đã thêm hàng `ollama` không mô hình rồi đo lại ⇒ đỏ ở đúng hàng đó. Gỡ dòng `shortcuts_import` khỏi `generate_handler!` ⇒ ca `ipc_contract` mới đỏ.
- Vitest: gỡ `await inFlightSave` ⇒ ca "đợi lưu khi rời ô" đỏ; vô hiệu kiểm `commitBindings` khi nhập ⇒ 3 ca tệp hỏng đỏ; gỡ vế "phím in được là chữ" ⇒ 1 ca đỏ; không đếm id lạ ⇒ 1 ca đỏ; gỡ xoá hàng thừa ở `writeOverrideDiff` ⇒ 2 ca đỏ; `settingsOwnsChord` luôn `false` ⇒ 2 ca đỏ; bỏ `update` khỏi `settingsSectionLocksScope` ⇒ 2 ca đỏ; bỏ `aiUsageVisible &&` ở từng trong 3 dòng token ⇒ ca tương ứng đỏ; gỡ kiểm "giá trị không đổi" ở `commitAiConfigField` ⇒ 2 ca đỏ.
- KHÔNG đối chứng: `layoutUserPresets` (dock thật) và `settingsReadingDataBodies`.

### Test cũ đã sửa

- `settingsFrame.test.ts` (danh sách 11 mục + dấu mốc thân), `settingsOverlayAiConfigTierRender.test.ts` (radio tầng -> thanh Phạm vi, viết lại), `aiBatchPanelRows.test.ts` (câu chưa cấu hình mới của Quyết định Ice), mock `config/aiconfig` thêm hai adapter ở `settingsFrame`/`aiConfigState`.
- `importPreviewTwoColumn.test.ts`: chỉ đổi mốc quét mã `isBlocked: () =>` thành `isBlocked: (event) =>` (Pha 2 đổi chữ ký, nội dung khẳng định về các cờ chặn giữ nguyên). Đây là chỗ DUY NHẤT một kỳ vọng văn bản đổi theo mã.
- Các ca timeout 5 giây mà Pha 2 ghi (`settingsOverlayAiConfigKeyRender`, `settingsState`, `settingsFrameDom`) KHÔNG tái hiện: xanh khi chạy riêng và trong lượt toàn bộ.
- `message_key` trên dây mang tiền tố `err.` (`err.ai_config.test_key_rejected`); bảng lỗi ở Pha 1 bỏ tiền tố. Test dùng bản có tiền tố.

### Lệnh đã chạy (lượt toàn bộ, vì động `lib.rs` / đăng ký lệnh)

- `npm run build`: xanh (hai `vue-tsc` gồm cây test, `vite build`).
- `cargo test --locked` (src-tauri): 98 khối kết quả (lib + tích hợp + doctest), 2.625 ca xanh, 0 đỏ, 27 ignored; thoát 0.
- `npm run test` (vitest): 203 tệp, 2.905 ca xanh; thoát 0 (lượt đầu 1 đỏ là mốc quét ở trên).
- 14 cổng của `.githooks/pre-push`: deps · tokens · i18n · commands · layout · panel-refs · doc-refs · dict · dict-manifest · lint · clippy · fmt · gates · debt-owner đều xanh.
- `npm run test:e2e -- --spec e2e/specs/story-12-18-settings.e2e.mjs` (ngoài sandbox): 4/4 xanh ở lượt cuối. Lượt 1 đỏ vì hạ tầng (Vite cổng 1420 vừa dậy trả `text/html` cho `/src/main.ts`, `assertModuleGraphHealthy` dừng bộ); lượt sau webview mount bình thường, KHÔNG đứng ở `tauri://localhost`. Ba lần đỏ tiếp là lỗi của spec: radio/thanh Phạm vi chỉ có ở mục AI và mục khoá (mở Cài đặt rồi phải bấm AI trước), `expect(x, msg)` không nhận hai đối số, và bấm chuột thật vào ô tìm không đặt tiêu điểm (không có sự kiện `focus` nào) nên spec gọi `.focus()` cho ô đó.
- `shortcuts-focus` và `shortcuts-capture-mouse` chạy lại sau Pha 2: cả hai xanh, không phải sửa.

### Sổ nợ

- Đóng nợ `unconfigured_local` (dòng 13206 trước lượt này) bằng dòng `→ ✅ ĐÃ ĐÓNG 2026-10-16 (Story 12-18)`.
- Thêm một mục `Chủ: Epic 12` ở cuối `deferred-work.md`: Ice bấm Thử kết nối khoá sai/đúng trên nhà cung cấp thật + Xuất/Nhập qua hộp thoại gốc + `⌘T`/`⌘⇧R` + áp preset trên dock thật.

### CHƯA làm / rủi ro

- Không có test chạy trong app thật: Thử kết nối với nhà cung cấp thật, hộp thoại Xuất/Nhập gốc, `⌘T`/`⌘⇧R` qua WKWebView, áp preset trên dock thật (nằm trong mục nợ `Chủ: Epic 12`).
- Trả tiêu điểm về `Công cụ ▾` sau `⌘T` không có test (chỉ `shortcuts-focus` hiện có cho đường mở bằng chuột).
- Trần 10 giây và trần danh sách 4 MiB của Thử kết nối không có test.
- Bấm chuột thật (WebDriver) không đặt tiêu điểm cho ô nhập nào trong e2e: ô tìm phím tắt và ô endpoint của mục AI đều ở lại `.set-panel`; ô tìm không có `mousedown`/`preventDefault` nào, probe thấy mousedown/mouseup/click tới ô mà không có `focus`. Kết luận: quirk của WebDriver/WKWebView, không phải lỗi app; chuột thật của người dùng chưa kiểm.
- Spec e2e tự đặt tiêu điểm bằng `.focus()` ở một chỗ; chuỗi `⌘T` -> tiêu điểm trả về chỗ cũ vẫn không được đo.
- Máy chủ giả của `connection_check_contract.rs` là bộ nhận HTTP/1.1 tối thiểu tự viết (một request một kết nối), đủ cho các hàng ma trận; không thay được nhà cung cấp thật.

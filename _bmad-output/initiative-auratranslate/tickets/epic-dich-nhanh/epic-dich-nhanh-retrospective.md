---
epic: epic-dich-nhanh
date: 2026-11-11
verdict: rejected
criteria: declared
headless: false
---

# Retrospective — Epic 15: Dịch nhanh

## Epic summary

- Epic: `epic-dich-nhanh` (id 15), Done when khai trong `epic-dich-nhanh.md` §Done when (6 mục). Ice không nêu trọng tâm riêng trước Phase 1.
- Vé (thứ tự build = thứ tự `status`): 15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7 — cả 7 `done`, `pending_tickets` rỗng. 15.1–15.4 lên `done` ở `d63cdae4`; 15.5–15.7 lên `done` trong cây làm việc, chưa commit (thay đổi của Ice, retro không đụng).
- Không vé nào còn ở `built`.

### Dải commit theo plan

| Vé | `baseline_revision` | Dải | Commit |
|---|---|---|---|
| 15.1 | `a9019def` | `a9019def..e0e5b48d` | `e0e5b48d` |
| 15.2 | `e0e5b48d` | `e0e5b48d..7d6e5186` | `7d6e5186` |
| 15.3 | `7d6e5186` | `7d6e5186..45e3dd48` | `45e3dd48` |
| 15.4 | `45e3dd48` | `45e3dd48..d63cdae4` | `ce0d78b6`, `d63cdae4` (docs) |
| 15.5 | `d63cdae4` | `d63cdae4..01fe4f12` | `01fe4f12` |
| 15.6 | `01fe4f12` | `01fe4f12..3dfcf11b` | `c292f35c`, `3dfcf11b` (gate) |
| 15.7 | `3dfcf11b` | `3dfcf11b..HEAD` (suy ra; HEAD = `33118293`) | `41ae3826`, `33118293` (docs) |

Lịch sử thẳng, 0 merge. Mã sản phẩm (`src`, `src-tauri/src`): 37 tệp, +4 333 / −202. Test (`tests`, `src-tauri/tests`, `e2e`): 28 tệp, +6 056 / −112.

### Bằng chứng có / thiếu

- Có: tệp epic, `.memlog.md`, 7 plan, 6 handoff (15.7 không có), `git_evidence.py` cho 7 dải, sổ nợ (`deferred-work.md:13880–13888`, `:14075–14100`), CI push 43048708565 (`41ae3826`) và 43055585867 (`33118293`) đều `success`.
- Thiếu: nightly e2e chứa bất kỳ revision nào của Epic 15 (nightly gần nhất 42917495709 ở `d2cf9d65`, trước epic). Không plan nào có khối `## Code Review` có ngày; review ghi ở Review Triage Log.
- Retro trước: epic liền trước theo thứ tự `epics` là `epic-san-chuot` (14) — không có tệp retrospective.
- Session log: có trong `~/.claude/projects/…` nhưng không đọc (thu hẹp phạm vi: bài học quy trình chỉ lấy từ plan, handoff, commit, sổ nợ).

## Findings

Ba góc nhìn chạy bằng agent riêng (review diff, góc nhìn tổng hợp, đối chiếu spec); mọi phát hiện dưới đây được đọc lại ở nguồn trước khi ghi. Dòng mã tính ở HEAD. `aitranslate.rs` = `src-tauri/src/commands/aitranslate.rs`. Mỗi mục có hai hướng xử lý: **Lần này** (sửa ngay / hoãn / chấp nhận) và **Lần sau** (bài học quy trình).

### Review diff — ranh giới giữa các story

- **F1 · medium · đã đọc mã.** Bảng chờ huỷ cả lượt bấm tay khi lưu cấu hình nhanh hoặc bật `Tự dịch`; màn xem trước chỉ huỷ lượt `auto`. `src/glossaryQueueState.ts:248-254` gọi `abortContextReference()` vô điều kiện trên `[quickAiAutoEnabled, quickAiConfigRevision]`; `src/importPreviewState.ts:1149-1157` chỉ huỷ khi `quickBatchRun.value?.trigger === 'auto'`. `quickAiConfigRevision` tăng ở `src/quickAiConfigState.ts:110,138,154` (lưu trường, lưu khoá, xoá khoá). AD-64 mục 8 chỉ kể "rời hàng, rời màn, đóng lớp phủ hoặc tắt tuỳ chọn". Kịch bản: Ice bấm `Dịch thử` ở Bảng chờ, sang Cài đặt lưu model nhanh → câu đang dịch bị huỷ, hàng về trạng thái cũ. Gốc: 15.5 (`01fe4f12`) trên `quickAiConfigRevision` của 15.2. **Lần này:** sửa ngay, sau khi Ice chốt lưu cấu hình có nên huỷ lượt bấm tay hay không. **Lần sau:** xem F5.
- **F2 · low · có thể xảy ra, chưa tái hiện.** Đường tự chạy có thể dùng cấu hình chính. `quick_auto_gate` (`aitranslate.rs:679-705`) đọc "đã khai riêng" một lần; `prepare` trong `spawn_blocking` (`:2950-2953`) đọc lại qua `resolve_quick_call_config` (`:984-994`), hàm không nhận `trigger` và rơi về `MainGlobal` khi endpoint/model nhanh trống. Nếu người dùng xoá cấu hình nhanh trong khe giữa cổng và `prepare`, một lời gọi `auto` đi bằng cấu hình và khoá chính, trái AD-64 mục 2 "Tự chạy không bao giờ dùng cấu hình chính". **Lần này:** sửa ngay (đưa `trigger` vào hàm phân giải, `auto` mà không phải `Dedicated` thì trả `auto_off`), cùng vé với F1. **Lần sau:** một luật "không bao giờ" phải nằm ở chỗ dùng giá trị, không ở cổng đọc trước đó.
- **F3 · low · có thể xảy ra.** Dọn kho đua với tự chạy. `quick_purge_wire` gọi `cancel_all` trước khi xoá (`aitranslate.rs` ~1216-1225), nhưng webview chỉ dừng hẹn giờ khi `markQuickStorePurged()` chạy sau `VACUUM` (`src/settingsQuickDataState.ts:162`). Một hẹn giờ 800 ms nổ trong khe đó lấy thế hệ mới, gọi mạng, và có thể ghi hàng sau `DELETE`, nên Cài đặt báo đã dọn N câu mà vẫn còn hàng. Ranh giới 15.4 × 15.5/15.6. **Lần này:** hoãn. **Lần sau:** —
- **F4 · low · đã đọc mã.** Màn xem trước bỏ qua cờ `saved` của sự kiện `received` (`src/config/quickBatch.ts:15` kiểm hình dạng, `importPreviewState.ts` ~1050 không đọc). Lưu lỗi thì khối vẫn hiện `done`, lần mở sau âm thầm trả tiền dịch lại. Đường một câu cố ý ghi log rồi trả `done` (15.1 Implementation Notes). **Lần này:** chấp nhận, vì 15.1 đã chọn có chủ đích; trường `saved` trên dây đang chết. **Lần sau:** —
- Các ranh giới đã đọc hết mà không thấy lỗi: `run_id` dùng chung qua `quickTranslateRun.ts`; `begin` trả `Busy` cho bề mặt khác, vé `drop` khoá theo `(surface, run_id)`; `save` kiểm `still_current` trong job của `Writer`; không đường dịch nhanh nào ghi `AiTranslateGeneration`. Ngoài phạm vi review: template `.vue`, `aiconfig`/keychain Rust, dây nối `lib.rs`, i18n, các spec e2e.

### Góc nhìn tổng hợp

- **F5 · Trùng lặp · đã đọc mã.** Hai bộ hẹn giờ tự dịch viết song song, không có helper chung. Tám mối lo trùng nhau: dọn hẹn giờ, cổng cho chạy, giờ 800 ms, tập lỗi bị bỏ qua, theo dõi cấu hình, theo dõi hoạt động, theo dõi dọn, trạng thái tạm dừng (`glossaryQueueState.ts:137-266` so với `importPreviewState.ts:877-1187`). Tập lỗi khác kiểu nhau (`Set<number>` / `Set<string>`). `importPreviewState.ts:82` lấy `QUICK_AUTO_WAIT_MS` từ `glossaryQueueState.ts`, thành một cạnh phụ thuộc giữa hai bề mặt. Hai wire `ai_context_reference_translate` (`aitranslate.rs:2915`) và `quick_translate_blocks` (`:2980`) có phần mở đầu gần giống nhau, khoảng 25 dòng. Phép thử "endpoint và model nhanh đều có" xuất hiện ba lần (`aitranslate.rs:695-700`, `:1004-1005`, `commands/aiconfig.rs:458`). F1 là hệ quả trực tiếp: hai bản sao lệch nhau. **Lần này:** hoãn (tách bộ hẹn giờ dùng chung vào `quickTranslateRun.ts` hoặc một module mới). **Lần sau:** story thứ hai của cùng một cơ chế phải đọc story thứ nhất và dùng lại, không chép.
- **F6 · Phình tệp · đã đo.** `aitranslate.rs` tăng 2 194 → 3 194 dòng (+46 %) qua 15.1–15.5. Riêng phần dịch nhanh chiếm khoảng `:670-720` (cổng), `:870-1300` (khe lượt, cấu hình, dây kho), `:1283-1625` (hai hàm chạy), cộng khoảng 8 lệnh trong `wire`. Spine §Cây nguồn nói `commands/` là "adapter, no business rules". AD-64 §Binds cho phép đặt `resolve_quick_call_config` và `QuickTranslateGeneration` ở đây, nhưng không cho phép `quick_translate`/`quick_translate_blocks`, nơi chứa tra kho, gọi nhà cung cấp, luật `maybe_truncated` và lưu. Tiền lệ có sẵn: `run_translate_operation` cũng nằm trong tệp này. `core/quick_translation/mod.rs` chỉ 290 dòng, toàn phần kho. `src/importPreviewState.ts` 2 898 dòng, 43 hàm export; `src/ImportPreviewOverlay.vue` 3 313 dòng. Repo không có cổng giới hạn cỡ tệp. **Lần này:** hoãn, đưa Winston xem việc tách phần chạy dịch nhanh khỏi `commands/`. **Lần sau:** —
- **F7 · Kiến trúc / đối chiếu spec · đã đọc mã.** 15.5 sửa `AiTranslateGeneration`: thêm bộ đếm hoạt động, `begin_activity`, lệnh `ai_translate_activity` và sự kiện rời `ai-translate-activity-v1`, phát ở `aitranslate.rs:2583, 2683, 2776, 2847` và nghe ở `src/config/aiTranslateActivity.ts:42`. Test canh cũ `the_command_file_carries_zero_emit_or_listen_calls_on_the_translate_path` được thay bằng `the_translate_path_emits_only_typed_activity_status_and_never_stream_tokens`, chừa đúng một dòng emit (`src-tauri/tests/ai_translate_contract.rs:1865-1905`, `01fe4f12`). Điều này trái chữ của Boundaries trong tệp epic ("Must not change: lô dịch thật và `AiTranslateGeneration`"), của R5 và của AD-64 mục 8 ("không đụng `AiTranslateGeneration`"). Người cho phép chỉ là một câu trong plan 15.5 (`story-tu-dich-hang-ang-mo-o-bang-cho-glossary-plan.md` §Implementation Notes); `grep -i activity` trên spine không ra gì. Bất biến AD-22 (luồng token chỉ qua Channel) vẫn giữ, và AD-64 mục 1 ("không bắt đầu khi lô dịch thật đang chạy") cần một tín hiệu như thế. **Lần này:** chấp nhận độ lệch; đề xuất Winston thêm dòng cập nhật có ngày vào AD-64 mục 1/8 (và một dòng ở AD-22) ghi bộ đếm và sự kiện. **Lần sau:** khi một test canh kiến trúc phải nới, người cho phép là Winston qua spine, không phải câu trong plan.
- **F8 · Mẫu khác lệ · đã đọc.** 3dfcf11b thêm 16 miễn trừ có tên vào `scripts/check-panel-refs.mjs`, mỗi miễn trừ có lý do: 10 là bất biến thật hoặc trạng thái kho Global; 6 là trạng thái màn hình được `loadQuickDataRow` đặt lại khi mount (`src/settingsQuickDataState.ts:129-136`), nhưng cổng chỉ nhận hàm tên `reset*`. AGENTS.md cấm nới danh sách loại trừ để xoá đỏ, trừ khi mỗi mục có tên, có lý do và có thể bỏ được, và các mục này đạt điều đó. Các sàn cổng (`FILE_FLOOR`, `COMMAND_FLOOR`, `CLICK_FLOOR`, `POINTER_FLOOR`, `WEBVIEW_FLOOR`, `PLANNED_TICKET_FLOOR`) chỉ tăng theo công thức `ceil(0.85 × live)` của `scripts/lib/floor-judge.mjs`; không sàn nào bị hạ. **Lần này:** chấp nhận. **Lần sau:** nếu một danh mục còn lặp lại (trạng thái kho Global, hàm đặt lại khi mount), cổng nên nhận danh mục thay vì từng tên.
- **F9 · Mẫu khác lệ · thấp.** Plan 15.2 và 15.5 viết tiếng Anh (6/96 và 5/93 dòng có dấu tiếng Việt); năm plan còn lại tiếng Việt. AGENTS.md không quy định ngôn ngữ plan. **Lần này:** chấp nhận. **Lần sau:** —

### Đối chiếu spec với mã

- R1, R2, R3, R4, R6, R7, R8, R9, R11: đạt, mỗi mục có test canh đọc ở nguồn (bảng đầy đủ nằm trong báo cáo của agent đối chiếu; retro không chạy lại test). R5: đạt về hành vi, lệch chữ (xem F7). R10: phần hiển thị đạt; vế "không ghi `AiCallRecords`, không đổi `LastAssembledPromptState`" chỉ đúng do cấu trúc (các hàm dịch nhanh không nhận hai state đó), không test nào canh. R12: vitest canh Esc, J/K, Tab, `mousedown.prevent`; kích hoạt bằng bàn phím trong app thật thuộc mục nợ dùng thật.
- **F10 · Đối chứng AD-64 mục 12 · đã đọc plan, handoff, commit.** (b), (c), (f), (g) có phép gỡ thật kèm lý do đỏ (handoff 15.2 :83-84; 15.3; 15.4 Implementation Notes). Ghi chép còn mỏng ở: (a) nửa Bảng chờ "tắt tuỳ chọn trong lúc chờ không có lời gọi" (đã gỡ cho màn xem trước ở 15.6, chưa thấy cho Bảng chờ); (d) vế "ảnh chụp `segment`/`tm_unit`/`glossary_entry` không đổi qua lệnh thật" chỉ có ở e2e `story-15-1`, không có phép gỡ được ghi lại, còn ca Rust gọi thẳng `quick_translate` (15.1 Review Triage #2); (e) và (h) chỉ có một câu trong commit `e0e5b48d` ("đỏ đúng lý do khi gỡ seam thật"), không kèm lý do đỏ. Ngoài ra (f) ca sai `run_id` và (g) gọi thẳng struct hoặc `quick_purge_wire`, không qua lệnh `wire::`. **Lần này:** sửa ngay trước khi đóng epic: chạy và ghi lại phép gỡ cho (a)-Bảng chờ, (d)-e2e, (e), (h), hoặc Ice chấp nhận ghi chép hiện có. **Lần sau:** lý do đỏ của từng chữ cái đối chứng ghi vào plan, không chỉ trong commit message.
- **F11 · Spec cũ · đã đọc spine.** Mục `Chưa chốt` của AD-64 (spine :1548) vẫn ghi `QUICK_BATCH_MAX_BLOCKS`, `QUICK_BATCH_MAX_CHARS`, thời gian chờ tự chạy và N "chưa có số đo". Thực tế đã chốt: 40 khối / 4 000 ký tự (đo trên bảy trang `6-1-ban-do`, plan 15.3), 800 ms (Ice duyệt, plan 15.5), N = 30 (`scope/store.rs:134`). Dòng cập nhật cho N mà tệp epic hứa (§Notes) chưa có. Open question trong §Notes của tệp epic cũng đã cũ. **Lần này:** đề xuất Winston thêm dòng cập nhật có ngày vào spine; không sửa tệp epic. **Lần sau:** —
- Phạm vi chưa xác minh được: lượt dùng thật, Windows, kích hoạt bằng bàn phím trong app thật, nightly e2e.

### Quy trình (từ plan, handoff, commit, sổ nợ; không đọc session log)

- **F12 · Đỏ truyền qua các story vì `pre-push` không chạy được · đã đối chiếu git.** Lần push cuối trước epic là `d2cf9d65` (2026-11-09). `check:clippy` đỏ từ 16-6 (`57810b33`, Epic 16, dòng `&PathBuf` ở `src-tauri/tests/convention_exchange_contract.rs:115`). `check:clippy` nằm trong vòng cổng của `.githooks/pre-push:64`, nên 22 commit (14 của Epic 16, 8 của Epic 15 tới `3dfcf11b`) nằm cục bộ tới `41ae3826`. Trong khoảng đó mỗi story chỉ chạy tập test riêng của mình, và ba cái đỏ tích lại mà không story nào thấy: `check:panel-refs` 16 ô từ 15.4 (§Verification của plan 15.4 không có `check:panel-refs`); `check:debt-owner` đỏ vì sàn từ plan 15.6; e2e `glossary-context-reference.e2e.mjs` đỏ từ 15.2 do readiness nạp bất đồng bộ. Cả ba được phát hiện và sửa ở 15.6/15.7. 15.1–15.4 lên `done` (`d63cdae4`) khi CI mới chỉ đọc ở `d2cf9d65`, revision có trước cả epic. Riêng 15.5 ghi khoảng trống này thành nợ. CI đầu tiên chứa Epic 15 là 43048708565, xanh cả hai nền tảng. **Lần này:** đã tự đóng (CI xanh). **Lần sau:** `pre-push` đỏ thì sửa trước khi bắt đầu story kế tiếp, vì đó là lượt chạy đủ duy nhất.
- **F13 · Câu sai trong Intent đã khoá · đã đếm.** Plan 15.7 §Intent ghi "22 commit của Epic 15 chưa push được". `git rev-list --count d2cf9d65..3dfcf11b` = 22, nhưng chỉ 8 commit là của Epic 15 (`a9019def..3dfcf11b`); 14 commit còn lại thuộc Epic 16. **Lần này:** sửa tại chỗ bằng một dòng cập nhật có ngày. **Lần sau:** —

## Behavior verification

Retro không tự chạy lại luồng nào (thu hẹp phạm vi: e2e cần dựng binary `--features wdio` và chạy ngoài sandbox). Bằng chứng hành vi đã có:

- Plan 15.7 §Implementation Notes: bốn spec `story-15-*` và `glossary-context-reference.e2e.mjs` xanh trên WKWebView (macOS, Debug, rustc 1.99.1) ở mã giống `41ae3826`, sau khi sửa spec đỏ từ 15.2. Lượt sau đó (`33118293`) chỉ sửa tài liệu.
- Số đo e2e của 15.6: 22 lời gọi, 60/60 khối gửi đúng một lần, khoảng cách giữa hai lời gọi 804–825 ms; khi tắt tuỳ chọn thì 0 lời gọi (plan 15.6).
- Push CI 43048708565 (`41ae3826`) và 43055585867 (`33118293`): `success`.
- Chưa có: nightly e2e chứa Epic 15, lượt dùng thật bằng khoá thật, Windows.


## Previous-retro follow-through

Không có gì để đối chiếu: `epic-san-chuot/epic-san-chuot-retrospective.md` không tồn tại (thiếu tệp, không phải "không có mục tồn").

## Action items

Tất cả mục dưới đây là **đề xuất**. Retro không sửa mã, spec, sổ nợ hay trạng thái nào.

| # | Việc | Nguồn | Chủ | Loại |
|---|---|---|---|---|
| A1 | Lượt dùng thật Done when 4: khoá thật, một Tác phẩm `zh`, một Tác phẩm `en`, một lần nhập URL, `Dọn` thu nhỏ tệp, Tab/Space/Enter ở công tắc `Tự dịch` (mục nợ có sẵn, `deferred-work.md:13882-13884`) | Done when 4 | Ice | chặn đóng epic |
| A2 | Bảng chờ chỉ huỷ lượt `auto` khi cấu hình nhanh được lưu hoặc `Tự dịch` đổi, như màn xem trước; lượt bấm tay chạy tiếp (Ice duyệt đề xuất, 2026-11-12) | F1 | Amelia (vé do Sofia mở) | sửa mã |
| A3 | Đường `auto` không bao giờ rơi về cấu hình chính: `resolve_quick_call_config` nhận `trigger`, `auto` mà không phải `Dedicated` thì trả `auto_off`; thêm ca canh. Gộp với A2 thành một story sửa lỗi | F2 | Amelia (vé do Sofia mở nếu Ice duyệt) | sửa mã |
| A4 | ~~Chạy lại phép gỡ cho (a)-Bảng chờ, (d)-lệnh thật, (e), (h)~~ Ice chấp nhận ghi chép hiện có (2026-11-12); không làm | F10 | — | đã quyết |
| A5 | Spine: dòng cập nhật có ngày cho AD-64 `Chưa chốt` (40 / 4 000 / 800 ms / N = 30); dòng cập nhật cho AD-64 mục 1/8 và AD-22 ghi bộ đếm hoạt động và sự kiện `ai-translate-activity-v1` | F7, F11 | Winston | đối chiếu spec |
| A6 | ~~Thêm dòng "`pre-push` đỏ ⇒ sửa trước story kế tiếp" vào AGENTS.md~~ Ice: không cần (2026-11-12); không làm | F12 | — | đã quyết |
| A7 | Sổ nợ: F3 (dọn kho đua với tự chạy) và F5/F6 (bộ hẹn giờ trùng nhau; tách phần chạy dịch nhanh khỏi `commands/aitranslate.rs`), mỗi mục tối đa 5 dòng, có `Chủ:` hợp lệ | F3, F5, F6 | Ice chọn chủ | hoãn |
| A8 | Đọc nightly e2e đầu tiên chứa `41ae3826` rồi đóng mục nợ CI (`deferred-work.md:14095-14100`) | Done when 6, AGENTS.md §Tests | Epic 15 | theo dõi |
| A9 | Plan 15.7 §Intent: sửa "22 commit của Epic 15" bằng dòng cập nhật có ngày (8 của Epic 15, 14 của Epic 16) | F13 | Ice | sổ sách |

Chấp nhận, không làm: F4 (cờ `saved` chết, 15.1 đã chọn có chủ đích), F8 (16 miễn trừ có lý do; sàn chỉ tăng), F9 (ngôn ngữ plan), R10 vế phủ định (đúng do cấu trúc).

## Acceptance verdict

**Machine verdict: rejected (not accepted)** · criteria: declared · Ice đã quyết A2, A4, A6 (2026-11-12) nhưng chưa ghi đè verdict.

| Done when | Kết quả | Bằng chứng |
|---|---|---|
| 1. Đối chứng (a)–(h) đỏ đúng lý do; test canh xanh | đạt theo quyết định của Ice | F10: (b)(c)(f)(g) có ghi chép đủ; (a)-Bảng chờ, (d)-lệnh thật, (e), (h) mỏng, Ice chấp nhận ghi chép hiện có (2026-11-12) |
| 2. e2e WKWebView mỗi bề mặt: trúng kho 0 lời gọi; tắt 0 lời gọi | đạt (macOS) | `story-15-1` (Bảng chờ, trúng kho), `story-15-3` (xem trước, trúng kho), `story-15-5` (Bảng chờ, tắt), `story-15-6` (xem trước, tắt); xanh ở 15.7 |
| 3. `check:commands` xanh, `POINTER_DEBT` không tăng | đạt | `POINTER_DEBT = []` ở `a9019def` và HEAD; cổng thoát 0 |
| 4. Mục nợ `Chủ: Epic 15` đóng bằng lượt dùng thật của Ice | **chưa** | `deferred-work.md:13882-13884` còn nửa đóng |
| 5. Phần chỉ-Windows nằm trong mục `B7` | đạt | `deferred-work.md:13885-13887`, `Chủ: B7` |
| 6. CI xanh cả nửa Windows | đạt | 43048708565, 43055585867 `success`; nightly e2e đầu tiên vẫn chờ (A8) |

Mọi vé đều `done`, `pending_tickets` rỗng. Verdict là rejected chỉ vì Done when 4 chưa đạt. Done when 1 đạt nhờ Ice chấp nhận ghi chép đối chứng (2026-11-12). Đây là trạng thái chờ Ice, không phải epic hỏng: khi Ice chạy xong A1, các tiêu chí đều đã có bằng chứng, và verdict có thể thành **accepted-with-open-items** (mở: A2/A3 nếu chưa sửa, A5, A7, A8). Ice có quyền ghi đè.

## Open questions

Ice trả lời 2026-11-12:
- F1/A2: theo đề xuất. Lưu cấu hình nhanh hoặc bật `Tự dịch` chỉ huỷ lượt `auto`, không huỷ lượt bấm tay ở Bảng chờ.
- A4: chấp nhận ghi chép đối chứng hiện có.
- A6: không thêm dòng vào AGENTS.md.

Còn mở:
- Retro có nên chạy lại bốn spec e2e `story-15-*` để có quan sát hành vi của riêng mình không? Lượt đó cần dựng binary `--features wdio` và chạy ngoài sandbox.

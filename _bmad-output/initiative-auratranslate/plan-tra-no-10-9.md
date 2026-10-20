---
title: 'Trả 26 mục nợ có chủ Story 10.9'
type: 'chore'
ticket: ''
created: '2026-10-20'
status: 'built'
baseline_revision: 'd78645ba9ae9a31e49f152253ffb370c848869c4'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 26 mục mở trong `deferred-work.md` có `Chủ:` cuối là Story 10.9 (`:35 :201 :407 :735 :834 :939 :1171 :1179 :2496 :5092 :5762 :5887 :7997 :8024 :8050 :8177 :8889 :8921 :9260 :9288 :9338 :9517 :9545 :9947 :11966 :11979`, số dòng ở `d78645b`). Story 10.9 (nghiệm thu NFR cuối, `planned`) chưa có plan. Có mục đã hết đúng, có mục đã có số đo mà vẫn ghi mở, có mục không thuộc phạm vi NFR của nó.

**Approach:** Mỗi mục nhận một dòng `→ … 2026-10-20`, theo khuôn của `plan-tra-no-2-4.md`. Đóng được thì đóng (`✅` · `🟡` · `KHÔNG LÀM`), chuyển chủ khi luật kho hoặc Ice chỉ chỗ, còn lại ghi `🔵` và giữ `Chủ: Story 10.9`. Ba mục được vá ngay.

**Decisions (Ice 2026-10-20):**
- Vá `:8889`: gác `contains('\r')` trước hai lượt `replace` của `normalize`.
- Vá `:9338`: cache `Regex` của mẫu phân tách Chương.
- Vá `:735`: Kiểm A của `check-i18n` miễn trừ có tên cho đối số của `console.*` trong `.vue`, rồi trả dấu cho 12 chuỗi chẩn đoán.
- `:9260` không vá, ghi 🔵, giữ Story 10.9.
- `:2496` và `:5092` (NFR2) chuyển `Chủ: Story 2.4`.
- `:8024` chuyển `Chủ: Winston` (soạn ad-brief đổi hình dạng truy vấn trigram).
- Giữ plan một tệp dù vượt 1 600 token.

## Boundaries & Constraints

**Always:** Mỗi bản vá có một phép đối chứng GỠ thật (AGENTS.md §Tests). Số đo nêu build, profile và quần thể. Dòng sổ ✅ không chứa cụm "còn mở"/"vẫn mở" (`HALF_ON_CLOSED_LINE_RE` hạ nó xuống 🟡).

**Never:** Không chạm nợ của chủ khác. Không tạo plan hay sửa ticket của Story 10.9. Không đo lại NFR3/4/5/6. Không đổi `[profile.release]`. Không dời chuỗi từ `.vue` sang `.ts` để qua cổng. Không thêm hằng số giới hạn cỡ cache chọn tay.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| normalize không `\r` | văn bản LF thuần | kết quả y hệt trước, không chạy `replace` | — |
| normalize CRLF / CR đơn | `a\r\nb\rc` | `a\nb\nc` như trước | — |
| Kiểm A, chuỗi có dấu trong `console.error('…')` của `.vue` | `<script>` | không hit | — |
| Kiểm A, chuỗi có dấu SAU dấu `)` đóng của `console.*` | `console.warn(x); const y = 'Đã lưu'` | vẫn hit | exit 1 |
| Kiểm A, `console.*(` lồng ngoặc/template | `console.error(\`… ${f('Đã')} …\`)` | không hit, miễn trừ kết thúc đúng ở `)` cân bằng | — |
| Mẫu Regex hỏng | `pattern = "("` | `match_starts` vẫn trả `Err(regex::Error)` | không cache lỗi thành thành công |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/segment/normalize.rs:86`, `:223` — `replace("\r\n","\n").replace('\r',"\n")` vô điều kiện. Probe: `tests/story_6_18_debt_probes.rs::perf_probe_normalize_two_pass_replace_cost_on_a_ten_megabyte_crlf_free_chapter`.
- `src-tauri/src/core/segment/chapterpattern.rs:48-53` — `#[derive(Debug, Clone, PartialEq, Eq)] struct ChapterPattern { kind, pattern }`. Ở `:80-100`, `match_starts` gọi `compile` (`:105`) mỗi lượt. `resolve_chapter_pattern` biên dịch để nghiệm thu rồi bỏ. Probe: `story_6_18_debt_probes.rs::perf_probe_match_starts_recompiles_the_regex_every_call`.
- `scripts/check-i18n.mjs` — `scanScript` (`:513`) là máy trạng thái `code/string/template/comment` có `lastSig` và ngăn xếp `interp`. `scanVue` (`:865`) gọi nó cho mỗi `<script>`. Kiểm A ở `:934-1030`. Khuôn tự kiểm nội tuyến: `rsLineExempt` + khối `{ … fail(...) }` (`:962-976`).
- `src/layout/WorkspaceDock.vue:252,280,390,465,475,769,817,843,876,961,977` (`console.error`) và `src/modes/WorkspaceMode.vue:86` (`console.warn`): chuỗi không dấu. Khối chú thích `WorkspaceMode.vue:19-30` và các chú thích "⚠️ Chẩn đoán viết KHÔNG DẤU" sẽ thành sai sau bản vá.
- `_bmad-output/initiative-auratranslate/deferred-work.md` — 26 mục. Bằng chứng cho từng dòng nằm ở Tasks.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/normalize.rs` — bọc hai lượt `replace` ở `:86` và `:223` trong `if text.contains('\r')`. Kiểm có sẵn ca cho CRLF và CR đơn; thiếu thì thêm vào tệp test của normalize.
- [x] `src-tauri/src/core/segment/chapterpattern.rs` — trước hết lần theo đường sản phẩm (`resolve_chapter_pattern` → `PipelineInput::chapter_pattern` → `match_starts`) xem CÙNG một chuỗi mẫu có bị biên dịch lại qua các lượt `run_pipeline` thật không. Có thì đặt cache ở chỗ tái dùng được: field lười kèm `PartialEq`/`Eq`/`Debug` tự viết chỉ so `kind`+`pattern`, hoặc bảng tĩnh theo chuỗi mẫu không trần chọn tay. Nếu mỗi lượt thật dùng một chuỗi khác nhau thì DỪNG và báo lại, không thêm cache vô ích.
- [x] `scripts/check-i18n.mjs` — trong `scanScript`, khi ở trạng thái `code` gặp `console.<tên>(`, các chuỗi trong đối số tới `)` cân bằng không đẩy hit. Tên miễn trừ in ra log Kiểm A. Thêm một khối tự kiểm nội tuyến phủ bốn hàng Kiểm A của ma trận.
- [x] `src/layout/WorkspaceDock.vue`, `src/modes/WorkspaceMode.vue` — trả dấu cho 12 chuỗi. Xoá khối `:19-30` và các chú thích "viết KHÔNG DẤU" trong dòng đổi.
- [x] `deferred-work.md` — mỗi mục một dòng `→ … 2026-10-20`, kết thúc bằng `**Chủ: …**` khi chủ đổi hoặc được giữ:
  - ✅ `:8889`, `:9338`, `:735`: bản vá cộng số đo trước/sau.
  - ✅ `:9545`: `7f7ee1d` (2026-09-07 14:59, sau khi mục được ghi lúc 08:04) thêm pool 8 client `shared_client()` (`core/webimport/fetcher.rs:153`, gọi ở `:266`). Số `bench-release` 2026-09-14 đo trên đường này.
  - ✅ `:9517`: 43 lượt CI xanh từ 2026-09-07 (`34737934378` … `39397681281`) chạy trọn `cargo test` ở hai nửa. Trong 47 lượt đỏ, `webimport_contract` đỏ một lần, `34553274876` (thiếu fixture `6-1-ban-do/…/a01.html`); bốn ca wall-clock không đỏ lần nào. Lượt đỏ ở `store_contract` không chạy tới bộ này vì `cargo test` dừng ở binary đỏ đầu tiên. Ở `39397681281`: 0,36 s trên macOS, 16,17 s trên Windows.
  - 🟡 `:201`: `⌘1`/`⌘2` có assert ở `e2e/specs/story-5-5-progress.e2e.mjs`, `⌘3` ở `story-5-13-reading-marks.e2e.mjs`, đều qua WebDriver trong WKWebView thật. Vế tầng OS giữ `Chủ: Story 10.9`.
  - 🟡 `:8050` (vế "hàng trăm `.atproj`" đo trên thư viện 5 000 Chương) và `:8921` (có vá hay không là kết luận A8 của 10.9): giữ 10.9.
  - `Chủ: B7` cho `:834` (luật "Windows-only checks go to B7"). `Chủ: Story 2.4` cho `:2496` và `:5092`. `Chủ: Winston` cho `:8024`.
  - 🔵 giữ 10.9: `:35` (`Cargo.toml:219-220`, chưa có panic hook), `:407`, `:939`/`:1171`/`:1179` (`lookupTiming.ts:40` tắt mặc định; `dict.rs:46-56` gọi `load_global_config` mỗi lượt tra), `:5762`/`:5887` (`chapter.rs:151,270,472`), `:7997`, `:8177`, `:9260` (`mod.rs:2460`), `:9288`, `:9947` (`ImportPreviewOverlay.vue:375-381`), `:11966` (`glossary/store.rs:1585-1612`), `:11979` (`ai_boundary.rs:1458`).

**Acceptance Criteria:**
- Given sổ sau lượt sửa, when chạy `node /private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/2a7a9ee8-3802-4596-a7ab-abf53e48af31/scratchpad/list109.mjs _bmad-output/initiative-auratranslate/deferred-work.md <out.md>` (in các mục mở hoặc 🟡 có chủ cuối Story 10.9), then 26 mục đều có dòng 2026-10-20. Các mục còn chủ 10.9 là đúng 17 mục (26 trừ năm mục ✅, một B7, hai Story 2.4, một Winston), và `check:debt-owner` xanh.
- Given `match_starts` sau bản vá, when chạy lại probe recompile ở `bench-release`, then khoảng chênh giữa 7 lượt gọi thật và 7 lượt quét trên regex biên dịch sẵn co về mức nhiễu. Gỡ cache thì khoảng chênh quay lại.
- Given probe normalize ở `bench-release`, when chạy lại, then `normalize()` trên 10 MB không `\r` giảm khoảng phần hai lượt `replace` đã đo. Gỡ lượt gác thì số trở lại.
- Given khối tự kiểm mới của Kiểm A, when gỡ nhánh miễn trừ `console.*` khỏi `scanScript`, then tự kiểm đỏ và 12 chuỗi có dấu đỏ ở Kiểm A.

## Implementation Notes

- Đường sản phẩm có biên dịch lại cùng một chuỗi mẫu: mỗi lượt `run_pipeline` dựng lại `ChapterPattern`, và đường song ngữ gọi `match_starts` mỗi hàng. Vì vậy cache là một ô tĩnh `Arc<Regex>` chứa mẫu gần nhất (`compile_cached`). Năm ứng viên xen kẽ vẫn biên dịch lại; một bảng không trần sẽ phình theo từng mẫu người dùng gõ.
- Bản đầu trả `Regex::clone()`. Mỗi bản sao của `regex-automata` mang một pool cache rỗng, nên còn dư 0,40 ms. Đổi sang `Arc` thì không còn phần dư vượt nhiễu.
- Agent đầu chèn dòng của `:2496` SAU `---`. Parser cắt mục ở `---`, nên dòng ấy không thuộc mục nào và cổng vẫn xanh. Dòng 🟡 của `:8050` từng khai vế "hàng trăm `.atproj`" là đã đo. Cả hai đã sửa tay.
- `normalize` chỉ giảm 9–10 ms, trong khi `two_pass_replace` mà probe đo riêng là 23,6–30,3 ms; phần chênh chưa đo tách. Tải máy giữa hai phía không đều (4,1–12,9).
- Đối chứng của `check-i18n` là xoá thật hai khối `console.*` trong `scanScript`: tự kiểm đỏ, Kiểm A đỏ 19 dòng.
- AC4 ghi "12 chuỗi đỏ". Thực tế 12 chuỗi nằm trên 19 dòng, vì Kiểm A đếm theo dòng.

## Plan Change Log

## Review Triage Log

Lens quick: 2 medium · 8 low · 2 false. Tất cả đều được vá, không có loopback.
- medium, patch: `compile_cached` trả `Regex::clone()`, mà mỗi bản sao mang một pool cache rỗng (`regex-automata` `meta/regex.rs:1916`). Nay dùng `Arc<Regex>`.
- medium, patch: số đo một lượt mỗi phía, không ghi máy/tải/rustc, và quy phần dư cho nguyên nhân chưa đo. Đã đo lại 4 lượt sau và 3 lượt trước, xen kẽ, ghi đủ điều kiện, bỏ phần quy nguyên nhân.
- low, patch: dòng `:9338` viết "bảng một ô … không trần chọn tay", tự mâu thuẫn. Nay ghi rõ một ô, năm ứng viên xen kẽ vẫn biên dịch lại.
- low, patch: `:8889` và `:9338` trình một lượt đo như hai (trước + đối chứng). Đã viết lại theo các lượt đo mới.
- low, patch: AC3 không có `two_pass_replace` trong sổ. Nay có, kèm khoảng chênh chưa giải thích.
- low, patch: số RSS ở `:8921` đo trước lượt gác ``. Dòng sổ nay nói rõ chưa đo lại.
- low, patch: AC4 đếm 12, sổ ghi 20. Đối chứng lại cho 19 dòng (12 chuỗi).
- low, patch: đối chứng vô hiệu hoá regex thay vì gỡ. Làm lại bằng cách xoá hai khối.
- low, patch: hai JSDoc tiếng Việt mới thêm, sai luật comment. Agent xoá một cái, cái còn lại tôi xoá tay.
- low, patch: header "BA NHÓM VI PHẠM" mục 3 lỗi thời. Đã thêm câu về miễn trừ `console.*`.
- false: AC1 dựa vào tệp tạm `list109.mjs`. Sửa thì phải sửa chính plan này, nên bác theo luật triage.
- false: plan chưa đánh dấu task và chưa có Implementation Notes. Cũng là sửa plan; đã điền trong bước này.

## Verification

**Commands:**
- `cargo test --manifest-path src-tauri/Cargo.toml --test segment_normalize_boundary` và `--test segment_chapterpattern_boundary`: xanh.
- `cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml --test story_6_18_debt_probes -- --nocapture perf_probe_`: ghi số trước/sau.
- `npm run check:i18n`: xanh. Sau đó chạy cả bộ một lần (đã chạm `scripts/check-*`).
- `npm run check:debt-owner`: xanh.

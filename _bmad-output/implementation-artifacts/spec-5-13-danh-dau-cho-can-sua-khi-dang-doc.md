---
title: 'Story 5.13: Đánh dấu chỗ cần sửa khi đang đọc'
type: 'feature'
created: '2026-08-31'
status: 'done'
baseline_revision: '0fb1fe219de71fb26f4526127f78f169cb5e8897'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
warnings: ['oversized']
deferred: []
---

<intent-contract>

## Intent

**Problem:** Chế độ đọc hiện không có cách ghi lại một câu cần sửa mà vẫn giữ nhịp đọc, không có
danh sách marker theo Tác phẩm, và không giữ một neo `segment.id` để trở lại đúng vị trí đọc. Một
marker trỏ vào segment về hưu còn có thể trôi vị trí sau một lượt gộp/tách kế tiếp vì `ord` là số
thứ tự khả biến.

**Approach:** Lưu marker trong `project.db` bằng ID segment gốc và một `navigation_segment_id` sống;
cập nhật neo trong cùng transaction gộp/tách. Rust trả trạng thái marker và danh sách đã phân giải,
còn Reading Mode chỉ quản lý aim, lớp phủ danh sách và neo cuộn theo ID; `M` đánh dấu tại chỗ,
`Enter` cục bộ hoặc chọn danh sách mở đúng segment trong Workspace.

## Boundaries & Constraints

**Always:** Marker thuộc đúng một Tác phẩm vì sống trong `project.db`; `segment_id` gốc không đổi và
`navigation_segment_id` luôn là segment sống dùng để mở. Regroup phải cập nhật neo trong cùng
transaction tạo ID thay thế. Mọi lượt mở Workspace dùng `(chapter_id, navigation_segment_id)` và chỉ
đổi mode sau khi `openChapterById` thành công. Neo vị trí đọc là `segment.id`, không pixel. Rust sở
hữu quy tắc tồn tại/retired/sắp thứ tự; frontend kiểm hình dạng IPC đầy đủ và mọi chuỗi hiển thị qua
`t()`. Affordance ẩn bằng cơ chế loại khỏi hit/focus khi câu không hover/focus.

**Block If:** Cần đổi bất biến AD-3/AD-5; cần đặt marker ngoài `project.db`; cần tham chiếu vị trí bằng
`ord`/pixel thay vì ID; hoặc yêu cầu mở rộng thành toggle, gỡ từng dấu, bỏ hết dấu hay sửa hàng loạt —
các hành vi đó không có trong AC Story 5.13 và cần Ice chốt riêng.

**Never:** Không sửa `target_text`, trạng thái segment hay ranh giới từ bề mặt marker. Không đăng ký
`Enter` trần toàn ứng dụng. Không dùng `ord` làm danh tính/neo bền vững. Không xoá marker khi segment
về hưu. Không thêm dependency, box-shadow, màu/cỡ chữ viết cứng, hay công cụ batch của mockup.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Đánh dấu đang đọc | Câu sống đang hover/focus, chưa có marker | `M` tạo đúng một marker, câu hiện đã đánh dấu; mode, focus và neo đọc không đổi | Lỗi ghi được hiển thị, không đổi state giả |
| Bấm M lặp lại | Câu đã có marker | Marker vẫn đúng một hàng; không biến thành thao tác gỡ | No error expected |
| M không có aim | Không câu nào hover/focus | Không ghi, không chuyển mode | No-op có chủ ý |
| Mở trực tiếp | Câu sống đang aim, bấm `Enter` trên wrapper | Workspace mở đúng `chapter_id` và `segment_id` | Mở trượt ⇒ ở lại Reading, báo lỗi |
| Danh sách Tác phẩm | Marker ở nhiều Chương của cùng Work | Một danh sách sắp theo `(chapter.ord, anchor.ord, segment_id)` | Không trộn marker Work khác |
| Segment về hưu | Marker gốc bị gộp/tách | Marker còn nguyên `segment_id`, mang `is_retired = true`, ghi chú “câu này đã đổi”, neo sang ID thay thế sống | Cập nhật neo cùng transaction regroup |
| Regroup lặp | Neo của marker retired lại bị gộp/tách | Neo tiếp tục chuyển sang ID thay thế mới; mở vẫn đúng vị trí | Không rơi về `ord` cũ |
| Neo vị trí đọc | Rời rồi quay lại Reading trong cùng phiên | Câu neo theo `segment.id` trở lại vùng nhìn, không dùng `scrollTop` lưu trữ | Neo biến mất ⇒ rơi về câu sống đầu và ghi chẩn đoán |
| Chưa mở Work | Gọi đọc/ghi marker khi `open = None` | Không dữ liệu marker được đọc/ghi | `work.none_open` |

</intent-contract>

## Code Map

- `src-tauri/src/core/store/schema.rs:777,1452-1559` — `CHAPTER_POSITION_DDL` là khuôn bảng chi
  tiết theo ID; `PROJECT_MIGRATIONS` đang có 16 mục và đích 17. Thêm hằng `READING_MARK_DDL` và bước
  18; sửa mọi doc-comment đếm/đích tại chỗ kèm 🔵.
- `src-tauri/src/commands/segment.rs:988-1195` — `ReadingSegment`/`read_reading_run`; thêm
  `is_marked` từ cùng snapshot đọc, không IPC phụ. `:2618-2688` `write_regroup` là transaction duy
  nhất retire + insert; cập nhật mọi `reading_mark.navigation_segment_id` đang neo vào ID bị retire
  sang ID mới đầu tiên trước commit. `:3014` là khuôn wire `try_state`.
- `src-tauri/src/commands/chapter.rs` — đường gộp/tách Chương đã dời cả segment sống lẫn retired;
  danh sách marker phải lấy Chương từ anchor sống sau tổ chức lại, không giữ `chapter_id` thừa.
- `src-tauri/src/lib.rs:385` — `generate_handler!`; đăng ký wire marker mới tại đây.
- `src-tauri/tests/segment_contract.rs:617,6623` — cổng ladder migration và khuôn test vị trí theo
  ID; thêm hợp đồng marker, idempotence, cách ly Work, retired và regroup lặp.
- `src-tauri/tests/project_contract.rs:1112-1139` — `NON_ENTITY_DETAIL_TABLES` có 7 tên;
  `reading_mark` là thuộc tính theo segment, không thực thể tầng ba, nên thêm kèm lý do.
- `src-tauri/tests/pinned_contract.rs:213` và `src-tauri/tests/ipc_contract.rs:810-980` — cập nhật số
  migration/đích, đóng băng khoá snake_case và sự hiện diện wire.
- `src/config/reading.ts` — adapter runtime-guarded hiện có; thêm `markReadingSegment` và
  `listReadingMarks`, mỗi hàm đúng một `invoke`, một `try/catch`, không ném.
- `src/modes/readingState.ts:95-270` — state nội dung và overlay TOC là khuôn cho aim/list/error/
  cursor/focus restore. Neo đọc theo ID phải qua `resetReading` đúng phạm vi và sống qua một lượt
  đổi mode có chủ ý.
- `src/modes/ReadingMode.vue:76-85,365-476` — wrapper câu, root focus và overlay TOC. Wrapper là một
  tab stop; local `keydown Enter` dispatch command keyless; affordance chỉ hiện `:hover`/
  `:focus-within`; marker list dùng cùng kỷ luật focus của TOC.
- `src/panels/editorPanelState.ts:1766` — `openChapterById(chapterId, segmentId?)` đã flush, nạp và
  đặt caret đúng segment; tái dùng nguyên, không dựng đường điều hướng thứ hai.
- `src/commands/index.ts:1362-1505` · `src/main.ts:250-470` — đăng ký `reading.mark_aimed` với bare
  `M`; lệnh mở aimed/list/key navigation không có phím global và được tiêm qua deps.
- `src/commands/keys.ts:399-488` — bare `M` đã bị chặn trong typing zone; duplicate chord ném. Bare
  `M` hiện chưa có chủ, còn `Mod+M`/`Mod+Alt+M` đã có chủ khác.
- `src/i18n/vi.json:96-110,289-330` và `src/tokens/tokens.json` — thêm nhãn command/status/aria/note;
  dùng token `surface-tm`, `tm-text`, `surface-accent`, không hardcode màu.
- `e2e/specs/story-5-12-reading-frontier.e2e.mjs` — khuôn fixture Chế độ đọc thật; story mới phải
  dùng `realClick` và kiểm computed visibility, mode, segment đích, marker retired.
- `_bmad-output/implementation-artifacts/deferred-work.md:5039-5067,8759-8766` — đóng tại chỗ món
  nợ vị trí đọc bằng neo `segment.id`; FR119 đang có chủ duy nhất Story 5.13.

## Tasks & Acceptance

**Execution:**

1. `src-tauri/src/core/store/schema.rs` + ba contract migration/detail-table — thêm migration 18
   `reading_mark(segment_id PRIMARY KEY, navigation_segment_id NOT NULL, marked_at NOT NULL)`, cập
   nhật đích/số mục/fixture newer-than-app và giải thích vì sao không FK/không `chapter_id`.
2. `src-tauri/src/commands/segment.rs` + `src-tauri/src/lib.rs` — thêm kiểu `ReadingMark`, hàm thuần
   mark-idempotent/list và wire mỏng; `read_reading_run` left-join trạng thái marker trong cùng
   snapshot; list join segment gốc + anchor sống + Chapter của anchor và sắp thứ tự ổn định.
3. `src-tauri/src/commands/segment.rs::write_regroup` — sau khi có các ID mới, rebase mọi marker có
   `navigation_segment_id` thuộc tập retire sang fresh ID đầu tiên trong cùng transaction; giữ
   `segment_id` gốc. Bổ sung đối chứng regroup lặp và rollback.
4. `src-tauri/tests/segment_contract.rs` + `src-tauri/tests/ipc_contract.rs` — phủ toàn bộ I/O Matrix,
   bao gồm Work isolation, idempotence, retired note data, regroup hai lượt, tổ chức lại Chương,
   unknown/no-work và khoá wire/registration.
5. `src/config/reading.ts` — mở rộng types/guards/adapters marker; payload sai phải thành lỗi phân
   biệt được, không danh sách rỗng giả.
6. `src/modes/readingState.ts` — thêm aimed segment, mark idempotent, overlay danh sách, điều hướng
   exact segment và neo đọc theo ID. Dọn mọi state theo Work trong reset; giữ neo vừa chọn qua lượt
   Reading → Workspace → Reading; reply cũ không được ghi đè Work mới.
7. `src/modes/ReadingMode.vue` — biến mỗi câu thành wrapper focusable, affordance ẩn hoàn toàn khi
   không hover/focus, chỉ báo marked thường trực, local Enter dispatch, danh sách marker và note
   retired; dọn aim khi deactivated và khôi phục neo sau render.
8. `src/commands/index.ts` + `src/main.ts` + `src/i18n/vi.json` — nối command/deps/i18n; bare `M` chỉ
   mark aimed, không mở list; Enter chỉ cục bộ. Nút mở list và các nút overlay dùng đúng một
   `dispatch('<id>')`.
9. `tests/**` + `e2e/specs/story-5-13-reading-marks.e2e.mjs` — test adapter/state/DOM và bề mặt
   WebKit: hover/focus visibility, M không đổi mode/vị trí, Enter exact, list nhiều Chương, retired
   qua regroup lặp và neo cuộn trở lại.
10. `_bmad-output/implementation-artifacts/deferred-work.md` + story/sprint tracking — đóng FR119 và
    món nợ vị trí đọc bằng chữ, chỉ sau khi có bằng chứng; không xoá lịch sử.

**Acceptance Criteria:**

- Given chuột hoặc tiêu điểm bàn phím chạm một câu, when Reading Mode vẽ lại, then affordance đánh
  dấu nhìn thấy và thao tác được; Given không câu nào được chạm, when Reading Mode vẽ trạng thái
  nghỉ, then affordance không nhìn thấy, không hit được và không là tab stop.
- Given một câu đang aim, when bấm `M`, then câu có marker bền vững và phiên đọc tiếp tục tại cùng
  mode, focus và neo; bấm lặp không tạo hàng thứ hai và không gỡ marker.
- Given một câu đang aim, when bấm `Enter` trên wrapper, then Workspace mở đúng segment; lỗi mở giữ
  nguyên Reading và hiện chẩn đoán.
- Given marker nằm ở nhiều Chương, when mở danh sách, then chỉ marker của Work đang mở xuất hiện
  theo thứ tự Chương/vị trí ổn định; chọn mục mở đúng segment neo.
- Given marker gốc đã về hưu qua một hoặc nhiều lượt gộp/tách, when hiển thị/chọn mục, then marker
  vẫn còn với ghi chú “câu này đã đổi” và Workspace mở ID neo sống đúng vị trí trong Chương.
- Given người dùng rời Reading rồi quay lại trong cùng phiên, when nội dung được kích hoạt, then câu
  neo bởi `segment.id` trở lại vùng nhìn dù cỡ chữ thay đổi; không giá trị pixel nào được lưu.
- Given cơ chế marker/navigation, when đối chiếu FR11, then không đường nào sửa nội dung, trạng thái
  hay ranh giới segment.

## Spec Change Log

## Review Triage Log

### 2026-08-31 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 20: (high 8, medium 7, low 5)
- defer: 0
- reject: 3: (high 0, medium 0, low 3)
- addressed_findings:
  - `[high]` `[patch]` Aim hoặc lượt cập nhật `ReadingRun` không còn kích hoạt `scrollIntoView`; neo chỉ khôi phục ở lượt nạp/kích hoạt.
  - `[high]` `[patch]` Nút marker không còn bị thay node sau ghi, nên click không làm tiêu điểm rơi khỏi câu.
  - `[medium]` `[patch]` Affordance marker rời khỏi luồng inline và trạng thái marked dùng nền + node ổn định, không tái dàn dòng đọc.
  - `[high]` `[patch]` Lỗi ghi marker được đưa cạnh toolbar thay vì nằm sau toàn bộ Tác phẩm dài.
  - `[medium]` `[patch]` Hai overlay Reading giữ Tab bên trong chuỗi điều khiển của dialog.
  - `[medium]` `[patch]` Dialog marker có accessible name qua `aria-labelledby`.
  - `[medium]` `[patch]` Mở marker list và mục lục nay loại trừ nhau, không còn hai `aria-modal` đồng thời.
  - `[low]` `[patch]` Marker list hiện trạng thái đang nạp đã nội địa hoá.
  - `[low]` `[patch]` Các nút trước/kế/mở bị vô hiệu hoá khi danh sách rỗng.
  - `[high]` `[patch]` Lượt đóng overlay do đổi mode không còn giật focus về Reading đã ẩn.
  - `[low]` `[patch]` Neo mất được thay bằng ID sống đầu tiên, không lặp cảnh báo fallback ở mỗi activation.
  - `[high]` `[patch]` Danh sách marker đối chiếu số hàng lưu với số hàng join được; neo thiếu/retired thành lỗi thay vì biến mất im lặng.
  - `[low]` `[patch]` Marker có bản dịch rỗng rơi về nguyên văn thay vì hiện một hàng không phân biệt được.
  - `[medium]` `[patch]` Chụp viewport ở cuối trang rơi về segment sống cuối thay vì giữ neo cũ.
  - `[high]` `[patch]` Adapter từ chối response marker hợp kiểu nhưng sai `segment_id`.
  - `[high]` `[patch]` Thêm đối chứng lỗi ghi: không tô state thành công giả và giữ lỗi cho UI.
  - `[high]` `[patch]` Thêm hai race test bỏ response marker/list cũ sau reset Work.
  - `[medium]` `[patch]` E2E WebKit chụp neo từ viewport thật khi rời Reading mà không mở marker.
  - `[medium]` `[patch]` Unit + E2E canh con trỏ marker trước/kế, biên và mục được Open.
  - `[low]` `[patch]` Sửa lời khai đầu tệp e2e: `mouseenter` sản phẩm được đo, pseudo-class `:hover` không bị khai đạt sai.

## Design Notes

Ice chọn phương án B ngày 2026-08-31: marker giữ danh tính gốc và neo điều hướng sống. Hai ID không
trùng vai: `segment_id` trả lời “câu nào đã được đánh dấu”, `navigation_segment_id` trả lời “hôm nay
mở ở đâu”. Nhiều marker được phép cùng neo vào một segment mới sau merge.

`epics.md:4354-4356` là nguồn chính thức cho `M`: đánh dấu câu đang aim. Dòng rút gọn
`EXPERIENCE.md:511` gọi `M` là đường mở danh sách xung đột với AC và với keymap không cho hai command
chung chord; vì vậy `M` chỉ đánh dấu, danh sách mở bằng nút/command keyless. Mockup có gỡ/batch nhưng
AC không có; story này dùng INSERT idempotent và không tự mở rộng thành toggle.

## Verification

**Commands:**

- `npm run check:deps && npm run check:tokens && npm run check:i18n && npm run check:commands && npm run check:layout && npm run check:panel-refs && npm run check:dict && npm run check:dict-manifest && npm run check:gates && npm run check:debt-owner` — mọi cổng tĩnh xanh.
- `npm test -- --run` — vitest frontend xanh, gồm adapter/state/DOM marker.
- `npm run build` — TypeScript và bundle xanh; tạo `dist/` trước Rust.
- `cargo test --locked --manifest-path src-tauri/Cargo.toml` — migration, marker, regroup và IPC xanh.
- `npm run test:e2e -- --spec e2e/specs/story-5-13-reading-marks.e2e.mjs` — bề mặt WebKit của Story
  5.13 xanh khi môi trường GUI khả dụng; nếu không chạy được phải ghi nợ có chủ, không suy luận đạt.

## Auto Run Result

### Summary

Story 5.13 đã hoàn tất theo phương án B Ice chọn: marker giữ `segment_id` gốc và
`navigation_segment_id` sống, tồn tại qua regroup lặp, liệt kê theo đúng Tác phẩm và mở đúng câu
trong Workspace. Reading Mode có aim bằng hover/focus, `M` idempotent, `Enter` cục bộ, marker list
điều hướng được và neo đọc theo ID không lưu pixel.

### Files changed

- `src-tauri/src/core/store/schema.rs` — migration project 18 và bảng `reading_mark`.
- `src-tauri/src/commands/segment.rs` — đọc/ghi/list marker, snapshot marked, rebase transaction và chốt chống marker biến mất im lặng.
- `src-tauri/src/lib.rs` — đăng ký hai IPC marker.
- `src-tauri/tests/{segment,project,ipc,pinned}_contract.rs` — hợp đồng migration, wire, cách ly Work, regroup/rollback và neo hỏng.
- `src/config/reading.ts` — kiểu/guard IPC marker, gồm bất biến response đúng ID yêu cầu.
- `src/modes/readingState.ts` — aim, mark, list, exact navigation, race guards và neo ID.
- `src/modes/ReadingMode.vue` — wrapper câu, affordance ổn định hình học, dialog marker, focus và khôi phục viewport.
- `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json` — command wiring và chuỗi giao diện.
- `src/panels/editorPanelState.ts` — phát lại đặt caret DOM sau khi Workspace đã active.
- `tests/frontend/readingMarks.test.ts` cùng các test Reading hiện có — adapter/state/DOM/race/focus regression.
- `e2e/specs/story-5-13-reading-marks.e2e.mjs` — bàn đo WebKit end-to-end.
- `_bmad-output/implementation-artifacts/{deferred-work.md,sprint-status.yaml}` — đóng nợ FR119/neo đọc và trạng thái story.

### Review findings

- Applied: 20 patches (high 8, medium 7, low 5), chi tiết trong Review Triage Log.
- Deferred: 0.
- Rejected: 3 low — hai suy đoán hiệu năng (`O(n)` list lúc mark và thiếu index neo) không có phép đo; các cách hiểu substring/toggle bỏ qua định danh Story 5.13 và nguồn AC nên không phải intent gap.
- Follow-up review: `true`; patched counts high 8 / medium 7 / low 5, score `3 × 7 + 5 = 26`, đồng thời có finding high.

### Verification performed

- Full static gates: xanh sau review.
- `npm test -- --run`: 56 tệp, 766 ca xanh.
- `npm run build`: TypeScript + bundle xanh trước Rust.
- `cargo test --locked --manifest-path src-tauri/Cargo.toml`: toàn suite xanh; `segment_contract` 153/153.
- `npm run test:e2e -- --spec e2e/specs/story-5-13-reading-marks.e2e.mjs`: WebKit 605.1.15, 1/1 xanh trong 1m28s.

### Residual risks

WDIO nhúng không tạo pseudo-class `:hover` bằng pointer Actions dù node đích đúng; ca e2e đo đường sản
phẩm `mouseenter` → aim → Vue/CSS và đường focus thật. CSS `:hover` vẫn tồn tại như đường bổ sung nhưng
không được khai là đã đo trực tiếp. Hai cảnh báo cleanup/window-state của `@wdio/tauri-service` xuất hiện
sau/trong lượt chạy nhưng runner trả exit 0 và spec PASS.

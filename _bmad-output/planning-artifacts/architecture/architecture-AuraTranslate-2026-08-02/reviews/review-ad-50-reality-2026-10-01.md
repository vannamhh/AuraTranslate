# Review thực tại AD-50 (HEAD 1464b2d, 2026-10-01)

Phạm vi: `ARCHITECTURE-SPINE.md` §AD-50 (dòng 796–) và 🔵 trong §AD-47 (dòng 702, 708, 720, 768) đối chiếu `src-tauri/src`, `src/`, `tests/`, `e2e/`, `src-tauri/tests/`.

## Phán quyết
AD-50 đứng được và danh mục người ghi đầy đủ, nhưng có 1 lỗ thật ở tách (split) và 1 câu sai của AD-47 ④ còn nguyên; hai chỗ còn lại là bổ sung nhỏ.

## 1. Danh mục người ghi `target_text` / `translation_origin`

Grep `INTO segment` / `UPDATE segment` / `target_text =` / `translation_origin =` toàn `src-tauri/src`: không sót người ghi nào. Đối chiếu:

| Chỗ | Ghi gì | Trong bảng cl.2? |
|---|---|---|
| `commands/segment.rs:145` `insert_segments` | hàng mới, origin `''` (?6), không target | có (hàng "còn lại"; baseline `''`/`''` = DEFAULT) |
| `segment.rs:194` `insert_bilingual_segments` | target + origin; origin `''` khi target rỗng (`:213-214`) | có; baseline phải ghi cùng giá trị `origin` đã tính, KHÔNG hằng `bilingual_import` |
| `segment.rs:862` `restore_segment_version` | target + status, không origin | có |
| `segment.rs:1972` `save_segment_targets` (flush) | target + updated_at | có (không chạm mốc) |
| `segment.rs:2197` `promote_ai_translation` | target + origin `other` + status draft | có |
| `segment.rs:2567` `confirm_segment` | status + origin | có (không chạm cột mốc) |
| `segment.rs:2945` (hạ confirmed→draft khi flush đổi chữ) | chỉ status | không phải người ghi mốc/origin; ổn |
| `segment.rs:3241` `write_regroup` INSERT (+ `regroup.rs:171` `merge`, `:296` `split_at`) | target + origin | có, xem finding 1 |
| `chapter.rs:872,1063` | chỉ `chapter_id`, `ord` | ngoài phạm vi (đúng AGENTS.md) |
| `schema.rs:1798` (bước 11), `:1641` | backfill cũ | không đụng |

`insert_segments` và `write_regroup` khai tường minh cột; sau bước 28 cả hai INSERT phải khai thêm hai cột mốc (bài học "khai đủ cột" ở `segment.rs:3176`), kể cả hàng import vì DEFAULT `''` chỉ đúng ngẫu nhiên.

## 2. Tách (split) và clause 2/3 — LỖ

- `regroup.rs:296-303`: mảnh đầu giữ `part.translation_origin` (cột `translation_origin` đang lưu, đọc ở `segment.rs:3112`), mảnh sau `target ''` + origin `''` (`:313-318`).
- Mảnh đầu ghi baseline = (target hiện tại, origin lưu) theo cl.2 "mọi hàng còn lại". Nhưng với segment người dùng đã viết lại (chưa ký), origin lưu là origin CŨ (vd `other`/`bilingual_import`), baseline text lại là chữ MỚI của người dùng ⇒ ký không sửa ⇒ nhãn *người khác dịch* cho chữ của chính họ. Đúng lỗ (3) mà AD-50 tuyên bố chặn, qua cửa tách.
- Cl.3 chỉ nói "giá trị mỗi mảnh ở AD-47 ④ là kết quả hàm"; không nói tách. Hiện `merge`/`split_at` đọc cột lưu (`regroup.rs:152`), không qua hàm.
- Sửa: cl.3/cl.6 ghi rõ cả `merge` lẫn mảnh đầu của `split_at` gọi hàm phân xử trên (target, baseline_target, baseline_origin) của segment gốc để lấy giá trị mảnh; `SegmentPart` (14 chỗ dựng, chỉ ở `segment.rs` và `tests/segment_contract.rs`) phải mang hai trường mốc hoặc một origin đã phân xử sẵn (giữ `core/segment` thuần).
- Mảnh sau: baseline (`''`,`''`) khớp cl.2/3 (target rỗng ⇒ `''`).

## 3. AD-47 ④ có câu SAI so với mã

AD-47 ④: "Tách là ca tầm thường… một nguồn ⇒ mọi mảnh cùng giá trị". Mã: mảnh sau mang `''` chứ không giá trị gốc (`regroup.rs:308-318`, có comment "SUY DẪN" nói đúng điều ngược lại). AD-50 cl.6 giữ ④ "chỉ được khoá đầu vào" nên câu sai sống tiếp. Sửa: thêm 🔵 vào ④: mảnh đầu giữ giá trị phân xử, mảnh không có bản dịch mang `''`.

Phụ (thấp): gộp một mảnh target rỗng (origin `''`) với mảnh có chữ vẫn ra `other` (đã là cái mất ghi ở ④); khi cl.3 trả `''` cho mảnh rỗng nó thành hệ quả máy móc. Cân nhắc bỏ mảnh target rỗng khỏi phép "đồng thuận" (`join_targets` đã lọc chúng, `regroup.rs:131`).

## 4. Cl.3 vs `confirm_segment` (`segment.rs:2530-2570`)

Hiện tại: `trim().nfc()` hai vế khác nhau **hoặc** cột `translation_origin` rỗng ⇒ `SelfTranslated`; còn lại `PairOrigin::from_stored(origin_at_load)` (webview), không hợp lệ ⇒ `UnknownOrigin`.

Cl.3 là tổng quát hoá trung thực, với các khác biệt cần ghi:
1. Điều kiện rỗng chuyển từ cột LIVE `translation_origin` (`:2558`) sang `baseline_translation_origin`. Hai cột tách nhau sau lượt ký đầu (confirm ghi `translation_origin`, không ghi baseline). Hệ quả đúng ý (ký→sửa→ký lại về xuất xứ lúc đặt mốc) nhưng là thay đổi hành vi có chủ đích.
2. Khôi phục FR101 (cl.2: chỉ ghi baseline text): sau khi khôi phục chữ ĐÃ KÝ cũ rồi ký không sửa, kết quả là `baseline_translation_origin` (có thể `other` từ lượt AI/import cũ) chứ KHÔNG `translation_origin` hiện tại (`self`). Hiện nay mốc webview = cột live, nên cho `self`. AD-47 ⑤ 🔵 vẫn nói "giữ nguyên xuất xứ hiện tại" ⇒ sai so với cl.2. Phải sửa lời (⑤ + cl.2) hoặc cho restore đặt luôn baseline origin = `translation_origin` hiện tại (đề nghị: ghi rõ đây là hành vi mới + test).
3. `UnknownOrigin` (`segment.rs:2327`, `:2559-2563`): cl.3 không nhắc. Giá trị lạ trên đĩa trong `baseline_translation_origin` vẫn phải bị từ chối (từ chối là nguồn hỏng đĩa chứ không còn lỗi webview); nêu rõ giữ `from_stored` và mã lỗi, nếu không `.unwrap_or(self)` lọt vào.
4. Mệnh đề "`target_text` rỗng ⇒ `''`" thừa với confirm (đã `NothingToConfirm` ở `:2509`), chỉ có nghĩa cho gộp/tách.
5. Hàm đọc "trong cùng giao dịch": confirm đã đọc trong `tx` (`:2470`), regroup đọc trong `load_segment_for_write` (`:3110`); cả hai chỉ cần thêm cột vào SELECT.

## 5. Cl.4: bán kính nổ khi bỏ `textAtLoad`/`originAtLoad`

Rust: `commands/segment.rs` (đọc/đối chiếu `:2440-2563`, wire `:3783-3794`, doc `:297`, `:2130`, `:2327`, `:2389`, `:3767-3773`, `:3987`).
Rust tests: `tests/ipc_contract.rs:2196-2216` (ghim chữ ký wire, test `the_confirm_segment_wire_keeps_its_origin_at_load_parameter` phải đổi/xoá); `tests/segment_contract.rs:5825-5997, 10674` (5 chỗ đọc/gửi mốc); tổng 72 lời gọi `confirm_segment(` trong `segment_contract.rs`, `tm_contract.rs`, `ai_translate_contract.rs`, `segment_role_contract.rs`.
Webview: `src/config/segment.ts:145,821-851,1259` (hàm `confirmSegment(id, textAtLoad, originAtLoad)`); `src/panels/editorPanelState.ts:1295-1316` (đọc `loaded.target_text`/`loaded.translation_origin` từ `segments`, nhánh `'no-caret'` cho "không tìm thấy trong ảnh chụp" chỉ còn lý do là id không có).
Vitest: `tests/frontend/editorConfirmSegment.test.ts:39-51` (`confirmMarks`/`confirmOrigins`, có thể là tài sản khẳng định mốc gửi đi — cần đổi sang khẳng định "không gửi mốc"), `tests/frontend/gridPanelRowErrorPriority.test.ts:28`; mock `confirmSegment` ở `editorCaretAtCellStart.test.ts`, `editorTypingZone.test.ts`.
E2E: `e2e/specs/grid-row-error-label.e2e.mjs:122`, `segment-history-restore.e2e.mjs:53-86,179-180`, `segment-merge-split.e2e.mjs:12,270,374` (invoke trực tiếp với `textAtLoad:''`; invoke thừa khoá vẫn có thể bị Tauri từ chối "unknown key"? Tauri bỏ qua khoá thừa nhưng kiểm lại).

Phụ thuộc khác của webview vào ảnh chụp lúc nạp làm mốc:
- `editorPanelState.ts:307` (doc `promoteAiTranslationToEditor`) nói thẳng "mirror để mốc không trỏ vào văn bản cũ" ⇒ sau AD-50 chỉ còn là hiển thị, comment phải gỡ.
- `replaceEditorSegment` (`editorPanelState.ts:268-278`) vẫn cần cho lưới hiển thị; `segmentHistoryState.ts:351` gọi nó. Không cái nào còn là mốc FR117. Không phụ thuộc mốc nào khác thấy trong `src/` (grep `baseline`/`loadedText` rỗng).
- `ChapterSegment.translation_origin` (`segment.rs:299`, `:947`) vẫn đi ra webview để hiển thị; AD-50 cl.4 "không nhận mốc" đúng vì hai cột mốc không nằm trong SELECT/struct này, nhưng cần ghi rõ `translation_origin` live KHÔNG là mốc để người sau không nối lại.

## 6. Cl.5: bước 28

`PROJECT_MIGRATIONS` kết thúc tại `schema.rs:2202-2205` `to_version: 27, sql: TM_UNIT_DDL` ⇒ bước 28 là kế tiếp, đúng.
Tiền lệ `ADD COLUMN … NOT NULL DEFAULT ''` + `UPDATE` trong cùng hằng: bước 9 (`schema.rs:1600-1641`, DDL+DML) và bước 11 `SEGMENT_TRANSLATION_ORIGIN_DDL` (`:1796-1799`); `migrate` chạy `execute_batch` trong một giao dịch (`:2635`). Không có ràng buộc ngăn. `UPDATE` đặt `baseline_translation_origin = translation_origin` đồng loạt khả thi.
Nhắc: ghi hai cột dưới dạng một hằng `concat!` (ADD COLUMN ×2 + UPDATE), hằng mới `..._DDL`, mỗi bước một hằng. Test fixture "newer than app" cần nâng (`tests/segment_contract.rs:2105-2127`: mảng `[Migration; 27]` → 28, bước giả `to_version: 29`) — cùng khuôn đã làm cho bước 27; có thể có nhiều test đếm bước, grep `to_version`/`[Migration;` trước khi thi hành.

## 7. Mệnh đề trong AD-50/AD-47 🔵 sai so với mã

- AD-47 ④ "một nguồn ⇒ mọi mảnh cùng giá trị": sai (mục 3).
- AD-47 ⑤ 🔵 + `⑤` gốc "giữ nguyên xuất xứ hiện tại": sai sau AD-50 (mục 4.2).
- AD-50 Prevents (1) "mỗi lượt nạp Chương đặt lại mốc": đúng (`editorPanelState.ts:1315-1316`).
- AD-47 ② 🔵 "nhập song ngữ, đưa đề xuất AI sang Editor và gộp/tách đã cài": đúng (`segment.rs:194`, `:2197`, `:3241`).
- AD-47 ③ hàng Review Mode FR94 / TM FR58: không có người ghi nào trong mã (grep sạch) — bảng đóng đúng, chưa cài.
- AD-50 cl.5 ⚠️ "câu đã ký tôi dịch rồi sửa ngược … ra tôi dịch": nhất quán với dữ liệu bước 28 (baseline = target hiện tại).

## Việc cần làm trước khi ký

1. (Cao) Cl.3: bắt `merge` và mảnh đầu `split_at` dùng hàm phân xử trên mốc của segment gốc, không đọc `translation_origin` lưu.
2. (Trung) AD-47 ④: 🔵 sửa câu "mọi mảnh cùng giá trị" cho tách.
3. (Trung) AD-47 ⑤ + cl.2: nêu hành vi mới của khôi phục (kết quả = origin lúc đặt mốc), hoặc cho restore đặt luôn baseline origin.
4. (Thấp) Cl.3 nêu rõ `UnknownOrigin` giữ cho `baseline_translation_origin`; cl.4 liệt kê `ipc_contract.rs:2196-2216` và e2e là bề mặt phải đổi.

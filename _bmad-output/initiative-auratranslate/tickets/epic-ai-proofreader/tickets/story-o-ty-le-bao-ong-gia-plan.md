---
title: 'Đo tỷ lệ báo động giả'
type: 'feature'
ticket: '8'
created: '2026-10-12'
status: 'built'
baseline_revision: 'd3fb84934c5ae0cb7082c5a28c05c26d236e2373'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR81 cần một số cho biết proofreader báo sai bao nhiêu trên bản dịch thật; 9.6 đã ghi `proof_ignore` và `proof_scan` nhưng chưa gì đọc chúng.

**Approach:** Công cụ chỉ đọc tính tỷ lệ từ `project.db`, kèm khoảng tin cậy và kết luận theo ngưỡng; số thật do lượt dùng của Ice tạo ra, không phải agent.

**Decisions (Ice, 2026-10-12):** (1) Một ngưỡng chung, đỗ khi ≤ 20%; không thêm cột vào `proof_scan`, không migration; tử số vẫn tách theo `proof_ignore.kind`. (2) Mẫu tối thiểu 30 phát hiện; dưới đó ghi tỷ lệ kèm khoảng Wilson 95% và kết luận "chưa đủ mẫu". (3) Quy trình: quét trọn một Chương thật đúng một lần; xử lý mọi phát hiện bằng chấp nhận hoặc "không phải lỗi"; không dùng "bỏ qua" của 9.4, không bỏ ghi nhớ, không quét lại. (4) Ngưỡng nới từ Tricorder (ICSE 2015, ~10%) vì đây là phán đoán nghĩa. (5) Mục nợ `Chủ: Story 9.8` về loại trong chữ ký ghi nhớ phải được đo hoặc giao lại có lý do. (6) Kết luận đỗ/trượt theo mẫu số `reported` (điều người dùng thấy); `reported + filtered` chỉ in để tham khảo. (7) Công cụ là script Node `node:sqlite` chỉ đọc (Node 22 có sẵn, chấp nhận cảnh báo experimental). (8) Số đo ghi vào `epic-ai-proofreader/measurements-bao-dong-gia.md`, mỗi lần đo một khối, cộng dồn. (9) Cửa sổ giữ cận dưới `scanned_at`; quy trình thêm "chờ lượt quét xong rồi mới xử lý"; công cụ đếm hàng `proof_ignore` nằm sau `scanned_at` của lượt quét trước (hoặc từ đầu nếu không có) và trước `scanned_at` của lượt này, in cảnh báo, và không kết luận "đỗ" khi số đó > 0. (10) Chương do Ice truyền `--chapter <id>`; công cụ kiểm Chương có trong DB rồi in `ord`, `title` cùng `work.name`; vẫn là lời khai vì `proof_scan` không ghi Chương.

## Boundaries & Constraints

**Always:** mở `project.db` chỉ đọc (lượt ghi qua kết nối của công cụ bị từ chối); tỷ lệ = hàng `proof_ignore` tạo trong cửa sổ của lượt quét / `proof_scan.reported`; báo cáo nêu git HEAD (sạch/bẩn) của repo chứa script, Tác phẩm, Chương, `scan.id`, n, tử số theo loại, tỷ lệ, Wilson, kết luận; không phụ thuộc mới.

**Never:** hiển thị tỷ lệ trong ứng dụng; migration; cổng `check:*`; agent tự điền số đo hay đánh dấu đỗ; chạy quét AI thật; sửa prompt (chỉ đề xuất khi vượt ngưỡng).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Đủ mẫu | `reported`=40, 6 hàng ignore | 15,0%, Wilson, "đỗ"; 12 hàng ⇒ "vượt" kèm ba đề xuất FR81 | Không lỗi |
| Thiếu mẫu | `reported`=12, 2 hàng | 16,7% kèm Wilson, "chưa đủ mẫu" | Không lỗi |
| `filtered` > 0 | Chương đã có ghi nhớ | kết luận theo `reported`, in thêm cách tính `reported + filtered` | Cảnh báo vi phạm quy trình |
| Nhiều lượt quét chương | `proof_scan` > 1 hàng | liệt kê, đòi `--scan <id>` | Thoát khác 0 |
| Không có `proof_scan` | DB rỗng/sai | không in tỷ lệ | Thoát khác 0, không phải 0% |
| Bấm giữa lượt quét | hàng ignore trước `scanned_at`, sau lượt trước | in số hàng đó, không kết luận "đỗ" | Cảnh báo vi phạm quy trình |
| Tử số > mẫu số | hàng trong cửa sổ > `reported` | không in tỷ lệ hay Wilson | Thoát khác 0 |
| Lượt quét không phải Chương | `scope` ≠ `chapter` | in `scope`, không kết luận "đỗ" | Cảnh báo |
| `--chapter` sai | id không có trong `chapter`, hoặc thiếu | không in tỷ lệ | Thoát khác 0 |

</frozen-after-approval>

## Code Map

- `core/store/schema.rs` (`PROOFREAD_MEMORY_DDL`): `proof_ignore.created_at`, `proof_scan.scanned_at`, cùng `strftime` UTC của SQLite, so chuỗi được.
- `commands/proofread.rs`: `scanned_at` ghi SAU `Done`; `INSERT OR IGNORE` gộp cùng chữ ký thành một hàng. `src/proofreadState.ts`: `reported` đã qua bộ lọc, tổng model báo = `reported + filtered`.
- `scripts/lib/floor-judge.mjs`: kiểu mô-đun thuần; vitest gom ở `tests/frontend/`; vitest không gói được `node:sqlite`, nạp qua `createRequire`.
- Định dạng thời điểm thật: `strftime('%Y-%m-%dT%H:%M:%fZ','now')` ở cả hai bảng (`commands/proofread.rs:350`, `:421`); fixture dùng đúng dạng này.
- `src/proofreadState.ts:129` `applyEvent` đổ phát hiện theo lô trong lúc quét và `ignoreProofreadFinding` không chặn `scanning` ⇒ hàng ignore có thể trước `scanned_at` (Decision 9). Không sửa mã ứng dụng.
- `work.name` (`schema.rs:849`), `chapter.ord`/`title` (`schema.rs:1428`).
- Đo `node:sqlite` 3.51.2 (Node 22.22): `readOnly: true` đọc được DB WAL, lượt ghi ném "attempt to write a readonly database"; mở ghi rồi chỉ đọc không đổi byte ⇒ so byte không canh được `readOnly`.
- Bản dựng lần 1 (bị hoàn tác) nằm ở scratchpad phiên; không dựa vào nó.

## Tasks & Acceptance

**Execution:**
- [x] `scripts/lib/falseAlarm.mjs` -- cửa sổ ignore (từ `scanned_at` tới lượt quét kế) và khoảng trước nó (Decision 9), Wilson, kết luận; ba đề xuất đúng lời FR81: chỉnh prompt, hạ độ nhạy, tách riêng hai loại phát hiện; `openReadOnly(path)` dùng chung cho công cụ và test -- tách số học khỏi DB
- [x] `scripts/measure-proofread-false-alarm.mjs` -- `<project.db> --chapter <id> [--scan <id>]` theo thứ tự bất kỳ; mở qua `openReadOnly`; git chạy với cwd là repo của script; in báo cáo, chẩn đoán phụ: số cặp (cụm, ngữ cảnh) có hai loại -- không là cổng, không chú thích đầu tệp
- [x] `tests/frontend/falseAlarmRate.test.ts` -- mỗi hàng ma trận một ca; DB tạm có `proof_scan`, `proof_ignore`, `work`, `chapter`; lỗi khẳng định trên `stderr` và mã thoát; ca ghi qua `openReadOnly` phải ném
- [x] `epic-ai-proofreader/measurements-bao-dong-gia.md` -- khuôn bản ghi (Decision 8), quy trình gồm bước chờ quét xong (Decision 9) và lệnh có `--chapter`
- [x] `deferred-work.md` -- mục nợ `Chủ: Epic 9` cho lượt dùng thật; nối `→ 🟡 ... Chủ: Epic 9` vào mục đổi loại (một lượt quét không làm xảy ra đổi loại; công cụ chỉ cho cận dưới)
- [ ] NGƯỜI: Ice quét một Chương, xử lý hết, chạy công cụ, ghi bản ghi

**Acceptance Criteria:**
- Given DB dựng sẵn 40 phát hiện, 6 hàng ignore, when chạy công cụ, then 15,0%, Wilson, "đỗ", DB không đổi byte.
- Given n < 30, when chạy, then "chưa đủ mẫu" dù tỷ lệ thấp.
- Given `proof_scan` rỗng, when chạy, then thoát khác 0, không in tỷ lệ.
- Given một hàng ignore trước `scanned_at`, when chạy, then cảnh báo nêu số hàng và kết luận không phải "đỗ".
- Given số hàng trong cửa sổ > `reported`, when chạy, then thoát khác 0, không in "NaN".
- Given `--chapter` trỏ Chương có thật, when chạy, then báo cáo in `work.name`, `ord`, `title`.
- Given chưa có bản ghi của Ice, then story dừng ở `built`, không đánh đỗ bằng suy ra.

## Implementation Notes

- Số thật chưa có: story dừng ở `built` tới khi Ice ghi khối đo đầu tiên (nợ `Chủ: Epic 9`); agent không điền, không suy ra đỗ.
- Phát hiện đổ theo lô giữa lúc quét và "không phải lỗi" không chặn `scanning` (`proofreadState.ts:129`), nên Ice chọn quy trình chờ quét xong và công cụ đếm hàng trước `scanned_at`, không kết luận đỗ khi có (Decision 9).
- `clearResult()` chạy đầu mỗi lượt quét (`proofreadState.ts:173`): mọi hàng ignore sau `scanned_at` và trước lượt kế đến từ chính lượt đó.
- `node:sqlite` 3.51.2: `readOnly` đọc được DB WAL và từ chối ghi; so byte không canh được chỉ-đọc nên ca canh là lượt ghi bị từ chối.
- Tử số là cận dưới (`INSERT OR IGNORE` gộp chữ ký); chấp nhận không để dấu nên "xử lý hết" là lời khai.
- Nợ đổi loại trong chữ ký giao lại `Chủ: Epic 9`: một lượt quét không làm xảy ra đổi loại, công cụ chỉ in cận dưới.
- Mã lần 1 hoàn tác sau review (intent_gap); đối chứng lần 2: gỡ `readOnly`, gỡ đếm trước `scanned_at`, bỏ cận dưới cửa sổ, gỡ chặn tử số > mẫu số ⇒ mỗi phép đỏ đúng một ca.

## Plan Change Log

- 2026-10-12, review lần 1: intent_gap (bấm giữa lượt quét bị loại im lặng; Chương không có nguồn) ⇒ Ice chọn Decision 9 (B) và 10 (`--chapter`); mã lần 1 hoàn tác; Code Map, Tasks, AC, ma trận và đối chứng nhận các phát hiện medium/low của Triage Log. Tránh: so byte làm cổng chỉ-đọc, bí danh `chapterScans`, "NaN%", đề xuất tự đặt thay lời FR81. GIỮ: tách số học (`falseAlarm.mjs`) khỏi CLI, nạp `node:sqlite` qua `createRequire`, mẫu số `reported`, mã thoát 2 cho lỗi, khuôn bản ghi và hai dòng sổ nợ của lần 1.

## Review Triage Log

- [Điều phối] `proofreadState.ts:129` `applyEvent` đổ phát hiện theo lô giữa lúc quét, `ignoreProofreadFinding` không chặn `scanning` ⇒ "không phải lỗi" giữa lượt quét có `created_at < scanned_at`, công cụ loại im lặng khỏi tử số — high, intent_gap (cận dưới cửa sổ là quyết định của Ice).
- [Quick] báo cáo không nêu Tác phẩm, Chương như Always đòi; `work.name` đọc được, `proof_scan` không giữ Chương — medium, intent_gap (Chương lấy từ đâu).
- [Quick] `chapterScans = scans` là bí danh chết, `scope` không dùng; lượt quét vùng chọn bị đo như Chương — medium, đưa vào plan.
- [Quick] tử số > `reported` ⇒ `Math.sqrt` âm, in "NaN%" và tỷ lệ > 100% không cảnh báo — medium, đưa vào plan.
- [Quick] ca "DB không đổi byte" không đỏ được: đo `node:sqlite` 3.51.2 trên WAL, mở ghi rồi đọc không đổi byte, `readOnly` ghi thì ném "attempt to write a readonly database" — medium, đưa vào plan (canh bằng lượt ghi bị từ chối).
- [Điều phối] ba đề xuất in khi vượt không phải ba đề xuất FR81 (chỉnh prompt, hạ độ nhạy, tách riêng hai loại) — medium, đưa vào plan.
- [Quick] `args.find` lấy "1" làm đường dẫn khi `--scan 1` đứng trước — low, đưa vào plan.
- [Quick] `gitState()` chạy theo cwd, không theo repo của script — low, đưa vào plan.
- [Quick] chú thích đầu tệp lặp lại tên tệp (AGENTS.md Code comments); doc comment `ignoreWindow` nêu cận loại trừ, giữ — low, đưa vào plan.
- [Quick] ca lỗi khẳng định `stdout` không chứa "tỷ lệ" trong khi lỗi ra `stderr`; thiếu ca `--scan` không nguyên và id không có — low, đưa vào plan.
- [Điều phối] fixture dùng `'2026-10-12 10:00:00'`, DB thật dùng `strftime('%Y-%m-%dT%H:%M:%fZ')` (`commands/proofread.rs:350`, `:421`) — low, đưa vào plan.
- [Quick] ô Tasks chưa đánh dấu, Implementation Notes rỗng — false, sửa plan của chính build bị loại theo luật triage.

- [Quick r2] `falseAlarmRate.test.ts:7` `@ts-expect-error` thừa, `vue-tsc` báo TS2578 (tsconfig `allowJs`) — medium, patch.
- [Điều phối r2] `SUGGESTIONS[2]` gọi hai loại là "chính tả và ngữ pháp", hai loại thật là chính tả/ngữ pháp và nghi về nghĩa (`isMeaningKind`) — medium, patch.
- [Quick r2] Verification của plan thiếu `--chapter` — false theo luật triage (sửa plan của chính build); `measurements-bao-dong-gia.md` có lệnh đúng.
- [Quick r2] cửa sổ lượt quét cuối không có cận trên, bấm trên Chương khác hay phiên sau làm phồng tử số — false: `clearResult()` chạy đầu mỗi lượt quét (`proofreadState.ts:173`), phát hiện chỉ sống trong bộ nhớ, quét Chương khác sinh hàng `proof_scan` đóng cửa sổ; phần dư (bấm giữa lượt quét KẾ được tính cả vào lượt trước) low, sửa cần thêm nhánh, bị loại.
- [Quick r2] "vượt" thắng cảnh báo quy trình — false: tử số khi `beforeScan` > 0 là cận dưới, nên vượt vẫn đúng.
- [Quick r2] thiếu ca `beforeScan` > 0 kèm tỷ lệ vượt — low, patch (khoá thứ tự đã chọn).
- [Quick r2] kết quả đối chứng chưa ghi — false theo luật triage (ghi ở commit/Implementation Notes bước 5); điều phối đã tự chạy bốn phép gỡ, mỗi phép đỏ đúng một ca.

## Design Notes

Cửa sổ theo thời gian vì `proof_ignore` không giữ id lượt quét; đúng khi quy trình (3) giữ, vì nó cấm quét lại và bỏ ghi nhớ. Tử số là cận dưới (gộp chữ ký); mẫu số gồm cả phát hiện Ice bỏ sót vì chấp nhận không để dấu, nên xử lý hết là lời khai của Ice. Build lấy từ git của cây chạy `tauri dev`.

## Verification

**Commands:**
- `npx vitest run tests/frontend/falseAlarmRate.test.ts` -- expected: đạt
- `node scripts/measure-proofread-false-alarm.mjs <project.db> --scan <id>` -- expected: báo cáo như ma trận

**Manual checks (if no CLI):**
- Đối chứng (gỡ thật, chạy lại riêng tệp test): bỏ `readOnly` khỏi `openReadOnly` ⇒ ca ghi bị từ chối đỏ; lấy cả hàng trước `scanned_at` vào tử số ⇒ ca cửa sổ đỏ; bỏ phép đếm hàng trước `scanned_at` ⇒ ca bấm giữa lượt quét đỏ; bỏ chặn tử số > mẫu số ⇒ ca đó đỏ.


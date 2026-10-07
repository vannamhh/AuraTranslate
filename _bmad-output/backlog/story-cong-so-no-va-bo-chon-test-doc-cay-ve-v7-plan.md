---
title: 'Cổng sổ nợ và bộ chọn test đọc cây vé v7'
type: 'chore'
ticket: 'story-cong-so-no-va-bo-chon-test-doc-cay-ve-v7'
created: '2026-10-06'
status: 'done'
baseline_revision: 'a7517c903d635cd45810d7f9a028ea52340e2925'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/scripts/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `check:debt-owner` (cổng thứ 12 của `pre-push`) và `test:story` còn đọc `_bmad-output/implementation-artifacts/`, thư mục đã mất sau di trú v7. Cổng dừng với lỗi hạ tầng, nên lần push kế bị chặn.

**Approach:** Một bộ đọc cây vé v7 dùng chung cho cả hai script. Kiểm C tra chủ `Story X.Y` và `Epic N` theo cây vé, `test:story` tìm plan theo ref. Khoảng 20 mục nợ có chủ cuối là Story 4.x được nối dòng chuyển chủ. Tiêu chí nghiệm thu là AC1–AC8 của ticket `_bmad-output/backlog/story-cong-so-no-va-bo-chon-test-doc-cay-ve-v7.md`.

## Boundaries & Constraints

**Always:** Kiểm A giữ nguyên luật và con số (`0/260 … 795 mục tổng, 63 nửa, 339 đóng` trên sổ chưa nối dòng). Cây vé đọc hỏng hoặc rỗng thì `abort()` với lỗi hạ tầng. Có sàn số ticket thay cho `SPRINT_KEY_FLOOR`, qua `judgeFloor`. `--report/--list/--surface/--file` giữ nguyên ngữ nghĩa. Sổ nợ chỉ được nối thêm dòng. Chủ mới mặc định là `Epic 4` (Ice, 2026-10-06). Quyết định (Ice, 2026-10-06): cây vé đọc bằng bộ đọc dòng tự viết, chặt, chỉ nhận `[[epic]]`/`[[entry]]` với khoá `id`/`slug` và ném lỗi khi gặp dạng lạ; không thêm dependency. Quyết định (Ice, 2026-10-06): plan không khai test thì `test:story` in cảnh báo và vẫn chạy test lấy từ diff, chỉ thoát khác 0 khi tập test rỗng. Quyết định (Ice, 2026-10-06): giữ cả hai script trong một story dù plan vượt 1.600 token.

**Never:** không gọi `uv`/`tickets.py` từ cổng (CI không cài `uv` hay python). Không thêm Kiểm mới. Không đổi danh sách cổng ở ba nơi. Không sửa `tickets.py`, plan hay tệp epic. Không sửa các con trỏ chết khác trong AGENTS.md.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Chủ story done | `Chủ: Story 4.8`, plan `status: done` | mục bị báo | — |
| Chủ story chưa xong | plan `in-progress`/`built`/`in-review`, hoặc entry không có plan | không báo | — |
| Chủ không tồn tại | `Story 4.99` hoặc `Epic 42` | mục bị báo | — |
| Id chữ | `Story 6.16c` | tra đúng entry `16c` của epic id 6 | — |
| Chủ epic | `Epic 4`, tệp epic `status: in-progress` hoặc không có `status:` | không báo; `done` thì báo | — |
| Giá trị có nháy | `status: 'done'`, `ticket: '5'` | đọc như không nháy | — |
| Cây vé hỏng | thiếu `tickets.toml`, dòng `id` lạ, id trùng | — | `abort()`, thoát khác 0 |
| `test:story 4-8` | dạng gạch ngang | cùng plan với `4.8` | — |
| Ref không có | `test:story 4.99` | — | thoát 1, in thư mục đã tìm |

</frozen-after-approval>

## Code Map

- `scripts/check-debt-owner.mjs` -- đường cũ ở :111, :115, :116 và selftest `MAC_DINH` :598. `STORY_OWNER_RE` :198 và `latestOwnerKey` :202-209 trả khoá `N-M`/`epic-N` (đã nhận `[a-z]?`). `parseSprintStatus` :217-232 sẽ bị thay. `staleOwnerItems` :235-242 giữ hình dạng `(items, Map)`. Lượt đọc cấp cao :659-664, sàn `SPRINT_KEY_FLOOR = 139` :666-676, Kiểm C :760-773. Selftest C ở :480-506 (6 khoá giả, 15 ca) và :568, :578.
- `scripts/lib/floor-judge.mjs` -- `judgeFloor`, dùng lại.
- `scripts/test-story.mjs` -- `SPEC_DIR` :73, `findSpec` :~97 (cần thay), `declaredFrom` :~103 regex cả văn bản (giữ, vì plan v7 dùng cùng dạng đường dẫn). Hợp diff + khai báo; run set rỗng thì thoát 1.
- Cây v7 -- `_bmad-output/initiative-auratranslate/tickets.toml` (`[[epic]]` `id`, `slug`). `epic-<slug>/tickets.toml` (`[[entry]]` `id`). Plan là `epic-<slug>/*.md` ở độ sâu 1 có `ticket:` trong frontmatter (141 plan: 115 `done`, 20 `'done'`, 6 `in-progress`; không trùng, không mồ côi). Trạng thái epic là `status:` của `epic-<slug>/epic-<slug>.md` (8/11 có, đều `in-progress`). Loại trừ `*-retrospective.md`. 174 entry, 33 entry chưa có plan.
- `AGENTS.md` gốc -- :34 và :62 nói Kiểm C/`sprint-status.yaml`. Phải giữ số dòng: `naming_boundary.rs` đọc dòng :41.
- `_bmad-output/initiative-auratranslate/deferred-work.md` -- 20 mục mở có chủ cuối Story 4.x, liệt kê theo dòng đầu: 643, 866, 878, 2760 (4.12) · 5104 (4.9) · 7289 (4.2) · 11382, 11601, 11655, 11688, 11713, 11771, 11876, 11895, 11918, 12059, 12120, 12205 (4.8) · 12230 (4.10) · 12285 (4.12).

## Tasks & Acceptance

**Execution:**
- [ ] `scripts/lib/ticket-tree.mjs` -- mới: `readTicketTree(root)` → `{ stories: Map<'N-M', status|null>, epics: Map<'epic-N', status|null>, planFor(ref) }`, ném lỗi khi hình dạng lạ -- một bộ đọc cho cả hai script.
- [ ] `scripts/check-debt-owner.mjs` -- đổi đường sổ nợ. Thay `parseSprintStatus`/`SPRINT_STATUS_PATH` bằng `readTicketTree`. Đặt sàn mới bằng ceil(0.85 × số entry thật). Đổi chữ trong thông báo Kiểm C. Thêm ca selftest cho epic `done`, id chữ, cây rỗng, trạng thái có nháy -- AC1–AC5.
- [ ] `scripts/test-story.mjs` -- `findSpec` → `planFor(ref)` (`X.Y`/`X-Y`). In cảnh báo "plan không khai test" khi `declaredFrom` rỗng, vẫn chạy test từ diff; ref không có thì in thư mục đã tìm -- AC7.
- [ ] `deferred-work.md` -- nối `→ 2026-10-06 (Story 4.x đã done) — Chủ: Epic 4 — <lý do>` cho 20 mục, hoặc chủ khác khi rõ ràng hơn -- AC6.
- [ ] `AGENTS.md` -- sửa :34 và :62 sang "cây vé", giữ số dòng -- AC8.

**Acceptance Criteria:**
- Given HEAD sau các task, when chạy `npm run check:debt-owner`, then thoát 0 và Kiểm C đối chiếu với số ticket thật.
- Given phép tra trạng thái trong `ticket-tree.mjs` bị gỡ thật, when chạy lại cổng, then Kiểm B đỏ đúng vì lý do đó.

## Implementation Notes

- Bộ đọc `scripts/lib/ticket-tree.mjs` nhận thêm chuỗi nhiều dòng `"""`/`'''` (32 `description` thật dùng), nhưng `id`/`slug` nhiều dòng vẫn ném.
- Kiểm C trên sổ thật chỉ báo 13 mục, không phải ~20 như số đếm bằng script lúc ký. Bộ đếm tạm đọc chữ `Chủ:` thô, còn cổng đọc `Chủ:` cụ thể cuối cùng theo luật của nó. 13 là số thật.
- 10 mục chuyển sang `Epic 4`. 3 mục tự ghi chủ dự kiến bằng "Chủ mới:" (cổng không đọc cụm này) nên chuyển theo đúng ý đã ghi: hai mục sang `Ice`, một mục sang `Story 10.9`.
- Đối chứng: gỡ thật `stories.set(key, frontmatterValue(…))` thành `null` ⇒ Kiểm B đỏ "bộ đọc cây vé đọc sai trạng thái", cổng thoát 1; trả lại ⇒ thoát 0. Kiểm C trên sổ thật vẫn xanh lúc gỡ, nên chỉ ca cây giả của Kiểm B canh mối nối này.
- Hai dòng `test:story` của bảng tình huống (`4-8`, `4.99`) được kiểm bằng chạy CLI tay; `test-story.mjs` không có bộ test tự động.
- AGENTS.md :11 vẫn trỏ `implementation-artifacts`/`sprint-status.yaml` (ngoài phạm vi theo plan).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 8 phát hiện — high 0 · medium 1 · low 2 · false 3 · bị bác vì sửa plan 2.
- medium · patch — `TICKET_FLOOR` chỉ đếm entry; plan mất `ticket:` thì mọi trạng thái null, Kiểm C xanh mà không quét gì. Thêm sàn thứ hai trên số trạng thái đọc được.
- low · patch — doc comment tiếng Việt trên dòng đã sửa và hai comment `// expected` mới, nhắc lại tên (AGENTS.md §Code comments). Xoá.
- low · patch — JSDoc `parseTicketsToml`/`readTicketTree` vượt hai dòng. Cắt còn hợp đồng không hiển nhiên.
- bác (sửa plan) — ô Execution chưa đánh dấu, Triage Log rỗng.
- bác (sửa plan) — "khoảng 20" so với 13 mục; Implementation Notes đã ghi 13 là số thật.
- false — Plan Change Log rỗng: log này chỉ dành cho loopback; AC7 nới bằng quyết định của Ice lúc lập plan.
- false — AC8 với AGENTS.md :11: dòng đó là con trỏ thư mục, không nói Kiểm C đọc `sprint-status.yaml`.
- false — mối nối `epics` không có ca: sổ thật có 10 mục `Chủ: Epic 4` ⇒ bỏ `epics` khỏi `ticketStatus` thì `epic-4` thành "không tồn tại" và Kiểm C đỏ.

## Verification

**Commands:**
- `npm run check:debt-owner` -- thoát 0. Kiểm A in `0/260 mục mở thiếu Chủ: — 795 mục tổng` trước khi nối dòng.
- `npm run test:story 4.8 -- --list` và `npm run test:story 4-8 -- --list` -- cùng một plan.
- `npm run test:story 4.99 -- --list` -- thoát 1.
- `cargo test --test naming_boundary` -- xanh (cần `npm run build` trước).

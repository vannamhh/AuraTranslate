---
type: migration
title: "Kế hoạch di trú BMad v6 → v7 cho AuraTranslate"
status: done
created: 2026-10-06
---

# Kế hoạch di trú BMad v6 → v7 — AuraTranslate

Bản NHÁP, chưa thi hành. Quy tắc nguồn: `/Users/hoangnam/.claude/skills/bmod-method/migration-1.toml` (`target`, `precautions`, `guide`, `checklist`). Mẫu thực thể: `/Users/hoangnam/.claude/skills/bmad-ticket/assets/` (`initiative-template.md`, `epic-template.md`, `story-template.md`, `tickets-template.toml`). Ngoài tệp này không có gì bị đụng tới: không di chuyển, không sửa, không xoá, không commit, chưa chạy test hay cổng nào. `deferred-work.md` (1.685.887 byte, 12.872 dòng) chỉ được `grep`/`wc`, không đọc.

Quy ước tên trong kế hoạch: mọi thư mục/tệp mới viết bằng slug ASCII kebab (bỏ dấu, `đ`→`d`); khoá `sprint-status.yaml` mang dấu tiếng Việt nên `v6_key` giữ NGUYÊN VĂN khoá đó (để tìm kiếm). Tên không mang ngày, giờ, số story/epic v6; ngày nằm ở `created`. Ngoại lệ do quy tắc: `archive-v6/`, `migration-v6-v7/`, tên anh em/đồng hành giữ nguyên (mục 5).

## 1. Tín hiệu `detect` đã khớp

Cả bốn tín hiệu đều có mặt; hai tín hiệu đầu chắc chắn là v6.

| Tín hiệu | Khớp | Bằng chứng |
|---|---|---|
| `epics.md` / `sprint-status.yaml` | CÓ | `_bmad-output/planning-artifacts/epics.md` (7.399 dòng, 11 `## Epic N:`, 154 `### Story`); `_bmad-output/implementation-artifacts/sprint-status.yaml` (1.173 dòng) |
| Tệp build `<epic>-<story>-<slug>.md` / `spec-<epic>-<story>-<slug>.md` có `route:`/`status:` | CÓ (một phần) | 127 tệp bản ghi cho 120 story; `status:` có ở 86 tệp, `route:` ở 46; Epic 1–2 (41 tệp: 25 + 16) chỉ có `baseline_commit:` nên trạng thái lấy từ `sprint-status.yaml` |
| Thư mục tài liệu có ngày dưới planning | CÓ | `prds/prd-AuraTranslate-2026-08-02/`, `briefs/brief-AuraTranslate-2026-08-02/`, `ux-designs/ux-AuraTranslate-2026-08-02/`, `architecture/architecture-AuraTranslate-2026-08-02/` |
| `specs/spec-<slug>/SPEC.md` dưới `core.output_folder` | CÓ | `_bmad-output/specs/spec-AuraTranslate/` (SPEC.md + 5 tệp đồng hành + `.memlog.md`); KHÔNG có `stories.yaml`, KHÔNG có `stories/` ⇒ là tài liệu quy hoạch, không phải epic |

Điều kiện áp dụng: không có `active_initiative` dưới `[core]` trong `_bmad/custom/config.user.toml` (tệp chỉ có chú thích), không có thư mục `initiative-*/` ⇒ di trú áp dụng. `_bmad/method/scripts/tickets.py` KHÔNG tồn tại (thấy `_bmad/scripts/` chỉ có `config_utils.py`, `memlog.py`, `render_skill.py`, `resolve_*.py`) ⇒ tiền đề, câu hỏi Q7.

## 2. Nguồn và kiểm kê

Cấu hình đọc từ `_bmad/config.toml` và `_bmad/custom/*.toml`: `core.project_name = "AuraTranslate"`; `core.output_folder = {project-root}/_bmad-output`; `modules.bmm.planning_artifacts = …/_bmad-output/planning-artifacts`; `modules.bmm.implementation_artifacts = …/_bmad-output/implementation-artifacts`. Hai tệp `_bmad/custom/config.toml`, `config.user.toml` chỉ có chú thích; `_bmad/custom/bmad-project-context.toml` có 2 dòng nhắc đường dẫn sẽ chuyển (mục 14).

Toàn bộ `_bmad-output/`: **777 tệp** = 431 do git theo dõi + 346 bị gitignore (đo bằng `git ls-files` / `--others --ignored`; con số 336 ban đầu là ước lượng, đo thật là 346: 333 trong `2-4-ban-do/` gồm 287 `.log`, 34 `.tsv`, 6 `.db*`, 2 `.png`, 4 `.txt`; 7 fixture HTML `6-1-ban-do/fixtures/html/a0*.html`; 3 `latest-run.log`; 3 `.DS_Store`). Tệp bị gitignore KHÔNG `git mv` được ⇒ chuyển bằng `mv` thường (mục 11).

| Thư mục | Tệp |
|---|---|
| `planning-artifacts/` | 104 (1 `epics.md`, 4 prd, 3 brief, 44 ux, 13 architecture + 2 spine-condense, 18 sprint-change-proposal, 6 ad-brief + 1 ad-51-draft, 1 readiness, 10 research, 1 `.DS_Store`) |
| `implementation-artifacts/` | 663 (206 tệp ở gốc + 456 trong 14 thư mục `*-ban-do/` + 1 `.DS_Store`) |
| `specs/spec-AuraTranslate/` | 7 |
| `party-mode/memories/installed/` | 1 (`.memlog.md`) |
| gốc `_bmad-output/` | `project-context.md`, `.DS_Store` |

Lệch so với ghi chú đầu việc (đã đo lại): (a) có **18** `sprint-change-proposal-*.md`, không phải 20; (b) có **6** `epic-N-context.md` (3, 4, 5, 6, 7, 11), không phải 7; (c) `sprint-status.yaml` có **93** `action_items`: 49 `done`, 40 `open`, 4 `in-progress` (44 mục phải chuyển, không phải ~30); (d) 346 tệp gitignore, không phải 336.

### Đối chiếu số story: 153 hay 154

- `sprint-status.yaml` `development_status`: **153** khoá story + 11 khoá epic (8 `in-progress`, 3 `backlog`) + 11 khoá retrospective (8 `done`, 3 `optional`). Trạng thái story: 103 `done`, 12 `review` (toàn bộ Epic 4), 5 `in-progress` (1-3, 1-20, 1-21, 2-3, 2-4), 33 `backlog` (6-16c và Epic 8, 9, 10 = 15 + 8 + 9 + 1).
- `epics.md`: **154** `### Story` (đếm bằng regex, không trùng id). Chênh 1 = **Story 6.18** *Đo lại NFR3, NFR4, NFR5 trên thư viện 5.000 Chương thật* (`epics.md:5493`): có tiêu đề, KHÔNG có khoá trong `sprint-status.yaml`. Đã soát hai chiều: mọi khoá sprint đều có tiêu đề, chỉ 6.18 có tiêu đề mà không có khoá.
- Lý do: 2026-09-15 `correct-course` GỘP 6.18 vào Story 10.9 (`sprint-change-proposal-2026-09-15.md`, comment `sprint-status.yaml:177-179` và `:242-244`); khoá bị gỡ khỏi sprint nhưng bản ghi `spec-6-18-…` còn `status: 'in-progress'`, `baseline_commit: 7863afb…`, cùng `6-18-ban-do/`, ba tệp `story_6_18_*.rs`, feature `nfr-bench`. Năm khối AC đã chuyển nguyên văn sang 10.9.
- Hệ quả: 154 entry trong `tickets.toml` (đúng quy tắc: mỗi `### Story` là một entry), nhưng đối chiếu checklist mục 4 phải ghi 153 (tracking) + 1 (6.18 chỉ có ở `epics.md`) ⇒ Q10.

## 3. Câu hỏi và câu trả lời

### 3.1 Đã trả lời (Ice)

| # | Câu hỏi | Trả lời | Lý do |
|---|---|---|---|
| 1 | Sao lưu `_bmad-output` thành `<output>-bak` trước? | CÓ → `_bmad-output-bak` | Git không phải bản sao lưu đủ: 346 tệp (≈11 MB) bị gitignore nằm trong `_bmad-output` (`2-4-ban-do/*.log`, `.db*`, fixture HTML 6-1, `latest-run.log`, `.DS_Store`). Chưa tạo; tạo sau khi duyệt kế hoạch này |
| 2 | Một hay nhiều initiative? | MỘT: `initiative-auratranslate` | `core.project_name = AuraTranslate`; toàn bộ PRD/epics/spec là một sản phẩm |
| 3 | Dự án có chạm kho khác? | KHÔNG | Không cần workspace |
| 4 | Kho riêng cho store? | KHÔNG: store vẫn do kho AuraTranslate (nhánh `master`) theo dõi | Không `git init` mới, không `git rm --cached`, không sửa `.gitignore` |
| 5 | 17 story dở dang: làm xong ở v6 hay di trú ngay? | DI TRÚ NGAY: `review` → `in-review`, `in-progress` giữ nguyên | 12 story Epic 4 (`review`) + 5 story `in-progress` (1-3, 1-20, 1-21, 2-3, 2-4) |
| 6 | Gộp tệp story chưa bắt đầu vào entry? | Vô hiệu | Đã kiểm: không story `backlog` nào có tệp bản ghi (33 story `backlog` đều không có tệp; xem mục 9) |

### 3.2 Còn mở: mặc định đề nghị (Ice sửa một lần, không hỏi từng tệp)

| # | Câu hỏi | Mặc định đề nghị | Đo/Căn cứ |
|---|---|---|---|
| Q7 | Tiền đề: thiếu `_bmad/method/scripts/tickets.py` (bmad setup báo script module "missing", script dùng chung "stale"); checklist mục 4 cần nó | Chạy `bmad setup` làm mới (chỉ script, không đổi cấu hình) NGAY SAU khi duyệt, trước bước 3 | `_bmad/method/` không tồn tại; không có tickets.py ⇒ không chạy được `status` |
| Q8 | `.claude/skills/`, `.agent/skills/`, `.agents/skills/` (gitignored, mỗi nơi 64 mục) giữ bản v6 cục bộ của mọi skill BMad + hai skill đã nghỉ `bmad-create-epics-and-stories`, `bmad-sprint-planning`; chúng che bản v7 ở `~/.claude/skills/` (`bmad-ticket` chỉ có bản toàn cục) | Gỡ bản cục bộ trùng và hai skill đã nghỉ qua `bmad setup` SAU khi kiểm xong di trú; Ice xác nhận việc xoá riêng | Sau di trú, `bmad-build`, `bmad-retrospective`, `bmad-project-context` cục bộ vẫn là bản v6 (chúng đọc `sprint-status.yaml`) ⇒ sẽ ghi/đọc sai chỗ |
| Q9 | Thứ tự `[[epic]]` trong `initiative-auratranslate/tickets.toml`: quy tắc nói "theo thứ tự `epics.md`" (1→11), mẫu nói thứ tự bảng = thứ tự xây, và `epics.md` §Epic List tự nói cột *Thứ tự* mới là trình tự thật | Thứ tự THỰC THI: 1, 2, 3, 5, 6, 4, 11, 7, 8, 9, 10 (id epic giữ số v6; không đổi id). Story 4.1 chạy ngay sau Epic 3: ghi ở Notes của epic-4, không đổi vị trí epic | `sprint-change-proposal-2026-08-13b-thu-tu-epic.md`, `…-2026-09-24-epic-11-tra-no-nen.md`; `sprint-status.yaml:2-21`. Phương án B (1→11) trung thực với chữ quy tắc nhưng `tickets.py next` có thể đề xuất Epic 4 trước Epic 5 (suy từ thứ tự bảng, chưa kiểm vì thiếu script) |
| Q10 | Story 6.18 (có tiêu đề ở `epics.md`, không có khoá sprint, đã gộp vào 10.9, bản ghi `in-progress`) | Một entry ở epic-duong-nhap (id `18`), kèm plan `in-progress` từ bản ghi, Notes ghi "gộp vào 10.9; AC đã ở 10.9"; entry 10.9 giữ AC. Đếm: 154 entry, trong đó 6.18 là entry duy nhất không có khoá sprint | Quy tắc "mỗi `### Story` là một entry" + "không suy ra baseline từ HEAD" (bản ghi có baseline thật). Phương án B: lưu bản ghi vào `archive-v6/` như mục gộp ⇒ mất một tệp bản ghi `in-progress` khỏi cây sống |
| Q11 | Epic 4: 12 bản ghi `status: done` nhưng `sprint-status` là `review` | `in-review` (sprint thắng; "spec `done` + sprint `review`" = chờ Ice ký) | Quy tắc "review stays in-review, nothing is relabeled built" |
| Q12 | Story 5.14: bản ghi `status: review`, `sprint-status` là `done` | `done` (đồng bộ theo sprint và retro Epic 5 đã `done`) | `spec-5-14-…` và `sprint-status.yaml:149`. Phương án B `in-review` làm số story `done` giảm 1 (103→102) |
| Q13 | Story nhiều bản ghi: 11.1 (4 lô A–D + 1 handoff lô D), 11.6 / 11.7 / 11.8 (2 lô mỗi story), 3.5 (+`spec-3-5-fix-review-findings.md`) | MỖI bản ghi là một plan riêng (checklist: "every build record one plan"), cùng `ticket:` = id entry, tên `story-<slug-bản-ghi>-plan.md`, trạng thái từng bản ghi (đều `done`); handoff `phases`/`task0` là bằng chứng (mục 5). Phương án B: một plan/entry (bản ghi lô A) và lô còn lại thành bằng chứng, an toàn hơn nếu `tickets.py` kén trùng `ticket:` | Chưa kiểm được hành vi `tickets.py` với nhiều plan cùng `ticket` (script thiếu ⇒ Q7). Sau khi có script, chạy `status`; nếu lỗi thì chuyển sang B |
| Q14 | 21 spec lẻ không số story (mục 7.3) | 14 spec có action item retro nêu đích danh, 6 spec `epic-3-review-cum-*` chỉ có bằng chứng yếu (tên và ngày); cả 20 spec ⇒ entry MỚI cuối epic của retro đó (id kế tiếp; entry `bug` nếu `type: bugfix`, còn lại `story`; `covers = []`); 1 spec không có bằng chứng (`spec-fix-hanviet-…`) ⇒ `backlog/bug-fix-hanviet-parallel-row-overflow.md` + plan. Phương án B: cả 21 vào `backlog/` phẳng (giữ số entry = 154, bớt 20 entry không có trong `epics.md`, nhưng lìa khỏi epic của retro) | Bằng chứng ghi ở mục 7.3 (id action item trong `sprint-status.yaml`). Phương án A thêm 20 entry vượt 154 ⇒ checklist mục 4 phải tách "entry thêm từ spec lẻ" |
| Q15 | Thư mục `*-ban-do/` (14 thư mục, 456 tệp, 5 tệp `*-ban-do-*.html` ở gốc) | Ở NGUYÊN CHỖ (không di chuyển) | 3 test Rust đọc đường dẫn cứng: `src-tauri/tests/docx_probe.rs:38` (`6-12-ban-do`), `webimport_probe.rs:72` + `webimport_contract.rs:1461-1463` (`6-1-ban-do`, kể cả fixture gitignore); `.githooks/pre-push:33` miễn trừ theo tên `-ban-do/`. Quy tắc cấm sửa tệp ngoài store. Phương án B: chuyển 12 thư mục vào epic, giữ 2 thư mục (6-1, 6-12) ⇒ 12 thư mục cần slug mới và 54 tham chiếu văn xuôi `*-ban-do` ở 29 tệp ngoài store lệch đường dẫn |
| Q16 | Kiểu (`type`) và dạng cho tệp bằng chứng mà tên và frontmatter không nói rõ | Theo bảng mục 5 và 7.4: `handoff` (phases/task0), `investigation` (`ho-so-dieu-tra-*`), `proposal` (`proposal-tauri-window-automation`), `report` (`bmad-build-auto-result-*`), `history` (`ai-4-loopback-1-history`, kèm `ai-4-implementation.patch` bên trong thư mục), `evidence` (`agent-rules-evidence`, `agent-token-economics`), `research` (research/\*), `data` (`research/2-4-so-do`), `plan`/`worksheet` (spine-condense) | Quy tắc: "kiểu từ frontmatter hoặc tên, hỏi khi cả hai không nói". Mỗi tệp một thư mục cùng tên chính |
| Q17 | `deferred-work.md` (quy tắc: về thư mục initiative, không đổi nội dung) và `sprint-status.yaml` (lưu trữ) bị `scripts/check-debt-owner.mjs` đọc cứng (cổng thứ 12 trong `pre-push`, bước CI `npm run check:debt-owner`) ⇒ cổng hỏng ngay sau di trú | Thi hành đúng quy tắc (chuyển `deferred-work.md`, lưu trữ `sprint-status.yaml`); ghi việc theo dõi thuộc Ice ở mục 13.3: sửa `check-debt-owner.mjs` TRƯỚC lần push kế. Không `--no-verify` | Phương án B (để `deferred-work.md` tại chỗ) không cứu được cổng vì Kiểm C vẫn cần `sprint-status.yaml`. `deferred-work.md` miễn viết lại đường dẫn (12.872 dòng, nơi khác trích theo số dòng `L1234`) |
| Q18 | Tên slug: epic `epic-<slug>` lấy phần tiêu đề TRƯỚC dấu `—` (vd. `epic-bien-tap-theo-segment`), tiêu đề đầy đủ vẫn ở `title`; tên plan = `story-<phần-sau-số-trong-tên-bản-ghi>-plan.md` | Như vậy | Tiêu đề đầy đủ sẽ cho slug 60–90 ký tự, và tiêu đề Epic 11 có chữ "Epic 1–6" (số epic v6, cấm trong tên). Chữ quy tắc: "slug là tiêu đề epic, kebab-case, bỏ số" ⇒ đây là một quyết định kế hoạch, nêu rõ |
| Q19 | `project-context.md` ở gốc store (AGENTS.md gọi là lịch sử đóng băng, 699 dòng) | `inbox/project-context.md` (nguyên trạng); 41 tham chiếu văn xuôi ở 24 tệp ngoài store (vd. `ci.yml:259`, `check-debt-owner.mjs:5,48`) thành tham chiếu lệch, chỉ liệt kê | Quy tắc: tệp ở gốc không nhận ra ⇒ đề xuất inbox, mặc định có. Phương án B: để nguyên tại chỗ (không lệch tham chiếu nào, store có một tệp lạ ở gốc, checklist cho phép "remnant liệt kê") |
| Q20 | Mảnh vụn không ai dùng: `party-mode/memories/installed/.memlog.md` (17 dòng, không có gì bên cạnh) và `research/thu-xin-phep-hvtdtd.md` (thư xin phép dùng dữ liệu HVTĐTD, chỉ được `brief/.memlog.md` nhắc) | Cả hai vào inbox: memlog ⇒ `inbox/archive-v6/.memlog.md`; thư ⇒ `inbox/thu-xin-phep-hvtdtd.md` (không đổi tên, không thêm frontmatter) | Quy tắc "stray memlog ⇒ inbox/archive-v6/"; thư là mảnh vụn không có bằng chứng gia nhập |
| Q21 | 6 `ad-brief-*.md` + `ad-51-draft-2026-10-02.md`: AGENTS.md ghi "số `AD` kế tiếp = quét spine VÀ MỌI `ad-brief-*.md` chưa viết" ⇒ đổi tên làm lệch phép tra này | Theo quy tắc tài liệu quy hoạch: `ad-brief-<slug>/ad-brief-<slug>.md` (+ `ad-51-draft/ad-51-draft.md`) trong initiative, kiểu = `ad-brief`/`ad`; ghi vào việc theo dõi của Ice (13.3): đổi câu quét trong `AGENTS.md` sang `initiative-auratranslate/ad-brief-*/ad-brief-*.md` | AGENTS.md là tệp ngoài store, không sửa trong di trú. AD-51 đã vào spine theo `ad-51-draft`, bản nháp đó chỉ còn là bằng chứng |
| Q22 | `.DS_Store` (3 tệp: gốc output, `ux-designs/`, `implementation-artifacts/`) | Để nguyên tại chỗ, không đụng (siêu dữ liệu macOS, gitignored); thư mục chỉ còn `.DS_Store` giữ lại tới khi Ice xoá | "Không xoá gì" + sao lưu đã chụp chúng |
| Q23 | `after` cho `[[epic]]`: quy tắc yêu cầu ghi khi `epics.md` NÊU Epic nào cần gì từ Epic nào | Không ghi `after` cấp epic (`epics.md` nêu thứ tự chạy, không nêu "cần"); thứ tự nằm ở bảng `[[epic]]` (Q9). Ghi vào Notes initiative: bốn FR nghiệm thu chia đôi (FR13: Epic 1⇄6; FR44, FR129: Epic 6⇄7; FR70: Epic 4⇄7) và entry 7.1 chờ phần cấu trúc ở 6.13; `after` cấp entry chỉ ghi nơi tiêu đề story nêu rõ phụ thuộc, soát lúc viết entry | Tìm thấy 0 câu "Epic X cần Y từ Epic Z" ở §Epic List |
| Q24 | `Done when` của 11 epic + initiative (quy tắc: viết từ Requirements và xác nhận với Ice) | Soạn nháp từ mục tiêu epic ở `epics.md` và PRD, trình Ice xác nhận một lượt ở bước viết epic, trước khi commit nhóm 3 | Quy tắc yêu cầu xác nhận |
| Q25 | `sprint-change-proposal-2026-09-24b-phieu-quyet.md` (phiếu quyết, anh em của `…-no-dung-ten-ice.md`) | Thư mục `change-` riêng (`change-phieu-quyet-no-dung-ten-ice-hang-p/`), không gộp | Tên và vai là một phiếu quyết, nhưng nguồn nó trỏ về đề xuất kia; gộp sẽ làm mất ranh giới tệp |


Tất cả mặc định trên nghĩa là "đồng ý hết" cũng là một câu trả lời hợp lệ.

### 3.3 Ice đã chốt (2026-10-06)

| # | Trả lời | Ghi chú |
|---|---|---|
| Q9 | Thứ tự THỰC THI: 1, 2, 3, 5, 6, 4, 11, 7, 8, 9, 10 | Lệch chữ quy tắc ("theo thứ tự `epics.md`") có chủ ý; lý do ở bảng 3.2 |
| Q14 | Phương án A: 20 spec lẻ thành entry mới cuối epic của retro; `spec-fix-hanviet-…` vào `backlog/` | |
| Q13 | Phương án B (ghi lại ở bước kiểm; thi hành ở commit `016f2ce`) | `tickets.py` không nhận hai plan cùng `ticket:` ⇒ bản ghi thứ hai của 3.5 và các lô B/C/D của 11.1, lô B của 11.6/11.7/11.8 thành tệp `evidence-*` (`type: evidence`, `relates_to`) |
| Q17 | Thi hành đúng quy tắc; ngay sau di trú, một story riêng viết lại `scripts/check-debt-owner.mjs` (đọc trạng thái từ plan v7 thay vì `sprint-status.yaml`) và `scripts/test-story.mjs`, rồi mới push một lượt | Di trú commit theo nhóm, KHÔNG push |

## 4. Sao lưu

- 🔵 2026-10-06: đã tạo; `find _bmad-output-bak -type f | wc -l` = 778 (777 đã kiểm kê + chính tệp kế hoạch); thuộc về Ice, tự xoá khi muốn; loại ở `.git/info/exclude`. Nội dung ban đầu của dòng này:
- Sau khi duyệt: `cp -Rp _bmad-output _bmad-output-bak` (nguyên 777 tệp, kể cả 346 tệp gitignore, `.DS_Store` và các tệp `.db*`), rồi xác nhận số tệp `find _bmad-output-bak -type f | wc -l` = 777 và nói với Ice: bản sao này THUỘC VỀ Ice, tự xoá khi muốn.
- `_bmad-output-bak/` không có trong `.gitignore` của kho ⇒ để khỏi bị `git add -A`, thêm `_bmad-output-bak/` vào `.git/info/exclude` (cục bộ, không đụng `.gitignore`) và mọi commit của di trú dùng `git add` theo đường dẫn. Không bao giờ commit `.db` (AD-25): các `.db*` trong bản sao lưu nằm ngoài mọi commit.
- Cây git hiện sạch (`git status` trống tại đầu phiên) ⇒ không cần commit "trạng thái hiện tại" trước; kế hoạch này là tệp mới duy nhất chưa theo dõi (di chuyển vào initiative ở nhóm 2).


## 5. Bốn danh sách (mỗi tệp đúng một lần)

Tổng 777 tệp = **joins 302** + **inbox 3** + **stays 464** + **archive-v6 8**. Đã kiểm bằng script: mọi tệp trong `_bmad-output/` rơi vào đúng một nhóm bên dưới.

### 5.1 Gia nhập initiative (302 tệp) — kèm bằng chứng

Bằng chứng chung: `epics.md` `inputDocuments` nêu PRD, addendum, spine, DESIGN/EXPERIENCE, toàn bộ `specs/spec-AuraTranslate/*`, brief; spec nêu PRD addendum làm đồng hành; bản ghi story và đề xuất đổi hướng trỏ vào phần còn lại.

| Nhóm | Tệp | v6 | Đích (trong `_bmad-output/initiative-auratranslate/`) | Ghi chú / bằng chứng |
|---|---|---|---|---|
| PRD | 4 | `planning-artifacts/prds/prd-AuraTranslate-2026-08-02/{prd.md, addendum.md, review-rubric.md, .memlog.md}` | `prd-auratranslate/prd-auratranslate.md` (từ `prd.md`) + `addendum.md`, `review-rubric.md`, `.memlog.md` giữ tên | `epics.md` `inputDocuments`. Frontmatter bổ sung `type: prd`, `title`, `status: final`, `created: 2026-08-02`, `skill: bmad-prd` |
| Brief | 3 | `briefs/brief-AuraTranslate-2026-08-02/{brief.md, addendum.md, .memlog.md}` | `brief-auratranslate/brief-auratranslate.md` + hai tệp anh em giữ tên | `epics.md` `inputDocuments` (brief.md, "đối chiếu bước 4"); `type: brief`, `skill: bmad-product-brief`, `status: final` |
| UX | 44 | `ux-designs/ux-AuraTranslate-2026-08-02/` (DESIGN.md, EXPERIENCE.md, 29 `mockups/*.html`, 12 `.working/*`, `.memlog.md`) | `ux-auratranslate/` giữ NGUYÊN mọi tệp + `ux-auratranslate.md` (bộ định tuyến: frontmatter + danh sách mỗi tệp là gì) | `epics.md` `inputDocuments`; `skill: bmad-ux` |
| Kiến trúc | 13 | `architecture/architecture-AuraTranslate-2026-08-02/{ARCHITECTURE-SPINE.md, spine-evidence.md, .memlog.md, reviews/*(11)}` | `architecture-auratranslate/architecture-auratranslate.md` (từ `ARCHITECTURE-SPINE.md`, liệt kê tệp anh em) + `spine-evidence.md`, `.memlog.md`, `reviews/*` giữ tên | `epics.md` `inputDocuments`; AGENTS.md trỏ spine là nguồn bất biến. Tên `ARCHITECTURE-SPINE.md` biến mất: 43 dòng ở 28 tệp ngoài store nhắc nó (mục 13.2) |
| Gọn hoá spine | 2 | `architecture/spine-condense-plan-2026-09-23.md`, `spine-condense-worksheet-2026-09-23.md` | `plan-condense-spine/plan-condense-spine.md`, `worksheet-condense-spine/worksheet-condense-spine.md` | Cả hai nhắm `ARCHITECTURE-SPINE.md` (frontmatter `target`/`base_file`); thuộc kiến trúc, không phải anh em trong thư mục dated nên theo quy tắc tài liệu một thư mục mỗi tệp |
| Spec | 7 | `specs/spec-AuraTranslate/{SPEC.md, requirements.md, glossary.md, data-sources.md, build-sequence.md, risks.md, .memlog.md}` | `spec-auratranslate/spec-auratranslate.md` (từ `SPEC.md`) + 5 tệp đồng hành và `.memlog.md` KHÔNG ĐỔI | `epics.md` `inputDocuments` (cả 6 tệp) và `companions` của SPEC.md. Không có `stories.yaml` ⇒ không dựng epic. Là nguồn Requirements (FR/NFR) cho initiative |
| Đề xuất đổi hướng | 18 | `planning-artifacts/sprint-change-proposal-*.md` (18 tệp) | `change-<slug>/change-<slug>.md`, bảng slug ở 5.5 | Được `sprint-status.yaml`, `epics.md` và ad-brief trích dẫn |
| AD brief / nháp | 7 | `planning-artifacts/ad-brief-*.md` (6), `ad-51-draft-2026-10-02.md` | `ad-brief-<slug>/ad-brief-<slug>.md` (6) và `ad-51-draft/ad-51-draft.md`, slug = tên bỏ ngày (5.5) | Hồ sơ bàn giao cho Winston để cấp AD mới (AD-51 đã vào spine theo chính `ad-51-draft`); Q21 |
| Báo cáo sẵn sàng | 1 | `planning-artifacts/implementation-readiness-report-2026-08-03.md` | `implementation-readiness-report/implementation-readiness-report.md` | Kiểu = tên tệp bỏ ngày (không có `type`); 5 tệp nhắc tới nó (`git grep`); `status: complete` giữ |
| Nghiên cứu | 2 | `research/technical-auratranslate-tauri-rust-local-first-research-2026-08-02.md`, `research/phase-0-spike-results-2026-08-02.md` | `research-technical-auratranslate-tauri-rust-local-first/…md`, `research-phase-0-spike-results/…md` | `inputDocuments` của tệp đầu nêu brief và addendum; tệp sau được 5 tệp planning và 5 bản ghi story trích |
| Nghiên cứu font (Story 1.1) | 5 | `research/font-spike-results-2026-08-03.md` + `research/font-spike-2026-08-03/*.png` (4) | `epic-nen-mong-…/research-font-spike-results/research-font-spike-results.md` + 4 `.png` cạnh nó giữ tên | Frontmatter `title: "Story 1.1 — Mũi thăm dò font…"`; 7 bản ghi story trích. Thuộc epic Nền móng |
| Số đo NFR18 (Story 2.4) | 2 | `research/2-4-so-do/{nfr18-4mib-kills.tsv, nfr18-4mib-wal.tsv}` | `epic-bien-tap-theo-segment/data-nfr18-4mib/data-nfr18-4mib.md` (bộ định tuyến) + 2 `.tsv` giữ tên | Tên mang "2-4" và NFR18, nhưng KHÔNG tệp nào trích nó (đo 0 tham chiếu) ⇒ gia nhập theo tên; Ice sửa được |
| Sổ nợ | 1 | `implementation-artifacts/deferred-work.md` | `deferred-work.md` ở gốc initiative, KHÔNG ĐỔI | Quy tắc; miễn viết lại đường dẫn (Q17) |
| Bằng chứng agent | 2 | `implementation-artifacts/agent-rules-evidence.md`, `agent-token-economics.md` | `evidence-agent-rules/evidence-agent-rules.md`, `evidence-agent-token-economics/evidence-agent-token-economics.md` | AGENTS.md (gốc + con) và `_bmad/custom/bmad-project-context.toml:20,25` trích; mức initiative |
| Retrospective | 8 | `implementation-artifacts/epic-<N>-retro-<ngày>.md` (N = 1, 2, 3, 4, 5, 6, 7, 11) | `epic-<slug>/epic-<slug>-retrospective.md` trong epic tương ứng | Quy tắc "retrospective" (không thư mục); frontmatter `epic: N` + `verdict` chỉ khi đã có (Epic 3 và 11 `accepted-with-open-items`; Epic 4, 5, 6, 7 `rejected`; Epic 1, 2 không có verdict ⇒ KHÔNG thêm) |
| Bản ghi build story | 127 | `implementation-artifacts/<N>-<M>-<slug>.md`, `spec-<N>-<M>-<slug>.md` (127) | `epic-<slug>/story-<slug>-plan.md`, một plan mỗi bản ghi | Mục 7 và 8; không bản nào được gấp (Q6 vô hiệu) |
| Spec lẻ không số story | 21 | `implementation-artifacts/spec-{ai-*,ca-wal-*,e2e-*,e7-*,epic-3-review-*,epic-5-retro-*,epic-7-retro-*,fix-hanviet-*}.md` (21) | 20 plan trong epic (entry mới) + 1 plan trong `backlog/` | Mục 7.3 (Q14) |
| Handoff phases/task0 | 29 | `implementation-artifacts/<N>-<M>[-lo-x]-phases-<ngày>.md` (21), `<N>-<M>-task0-<ngày>.md` (7), `epic-7-retro-r-5-phases-<ngày>.md` (1) — tổng 29 (22 phases + 7 task0) | `epic-<slug>/handoff-<slug-bản-ghi>-{phases|task0}/handoff-….md` | Mục 7.4. Ghi chú nội dung: "working notes for the next agent, never copied into the spec" ⇒ bằng chứng, không phải retrospective. Tên đích lấy từ bản ghi tương ứng chứ không từ tên handoff, vì bỏ ngày và số story thì mọi `phases`/`task0` trùng nhau |
| Bằng chứng lẻ | 6 | `ai-4-implementation-2026-09-15.patch`, `ai-4-loopback-1-history-2026-09-15.md`, `bmad-build-auto-result-5-13-….md`, `ho-so-dieu-tra-6-7-….md`, `ho-so-dieu-tra-6-10-….md`, `proposal-tauri-window-automation-2026-08-11.md` | Mục 7.4 | Được đề xuất đổi hướng 09-08/09-15, retro Epic 1 và bản ghi story trích |

### 5.2 Vào `inbox/` (3 tệp, mặc định CÓ; tạo `inbox/space.md`)

| Tệp | Đích | Ghi chú |
|---|---|---|
| `project-context.md` | `inbox/project-context.md` | Q19; nguyên trạng, không đổi tên, không frontmatter |
| `research/thu-xin-phep-hvtdtd.md` | `inbox/thu-xin-phep-hvtdtd.md` | Q20 |
| `party-mode/memories/installed/.memlog.md` | `inbox/archive-v6/.memlog.md` | Q20; memlog lạc không có gì bên cạnh |

`inbox/space.md` (frontmatter `type: space`, `title`, `created: 2026-10-06`) được tạo vì có tệp vào. Không có `intent-*.md`: không có tệp ý tưởng/intent rời; `brief-auratranslate/addendum.md` mang "PRD v8.0 nguyên văn do Ice cung cấp" nhưng là tệp đồng hành của brief nên đi theo brief; ⇒ không tạo `intent.md`.

### 5.3 Ở nguyên chỗ (464 tệp, không di chuyển)

| Nhóm | Tệp | Ghi chú |
|---|---|---|
| 14 thư mục `implementation-artifacts/<…>-ban-do/` (`2-2, 2-3, 2-4 (366), 2-5, 2-5b, 2-5d, 2-8, 2-9, 2-10, 5-14, 6-1, 6-12, 6-18, e7-r4`) | 456 | Q15. Gồm 333 tệp gitignore (`2-4-ban-do`) + 7 fixture HTML 6-1 + 3 `latest-run.log`: không bị đụng, nên không cần `mv` thường |
| 5 tệp HTML gốc `2-2-ban-do-editor.html`, `2-3-ban-do-vung-go.html`, `2-5-ban-do-hai-vach.html`, `2-5b-ban-do-luoi.html`, `3-4b-ban-do-danh-dau.html` | 5 | Cùng họ `-ban-do`; `segment_contract.rs:2613` nhắc `2-2-ban-do-editor.html` (chú thích) |
| 3 `.DS_Store` | 3 | Q22 |

Hệ quả: `implementation-artifacts/` và `planning-artifacts/ux-designs/` (chỉ `.DS_Store`) KHÔNG bị xoá; thư mục còn lại ở `implementation-artifacts/` chỉ chứa 14 thư mục `*-ban-do/`, 5 HTML và `.DS_Store`.

### 5.4 Lưu trữ `initiative-auratranslate/archive-v6/` (8 tệp, KHÔNG ĐỔI nội dung)

| Tệp | Ghi chú |
|---|---|
| `planning-artifacts/epics.md` | Nguồn tracking; sau khi đã đọc hết để viết entry |
| `implementation-artifacts/sprint-status.yaml` | Nguồn tracking; 44 action item mở được chuyển sang Notes (mục 10) |
| `implementation-artifacts/epic-3-context.md`, `epic-4-context.md`, `epic-5-context.md`, `epic-6-context.md`, `epic-7-context.md`, `epic-11-context.md` | Bộ nhớ đệm bối cảnh epic (6 tệp). Chúng còn được nhiều bản ghi trích trong `context:` (79 tệp trong store nhắc `epic-N-context`); sau chuyển, đường dẫn trong `context:` của tệp sống được viết lại tới `archive-v6/` |

Không tệp story nào bị gấp (Q6 vô hiệu). Không va chạm tên (5.5) ⇒ không tệp nào vào `archive-v6/` vì trùng tên.

### 5.5 Tên mới cho nhóm có ngày: slug và va chạm

Quy tắc va chạm tên (bỏ ngày/số mà trùng ⇒ bản ngày cao nhất giữ tên, bản khác vào `archive-v6/`): đã áp dụng THẬT. Các hậu tố `a/b/c` của đề xuất cùng ngày không phải khoá va chạm vì slug lấy từ TIÊU ĐỀ, và 18 slug dưới đây khác nhau từng đôi ⇒ 0 va chạm, 0 tệp bị đẩy vào archive. Slug cho tiêu đề chỉ có ngày lấy từ dòng §1 / phụ đề (cột cuối).

| Đề xuất (`sprint-change-proposal-` + …`.md`) | Thư mục đích `change-<slug>/` | Nguồn slug |
|---|---|---|
| `2026-08-05` | `change-duong-tieng-anh-bi-roi-khoi-epic` | Đường tiếng Anh bị rơi khỏi Epic 1 |
| `2026-08-11` | `change-ra-soat-tai-lieu-voi-ma-nguon` | rà soát tài liệu vs mã nguồn |
| `2026-08-13` | `change-auto-lookup-thu-ve-panel-source` | Auto-Lookup thu về Panel Source… |
| `2026-08-13b-thu-tu-epic` | `change-thu-tu-thuc-thi-epic` | Epic 4 (AI) lùi xuống sau đường nhập |
| `2026-08-14` | `change-be-mat-nhap-lat-sang-luoi-hai-cot` | Bề mặt nhập lật sang lưới hai cột |
| `2026-08-18` | `change-story-thieu-o-tang-quy-hoach` | tiêu đề chỉ có ngày; §1: "Story 2.12 tồn tại ở tầng thực thi nhưng không ở tầng quy hoạch" |
| `2026-08-18b-mo-hinh-hoan-tac` | `change-rut-hoan-tac-cho-gop-tach` | rút ⌘Z cho gộp/tách |
| `2026-08-18c-nfr2-be-mat-luoi` | `change-nfr2-be-mat-luoi` | tiêu đề chỉ có ngày; phụ đề: nửa NFR2 của 2.4 đo một bề mặt đã bị xoá |
| `2026-08-21-story-3-4b` | `change-tach-nua-giao-dien-fr50` | tách nửa giao diện FR50 thành Story 3.4b |
| `2026-08-25-story-3-10b` | `change-tach-nua-chon-tep-glossary` | tiêu đề chỉ có ngày; §1: nửa chọn tệp của xuất/nhập Glossary |
| `2026-09-02` | `change-bu-hai-story-thieu-trong-epics` | tiêu đề chỉ có ngày; §1: hai story Epic 1 đã dựng mà không có trong epics.md |
| `2026-09-08-story-6-10` | `change-bo-loc-can-xem-dung-tren-nang-luc-chua-dung` | tiêu đề chỉ có ngày; §1: AC của Story 6.10 đứng trên một năng lực chưa dựng |
| `2026-09-08b-nguyen-nhan-thu-nam` | `change-nguyen-nhan-can-xem-thu-nam` | Nguyên nhân "cần xem" thứ năm cho FR132 |
| `2026-09-15` | `change-bon-muc-cho-quyet-cua-retro` | Bốn mục chờ quyết của retro Epic 6 |
| `2026-09-24-epic-11-tra-no-nen` | `change-epic-tra-no-nen` | Epic 11 trả nợ nền |
| `2026-09-24b-no-dung-ten-ice` | `change-no-dung-ten-ice-ngoai-luot-ra-soat` | Nợ đứng tên Ice ngoài lượt rà |
| `2026-09-24b-phieu-quyet` | `change-phieu-quyet-no-dung-ten-ice-hang-p` | Phiếu quyết — nợ đứng tên Ice, hạng P |
| `2026-09-30-story-11-8` | `change-sua-loi-ranh-gioi` | Story 11.8 sửa lỗi ranh giới |

`ad-brief-2026-08-16-xuat-xu-ban-dich` → `ad-brief-xuat-xu-ban-dich`; `ad-brief-2026-08-17-mo-hinh-hoan-tac` → `ad-brief-mo-hinh-hoan-tac`; `ad-brief-2026-08-17-vach-le-cau-cuoi-chuong` → `ad-brief-vach-le-cau-cuoi-chuong`; `ad-brief-2026-08-24-hop-thoai-chon-tep` → `ad-brief-hop-thoai-chon-tep`; `ad-brief-2026-10-01-moc-so-xuat-xu-luu-phia-rust` → `ad-brief-moc-so-xuat-xu-luu-phia-rust`; `ad-brief-2026-10-02-diff-khop-mo-tm` → `ad-brief-diff-khop-mo-tm`; `ad-51-draft-2026-10-02` → `ad-51-draft`. Mọi tài liệu quy hoạch di chuyển được thêm frontmatter `type`, `title`, `status` (của nó, không có thì `done`), `created` (ngày trong tên thư mục/tệp, không có thì commit đầu, không có thì 2026-10-06), `skill` (skill v6 đã viết). `ad-brief` không có frontmatter ⇒ `type: ad-brief`, `created` = ngày trong tên, `skill: bmad-architecture`.

## 6. Cây đích

```
_bmad-output/
  initiative-auratranslate/
    initiative-auratranslate.md           # phong bì (mẫu initiative); covers = FR1–FR135 trừ FR20 (rút)
    tickets.toml                          # 11 [[epic]], thứ tự thực thi (Q9)
    deferred-work.md                      # không đổi
    prd-auratranslate/  brief-auratranslate/  ux-auratranslate/  architecture-auratranslate/  spec-auratranslate/
    plan-condense-spine/  worksheet-condense-spine/  implementation-readiness-report/
    research-technical-auratranslate-tauri-rust-local-first/  research-phase-0-spike-results/
    change-*/ (18)   ad-brief-*/ (6)   ad-51-draft/
    evidence-agent-rules/  evidence-agent-token-economics/
    epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi/  epic-bien-tap-theo-segment/  epic-glossary/
    epic-library/  epic-duong-nhap/  epic-ai-mo-va-smart-rag-injector/  epic-tra-no-nen/
    epic-translation-memory/  epic-cau-noi-reviewer/  epic-ai-proofreader/  epic-phat-hanh-va-tin-cay/
      # mỗi epic: epic-<slug>.md, tickets.toml, story-<slug>-plan.md / bug-<slug>-plan.md, handoff-*/, research-*/ | data-*/, epic-<slug>-retrospective.md
    archive-v6/   epics.md  sprint-status.yaml  epic-{3,4,5,6,7,11}-context.md
    migration-v6-v7/migration-v6-v7.md    # tệp này, chuyển vào ở nhóm commit 2
  backlog/    bug-fix-hanviet-parallel-row-overflow.md  bug-fix-hanviet-parallel-row-overflow-plan.md
  inbox/      space.md  project-context.md  thu-xin-phep-hvtdtd.md  archive-v6/.memlog.md
  implementation-artifacts/   # còn lại: 14 *-ban-do/, 5 *-ban-do-*.html, .DS_Store (Q15, Q22)
  planning-artifacts/ux-designs/.DS_Store   # còn lại
  .DS_Store
```

Không có `intent.md` (không có tệp intent). `backlog/` được tạo vì có 1 tệp; `backlog-<slug>/` không cần. Gốc store sau di trú chỉ còn: `initiative-auratranslate/`, `backlog/`, `inbox/`, `implementation-artifacts/` và `planning-artifacts/` (hai thư mục v6 còn lại chỉ chứa các tệp "ở nguyên chỗ" ở 5.3: khai là remnant được liệt kê, checklist mục 3), `.DS_Store`. `_bmad-output-bak/` nằm ngang hàng `_bmad-output/`, ngoài store.

## 7. Cây vé: `tickets.toml` của initiative và các epic

### 7.1 `initiative-auratranslate/tickets.toml` — 11 `[[epic]]` (thứ tự thực thi, Q9)

Phong bì initiative: tiêu đề "AuraTranslate"; Description/Outcome từ tóm tắt PRD; Done when từ mục tiêu PRD (Q24); References nêu PRD, kiến trúc, UX đã chuyển, spec; `covers` = mọi mã yêu cầu PRD (FR1–FR135 trừ FR20 đã rút; NFR1…). Notes mang 44 dòng `Action item:` (mục 10) và Decision/Parked: FR20 rút (2026-08-14), Epic 11 chạy sau Epic 4.

| `id` | `slug` | `title` | `covers` (FR từ `**FRs covered:**`) | `after` | `status` v7 |
|---|---|---|---|---|---|
| 1 | `epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi` | Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì | FR13, FR16, FR17, FR18, FR19, FR21, FR22, FR27, FR28, FR29, FR30, FR31, FR32, FR33, FR34, FR35, FR36, FR37, FR38, FR39, FR40, FR41, FR96, FR97, FR102, FR103, FR104, FR135 | — | in-progress |
| 2 | `epic-bien-tap-theo-segment` | Biên tập theo segment — một vòng dịch tay hoàn chỉnh | FR16, FR19, FR21, FR23, FR24, FR25, FR26, FR78, FR100, FR101, FR117, FR133, FR134 | — | in-progress |
| 3 | `epic-glossary` | Glossary — chốt thuật ngữ một lần, dùng mãi | FR46, FR47, FR48, FR49, FR50, FR51, FR52, FR53, FR55, FR113, FR114 | — | in-progress |
| 5 | `epic-library` | Library — kho tác phẩm, tìm kiếm, và đọc lại thành quả | FR1, FR2, FR3, FR4, FR5, FR6, FR7, FR8, FR9, FR10, FR11, FR12, FR15, FR98, FR99, FR119, FR120 | — | in-progress |
| 6 | `epic-duong-nhap` | Đường nhập — mọi nguồn văn bản vào được, và không hỏng im lặng | FR13, FR14, FR42, FR43, FR44, FR45, FR115, FR116, FR122, FR123, FR124, FR125, FR126, FR127, FR128, FR129, FR132 | — | in-progress |
| 4 | `epic-ai-mo-va-smart-rag-injector` | AI mở & Smart RAG Injector | FR65, FR66, FR67, FR68, FR69, FR70, FR71, FR72, FR73, FR74, FR75, FR76, FR77, FR79 | — | in-progress |
| 11 | `epic-tra-no-nen` | Trả nợ nền — đóng nợ đã hoãn của Epic 1–6 trước khi xây tiếp | — (Epic 11: "FRs covered: không") | — | in-progress |
| 7 | `epic-translation-memory` | Translation Memory — không dịch lại, không tra lại thứ đã dịch | FR56, FR57, FR58, FR59, FR60, FR61, FR62, FR63, FR64, FR118 | — | in-progress |
| 8 | `epic-cau-noi-reviewer` | Cầu nối Reviewer — xuất, nhập lại, đối chiếu, và hấp thụ bài học | FR54, FR87, FR88, FR89, FR90, FR91, FR92, FR93, FR94, FR95, FR121, FR130, FR131 | — | (không ghi, backlog) |
| 9 | `epic-ai-proofreader` | AI Proofreader — bắt lỗi trước khi bàn giao | FR80, FR81, FR82, FR83, FR84, FR85, FR86 | — | (không ghi, backlog) |
| 10 | `epic-phat-hanh-va-tin-cay` | Phát hành & tin cậy — vượt rào cản không ký số | FR105, FR106, FR107, FR108, FR109, FR110, FR111, FR112 | — | (không ghi, backlog) |

Ghi chú: (a) `status` lấy từ `sprint-status.yaml`; Epic 1–7 và 11 đều `in-progress` dù `epic-N-retrospective: done` (bình thường: còn story dở, hoặc chờ Ice ký) ⇒ mỗi dòng `epic-N-retrospective: done` thành dòng `Retrospective:` trong Notes epic, trỏ `epic-<slug>-retrospective.md`; Epic 8, 9, 10 `backlog` ⇒ không dòng `status`; retrospective `optional` của 8–10 bỏ qua. (b) `FR13` ⇄ Epic 1 và 6; `FR44`, `FR129` ⇄ Epic 6 và 7; `FR70` ⇄ Epic 4 và 7 (map dùng ký hiệu ⇄, nên bảng map của `epics.md` chứa FR13/FR44/FR70/FR129 ở dạng "FRn ⇄"); FR20 rút; FR135 nằm ở dòng `FRs covered` của Epic 1 nhưng không có hàng "FR135" trong map (regex đo được 129 hàng thường + 4 hàng ⇄ + FR20 gạch bỏ = 134; còn FR135 ⇒ soát tay lúc kiểm checklist mục 7, ghi vào Notes initiative nếu thật sự thiếu).
(c) Đối chiếu phủ trước khi làm: mọi FR trong 129 hàng thường của map xuất hiện ít nhất ở một `**Covers:**` của một story (0 FR mồ côi; 4 hàng ⇄ và FR135 soát tay lúc kiểm); FR trong dòng `FRs covered` của từng epic đều có story trích (0 thiếu).

### 7.2 Entry cho từng story (154)

Cột: `id` đúng như v6 viết (`"6a"` có chữ ⇒ đặt trong dấu nháy); `v6_key` = khoá sprint nguyên văn, tra ở `archive-v6/sprint-status.yaml` (cột bỏ để gọn; 6.18 không có khoá ⇒ `v6_key = "6-18-do-lai-nfr3-nfr4-nfr5-tren-thu-vien-5-000-chuong-that"`, lấy từ tên bản ghi); `v7` = trạng thái của plan (`—` = chỉ entry, KHÔNG dòng status); `record` = tệp bản ghi v6, plan mới = `story-<tên bản ghi bỏ "spec-" và "<N>-<M>-">` với đuôi `-plan.md`; `type*` = kiểu build đề xuất vì bản ghi không khai (`feature`, hoặc `chore` khi tên bắt đầu `mui-tham-do`/`do-`/`ci-`/`dong-goi`/`ha-tang`/`dung-du-lieu`/`scaffold`/`e2e`); `base` = `baseline_commit` có trong bản ghi (đã kiểm `git cat-file -e` mọi giá trị còn tồn tại trong kho) ⇒ viết thành `baseline_revision`; `covers` = FR/NFR/UX-DR trong dòng `**Covers:**` (mã AD/A#/Q# là ràng buộc, đi vào References, không vào `covers`). `description`, `verify`, `after = []`, `hitl = false`, `risk = "low"`, entry không có bản ghi giữ NGUYÊN AC gốc trong `description`: viết ở bước thi hành, từ `epics.md`.

#### Epic 1 — `epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi` — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Mũi thăm dò font — đo dung lượng thật và rà giấy phép | done | `1-1-mui-tham-do-font-do-dung-luong-that-va-ra-giay-phep.md` | chore* | có | NFR6, NFR15 |
| 2 | Scaffold dự án và khoá phạm vi filesystem, phạm vi mạng | done | `1-2-scaffold-du-an-va-khoa-pham-vi-filesystem-pham-vi-mang.md` | chore* | có | FR104, NFR12, NFR14 |
| 3 | CI tối thiểu — hai nền tảng, mỗi lần push | in-progress | `1-3-ci-toi-thieu-hai-nen-tang-moi-lan-push.md` | chore* | có | NFR14, NFR15, NFR6, FR107 |
| 4 | Bộ token màu và chữ hai theme, có kiểm tương phản tự động | done | `1-4-bo-token-mau-va-chu-hai-theme-co-kiem-tuong-phan-tu-dong.md` | feature* | có | NFR17, UX-DR1, UX-DR2, UX-DR3, UX-DR5, UX-DR6, UX-DR10, UX-DR11, UX-DR14, UX-DR16 |
| 5 | Tài nguyên chuỗi giao diện và hình dạng lỗi qua IPC | done | `1-5-tai-nguyen-chuoi-giao-dien-va-hinh-dang-loi-qua-ipc.md` | feature* | có | NFR16 |
| 6 | CommandRegistry, ba chế độ, và tiêu điểm bàn phím | done | `1-6-commandregistry-ba-che-do-va-tieu-diem-ban-phim.md` | feature* | có | FR22 |
| 7 | Tầng ghi dữ liệu — một writer nối tiếp và lược đồ có phiên bản | done | `1-7-tang-ghi-du-lieu-mot-writer-noi-tiep-va-luoc-do-co-phien-ban.md` | feature* | có | NFR10 |
| 8 | Phân giải cấu hình hai tầng | done | `1-8-phan-giai-cau-hinh-hai-tang.md` | feature* | có | FR103 |
| 9 | Dựng dữ liệu từ điển lớp nền | done | `1-9-dung-du-lieu-tu-dien-lop-nen.md` | chore* | có | FR27, NFR6 |
| 10 | Đóng gói bốn lớp gỡ rời thành file độc lập | done | `1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap.md` | chore* | có | FR27, FR36 |
| "10b" | Dựng dữ liệu từ điển tiếng Anh | done | `1-10b-dung-du-lieu-tu-dien-tieng-anh.md` | chore* | có | FR34, NFR6, NFR8 |
| "10c" | Âm Hán Việt — đúng nguồn và đúng nhãn | done | `1-10c-am-han-viet-dung-nguon-va-dung-nhan.md` | feature* | có | FR33, FR113, FR27, FR32, NFR6, NFR13, NFR14, NFR15 |
| 11 | Ba nhánh truy vấn tiếng Trung | done | `1-11-ba-nhanh-truy-van-tieng-trung.md` | feature* | có | FR39, NFR1 |
| "11b" | Đường tra cứu tiếng Anh | done | `1-11b-duong-tra-cuu-tieng-anh.md` | feature* | có | FR34, FR19, FR40 |
| 12 | Matcher dùng chung | done | `1-12-matcher-dung-chung.md` | feature* | có | FR40 |
| 13 | Đường tra cứu giữ nguyên bất đồng giữa các nguồn | done | `1-13-duong-tra-cuu-giu-nguyen-bat-dong-giua-cac-nguon.md` | feature* | có | FR29, FR30, FR31, FR32, FR34, FR35 |
| 14 | Khung bốn panel | done | `1-14-khung-bon-panel.md` | feature* | có | FR16, FR17, FR18 |
| 15 | Tác phẩm trên đĩa và đường vào văn bản tối thiểu | done | `1-15-tac-pham-tren-dia-va-duong-vao-van-ban-toi-thieu.md` | feature* | có | FR13, FR96, FR97, FR102 |
| 16 | Panel Source và tab Hán Việt | done | `1-16-panel-source-va-tab-han-viet.md` | feature* | có | FR19, FR33 |
| 17 | Panel Lookup — bản ghi có cấu trúc | done | `1-17-panel-lookup-ban-ghi-co-cau-truc.md` | feature* | có | FR28, FR32 |
| 18 | Auto-Lookup | done | `1-18-auto-lookup.md` | feature* | có | FR21 |
| "18b" | Tách từ tiếng Trung cho tab Hán Việt — double-click chọn CỤM TỪ | done | `1-18b-tach-tu-tieng-trung-tab-han-viet.md` | feature* | có | FR135, NFR1, NFR13, NFR14, NFR15, NFR16, NFR17 |
| 19 | Bật tắt nguồn từ điển và ghi công | done | `1-19-bat-tat-nguon-tu-dien-va-ghi-cong.md` | feature* | có | FR37, FR38 |
| 20 | Lịch sử tra cứu và mục đã ghim | in-progress | `1-20-lich-su-tra-cuu-va-muc-da-ghim.md` | feature* | có | FR41 |
| 21 | Phím tắt cấu hình lại được | in-progress | `1-21-phim-tat-cau-hinh-lai-duoc.md` | feature* | có | FR22 |
| 22 | Bộ chạy e2e trong webview thật | done | **BẢN GHI THIẾU** ⇒ plan tối thiểu `story-bo-chay-e2e-trong-webview-that-plan.md` (status `done`, ghi "bản ghi thiếu", không baseline) | — | — | — |

#### Epic 2 — `epic-bien-tap-theo-segment` — Biên tập theo segment — một vòng dịch tay hoàn chỉnh

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Tách segment cấp câu và cờ kết đoạn | done | `2-1-tach-segment-cap-cau-va-co-ket-doan.md` | feature* | có | FR23 |
| 2 | Panel Editor liền mạch | done | `2-2-panel-editor-lien-mach.md` | feature* | có | UX-DR19, UX-DR20, UX-DR2, UX-DR12, UX-DR7, UX-DR13 |
| 3 | Hợp đồng flush và trạng thái đã lưu | in-progress | `2-3-hop-dong-flush-va-trang-thai-da-luu.md` | feature* | có | FR100 |
| 4 | Mũi thăm dò — đo NFR18 và NFR2 đồng thời | in-progress | `2-4-mui-tham-do-do-nfr18-va-nfr2-dong-thoi.md` | chore* | có | NFR2, NFR18 |
| 5 | Xác nhận segment và máy trạng thái | done | `2-5-xac-nhan-segment-va-may-trang-thai.md` | feature* | có | FR24 |
| "5b" | Lưới hai cột đối chiếu | done | `2-5b-luoi-hai-cot-doi-chieu.md` | feature* | có | UX-DR13, UX-DR15, UX-DR19, FR16, FR19, FR21 |
| "5c" | Cắt bỏ câu khỏi bản dịch | done | `2-5c-cat-bo-cau-khoi-ban-dich.md` | feature* | có | FR133 |
| "5d" | Ngắt đoạn của bản dịch | done | `2-5d-ngat-doan-ban-dich.md` | feature* | có | FR134 |
| 6 | Lịch sử phiên bản segment và khôi phục | done | `2-6-lich-su-phien-ban-segment-va-khoi-phuc.md` | feature* | có | FR101 |
| 7 | Xuất xứ bản dịch cấp segment | done | `2-7-xuat-xu-ban-dich-cap-segment.md` | feature* | có | FR117 |
| 8 | Gộp và tách segment tường minh | done | `2-8-gop-va-tach-segment-tuong-minh.md` | feature* | có | FR78 |
| 9 | Gộp bằng `Backspace` ở đầu ô | done | `2-9-gop-bang-backspace-dau-o.md` | feature* | có | FR78 |
| 10 | Điều hướng segment | done | `2-10-dieu-huong-segment.md` | feature* | có | FR25 |
| 11 | Chuyển Chương trong Workspace | done | `2-11-chuyen-chuong-trong-workspace.md` | feature* | có | FR26 |
| 12 | Hạ tầng e2e và cổng còn thiếu | done | `2-12-ha-tang-e2e-va-cong-con-thieu.md` | chore* | có | — |
| 13 | Phân loại sổ nợ — 199 mục mồ côi và một luật đang bị phá | done | `2-13-phan-loai-so-no-va-luat-khong-mo-coi.md` | feature* | có | — |

#### Epic 3 — `epic-glossary` — Glossary — chốt thuật ngữ một lần, dùng mãi

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Mô hình Glossary hai tầng và vòng đời ba trạng thái | done | `3-1-mo-hinh-glossary-hai-tang-va-vong-doi-ba-trang-thai.md` | feature | có | FR46, FR47 |
| 2 | Bảng chờ ứng viên tách hẳn khỏi Glossary | done | `3-2-bang-cho-ung-vien-tach-han-khoi-glossary.md` | feature | có | FR55 |
| 3 | Thêm nhanh thuật ngữ từ bất kỳ panel nào | done | `3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md` | feature | có | FR48 |
| 4 | Khớp thuật ngữ theo ngôn ngữ qua Matcher dùng chung | done | `3-4-khop-thuat-ngu-theo-ngon-ngu-qua-matcher-dung-chung.md` | feature | có | FR51, FR50 |
| "4b" | Đánh dấu thuật ngữ ở cột nguyên văn của lưới | done | `3-4b-danh-dau-thuat-ngu-o-cot-nguyen-van-cua-luoi.md` | feature | có | FR50 |
| 5 | Quét ứng viên khi nhập tài liệu | done | `3-5-quet-ung-vien-khi-nhap-tai-lieu.md`; `spec-3-5-fix-review-findings.md` | feature, bugfix | có, có | FR52 |
| 6 | Trạng thái chờ chốt và dải mọc chốt lần đầu gặp | done | `3-6-trang-thai-cho-chot-va-dai-moc-chot-lan-dau-gap.md` | feature | có | FR114 |
| 7 | Đề xuất bản dịch bằng âm Hán Việt | done | `3-7-de-xuat-ban-dich-bang-am-han-viet.md` | feature | có | FR113 |
| 8 | Duyệt hàng loạt một phím | done | `3-8-duyet-hang-loat-mot-phim.md` | feature | có | FR53 |
| 9 | Quản lý Glossary | done | `3-9-quan-ly-glossary.md` | feature | có | FR49 |
| 10 | Xuất và nhập Glossary qua CSV/TSV | done | `3-10-xuat-va-nhap-glossary-qua-csv-tsv.md` | feature | có | FR49 |
| "10b" | Nối hộp thoại chọn tệp vào xuất/nhập Glossary | done | `3-10b-noi-hop-thoai-chon-tep-vao-xuat-nhap-glossary.md` | feature | có | FR49 |
| 11 | (entry MỚI, spec lẻ) Cụm A — sáu chỗ có bản vá đúng ở ngay cạnh mà chỗ này bỏ sót | done | `spec-epic-3-review-cum-a-khuon-bo-sot.md` | bugfix | có | — |
| 12 | (entry MỚI, spec lẻ) Cụm B — chín chỗ hỏng ở đường phân tích CSV/TSV và đường ghi tệp | done | `spec-epic-3-review-cum-b-csv-tsv-va-ghi-tep.md` | bugfix | có | — |
| 13 | (entry MỚI, spec lẻ) Cụm C — mất cập nhật im lặng ở nhịp hai của lượt nhập Glossary | done | `spec-epic-3-review-cum-c-dong-thoi-duong-commit-nhap.md` | bugfix | có | — |
| 14 | (entry MỚI, spec lẻ) Cụm D — mười một chỗ hỏng ở frontend Glossary: dây không ai kiểm, cờ kẹt, và một phím xoá  | done | `spec-epic-3-review-cum-d-guard-ipc-va-thao-tac-pha-huy.md` | bugfix | có | — |
| 15 | (entry MỚI, spec lẻ) Cụm E — ba lỗ hổng canh gác: cắt vệ đi rồi bộ test vẫn xanh trọn | done | `spec-epic-3-review-cum-e-le-hong-canh-gac.md` | bugfix | có | — |
| 16 | (entry MỚI, spec lẻ) Cụm F — mục rải rác bốn tầng, và ba phát hiện bị chính phép đo bác | done | `spec-epic-3-review-cum-f-muc-rai-rac-bon-tang.md` | bugfix | có | — |

#### Epic 5 — `epic-library` — Library — kho tác phẩm, tìm kiếm, và đọc lại thành quả

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Mô hình Library hai tầng | done | `5-1-mo-hinh-library-hai-tang.md` | chore | có | FR1, FR2, FR3, FR4 |
| 2 | Chỉ mục Library dẫn xuất, một đường ghi duy nhất | done | `5-2-chi-muc-library-dan-xuat-mot-duong-ghi-duy-nhat.md` | feature | có | FR98 |
| 3 | Quét lại thư mục | done | `5-3-quet-lai-thu-muc.md` | feature | có | FR99 |
| 4 | Bốn trạng thái vòng đời | done | `5-4-bon-trang-thai-vong-doi.md` | feature | có | FR5, FR6 |
| 5 | Tiến độ Tác phẩm | done | `5-5-tien-do-tac-pham.md` | feature | có | FR7 |
| 6 | Lưới Tác phẩm, lọc và sắp xếp | done | `5-6-luoi-tac-pham-loc-va-sap-xep.md` | feature | **thiếu** | FR10 |
| 7 | Danh sách Chương và mở Chương vào Workspace | done | `5-7-danh-sach-chuong-va-mo-chuong-vao-workspace.md` | feature | **thiếu** | FR12 |
| 8 | Tổ chức lại Chương sau khi nhập | done | `5-8-to-chuc-lai-chuong-sau-khi-nhap.md` | feature | **thiếu** | FR15 |
| 9 | Tìm kiếm full-text xuyên Library | done | `5-9-tim-kiem-full-text-xuyen-library.md` | feature | **thiếu** | FR8, NFR3 |
| 10 | Hai chế độ dấu | done | `5-10-hai-che-do-dau.md` | feature | **thiếu** | FR9 |
| 11 | Chế độ đọc — typography và bố cục đọc dài | done | `5-11-che-do-doc-typography-va-bo-cuc-doc-dai.md` | feature | **thiếu** | FR11 |
| 12 | Chế độ đọc chỉ đọc phần đã xong | done | `5-12-che-do-doc-chi-doc-phan-da-xong.md` | feature | **thiếu** | FR120 |
| 13 | Đánh dấu chỗ cần sửa khi đang đọc | done | `spec-5-13-danh-dau-cho-can-sua-khi-dang-doc.md` | feature | **thiếu** | FR119 |
| 14 | Đo NFR3, NFR4, NFR5 và ghi lại trạng thái ba ngưỡng tạm | done | `spec-5-14-do-nfr3-nfr4-nfr5-va-ghi-lai-trang-thai-ba-nguong-tam.md` | chore | có | NFR3, NFR4, NFR5 |
| 15 | (entry MỚI, spec lẻ) Retro Epic 5 · AI-2 + AI-3 — sàn phiên bản cho lượt thu hoạch, và bề mặt cho phần bị bỏ qu | done | `spec-epic-5-retro-ai-2-ai-3-san-phien-ban-va-be-mat-bo-qua.md` | bugfix | có | — |
| 16 | (entry MỚI, spec lẻ) e2e: no spec inherits the previous spec in-session state | done | `spec-e2e-cach-ly-trang-thai-giua-cac-spec.md` | bugfix | có | — |
| 17 | (entry MỚI, spec lẻ) e2e: the two form-driven specs create their Work through the import preview | done | `spec-e2e-g2-tao-tac-pham-qua-lop-xem-truoc.md` | bugfix | có | — |

#### Epic 6 — `epic-duong-nhap` — Đường nhập — mọi nguồn văn bản vào được, và không hỏng im lặng

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Mũi thăm dò ba lựa chọn thư viện | done | `spec-6-1-mui-tham-do-ba-lua-chon-thu-vien.md` | chore | có | — |
| 2 | Pipeline nhập một chuỗi thứ tự cố định, dùng chung mọi nguồn | done | `spec-6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon.md` | refactor | có | — |
| 3 | Bảng mã — phát hiện và dải đối chiếu năm bản dựng thật | done | `spec-6-3-bang-ma-phat-hien-va-dai-doi-chieu-nam-ban-dung-that.md` | feature | có | FR126 |
| 4 | Chuẩn hoá xuống dòng và khoảng trắng | done | `spec-6-4-chuan-hoa-xuong-dong-va-khoang-trang.md` | feature | có | FR125 |
| 5 | Luật làm sạch lộ ra và hiện thứ sắp xoá | done | `spec-6-5-luat-lam-sach-lo-ra-va-hien-thu-sap-xoa.md` | feature | có | FR124 |
| 6 | Tách Chương theo mẫu phân tách | done | `spec-6-6-tach-chuong-theo-mau-phan-tach.md` | feature | có | FR14 |
| "6b" | Nhập nhiều tệp cùng lúc | done | `spec-6-6b-nhap-nhieu-tep-cung-luc.md` | feature | có | FR14 |
| 7 | Nhập từ URL bằng danh sách link | done | `spec-6-7-nhap-tu-url-bang-danh-sach-link.md` | feature | có | FR122 |
| "7b" | Thêm Chương vào Tác phẩm có sẵn | done | `spec-6-7b-them-chuong-vao-tac-pham-co-san.md` | feature | có | FR122 |
| 8 | Allowlist mạng hai tầng và nhật ký domain | done | `spec-6-8-allowlist-mang-hai-tang-va-nhat-ky-domain.md` | feature | có | NFR19, NFR12 |
| 9 | Bóc nội dung chính và sửa ranh giới bằng bàn phím | done | `spec-6-9-boc-noi-dung-chinh-va-sua-ranh-gioi-bang-ban-phim.md` | feature | có | FR123 |
| "10a" | Xem trước theo từng Chương và điều hướng Chương | done | `spec-6-10a-xem-truoc-theo-tung-chuong-va-dieu-huong-chuong.md` | feature | có | FR132 |
| 10 | Bộ lọc "cần xem" | done | `spec-6-10-bo-loc-can-xem.md` | feature | có | FR132 |
| 11 | Ảnh tải về `.atproj`, neo vị trí, và URL gốc | done | `spec-6-11-anh-tai-ve-atproj-neo-vi-tri-va-url-goc.md` | feature | có | FR45, FR127 |
| 12 | Đọc `.docx` | done | `spec-6-12-doc-docx.md` | feature | có | FR13 |
| 13 | Alt-text và caption là hai `Segment` mang trường vai | done | `spec-6-13-alt-text-va-caption-la-hai-segment-mang-truong-vai.md` | feature | có | FR44, FR129 |
| 14 | Hiển thị ảnh đúng vị trí | done | `spec-6-14-hien-thi-anh-dung-vi-tri.md` | feature | có | FR42, FR43 |
| 15 | Xuất xứ tài liệu ở tầng Chương | done | `spec-6-15-xuat-xu-tai-lieu-o-tang-chuong.md` | feature | có | FR128 |
| 16 | Nhập tài liệu song ngữ hai cột | done | `spec-6-16-nhap-tai-lieu-song-ngu-hai-cot.md` | feature | có | FR115 |
| 17 | Khớp câu trong từng cặp hàng | done | `spec-6-17-khop-cau-trong-tung-cap-hang.md` | feature | có | FR116 |
| "16b" | Bộ lọc "cần xem" cho bản xem trước song ngữ | done | `spec-6-16b-bo-loc-can-xem-cho-ban-xem-truoc-song-ngu.md` | feature | có | FR132 |
| 18 | Đo lại NFR3, NFR4, NFR5 trên thư viện 5.000 Chương thật | in-progress | `spec-6-18-do-lai-nfr3-nfr4-nfr5-tren-thu-vien-5-000-chuong-that.md` | chore | có | — |
| "16c" | Nhập song ngữ từ bảng `.docx` | — | chỉ entry | — | — | FR115 |
| 19 | (entry MỚI, spec lẻ) store_contract WAL ceiling drifts with schema size, not with writes | done | `spec-ca-wal-do-tren-windows.md` | bugfix | có | — |
| 20 | (entry MỚI, spec lẻ) AI-4 — six import commands leave the UI thread, and the gate that says so stops missing wh | done | `spec-ai-4-sau-lenh-nhap-roi-luong-giao-dien.md` | bugfix | có | — |
| 21 | (entry MỚI, spec lẻ) AI-6 stage 1 — lift `wire` and `tests` out of `commands/project.rs` | done | `spec-ai-6-tach-commands-project-rs.md` | refactor | có | — |
| 22 | (entry MỚI, spec lẻ) AI-7 — one named trim for the ChapterOrigin rule, and the second divergence the 2026-09-07 | done | `spec-ai-7-mot-ham-cat-khop-trim-cua-javascript.md` | bugfix | có | — |

#### Epic 4 — `epic-ai-mo-va-smart-rag-injector` — AI mở & Smart RAG Injector

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Module `ai/` cô lập và test cưỡng chế ranh giới | in-review | `4-1-module-ai-co-lap-va-test-cuong-che-ranh-gioi.md` | feature | có | FR77 |
| 2 | Cấu hình nhà cung cấp AI | in-review | `spec-4-2-cau-hinh-nha-cung-cap-ai.md` | feature | có | FR65, FR66, FR68 |
| 3 | API key trong keychain | in-review | `spec-4-3-api-key-trong-keychain.md` | feature | có | FR67, NFR11 |
| 4 | Bộ prompt theo thể loại | in-review | `spec-4-4-bo-prompt-theo-the-loai.md` | feature | có | FR69 |
| 5 | Xuất và nhập bộ prompt | in-review | `spec-4-5-xuat-va-nhap-bo-prompt.md` | feature | có | FR79 |
| 6 | Smart RAG Injector là một hàm thuần | in-review | `spec-4-6-smart-rag-injector-ham-thuan.md` | feature | có | FR70 |
| 7 | Xem prompt cuối cùng đã gửi | in-review | `spec-4-7-xem-prompt-cuoi-cung-da-gui.md` | feature | có | FR71 |
| 8 | Dịch một segment với kết quả chảy dần | in-review | `spec-4-8-dich-mot-segment-voi-ket-qua-chay-dan.md` | feature | có | FR72, FR74 |
| 9 | Dịch theo lô và huỷ giữa chừng | in-review | `spec-4-9-dich-theo-lo-va-huy-giua-chung.md` | feature | có | FR73 |
| 10 | Lỗi mạng và lỗi API | in-review | `spec-4-10-loi-mang-va-loi-api.md` | feature | có | FR75 |
| 11 | Số token và ước tính chi phí | in-review | `spec-4-11-so-token-va-uoc-tinh-chi-phi.md` | feature | có | FR76 |
| 12 | Bố cục màn hình hẹp và hiệu chỉnh ngưỡng | in-review | `spec-4-12-bo-cuc-man-hinh-hep-va-hieu-chinh-nguong.md` | feature | có | UX-DR15 |

#### Epic 11 — `epic-tra-no-nen` — Trả nợ nền — đóng nợ đã hoãn của Epic 1–6 trước khi xây tiếp

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Trả nợ cổng và công cụ kiểm | done | `spec-11-1-lo-a-check-commands.md`; `spec-11-1-lo-b-rust-boundary.md`; `spec-11-1-lo-c-webview-gates.md`; `spec-11-1-lo-d-floors-and-remainder.md` | chore, chore, chore, chore | có, có, có, có | — |
| 2 | Trả nợ hạ tầng e2e và bộ chạy test | done | `spec-11-2-e2e-and-test-runner-debt.md` | chore | có | — |
| 3 | Trả nợ tra cứu và dữ liệu từ điển | done | `spec-11-3-lookup-and-dictionary-debt.md` | chore | có | — |
| 4 | Trả nợ Glossary | done | `spec-11-4-glossary-debt.md` | chore | có | — |
| 5 | Trả nợ editor, segment và tầng ghi | done | `spec-11-5-editor-segment-and-write-layer-debt.md` | chore | có | — |
| 6 | Trả nợ đường nhập và Library | done | `spec-11-6-lo-a-library-and-file-import.md`; `spec-11-6-lo-b-url-import-images-and-append.md` | chore, chore | có, có | — |
| 7 | Trả nợ nền giao diện dùng chung và AI | done | `spec-11-7-lo-a-shared-ui-foundation.md`; `spec-11-7-lo-b-ai-module.md` | chore, chore | có, có | — |
| 8 | Sửa lỗi ranh giới giữa các story của Epic 11 | done | `spec-11-8-lo-a-rust-boundary-and-guards.md`; `spec-11-8-lo-b-webview-e2e-ci.md` | bugfix, bugfix | có, có | — |

#### Epic 7 — `epic-translation-memory` — Translation Memory — không dịch lại, không tra lại thứ đã dịch

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Ghi TM tự động, khoá theo cặp văn bản | done | `spec-7-1-ghi-tm-tu-dong-khoa-theo-cap-van-ban.md` | feature | có | FR44, FR56, FR129 |
| 2 | Xuất xứ trên từng cặp TM | done | `spec-7-2-xuat-xu-tren-tung-cap-tm.md` | feature | có | FR118 |
| 3 | TM phạm vi kép và thứ tự sắp xếp hai khoá | done | `spec-7-3-tm-pham-vi-kep-va-thu-tu-sap-xep-hai-khoa.md` | feature | có | FR57 |
| 4 | Khớp tuyệt đối 100% | done | `spec-7-4-khop-tuyet-doi-100.md` | feature | có | FR58 |
| 5 | Khớp mờ | done | `spec-7-5-khop-mo.md` | feature | có | FR59 |
| 6 | Thuật toán khớp theo ngôn ngữ | done | `spec-7-6-thuat-toan-khop-theo-ngon-ngu.md` | feature | có | FR61 |
| 7 | Concordance | done | `spec-7-7-concordance.md` | feature | có | FR60 |
| 8 | Nhiều bản dịch cho cùng một câu nguồn | done | `spec-7-8-nhieu-ban-dich-cho-cung-mot-cau-nguon.md` | feature | có | FR63 |
| 9 | Quản lý Translation Memory | done | `spec-7-9-quan-ly-translation-memory.md` | feature | có | FR62 |
| 10 | Xuất và nhập TMX | done | `spec-7-10-xuat-va-nhap-tmx.md` | feature | có | FR64 |
| 11 | Smart RAG ưu tiên cặp của chính người dùng | done | `spec-7-11-smart-rag-uu-tien-cap-cua-chinh-nguoi-dung.md` | feature | có | FR70, FR118 |
| 12 | (entry MỚI, spec lẻ) Epic 7 retro R-2 — Chapter load never fails because of TM pre-fill | done | `spec-e7-r2-nap-chuong-khong-hong-vi-tm.md` | bugfix | có | — |
| 13 | (entry MỚI, spec lẻ) Epic 7 retro R-3 — TMX import/export stop holding OpenWorkState beyond the store access | done | `spec-e7-r3-thu-hep-khoa-openworkstate-tm.md` | bugfix | có | — |
| 14 | (entry MỚI, spec lẻ) Epic 7 retro R-4 — measure how long TM-reading paths hold OpenWorkState in the packaged re | done | `spec-epic-7-retro-r-4-measure-openworkstate-hold.md` | chore | có | — |
| 15 | (entry MỚI, spec lẻ) Epic 7 retro R-6 — one row per (source, target) in the fuzzy strip and Concordance | done | `spec-epic-7-retro-r-6-dedupe-fuzzy-and-concordance.md` | bugfix | **thiếu** | — |
| 16 | (entry MỚI, spec lẻ) Epic 7 retro R-9 — no bare `origin` identifier, and a guard that keeps it so | done | `spec-epic-7-retro-r-9-subject-bearing-origin-names.md` | refactor | có | — |
| 17 | (entry MỚI, spec lẻ) Epic 7 retro R-10 — debt ledger: close, correct and record the retro's ledger items | done | `spec-epic-7-retro-r-10-debt-ledger.md` | chore | có | — |
| 18 | (entry MỚI, spec lẻ) Epic 7 retro R-5 — TM management tells what a filtered action leaves untouched; TMX import | done | `spec-epic-7-retro-r-5-tm-honest-scope-and-tmx-ownership.md` | bugfix | có | — |

#### Epic 8 — `epic-cau-noi-reviewer` — Cầu nối Reviewer — xuất, nhập lại, đối chiếu, và hấp thụ bài học

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Mũi thăm dò thư viện diff | — | chỉ entry | — | — | NFR15 |
| 2 | Phạm vi xuất | — | chỉ entry | — | — | FR89 |
| 3 | Xuất `.docx` bảng hai cột theo segment | — | chỉ entry | — | — | FR87 |
| 4 | Xuất `.docx` một khối theo đoạn cho đăng bài | — | chỉ entry | — | — | FR121 |
| 5 | Chọn cách xuất hình ảnh | — | chỉ entry | — | — | FR130 |
| 6 | Xuất `.md` và text thuần | — | chỉ entry | — | — | FR88 |
| 7 | Khối ghi nguồn | — | chỉ entry | — | — | FR131 |
| 8 | Cổng kiểm hình dạng bảng `.docx` | — | chỉ entry | — | — | — |
| 9 | Nhập lại file reviewer đã sửa | — | chỉ entry | — | — | FR90 |
| 10 | Segment alignment — máy khớp, người sửa | — | chỉ entry | — | — | FR91 |
| 11 | Review Mode — bố cục hai cửa sổ side-by-side | — | chỉ entry | — | — | FR92 |
| 12 | Diff bôi màu, ẩn văn bản gốc | — | chỉ entry | — | — | FR93 |
| 13 | Chấp nhận từng thay đổi | — | chỉ entry | — | — | FR94 |
| 14 | Thu hoạch thuật ngữ từ bản review | — | chỉ entry | — | — | FR54 |
| 15 | Thu hoạch chạy độc lập với Review Mode | — | chỉ entry | — | — | FR95 |

#### Epic 9 — `epic-ai-proofreader` — AI Proofreader — bắt lỗi trước khi bàn giao

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Quét chính tả và ngữ pháp tiếng Việt | — | chỉ entry | — | — | FR80 |
| 2 | Đối chiếu bản dịch với bản gốc | — | chỉ entry | — | — | FR81 |
| 3 | Chạy theo yêu cầu, không chạy nền | — | chỉ entry | — | — | FR82 |
| 4 | Hình dạng một phát hiện và xử lý từng cái một | — | chỉ entry | — | — | FR83 |
| 5 | Hiển thị tại chỗ bằng gạch chân lượn sóng | — | chỉ entry | — | — | FR86 |
| 6 | Bỏ qua có ghi nhớ | — | chỉ entry | — | — | FR84 |
| 7 | Proofreader không tự sửa văn bản | — | chỉ entry | — | — | FR85 |
| 8 | Đo tỷ lệ báo động giả | — | chỉ entry | — | — | FR81 |

#### Epic 10 — `epic-phat-hanh-va-tin-cay` — Phát hành & tin cậy — vượt rào cản không ký số

| id | Tiêu đề | v7 | record (v6) | type | base | covers |
|---|---|---|---|---|---|---|
| 1 | Build công khai qua GitHub Actions | — | chỉ entry | — | — | FR107 |
| 2 | Phát hành cho macOS và Windows | — | chỉ entry | — | — | FR105 |
| 3 | Checksum SHA-256 | — | chỉ entry | — | — | FR106 |
| 4 | Màn hình Attribution | — | chỉ entry | — | — | FR109 |
| 5 | Giấy phép trong bản phát hành | — | chỉ entry | — | — | FR110 |
| 6 | Hướng dẫn cài đặt có ảnh chụp màn hình | — | chỉ entry | — | — | FR108 |
| 7 | Cập nhật chỉ kiểm tra và thông báo | — | chỉ entry | — | — | FR111 |
| 8 | Chính sách gỡ bỏ dữ liệu | — | chỉ entry | — | — | FR112 |
| 9 | Nghiệm thu cuối các ngưỡng phi chức năng | — | chỉ entry | — | — | NFR1, NFR19, NFR3, NFR4, NFR5 |

Ghi chú đặc biệt trong các bảng: **1.22** `done` không có bản ghi ⇒ plan tối thiểu ("bản ghi thiếu"), không baseline; **6.18** (Q10); **5.14** (Q12); **Epic 4 (12 story `review`)** (Q11) — v7 `in-review`; **4.1** chạy sau Epic 3 (Notes epic-4); **3.5** (hai bản ghi), **11.1, 11.6, 11.7, 11.8** (nhiều lô) (Q13); **10.9** nhận AC của 6.18 (Notes: gộp 2026-09-15); 33 entry `backlog` (6.16c, 8.1–8.15, 9.1–9.8, 10.1–10.9) không bản ghi ⇒ chỉ entry với AC gốc trong `description`.

### 7.3 Spec lẻ không số story (21) — đặt chỗ theo bằng chứng action item (Q14)

Id mới = số nguyên kế tiếp trên id số lớn nhất của epic (Epic 3 → 11…, Epic 5 → 15…, Epic 6 → 19…, Epic 7 → 12…), theo thứ tự bảng. Entry viết từ ý định của chính tệp (quy tắc: "entry viết từ ý định của tệp, kế hoạch gắn cờ"); `v6_key` bỏ trống. Kiểu entry: `bug` nếu `type: bugfix`, còn lại `story`; tên plan `bug-<slug>-plan.md` / `story-<slug>-plan.md`.

| Epic.id | Kiểu entry | Bản ghi v6 | Slug plan | Kiểu build | Baseline | Bằng chứng đặt chỗ |
|---|---|---|---|---|---|---|
| 3.11 | bug | `spec-epic-3-review-cum-a-khuon-bo-sot.md` | `bug-cum-a-khuon-bo-sot-plan.md` | bugfix | có | BẰNG CHỨNG YẾU: không action item nào nêu; chỉ có tên "epic-3-review-cum-*" và ngày 2026-08-25/26, ngay trước retro Epic 3 (2026-08-26) |
| 3.12 | bug | `spec-epic-3-review-cum-b-csv-tsv-va-ghi-tep.md` | `bug-cum-b-csv-tsv-va-ghi-tep-plan.md` | bugfix | có | BẰNG CHỨNG YẾU: không action item nào nêu; chỉ có tên "epic-3-review-cum-*" và ngày 2026-08-25/26, ngay trước retro Epic 3 (2026-08-26) |
| 3.13 | bug | `spec-epic-3-review-cum-c-dong-thoi-duong-commit-nhap.md` | `bug-cum-c-dong-thoi-duong-commit-nhap-plan.md` | bugfix | có | BẰNG CHỨNG YẾU: không action item nào nêu; chỉ có tên "epic-3-review-cum-*" và ngày 2026-08-25/26, ngay trước retro Epic 3 (2026-08-26) |
| 3.14 | bug | `spec-epic-3-review-cum-d-guard-ipc-va-thao-tac-pha-huy.md` | `bug-cum-d-guard-ipc-va-thao-tac-pha-huy-plan.md` | bugfix | có | BẰNG CHỨNG YẾU: không action item nào nêu; chỉ có tên "epic-3-review-cum-*" và ngày 2026-08-25/26, ngay trước retro Epic 3 (2026-08-26) |
| 3.15 | bug | `spec-epic-3-review-cum-e-le-hong-canh-gac.md` | `bug-cum-e-le-hong-canh-gac-plan.md` | bugfix | có | BẰNG CHỨNG YẾU: không action item nào nêu; chỉ có tên "epic-3-review-cum-*" và ngày 2026-08-25/26, ngay trước retro Epic 3 (2026-08-26) |
| 3.16 | bug | `spec-epic-3-review-cum-f-muc-rai-rac-bon-tang.md` | `bug-cum-f-muc-rai-rac-bon-tang-plan.md` | bugfix | có | BẰNG CHỨNG YẾU: không action item nào nêu; chỉ có tên "epic-3-review-cum-*" và ngày 2026-08-25/26, ngay trước retro Epic 3 (2026-08-26) |
| 5.15 | bug | `spec-epic-5-retro-ai-2-ai-3-san-phien-ban-va-be-mat-bo-qua.md` | `bug-san-phien-ban-va-be-mat-bo-qua-plan.md` | bugfix | có | `epic-5-retro-item-51/52` (AI-2, AI-3) |
| 5.16 | bug | `spec-e2e-cach-ly-trang-thai-giua-cac-spec.md` | `bug-e2e-cach-ly-trang-thai-giua-cac-spec-plan.md` | bugfix | có | retro Epic 5 AI-5 `epic-5-retro-item-54` (đưa e2e về xanh); `deferred-work` G1 |
| 5.17 | bug | `spec-e2e-g2-tao-tac-pham-qua-lop-xem-truoc.md` | `bug-e2e-tao-tac-pham-qua-lop-xem-truoc-plan.md` | bugfix | có | cùng AI-5; spec nêu "G2" của `deferred-work.md` |
| 6.19 | bug | `spec-ca-wal-do-tren-windows.md` | `bug-ca-wal-do-tren-windows-plan.md` | bugfix | có | `epic-6-retro-item-60` AI-1 nêu đích danh `spec-ca-wal-do-tren-windows` |
| 6.20 | bug | `spec-ai-4-sau-lenh-nhap-roi-luong-giao-dien.md` | `bug-sau-lenh-nhap-roi-luong-giao-dien-plan.md` | bugfix | có | `epic-6-retro-item-61` (AI-4) |
| 6.21 | story | `spec-ai-6-tach-commands-project-rs.md` | `story-tach-commands-project-rs-plan.md` | refactor | có | `epic-6-retro-item-67` (AI-6) |
| 6.22 | bug | `spec-ai-7-mot-ham-cat-khop-trim-cua-javascript.md` | `bug-mot-ham-cat-khop-trim-cua-javascript-plan.md` | bugfix | có | `epic-6-retro-item-62` (AI-7) |
| 7.12 | bug | `spec-e7-r2-nap-chuong-khong-hong-vi-tm.md` | `bug-nap-chuong-khong-hong-vi-tm-plan.md` | bugfix | có | `epic-7-retro-item-81` R-2 |
| 7.13 | bug | `spec-e7-r3-thu-hep-khoa-openworkstate-tm.md` | `bug-thu-hep-khoa-openworkstate-tm-plan.md` | bugfix | có | `epic-7-retro-item-82` R-3 |
| 7.14 | story | `spec-epic-7-retro-r-4-measure-openworkstate-hold.md` | `story-measure-openworkstate-hold-plan.md` | chore | có | `epic-7-retro-item-83` R-4 |
| 7.15 | bug | `spec-epic-7-retro-r-6-dedupe-fuzzy-and-concordance.md` | `bug-dedupe-fuzzy-and-concordance-plan.md` | bugfix | **thiếu** | `epic-7-retro-item-85` R-6 |
| 7.16 | story | `spec-epic-7-retro-r-9-subject-bearing-origin-names.md` | `story-subject-bearing-origin-names-plan.md` | refactor | có | `epic-7-retro-item-88` R-9 |
| 7.17 | story | `spec-epic-7-retro-r-10-debt-ledger.md` | `story-debt-ledger-plan.md` | chore | có | `epic-7-retro-item-89` R-10 |
| 7.18 | bug | `spec-epic-7-retro-r-5-tm-honest-scope-and-tmx-ownership.md` | `bug-tm-honest-scope-and-tmx-ownership-plan.md` | bugfix | có | `epic-7-retro-item-84` R-5 |
| backlog/ | bug | `spec-fix-hanviet-parallel-row-overflow.md` | `bug-fix-hanviet-parallel-row-overflow-plan.md` | bugfix | có | KHÔNG có action item hay story nào nêu (2026-09-25, giữa lúc Epic 11 chạy) ⇒ `backlog/` |

### 7.4 Handoff và bằng chứng lẻ — đích (29 + 6)

Mỗi tệp một thư mục `<kiểu>-<slug>/<kiểu>-<slug>.md` trong epic (frontmatter bổ sung: `type: handoff`, `title` = tiêu đề đầu tệp, `status: done`, `created` = ngày trong tên v6, `skill: bmad-build`, giả định vì tệp không ghi skill). `slug` = tên bản ghi tương ứng bỏ `spec-` và `<N>-<M>-`. `task0` của story nhiều lô gắn với bản ghi lô đầu theo thứ tự chữ.

| Epic | Tệp v6 | Thư mục đích |
|---|---|---|
| 4 | `4-10-phases-2026-09-22.md` | `handoff-loi-mang-va-loi-api-phases/` |
| 4 | `4-12-phases-2026-09-22.md` | `handoff-bo-cuc-man-hinh-hep-va-hieu-chinh-nguong-phases/` |
| 4 | `4-3-phases-2026-09-17.md` | `handoff-api-key-trong-keychain-phases/` |
| 4 | `4-4-phases-2026-09-17.md` | `handoff-bo-prompt-theo-the-loai-phases/` |
| 7 | `7-10-phases-2026-10-04.md` | `handoff-xuat-va-nhap-tmx-phases/` |
| 7 | `7-11-phases-2026-10-04.md` | `handoff-smart-rag-uu-tien-cap-cua-chinh-nguoi-dung-phases/` |
| 7 | `7-5-phases-2026-10-02.md` | `handoff-khop-mo-phases/` |
| 7 | `7-7-phases-2026-10-03.md` | `handoff-concordance-phases/` |
| 7 | `7-8-phases-2026-10-03.md` | `handoff-nhieu-ban-dich-cho-cung-mot-cau-nguon-phases/` |
| 7 | `7-9-phases-2026-10-04.md` | `handoff-quan-ly-translation-memory-phases/` |
| 7 | `epic-7-retro-r-5-phases-2026-10-06.md` | `handoff-tm-honest-scope-and-tmx-ownership-phases/` |
| 11 | `11-1-lo-d-phases-2026-09-25.md` | `handoff-lo-d-floors-and-remainder-phases/` |
| 11 | `11-2-phases-2026-09-26.md` | `handoff-e2e-and-test-runner-debt-phases/` |
| 11 | `11-2-task0-2026-09-25.md` | `handoff-e2e-and-test-runner-debt-task0/` |
| 11 | `11-3-phases-2026-09-26.md` | `handoff-lookup-and-dictionary-debt-phases/` |
| 11 | `11-3-task0-2026-09-26.md` | `handoff-lookup-and-dictionary-debt-task0/` |
| 11 | `11-4-phases-2026-09-26.md` | `handoff-glossary-debt-phases/` |
| 11 | `11-4-task0-2026-09-26.md` | `handoff-glossary-debt-task0/` |
| 11 | `11-5-phases-2026-09-27.md` | `handoff-editor-segment-and-write-layer-debt-phases/` |
| 11 | `11-5-task0-2026-09-27.md` | `handoff-editor-segment-and-write-layer-debt-task0/` |
| 11 | `11-6-lo-a-phases-2026-09-28.md` | `handoff-lo-a-library-and-file-import-phases/` |
| 11 | `11-6-lo-b-phases-2026-09-28.md` | `handoff-lo-b-url-import-images-and-append-phases/` |
| 11 | `11-6-task0-2026-09-28.md` | `handoff-lo-a-library-and-file-import-task0/` |
| 11 | `11-7-lo-a-phases-2026-09-29.md` | `handoff-lo-a-shared-ui-foundation-phases/` |
| 11 | `11-7-lo-b-phases-2026-09-30.md` | `handoff-lo-b-ai-module-phases/` |
| 11 | `11-7-task0-2026-09-29.md` | `handoff-lo-a-shared-ui-foundation-task0/` |
| 11 | `11-8-lo-a-phases-2026-09-30.md` | `handoff-lo-a-rust-boundary-and-guards-phases/` |
| 11 | `11-8-lo-b-phases-2026-10-01.md` | `handoff-lo-b-webview-e2e-ci-phases/` |
| 11 | `11-8-task0-2026-09-30.md` | `handoff-lo-a-rust-boundary-and-guards-task0/` |
| 1 | `proposal-tauri-window-automation-2026-08-11.md` | `proposal-tauri-window-automation/` (sinh từ retro Epic 1 A4/A5/A7) |
| 5 | `bmad-build-auto-result-5-13-danh-dau-cho-can-sua-khi-dang-doc.md` | `report-danh-dau-cho-can-sua-khi-dang-doc/` (`type: report`; bản ghi 5.13 có `status: done`) |
| 6 | `ho-so-dieu-tra-6-7-nhap-tu-url-2026-09-06.md` | `investigation-nhap-tu-url-bang-danh-sach-link/` |
| 6 | `ho-so-dieu-tra-6-10-bo-loc-can-xem-2026-09-08.md` | `investigation-bo-loc-can-xem/` (đề xuất 09-08 trích) |
| 6 | `ai-4-loopback-1-history-2026-09-15.md` + `ai-4-implementation-2026-09-15.patch` | `history-sau-lenh-nhap-loopback/history-sau-lenh-nhap-loopback.md` + `implementation.patch` cùng thư mục (không đổi nội dung); gắn entry 6.20 (AI-4). Lưu ý `.gitignore` chặn `_bmad-output/*.patch` ở GỐC `_bmad-output/`, không chặn thư mục con ⇒ patch đi theo git bình thường |

`epic-7-retro-r-5-phases-<ngày>.md` KHÔNG vào luật retrospective như ghi chú đầu việc dự đoán: nó là ghi chú bàn giao pha của spec R-5 ("Working notes for the next agent … Nothing is committed"), không phải retro ⇒ `handoff` trong epic-translation-memory. Hai tệp `epic-7-retro-2026-10-05.md` và các `epic-N-retro-<ngày>.md` khác mới là retrospective.

## 8. Ánh xạ bản ghi build (v6 → v7)

Frontmatter plan: `ticket` = id entry (plan trong `backlog/`: gốc tên tệp); `status` theo bảng dưới; `type` = kiểu build (`feature`/`bugfix`/`refactor`/`chore`, KHÔNG `story`); `baseline_revision` = giá trị `baseline_commit` của bản ghi khi có (đã kiểm: 119/127 bản ghi có, cả 119 giá trị đều là commit tồn tại trong kho; 7 ký tự viết tắt cũng phân giải được), thiếu thì BỎ khoá và ghi vào báo cáo. Giữ nguyên toàn bộ tiêu chí, lịch sử cài đặt, phát hiện review; AC từ `epics.md` được gộp vào. Phần `<frozen-after-approval>` giữ nguyên. Không bao giờ suy baseline từ HEAD.

| v6 (`sprint-status` ⊕ bản ghi) | Số bản ghi | v7 `status` |
|---|---|---|
| `done` | 109 (gồm cả bản ghi phụ của 3.5, 11.1, 11.6–11.8) | `done` |
| `review` (12 story Epic 4; bản ghi nói `done`) | 12 | `in-review` (Q11) |
| `in-progress` (1-3, 1-20, 1-21, 2-3, 2-4) | 5 | `in-progress` (bản ghi không có `status:` ⇒ lấy từ sprint) |
| 6.18 (không khoá sprint; bản ghi `in-progress`) | 1 | `in-progress` (Q10) |
| `backlog` | 0 (33 story không có bản ghi) | không plan; entry không dòng status |
| `ready-for-dev` / `drafted` / `contexted` / `blocked` | 0 | — (không giá trị nào xuất hiện ở sprint hay bản ghi) |

Đối chiếu bản ghi ↔ sprint: lệch 14 chỗ, đã xử lý: 12 Epic 4 (Q11), 5.14 (Q12), 6.18 (Q10). Tổng bản ghi story = 127 (120 story có bản ghi; 7 bản ghi phụ ở 3.5, 11.1, 11.6, 11.7, 11.8). Cộng 21 spec lẻ (mục 7.3), 1 plan tối thiểu cho 1.22 ⇒ **149 plan** sau di trú (127 + 21 + 1). Baseline: 119 trong 127 bản ghi story có; trong 21 spec lẻ, 20 có và `spec-epic-7-retro-r-6-…` (oneshot) thiếu; plan 1.22 thiếu ⇒ 10 plan không `baseline_revision` (8 + 1 + 1), 139 có.

Kiểu build đề xuất: 66 bản ghi `feature`, 16 `chore`, 3 `bugfix`, 1 `refactor` khai sẵn trong frontmatter; **41 bản ghi không khai `type`** (toàn bộ Epic 1: 25 và Epic 2: 16) ⇒ đề xuất theo heuristic ở 7.2 (cột `type*`), Ice sửa được.
Baseline THIẾU (8 bản ghi story, cộng r-6 và 1.22 như trên): `5-6`, `5-7`, `5-8`, `5-9`, `5-10`, `5-11`, `5-12` (Story 5.6–5.12) và `spec-5-13-…` (Story 5.13) — cùng khối Epic 5 không có `baseline_commit` ⇒ plan không có `baseline_revision`, liệt kê trong báo cáo cuối. Ngoài ra 1.22 (plan tối thiểu, bản ghi thiếu) cũng không có baseline.

## 9. Story chưa bắt đầu có tệp (câu hỏi 6)

Đo: 33 khoá `backlog` trong `sprint-status.yaml` (6-16c; 8-1…8-15; 9-1…9-8; 10-1…10-9) — tìm `ls implementation-artifacts | grep -E "^(spec-)?(6-16c|8-|9-|10-)"` trả 0 tệp. Không có tệp để gấp, không tệp nào vào `archive-v6/` vì lý do này. Mọi entry `backlog` giữ AC gốc đầy đủ trong `description`.

## 10. Action item → dòng `Action item:` trong Notes initiative

93 mục: 49 `done` (bỏ), **44** mở/đang làm ⇒ 44 dòng `Action item:` (dạng `Action item: <nội dung rút gọn> — owner: <owner>; epic: <N>; status: <open|in-progress>; id: <id nếu có>`), nội dung giữ nguyên văn không rút gọn ý. Theo epic: Epic 1: 5, Epic 2: 6, Epic 3: 10, Epic 5: 7, Epic 6: 4, Epic 7: 6, Epic 10: 1, Epic 11: 5. Trạng thái: 40 `open`, 4 `in-progress` (1 của Epic 1 "Bộ e2e chấp chọn" và 3 của Epic 5: AI-5, AI-6, AI-7). Các mục từ `epic-3-retro-item-35` trở đi có `id`; mục cũ hơn không có id, giữ theo thứ tự tệp. Mỗi mục mang owner (Ice, Dev, Winston, Amelia, "Ice · Amelia") và epic của nó. Sau đó `sprint-status.yaml` mới được lưu trữ.

Hệ quả cần biết: `check-debt-owner` coi `Chủ: B7`, `Chủ: Epic N`, `Chủ: Story X.Y` là mục sprint-status; sau khi lưu trữ, các chủ đó chỉ còn ở plan/entry (mục 13.3).

## 11. Nhóm commit (theo thứ tự, guide bước 4) và cách di chuyển

Kho đã tồn tại và đang sạch ⇒ không cần commit "trạng thái hiện tại" trước. Mỗi nhóm một commit, thông điệp theo quy ước kho `type(scope): câu tiếng Việt nêu điều tìm thấy` (id story dạng `5-9`), kèm dòng đồng tác giả. Di chuyển: `git mv` cho tệp được git theo dõi (431); tệp bị gitignore (346) KHÔNG `git mv` được ⇒ `mv` thường, và vì cả 346 tệp đó nằm trong các thư mục "ở nguyên chỗ" (mục 5.3) hay là `.DS_Store`, di trú không cần `mv` thường nào ở chế độ mặc định (Q15); chỉ khi Ice chọn Q15-B thì 6-1 (7 tệp), 5-14, 6-18, e7-r4 và 2-4 (333 tệp) cần `mv` thường.

| # | Nhóm | Nội dung | Tệp chạm |
|---|---|---|---|
| 0 | (không commit) | Tiền đề: Q7 `bmad setup` làm mới; sao lưu (mục 4) | — |
| 1 | Kho (workspace) | KHÔNG CÓ (câu 3, 4 đều "không") | 0 |
| 2 | Initiative và phong bì | `initiative-auratranslate/` + `initiative-auratranslate.md` + `tickets.toml` (11 `[[epic]]`); chuyển kế hoạch này vào `migration-v6-v7/migration-v6-v7.md` | 2 mới + 1 chuyển |
| 3 | Tài liệu quy hoạch | prd, brief, ux, architecture (+2 spine-condense), spec, readiness, research (3 initiative), 18 change, 6 ad-brief + ad-51-draft, 2 evidence-agent; thêm frontmatter, bộ định tuyến `ux-auratranslate.md`, danh sách anh em trong `architecture-auratranslate.md` | 103 chuyển (+ 1 router `ux-auratranslate.md`) |
| 4 | Epic, entry, plan | 11 `epic-<slug>/` (+ `epic-<slug>.md`, `tickets.toml` có 154 + 20 entry), 148 plan (127 bản ghi + 20 spec lẻ + 1 plan tối thiểu 1.22), 8 retrospective, handoff/bằng chứng (35), `research-font`, `data-nfr18-4mib` | 197 chuyển + 11 `epic-<slug>.md` + 11 `tickets.toml` + 1 plan tối thiểu mới |
| 5 | Tệp lẻ và inbox | `backlog/` (entry + plan của spec hanviet), `inbox/` (3 tệp + `space.md`), `deferred-work.md` | 5 chuyển + `space.md` mới |
| 6 | Lưu trữ | `archive-v6/` ← `epics.md`, `sprint-status.yaml`, 6 `epic-N-context.md`; dòng `Action item:` đã vào Notes trước đó | 8 |
| 7 | Viết lại đường dẫn trong store | mục 12 | tới 207 tệp (đo ở mục 12) |
| 8 | Cấu hình | `_bmad/custom/config.user.toml` và `bmad-project-context.toml`: cả thư mục `_bmad/` bị gitignore (`.gitignore:86`, 0 tệp được theo dõi) ⇒ KHÔNG có commit; sửa tại chỗ sau khi duyệt | 0 commit |

Xoá thư mục v6 rỗng còn lại (sau khi dọn `specs/`, `party-mode/`, `planning-artifacts/` trừ `.DS_Store`). Chạy `checklist` sau nhóm 8 (guide bước 5), ghi kết quả vào mục 15, đặt `status: done`, báo cáo (guide bước 7). KHÔNG push.

## 12. Viết lại đường dẫn bên trong store

Đây là phép TÌM KIẾM, không đọc từng tài liệu. Đo hôm nay (`git grep` trong `_bmad-output/`): 1.032 dòng / 207 tệp chứa đường dẫn tuyệt đối `_bmad-output/{planning-artifacts|implementation-artifacts|specs}/…` hay `{project-root}/_bmad-output`; 115 tệp có khoá frontmatter `context:` / `inputDocuments:` / `companions:` / `relates_to:` / `sources:`; 109 tệp nhắc `ARCHITECTURE-SPINE`; 113 tệp nhắc `sprint-status`; 149 tệp nhắc `epics.md`; 79 tệp nhắc `epic-N-context`.

Cách làm: dựng bảng ánh xạ cũ→mới từ chính danh sách di chuyển (mục 5, 7), rồi (1) `git grep -nE` từng đường dẫn cũ (đầy đủ và tên tệp trần có dấu `/` hoặc trong backtick/frontmatter); (2) viết lại CHỈ tham chiếu sống: khoá frontmatter `companions:`, `inputDocuments:`, `context:`, `relates_to:`, `ref:`, phần References, liên kết văn xuôi dạng đường dẫn; (3) với mỗi tệp đã chuyển ngoài `inbox/` và `archive-v6/`, tính lại đường dẫn tương đối từ thư mục mới sao cho vẫn tới đúng đích; (4) KHÔNG sửa: `inbox/`, `archive-v6/`, `deferred-work.md` (12.872 dòng, được trích theo số dòng), khối `<frozen-after-approval>` (ý định do người giữ), và lời nhắc lịch sử trần không phải đường dẫn (`§Deferred from: code review of 1-1-…`, "xem `deferred-work.md`"). Sau cùng chạy lại `git grep` từng đường dẫn cũ ⇒ phải còn 0 tham chiếu sống chưa viết lại; chỗ chết sẵn từ v6 liệt kê riêng.

Đích đặc biệt: `epic-N-context.md` ⇒ `archive-v6/epic-N-context.md`; `epics.md`, `sprint-status.yaml` ⇒ `archive-v6/…`; `ARCHITECTURE-SPINE.md` ⇒ `architecture-auratranslate/architecture-auratranslate.md`; `SPEC.md` ⇒ `spec-auratranslate/spec-auratranslate.md`; `prd.md` ⇒ `prd-auratranslate/prd-auratranslate.md`; `<N>-<M>-<slug>.md`/`spec-…` ⇒ `epic-<slug>/story-<slug>-plan.md`; `*-ban-do/` KHÔNG đổi.

## 13. Tham chiếu bên ngoài store (chỉ liệt kê, không sửa)

Lệnh: `git grep -nE '_bmad-output|implementation-artifacts|planning-artifacts|sprint-status\.yaml|deferred-work\.md|ARCHITECTURE-SPINE|epic-[0-9]+-context\.md' -- . ':(exclude)_bmad-output' ':(exclude)_bmad'` ⇒ **498 dòng ở 175 tệp**. Phần lớn là chú thích/văn xuôi trong mã; mục 13.1 là ĐỌC MÁY (hỏng thật), 13.2 là văn xuôi.

### 13.1 Đọc máy sẽ hỏng (tệp:dòng, lý do)

| Tệp:dòng | Đọc cái gì | Hỏng khi |
|---|---|---|
| `scripts/check-debt-owner.mjs:111` (`join(repoRoot,'_bmad-output','implementation-artifacts','deferred-work.md')`), `:115` `REAL_DEBT_PATH`, `:598` `MAC_DINH` | sổ nợ | `deferred-work.md` chuyển vào initiative ⇒ `readFileSync` (`:651`) ném ⇒ cổng `debt-owner` đỏ. Cổng này là cổng thứ 12 của `.githooks/pre-push:53` và bước CI `ci.yml:268` ⇒ chặn mọi push (không `--no-verify`) |
| `scripts/check-debt-owner.mjs:116` `SPRINT_STATUS_PATH`; `:661` `parseSprintStatus(readFileSync(...))`; regex `:212-214` (`SPRINT_STORY_RE`, `SPRINT_EPIC_RE`, `SPRINT_RETRO_RE`) và `:666-668` `SPRINT_KEY_FLOOR = 139` | trạng thái story/epic cho **Kiểm C** (mục nợ MỞ/🟡 mà `Chủ:` cuối trỏ story/epic đã `done` hoặc vắng) | `sprint-status.yaml` được lưu trữ ⇒ đường dẫn chết; về mặt khái niệm, trạng thái story nay nằm ở plan `status` và `tickets.toml`, không còn ở một tệp phẳng; sàn 139 khoá cũng không còn ý nghĩa |
| `scripts/test-story.mjs:73` `SPEC_DIR`; `findSpec(id)` (`:90-95`) | `readdirSync(SPEC_DIR)` tìm `spec-<N>-<M>-*` | bản ghi chuyển vào `epic-<slug>/story-<slug>-plan.md` (tên không còn số story) ⇒ `npm run test:story <id>` không tìm thấy lời khai spec (không ném, chỉ mất việc chọn test); các bản ghi Epic 1–3 vốn không mang tiền tố `spec-` |
| `src-tauri/tests/docx_probe.rs:38` | `../_bmad-output/implementation-artifacts/6-12-ban-do` | KHÔNG hỏng khi mặc định Q15 (ở nguyên chỗ); hỏng nếu thư mục chuyển |
| `src-tauri/tests/webimport_probe.rs:72`, `webimport_contract.rs:1461-1463` | `…/implementation-artifacts/6-1-ban-do` (cả fixture HTML gitignore `a01…a07`) | như trên |
| `.githooks/pre-push:34` (nhánh awk: tệp dưới `_bmad-output/` hoặc đuôi `.md` thì bỏ qua) và `:33` (nhánh `-ban-do/`) | quy tắc "chỉ tài liệu" bỏ qua vitest/build/cargo test | KHÔNG hỏng: initiative ở dưới `_bmad-output/` nên vẫn miễn trừ; `-ban-do/` giữ tên; lưu ý tệp `.toml`/`.patch` dưới `_bmad-output/` cũng được coi là tài liệu (đã đúng từ v6) |
| `.githooks/pre-push:53` / `package.json:26` / `ci.yml:268` | chạy `check:debt-owner` | hậu quả của dòng đầu bảng |
| `.gitignore:80-83` | `_bmad-output/*.diff`, `*.patch`, `append.py`, `adversarial_findings.md` | chỉ khớp ở GỐC `_bmad-output/`, không ảnh hưởng thư mục con; không hỏng |
| `src-tauri/tests/naming_boundary.rs:1010` | `AGENTS.md` GỐC (không phải store) | KHÔNG hỏng |
| `_bmad/custom/bmad-project-context.toml:20,25` | `external_sources = file:{project-root}/_bmad-output/implementation-artifacts/agent-token-economics.md`; câu nhắc `_bmad-output/implementation-artifacts/agent-rules-evidence.md` | nằm trong `_bmad/custom/` ⇒ theo quy tắc cấu hình, được VIẾT LẠI khi duyệt (mục 14), không phải "liệt kê" |

Đã quét thêm `scripts/`, `.githooks/`, `.github/`, `src-tauri/tests/`, `e2e/`, `tools/`, `package.json`, `vitest.config.ts`, `src/`, `tests/`: ngoài các mục trên, không có tệp nào ĐỌC đường dẫn store bằng mã (mọi dòng còn lại là chú thích hay chuỗi in ra).

### 13.2 Tham chiếu văn xuôi (đếm theo mục tiêu; ước lượng bằng `git grep`, ở mức dòng/tệp)

| Mục tiêu bị nhắc | Dòng | Tệp | Bị ảnh hưởng thế nào sau di trú |
|---|---|---|---|
| `deferred-work.md` | 405 | 149 | Tên giữ nguyên, nhưng đường dẫn đổi; hầu hết là lời nhắc trần ⇒ vẫn đọc được |
| `ARCHITECTURE-SPINE.md` | 43 | 28 | Tên đổi thành `architecture-auratranslate.md` (vd. `src-tauri/Cargo.toml:102,135,184`, `scripts/check-tokens.mjs:11,20`, `tools/dict-build/README.md:198`) |
| `project-context.md` | 41 | 24 | Q19 (vào inbox) |
| `*-ban-do` (14 thư mục) | 54 | 29 | không đổi (Q15) |
| `epics.md` | 36 | 19 | lưu trữ; nhiều chỗ là ràng buộc `check-doc-refs` cấm neo số dòng, không phải đường dẫn |
| `sprint-status` | 20 | 3 | lưu trữ (chủ yếu `check-debt-owner.mjs`, mục 13.1) |
| `epic-N-context.md` | 8 | 7 | lưu trữ |
| `agent-rules-evidence.md` / `agent-token-economics.md` | 10 | 8 | AGENTS.md gốc và các AGENTS.md con (`src/`, `scripts/`, `tests/`, `e2e/`, `tools/dict-build/`…) cùng `.githooks/pre-push:5`: tên đổi thành thư mục `evidence-*/` |
| `implementation-artifacts/<bản ghi story>` | 6 | 6 | đường dẫn chết |
| `planning-artifacts` hay `_bmad-output` (đường dẫn trần) | 3 + 35 | 3 + 22 | tuỳ mục tiêu |
| `sprint-change-proposal` | 1 | 1 | thư mục `change-*` |
| `ad-brief` | 3 | 3 | Q21 |

Tệp ngoài store có nhiều tham chiếu nhất: `scripts/check-debt-owner.mjs` (24 dòng), `src-tauri/src/commands/project/mod.rs` (18), `src/commands/index.ts` (15), `src-tauri/src/commands/segment.rs` (12), `src/panels/editorPanelState.ts` (11), `src-tauri/tests/cleanup_contract.rs` (10); toàn bộ là chú thích. Danh sách đầy đủ có thể tái sinh bằng lệnh `git grep` ở trên.

### 13.3 Việc theo dõi do Ice sở hữu (mã/tài liệu ngoài di trú, KHÔNG làm trong di trú)

1. **Viết lại `scripts/check-debt-owner.mjs`** (bắt buộc trước push kế tiếp): trỏ Kiểm A tới `initiative-auratranslate/deferred-work.md`; viết lại **Kiểm C**: hiện nó đòi mọi chủ `Story X.Y` / `Epic N` còn tồn tại trong `sprint-status.yaml` (`:201-214`, `:234`, `:502-503`) ⇒ cần đọc từ plan `status` + `tickets.toml` (hoặc `tickets.py status`); `SPRINT_KEY_FLOOR = 139` đổi theo; bộ tự kiểm giả (`:479`) phải đổi theo. Đây là thay đổi mã, có gác cổng riêng.
2. `scripts/test-story.mjs:73-95`: `findSpec` phải tìm plan theo `ticket:`/tên epic trong thư mục epic thay vì `spec-<N>-<M>-*`.
3. `AGENTS.md` (gốc): đường dẫn `_bmad-output/planning-artifacts/architecture/…/ARCHITECTURE-SPINE.md` và `_bmad-output/implementation-artifacts/` (Story specs, `sprint-status.yaml`, `deferred-work.md`), câu "scan the spine AND every unwritten `ad-brief-*.md`" (Q21), và các `AGENTS.md` con có bình luận `agent-rules-evidence.md`. Qua `bmad-project-context` (kế hoạch bước 7), không sửa tay.
4. `.github/workflows/ci.yml` (9 dòng), `src-tauri/Cargo.toml` (3), `e2e/**`, `tools/dict-build/**`: lời nhắc trong chú thích; sửa khi tiện, không cản.
5. Sau di trú, mọi mục nợ `deferred-work.md` mang `Chủ: Story X.Y` / `Epic N` đang tham chiếu ID v6 trong sprint-status; nên đối chiếu với `tickets.toml` khi viết lại Kiểm C.

## 14. Cấu hình

- `_bmad/custom/config.user.toml` (hiện chỉ chú thích; gitignored): thêm `[core]` `active_initiative = "initiative-auratranslate"`.
- `core.output_folder` vẫn `{project-root}/_bmad-output` (store không chuyển ⇒ không ghi `config.toml` team). KHÔNG đụng `planning_artifacts`, `implementation_artifacts`, và `_bmad/config.toml` (do installer sở hữu).
- `_bmad/custom/bmad-project-context.toml:20`: `file:{project-root}/_bmad-output/implementation-artifacts/agent-token-economics.md` ⇒ `file:{project-root}/_bmad-output/initiative-auratranslate/evidence-agent-token-economics/evidence-agent-token-economics.md`.
- `_bmad/custom/bmad-project-context.toml:25`: `_bmad-output/implementation-artifacts/agent-rules-evidence.md` ⇒ `_bmad-output/initiative-auratranslate/evidence-agent-rules/evidence-agent-rules.md`.
- Hai dòng đó được viết lại khi duyệt (quy tắc: override `_bmad/custom/` nêu đường dẫn đã chuyển).

## 15. Kết quả kiểm `checklist` (thi hành 2026-10-06, sau nhóm 8)

Kiểm bằng lệnh thật trên cây HEAD `ff5ef4d` cộng các sửa nhỏ của bước kiểm (mục "Sửa trong bước kiểm" bên dưới). Baseline: `153e286`; bản sao lưu `_bmad-output-bak/`.

| # | Mục | Kết quả | Bằng chứng (lệnh ⇒ số) | Ghi chú |
|---|---|---|---|---|
| 1 | Mọi tệp nguồn v6 ở initiative / `backlog/` / `inbox/` / `archive-v6/` hoặc "ở nguyên chỗ" | pass với ghi chú | Hợp `git ls-tree -r 153e286 _bmad-output` (432) và `find _bmad-output-bak -type f` (778) ⇒ 778 tệp, đối chiếu với bảng ánh xạ cũ→mới: 302 initiative + 1 `backlog/` + 3 `inbox/` + 8 `archive-v6/` + 464 ở nguyên chỗ = 778; 0 tệp mất; 0 tệp đích không tồn tại | 778 = 777 đã kiểm kê + chính tệp kế hoạch (nằm trong bản sao lưu vì được commit ở `153e286` trước khi sao lưu). `backlog/` có 1 tệp nguồn (`spec-fix-hanviet-…`, Q14); 346 tệp gitignore đều có mặt ở nơi cũ hoặc đích |
| 2 | Thư mục có tệp chính cùng tên; retrospective đúng chỗ; tên không ngày/số v6 | pass với ghi chú | `find` thư mục dưới `initiative-auratranslate/`, `backlog/`, `inbox/` thiếu `<tên>/<tên>.md` ⇒ chỉ `ux-auratranslate/.working`, `ux-auratranslate/mockups`, `architecture-auratranslate/reviews` (thư mục con của tài liệu, tệp anh em giữ tên v6 theo quy tắc) và hai thư mục không-initiative `backlog/`, `inbox/` (tuân cây riêng); 8 retrospective trực tiếp trong epic; tên có ngày/số: 10 tệp `review-*` trong `architecture-auratranslate/reviews/` (tên anh em của spine, giữ nguyên) và `ad-51-draft` (số AD), `…5-000-chuong…` (số trong tên chuẩn) | Sửa: hai retro Epic 1 và 2 mang `epic: 1` / `epic: 2` (số epic v6) trong khi 6 retro kia mang slug ⇒ đổi thành `epic: epic-<slug>`. Verdict chỉ có ở 6 retro có bằng chứng (2 `accepted-with-open-items`: Glossary, Trả nợ nền; 4 `rejected`: AI mở, Đường nhập, Library, Translation Memory; khớp bảng mục 5); Epic 1, 2 không verdict |
| 3 | Mục ở gốc store hợp lệ hoặc là remnant được liệt kê | pass với ghi chú | `ls _bmad-output` ⇒ `initiative-auratranslate/`, `backlog/`, `inbox/` (có `space.md`), `implementation-artifacts/`, `planning-artifacts/` | Hai thư mục cuối là remnant đã liệt kê ở mục 5.3: `implementation-artifacts/` chỉ còn 14 thư mục `*-ban-do/`, 5 HTML `*-ban-do-*` và `.DS_Store`; `planning-artifacts/` chỉ còn `ux-designs/.DS_Store`. `specs/`, `party-mode/` đã xoá (rỗng) |
| 4 | `tickets.py status` thoát 0; số story khớp tracking | pass | `uv run _bmad/method/scripts/tickets.py --project-root . status _bmad-output/initiative-auratranslate` ⇒ exit 0, 174 entry, 11 epic, `order_conflict` 0, `undeclared_after` 0; `status _bmad-output/backlog` ⇒ exit 0, 1 entry (`done`); `status _bmad-output/inbox` ⇒ exit 0 | `archive-v6/sprint-status.yaml` loại khoá epic và retrospective: 153 story = 103 `done` + 12 `review` + 5 `in-progress` + 33 `backlog`. v7: 174 = 153 + 1 (Story 6.18, chỉ có ở `epics.md`) + 20 (entry Q14 từ spec lẻ). Khớp từng epic: E1 26=26, E2 16=16, E3 12+6, E4 12=12, E5 14+3, E6 22+1+4, E7 11+7, E8 15, E9 8, E10 9, E11 8 (số sau dấu `+` là 6.18 và Q14). Trạng thái: `done` 123 = 103 + 20; `in-review` 12; `in-progress` 6 = 5 + 6.18; `planned` 33 = 33 `backlog`. Backlog đếm riêng (33 `planned`, không plan) |
| 5 | Mỗi story một entry, mỗi bản ghi một plan, không file story epic | pass | 174 entry; 141 `*-plan.md` trong `epic-*/` (= 123 + 12 + 6 entry không `planned`), 0 plan trùng `ticket:`; 0 tệp `story-*.md` không đuôi `-plan`; 7 thư mục `evidence-*` trong `epic-*/` (`type: evidence`, mỗi thư mục một tệp chính cùng tên) cho bản ghi thứ hai của 3.5 và các lô B/C/D của 11.1, 11.6, 11.7, 11.8 (Q13, phương án B); 1 plan trong `backlog/` | Entry `planned` (33) không có plan, đúng quy tắc. Sửa: Epics 1 và 2 (42 entry) thiếu `v6_key` ⇒ đã bổ sung từ `archive-v6/sprint-status.yaml` (khớp theo `N-id-`); sau đó cả 174 entry có `v6_key`: 153 theo khoá sprint, 6.18 theo tên bản ghi, 20 entry Q14 theo tên tệp spec nguồn (mục 7.3 ghi "bỏ trống"; giữ tên tệp để tra cứu) |
| 6 | Plan có kiểu build và status đã ánh xạ; baseline thiếu được báo | pass với ghi chú | `yaml.safe_load` frontmatter 141 plan ⇒ type: 99 `feature`, 22 `chore`, 17 `bugfix`, 3 `refactor`, 0 `story`; status: 123 `done`, 12 `in-review`, 6 `in-progress`; 139 plan có `baseline_revision`, 139/139 qua `git cat-file -e <sha>^{commit}` | Baseline thiếu (2, đều `done`, không suy từ HEAD): `epic-translation-memory/bug-dedupe-fuzzy-and-concordance-plan.md`, `epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi/story-bo-chay-e2e-trong-webview-that-plan.md`. Epic 4 giữ `in-review` (12), Story 5.14 `done` (Q11, Q12) |
| 7 | `covers` tồn tại; mỗi yêu cầu trong coverage map có ≥ 1 entry | pass với ghi chú | Script đối chiếu: `covers` của 11 epic ⊆ nguồn yêu cầu (PRD + định nghĩa trong `epics.md`) ⇒ 0 id lạ; `covers` ở `tickets.toml` = frontmatter epic = mục Requirements ⇒ 0 lệch; mọi id trong `covers` của entry ∈ Requirements của epic (FR) hoặc nguồn (NFR, UX-DR) ⇒ 0 lệch sau sửa; 133 FR sống của coverage map đều có ≥ 1 entry; mọi `covers` epic có ≥ 1 entry | Kiểm tay: FR20 rút ⇒ không epic nào nhận (đúng); FR13 (Epic 1 ⇄ 6), FR44, FR129 (6 ⇄ 7), FR70 (4 ⇄ 7): cả hai nửa có chủ; FR135 không có hàng trong coverage map nhưng ở Epic 1 `covers` và có entry (Story 1.18b, entry `18b`); UX-DR20 là id đã rút (gạch bỏ ở `epics.md`) nhưng Story 2.2 còn trích AC4–AC5 ⇒ giữ. Sửa: (a) 3 id nửa TM (FR44, FR70, FR129) có entry ở Epic 7 nhưng chưa ở Epic 7 `covers` ⇒ thêm vào `tickets.toml`, frontmatter và Requirements của epic-translation-memory; (b) entry 1.3 mang FR107 trong khi `epics.md` viết "không phải FR107" ⇒ gỡ; (c) entry 1.10c mang FR113 (Epic 3 chủ, đóng ở Story 3.7) ⇒ gỡ, giữ FR33 |
| 8 | Đường dẫn sống ở `companions:`, `inputDocuments:`, References, liên kết văn xuôi đều tới được; chỗ chết từ v6 liệt kê riêng | pass với ghi chú | Quét 282 tệp (ngoài `inbox/`, `archive-v6/`, `deferred-work.md`, kế hoạch này, `*-ban-do/`): 5 khoá frontmatter (`context` 476, `sources` 19, `relates_to` 16, `companions` 12, `inputDocuments` 3) ⇒ 0 chỗ không tới được; đường gốc `_bmad-output/…` chết: 7 dòng; liên kết tương đối chết: 45 dòng / 14 tệp | Sửa 1 lỗi di trú gây ra: `story-phan-loai-so-no-va-luat-khong-mo-coi-plan.md:695` liên kết `../../.githooks/pre-push` ⇒ `../../../.githooks/pre-push`. Chết từ trước v6 (không tính lỗi): 45 liên kết tới `project.rs` (42) và `GlossarySettingsOverlay.vue` (3), mã đã chuyển trước di trú; 6 đường gốc đã chết ở `153e286` (`editor-perf-spike-results-2026-08-XX.md` x2, `1-19-` cụt, 3 mẫu fixture `a0…`/`a01..a07`/`encoding/`); 1 nhắc thư mục `_bmad-output/planning-artifacts/research/` (dòng 279 của plan Story 1.1, thư mục nay là gốc rỗng ⇒ mô tả ý định lúc đó, không phải liên kết). Còn 73 nhắc thư mục/glob và đường trần ở nhóm 7 giữ nguyên có chủ ý |
| 9 | `config.user.toml` có `active_initiative`; `output_folder` trỏ store | pass | `tomllib` đọc `_bmad/custom/config.user.toml` ⇒ `{'core': {'active_initiative': 'initiative-auratranslate'}}`; `_bmad/config.toml:16` `output_folder = "{project-root}/_bmad-output"` ⇒ thư mục tồn tại | Tệp bị gitignore (`.gitignore:86`, cả `_bmad/`) ⇒ không có commit; `bmad-project-context.toml:20,25` đã trỏ đường mới |
| 10 | Store dưới git như đã trả lời; không tệp nào hai kho theo dõi | pass | Câu 3, 4 đều "không" ⇒ một kho duy nhất: `git ls-files _bmad-output` ⇒ 461 tệp, 0 trùng; `find _bmad-output -name .git` ⇒ 0; 0 tệp `.db` được theo dõi (AD-25); `git ls-files _bmad-output-bak` ⇒ 0 (loại ở `.git/info/exclude`) | Phần workspace: không áp dụng (không workspace, không kho mới ⇒ không `bmad status` ở gốc workspace) |
| 11 | Kế hoạch ghi đủ câu hỏi, trả lời, sao lưu, tệp Ice chỉ định, kết quả | pass | Mục 3.1 (6 câu đã trả lời), 3.2 (Q7–Q25 mặc định), 3.3 (Q9, Q14, Q17 Ice chốt; Q13 phương án B ghi lại ở bước kiểm), mục 4 (sao lưu, cập nhật 778 tệp), mục 5 (bốn danh sách), mục 15 (kết quả) | Bổ sung ở bước kiểm: dòng Q13 vào 3.3 và trạng thái sao lưu vào mục 4 (chữ cũ "chưa tạo" đã lỗi thời) |

### Sửa trong bước kiểm (đều là sửa nhỏ trong store, đã kiểm lại)

- `epic: 1` / `epic: 2` ⇒ `epic: <slug>` ở hai tệp retrospective Epic 1 và 2 (mục 2).
- 42 dòng `v6_key` bổ sung cho entry Epic 1 và 2 (mục 5).
- `epic-translation-memory`: thêm FR44, FR70, FR129 vào `covers` (initiative `tickets.toml`, frontmatter) và Requirements (mục 7); gỡ FR107 khỏi entry 1.3 và FR113 khỏi entry 1.10c (mục 7).
- Một liên kết tương đối `../../.githooks/pre-push` (mục 8).

Sau các sửa: `tickets.py status` vẫn thoát 0 với 174 entry (123 `done`, 12 `in-review`, 6 `in-progress`, 33 `planned`).

### Tổng kết cho báo cáo (guide bước 7)

- Baseline thiếu (2): hai plan ở mục 6. 33 entry `planned` không có plan nên không cần baseline.
- Nguồn đã lưu trữ ở `archive-v6/` (8, nguyên văn): `epics.md`, `sprint-status.yaml`, `epic-3-context.md`, `epic-4-context.md`, `epic-5-context.md`, `epic-6-context.md`, `epic-7-context.md`, `epic-11-context.md`.
- Tệp ở nguyên chỗ (464): 14 thư mục `implementation-artifacts/*-ban-do/` (456 tệp, Q15), 5 tệp HTML `*-ban-do-*.html`, 3 `.DS_Store` (Q22).
- Tệp không vào initiative: `inbox/` (3 tệp nguồn: `project-context.md`, `thu-xin-phep-hvtdtd.md`, `archive-v6/.memlog.md`, cùng `space.md` mới); `backlog/` (1 bug và plan của nó).
- Bản sao lưu `_bmad-output-bak/` (778 tệp) thuộc về Ice, tự xoá khi muốn; không commit (`.git/info/exclude`).
- Ngoài kho di trú: 176 tệp ngoài store nhắc đường cũ (523 dòng ở bảng tệp (b), 498 dòng ở mục 13); không sửa.

### Tham chiếu ngoài store Ice tự cập nhật (bản chép từ scratchpad; di trú không sửa các tệp này)

Đích mới của các mục tiêu hay bị nhắc nhất (cũ ⇒ mới):
- `_bmad-output/implementation-artifacts/deferred-work.md` -> `_bmad-output/initiative-auratranslate/deferred-work.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml` -> `_bmad-output/initiative-auratranslate/archive-v6/sprint-status.yaml`
- `_bmad-output/planning-artifacts/epics.md` -> `_bmad-output/initiative-auratranslate/archive-v6/epics.md`
- `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md` -> `_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md`
- `_bmad-output/implementation-artifacts/agent-rules-evidence.md` -> `_bmad-output/initiative-auratranslate/evidence-agent-rules/evidence-agent-rules.md`
- `_bmad-output/implementation-artifacts/agent-token-economics.md` -> `_bmad-output/initiative-auratranslate/evidence-agent-token-economics/evidence-agent-token-economics.md`
- `_bmad-output/implementation-artifacts/spec-<id>-*.md` (story specs) -> `_bmad-output/initiative-auratranslate/epic-*/story-*-plan.md` (full old->new list: scratchpad `path-map.tsv`)
- `_bmad-output/project-context.md` -> `_bmad-output/inbox/project-context.md`
- `*-ban-do/` folders and `*-ban-do*.html` stayed in place: references to them are unchanged and still valid.


#### (a) Mã đọc máy sẽ hỏng

| file:line | reads | new target |
|---|---|---|
| scripts/check-debt-owner.mjs:111 | `_bmad-output/implementation-artifacts/deferred-work.md` (path join) | `_bmad-output/initiative-auratranslate/deferred-work.md` |
| scripts/check-debt-owner.mjs:115 | `REAL_DEBT_PATH` same file | same |
| scripts/check-debt-owner.mjs:598 | `MAC_DINH` same file | same |
| scripts/check-debt-owner.mjs:116 | `_bmad-output/implementation-artifacts/sprint-status.yaml` | `_bmad-output/initiative-auratranslate/archive-v6/sprint-status.yaml` (frozen v6 copy; Kiem C reads sprint keys from it) |
| scripts/test-story.mjs:73 | `SPEC_DIR = _bmad-output/implementation-artifacts`; `findSpec` looks for `spec-<id>-*.md` there (`npm run test:story`) | story specs now `initiative-auratranslate/epic-*/story-*-plan.md`; naming scheme changed, finder needs rewriting |
| AGENTS.md:10-11,13 (root; `naming_boundary.rs` reads it) | prose pointers to spine, `implementation-artifacts/`, `agent-rules-evidence.md` | per table above; still parses, but the paths are dead |

Still valid (checked): `.githooks/pre-push:34` (`^_bmad-output/` docs-only filter, any path under it), `.githooks/pre-push:30` (`-ban-do/`), `src-tauri/tests/docx_probe.rs:38`, `webimport_contract.rs:1461-1467`, `webimport_probe.rs:72` (all read `-ban-do/` fixtures that stayed), `.gitignore:80-83`.
scripts/check-doc-refs.mjs scans for `epics.md:<line>` in code, not a path read: unaffected.


#### (b) Nhắc văn xuôi (chú thích, tài liệu) ngoài store

Tổng 523 dòng ở 176 tệp; phần lớn chỉ nhắc trần `deferred-work.md`, `epics.md`, `ARCHITECTURE-SPINE.md`, nên đích mới giống bảng trên. Nhiều nhất:

| Tệp | Dòng |
|---|---|
| scripts/check-debt-owner.mjs | 24 |
| src-tauri/src/commands/project/mod.rs | 19 |
| src/commands/index.ts | 15 |
| src/panels/editorPanelState.ts | 12 |
| src-tauri/src/commands/segment.rs | 12 |
| scripts/check-doc-refs.mjs | 12 |
| .github/workflows/ci.yml | 12 |
| src-tauri/tests/cleanup_contract.rs | 10 |

Danh sách đầy đủ từng tệp được tái tạo bằng lệnh ở mục 13. Đây là danh sách giữ lại cho Ice; việc viết lại `scripts/check-debt-owner.mjs` và `scripts/test-story.mjs` là story riêng (Q17).

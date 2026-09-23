---
name: 'Kế hoạch gọn hoá ARCHITECTURE-SPINE.md'
date: '2026-09-23'
status: done — thi hành 2026-09-23
target: '_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md'
---

# Kế hoạch gọn hoá Architecture Spine

Đo bằng `wc -c` + một script Python (offset/heading), không ước lượng. Tệp đích hôm nay: **155.012 byte / 1.157 dòng**, 48 `AD`. Tiền lệ đã chạy: gọn `AGENTS.md` gốc 19.769 → 10.301 byte (giữ **52%**), phần còn lại chuyển nguyên văn sang `agent-rules-evidence.md` (commit `a0ddf46`).

## 1. Bảng kích thước đo được

### 1.1 Theo mục lớn (đo bằng offset heading → heading tiếp theo cùng cấp)

| Mục | Byte | % tệp |
|---|---:|---:|
| `## Invariants & Rules` (48 AD) | 87.152 | 56,2% |
| `## Stack` | 32.524 | 21,0% |
| `## Deferred` | 18.465 | 11,9% |
| `## Structural Seed` (2 mermaid + cây nguồn) | 6.966 | 4,5% |
| `## Consistency Conventions` | 5.091 | 3,3% |
| `## Design Paradigm` | 2.308 | 1,5% |
| `## Capability → Architecture Map` | 1.515 | 1,0% |
| frontmatter + tiêu đề | ~991 | 0,6% |
| **Tổng** | **155.012** | 100% |

### 1.2 Top 12/48 `AD` theo byte — chiếm 60,5% của 87.129 byte nội dung AD

| AD | Byte | AD | Byte | AD | Byte |
|---|---:|---|---:|---|---:|
| AD-47 | 9.397 | AD-46 | 3.290 | AD-37 | 2.478 |
| AD-44 | 9.282 | AD-41 | 2.938 | AD-31 | 2.204 |
| AD-48 | 6.765 | AD-40 | 2.011 | AD-45 | 2.138 |
| AD-39 | 5.283 | AD-42 | 1.994 | AD-18 | 4.973 |

36 AD còn lại chia nhau 39,5% còn lại — trung bình **~955 byte/AD**, khuôn `Binds`/`Prevents`/`Rule` 3-8 dòng đã gọn sẵn (xem AD-1..AD-13, đọc đủ, không AD nào vượt 1.500 byte).

### 1.3 Cách phân loại (a)(b)(c)(d)(e) — đọc trực tiếp, không suy đoán

Cắt theo ranh **Prevents:**/**Rule:**; câu mang **ngày+số đo+tên Story** hoặc trỏ `deferred-work.md`/`epics.md` → (c)/(e); câu mang 🔵 → (d); phần mệnh lệnh còn lại của `Rule:` → (a); `Prevents:` → (b) trừ khi tự mang số đo. Đo trực tiếp 3 AD lớn nhất (29% byte-AD) làm mẫu:

| AD | Byte | Prevents (b) | Rule (a) | (c)/(e) đo+lịch sử+nợ | 🔵 (d) |
|---|---:|---:|---:|---|---:|
| AD-47 | 9.397 | 1.948 (21%) | 7.294 (78%) | mỗi mục ①-⑧ neo Story/ngày/`deferred-work.md:3685-3697` | 0 |
| AD-48 | 6.765 | 1.301 (19%) | 1.778 (26%) | 2.607 (38%) — "vì sao plugin" + giải thích `check-deps.mjs` | 989 (15%) |
| AD-44 | 9.282 | ~2.100 (23%) | ~2.000 (22%) | ~5.180 (56%) — bảng recall/corpus/ký tự Hán | 0 |

⇒ **Khái quát 12 AD lớn (52.753B):** (a) ~25% · (b) ~15% · (c)/(e) ~55% · (d) ~5%. **36 AD nhỏ (34.376B):** mẫu AD-1..13 (đọc đủ) ~85% (a)+(b), ~15% (c). Không suy rộng sang 23 AD chưa đọc hết — đọc nốt ở Pha 1.

### 1.4 `## Stack` (32.524B) và `## Deferred` (18.465B)

- `Stack`: đo ngoặc `(...)` ngoài cùng, đoạn 803-987 → **6.750B (20,8%)** là chú thích từng dòng (Story nào thêm, `cargo tree` xác nhận) — (c)/(e), trùng sổ NFR15/`deferred-work.md`. 79,2% còn lại là bảng Name/Version/Giấy phép — (a) thuần, và `lint_spine.py` cưỡng chế cột Version phải còn.
- `Deferred`: mỗi hàng `✅ ĐÃ ĐÓNG` mang nguyên văn phép đo đã có bản sao ở `deferred-work.md` (vd. "Chi phí byte NFR6 THẬT của `dom_smoothie`") — gần như thuần (c)+(e).

## 2. Không được đổi

**Quy tắc biên tập:** không AD nào đổi số, đổi phạm vi `Binds`, hay đổi ý nghĩa của `Rule`. Đổi ý = một AD mới, không phải một câu sửa. Rút gọn được: câu ví dụ, số đo lặp lại số đã có ở `deferred-work.md`, đoạn tường thuật quyết định.

### 2.1 Người đọc bằng máy — đã grep toàn repo (`src-tauri/`, `scripts/`, `tests/`, `.claude/skills/`, `_bmad/`)

| Người đọc | Cái nó khớp | Có đọc NỘI DUNG spine không | Sống được sau gọn không |
|---|---|---|---|
| `.claude/skills/bmad-architecture/scripts/lint_spine.py` | `^#{2,4}\s*AD-(\d+)\b` phải tăng dần, không trùng; khối AD phải chứa cả 3 chữ (không phân biệt hoa/thường) `binds`/`prevents`/`rule`; `## Stack` mỗi hàng phải có cột Version không rỗng; cấm `TBD/TODO/FIXME/XXX`, `"similar to AD-n"`, `{token}` chưa điền | **Có** — đọc thật `ARCHITECTURE-SPINE.md`, parse regex | **Có, có điều kiện**: dạng AD rút gọn 1-3 dòng vẫn phải giữ ba nhãn **Binds:** **Prevents:** **Rule:**; bảng Stack không được bỏ cột Version |
| `src-tauri/tests/scope_contract.rs:116,183` | chuỗi thông điệp panic trỏ `ARCHITECTURE-SPINE.md#AD-18` (neo theo tên AD, không theo số dòng) | Không — chỉ trích dẫn trong message | Có — chỉ cần heading `### AD-18` còn tồn tại |
| `src-tauri/tests/webimport_contract.rs`, `matching_boundary.rs`, `src-tauri/src/core/{matching,segment/{split,regroup},scope,library/atproj}.rs`, `commands/{segment,project/mod}.rs`, `src/panels/README.md` | comment trỏ **số dòng chính xác** (`ARCHITECTURE-SPINE.md:236`, `:189`, `:542`, `:604-618`, …) | Không — comment tĩnh, không parse | **Không** — mọi trích dẫn số dòng lệch khi cấu trúc đổi. Đếm được: **211 chỗ trích `ARCHITECTURE-SPINE.md:<số>`** trong toàn repo; trong đó **10 tệp mã sống** (danh sách ở trên) cần sửa lại số dòng sau gọn, 45 tệp còn lại là hồ sơ story/ad-brief/sprint-change-proposal đã đóng — giữ nguyên như bản ghi lịch sử, không sửa |
| `scripts/check-tokens.mjs` | chỉ nêu tên `ARCHITECTURE-SPINE.md` trong comment giải thích lý do (AD-34), không đọc file | Không | Có, không phụ thuộc |
| `scripts/check-debt-owner.mjs:494` | regex `/…|ARCHITECTURE-SPINE\.md|…/ ` khớp **chuỗi tên file** xuất hiện bên trong `deferred-work.md` (phân loại nguồn tham chiếu), không đọc spine | Không | Có — chỉ cần **giữ nguyên tên file** `ARCHITECTURE-SPINE.md`, không đổi path/tên |

Không tìm thấy người đọc máy nào khác ngoài `lint_spine.py` (đã grep hết `_bmad/`, `.claude/skills/`, không có script thứ hai parse nội dung spine).

## 3. Hình dạng đích đề xuất

- **Giữ nguyên cấu trúc mục lớn** (Design Paradigm · Invariants & Rules · Consistency Conventions · Stack · Structural Seed · Capability Map · Deferred) — không đổi tên heading `##` (không máy nào phụ thuộc tên này, nhưng người đã quen tra theo mục).
- **Khuôn 1 AD rút gọn** (3 nhãn `lint_spine.py` đòi vẫn còn, nội dung cô lại):
  ```
  ### AD-N — <tên>
  - **Binds:** <danh sách>
  - **Prevents:** <1 câu, lớp hỏng cụ thể — không kể lại 5 ví dụ nếu 1 câu đủ nghĩa>
  - **Rule:** <1-3 câu mệnh lệnh, giữ số/tên chuỗi ký hiệu bắt buộc như `DECLARED_KIND_COUNT`>
  → chi tiết & bằng chứng: `spine-evidence.md#AD-N`
  ```
- **Tệp mới `spine-evidence.md`** cạnh spine (cùng thư mục `architecture-AuraTranslate-2026-08-02/`), một heading `## AD-N` mỗi mục, chép **nguyên văn** phần bị cắt (Prevents mở rộng, đo đạc, tường thuật quyết định, 🔵) — không diễn giải lại, như `agent-rules-evidence.md` đã làm.
- **Stack**: giữ bảng Name/Version/Giấy phép nguyên trạng (phần `lint_spine.py` canh + tra cứu nhanh nhất); chuyển chú thích dài trong ngoặc sang `spine-evidence.md#Stack`, để lại tối đa nửa dòng lý do (vd. `(Story 6.9, nâng bắc cầu→trực tiếp)`).
- **Deferred**: mục `✅ ĐÃ ĐÓNG` đã có bản sao ở `deferred-work.md` → rút còn `✅ ĐÃ ĐÓNG <ngày> (Story x.y) — xem deferred-work.md`; mục còn mở giữ nguyên (nguồn sự thật duy nhất).
- **Kích cỡ đích**: không chốt cứng — đưa vào quyết định #1 ở §6, vì tỉ lệ (c)/(e) không trùng nơi khác trong 12 AD lớn mới chỉ ước tính (§1.3: 55%), chưa đối chiếu từng câu với `deferred-work.md`.

## 4. Vấn đề mở cần Ice chốt trong lượt này

1. **`AD-48` bị cấp hai lần.** `ad-brief-2026-08-17-mo-hinh-hoan-tac.md` §11.4 (dòng 415) dành `AD-48` cho mô hình hoàn tác. `ad-brief-2026-08-24-hop-thoai-chon-tep.md:26` đo "spine dừng ở AD-47" rồi tự cấp lại `AD-48` cho hộp thoại chọn tệp — bản này đã **viết vào spine** (2026-08-25, `ARCHITECTURE-SPINE.md:746`). `sprint-status.yaml` mục `epic-3-retro-item-35-ai-1…` (dòng 568-583, **status: open**, chủ Ice) đã tự ghi nhận đúng lỗi và yêu cầu sửa văn bản B6 + cấp AD MỚI cho mô hình hoàn tác. Quét cả spine (tới 48) lẫn ad-brief chưa lên spine (chỉ còn mô hình hoàn tác) → số trống kế tiếp là **AD-49**. Soạn AD-49 giao Winston; plan này chỉ đề nghị sửa văn bản B6.
2. **Frontmatter `updated: '2026-08-16'` đã hết đúng** — AD-48 tự ghi nó viết ngày `2026-08-25`. Sửa `updated` là việc rẻ, làm cùng lượt gọn.
3. **AD-48 tự mang 1 đoạn 🔵 sửa số liệu của chính nó** (crate 9→3). Giữ nguyên trong AD khi gọn — đoạn đó tự khai lý do không được xoá ("đừng trích lại con số chín"), và nó là một phần `Rule` (điều kiện xét lại), không phải lịch sử rời để chuyển ra `spine-evidence.md`.
4. **AD-12** phủ định một nghiên cứu ngoài ("WAL2 … không tồn tại — xem `.memlog.md`") — kiểm `.memlog.md` còn tồn tại/còn đúng trước khi giữ nguyên.
5. Không AD nào khác tự khai bị AD sau ghi đè/mâu thuẫn, ngoài AD-47 (đã tự ghi bảng "AD-31/AD-5 đổi gì / không đổi gì" ở §⑦ — giữ nguyên).

## 5. Chia pha thực thi (mỗi agent qua FILE, dưới ~250k context — theo AGENTS.md)

| Pha | Việc | Input file | Output file | Agent mới |
|---|---|---|---|---|
| 0 | Ice chốt §6 | plan này | quyết định ghi vào đầu `spine-evidence.md` | không cần agent, Ice tự quyết |
| 1 | Đọc hết 23 AD nhỏ chưa đọc trọn vẹn (bù §1.3), khoá bảng phân loại (a)(b)(c)(d)(e) từng AD, viết `spine-condense-worksheet.md` | spine gốc | worksheet | agent mới, chỉ đọc — không sửa spine |
| 2 | Viết `spine-evidence.md` (chép nguyên văn phần bị cắt của 48 AD + Stack + Deferred đã đóng) | worksheet + spine gốc | `spine-evidence.md` | agent mới |
| 3 | Sửa `ARCHITECTURE-SPINE.md` tại chỗ theo khuôn §3, cập nhật `updated:` frontmatter, thêm dòng 🔵 nếu cần (mục §4.2) | worksheet + spine gốc | spine đã gọn | agent mới |
| 4 | Sửa 10 tệp mã sống có trích số dòng (bảng §2.1) sang trích theo `#AD-N` hoặc số dòng mới; chạy đối chứng | spine đã gọn | patch cho 10 tệp | agent mới |

**Đối chứng bắt buộc trước khi giao:**
- `python3 .claude/skills/bmad-architecture/scripts/lint_spine.py --workspace <thư mục spine>` → `"ok": true`, `0` finding.
- `cargo test --locked --test scope_contract --test webimport_contract --test matching_boundary` xanh (không phải suy luận từ suite đầy đủ).
- `npm run check:tokens && npm run check:debt-owner` xanh (hai script này chỉ khớp TÊN FILE, không nội dung — xanh nếu tên file spine không đổi).
- **Diff ID**: `grep -oE '^### AD-[0-9]+' ARCHITECTURE-SPINE.md` trước và sau, `diff` hai danh sách — phải **rỗng** (không AD nào biến mất, không AD nào đổi số).
- Grep 211 chỗ trích `ARCHITECTURE-SPINE.md:<số>` — đếm lại đúng 10 tệp mã sống đã sửa, 45 tệp lịch sử không đụng.

## 6. Quyết định Ice cần chốt

| # | Quyết định | A | Hệ quả A | B | Hệ quả B |
|---|---|---|---|---|---|
| 1 | Mục tiêu kích cỡ | Cắt mạnh, mô phỏng tỉ lệ AGENTS.md (giữ ~52%) | Spine còn ~80 KB; đòi cắt cả (a)/(b) ở 12 AD lớn — rủi ro mất một phần `Rule` thật (AD-47 Rule 7.294B vì mỗi mục ①-⑧ mang một điều kiện thật, không chỉ ví dụ) | Cắt đúng ranh (c)/(d)/(e), giữ mọi (a)/(b) | Spine còn ước **~95-100 KB** (87.129 − ~29.000 của Invariants − 6.750 Stack − ~15.000 Deferred) — an toàn hơn cho AD-47/AD-44, cắt ít hơn AGENTS.md |
| 2 | AD-48 trùng số (§4.1) | Sửa văn bản B6 ngay, cấp AD-49 cho mô hình hoàn tác (Winston soạn sau) | Đóng 1 mục nợ mở ở `sprint-status.yaml`, không chặn việc gọn spine | Để nguyên, chỉ ghi "đã biết, chưa xử" vào `spine-evidence.md` | Mục nợ tiếp tục treo; rủi ro người đọc B6 tưởng đã xong — đúng lớp lỗi mục nợ đó cảnh báo |
| 3 | 45 tệp lịch sử có trích số dòng cũ | Không sửa — bản ghi tại thời điểm viết | 0 rủi ro kỹ thuật; người đọc sau tự biết số dòng đã cũ | Sửa luôn theo mốc AD-N | Tốn thêm ~1 pha; không tệp nào bị máy đọc, lợi ích chỉ là tiện tra cứu |
| 4 | Tự soạn AD-49 trong lượt này? | Không — theo AGENTS.md "AD mới giao Winston" | Đúng chính sách; B6 treo tới khi Winston soạn | Có, gộp luôn | Vi phạm chính sách đã ghi (và bài học đã lưu `ad-moi-giao-winston-khong-dev-tu-soan`) — không đề xuất |

**Ice chốt 2026-09-23:** #1 → B (cắt đúng ranh, giữ mọi Rule/Prevents, đích ~95–100 KB) · #2 → A (B6 đã sửa, mô hình hoàn tác nhận `AD-49`, Winston soạn trong lượt riêng) · #3 → A (45 tệp lịch sử giữ nguyên, chỉ sửa 10 chỗ trong mã đang sống) · #4 → A (không soạn AD-49 trong lượt gọn spine).

🔵 2026-09-23 — Đích ~95–100 KB ở #1 hết đúng: spine sau gọn còn **131.386 byte** (từ 158.666, mốc `d861aab`). Đọc hết 49 AD thì phần cắt được thật của `Invariants & Rules` chỉ **6,4%**; 38 AD không mang ngày, tên Story, 🔵 hay trỏ nợ nào. Ước 55% ở §1.3 suy rộng từ 3 AD dày nhất. Cắt theo đúng ranh của B, không cắt vào Rule/Prevents để ép số. Chỗ trích số dòng trong mã sống là **28 chỗ / 16 tệp**, không phải 10 tệp như §2.1.

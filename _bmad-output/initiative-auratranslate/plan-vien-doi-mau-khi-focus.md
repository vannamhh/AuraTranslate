---
title: 'Phần tử có viền: bỏ vòng focus, đổi màu viền khi focus'
type: 'feature'
ticket: ''
created: '2026-10-25'
status: 'done'
baseline_revision: '00a33b8f9227c6c6d73eb60446d3863711dfe123'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/plan-vien-doi-mau-khi-focus-inventory.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Mọi phần tử nhận focus đều hiện vòng focus xanh mặc định của WebKit, kể cả khi phần tử đã có viền riêng (ảnh: ô tìm Library vẽ vòng xanh bên trong khung viền). Hai đường viền chồng nhau, trái hướng "bàn viết" của DESIGN.md.

**Approach:** Phần tử có viền (của chính nó hoặc khung bọc) bỏ outline khi focus và đổi màu viền sang `var(--color-primary)`; phần tử không viền giữ nguyên vòng focus. Kiểm H của `check:tokens` học luật mới thay cho miễn trừ từng chỗ.

**Quyết định của Ice (2026-10-25):**
- Phạm vi: nhóm 1–5 của bảng kê (nút có khung, ô nhập, khung bọc control, nút gạch chân, phần tử tabindex có khung kể cả 18 gốc lớp phủ). Viền làm chỉ báo "đang chọn" (mục *Ngoài phạm vi*) giữ nguyên.
- Biến thể đã viền `primary` (nút chính, `aria-pressed`, đang bật, thẻ đang chọn): chỉ đổi màu, nên các biến thể này **giữ vòng focus của trình duyệt**.
- Kiểm H chấp nhận `outline: none` khi CÙNG khối rule đặt màu viền bằng token; khối tắt outline mà không đổi viền vẫn đỏ. Khung bọc (outline tắt ở control bên trong, viền đổi ở khung) dùng miễn trừ có tên.

## Boundaries & Constraints

**Always:** chỉ đổi màu viền, không đổi độ dày hay padding (không xô bố cục; `.pl-ta-input` đè gương `.pl-ta-mirror`). Màu là token `var(--color-primary)`. Rule focus đặt trong `<style scoped>` của chính component, cạnh rule viền gốc. Dùng `:focus-visible` (khung bọc: `:focus-within` hoặc `:has(:focus-visible)` theo khuôn đã có). Biến thể đã viền primary bị loại bằng `:not(...)` trong selector, không bằng `outline: revert`. Nút gạch chân đổi `border-bottom-color`.

**Never:** rule toàn cục kiểu `:where(input, button):focus { outline: none }` (lấy mất chỉ báo của phần tử không viền). Không `box-shadow` (DESIGN.md: không bóng). Không lợi dụng lỗ của Kiểm H (`outline: 2px solid transparent`, `outline-width: 0`). Không đụng mục *Ngoài phạm vi*. Không token màu mới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Nút có khung | Tab tới `.gm-act` | viền `primary`, không vòng xanh | — |
| Nút đã viền primary | Tab tới `.gm-act-primary` | vòng focus của trình duyệt vẫn hiện | — |
| Ô nhập trong khung | bấm vào ô tìm Library | khung `.search-field` viền primary, không vòng xanh trong ô | — |
| Phần tử không viền | Tab tới mục menu `.tools-item` | vòng focus vẫn hiện | — |
| Gate: tắt outline có đổi viền | khối `.x:focus-visible { outline: none; border-color: var(--color-primary) }` | Kiểm H xanh, không cần chú thích | — |
| Gate: tắt outline không đổi viền | khối `.x:focus-visible { outline: none }` | Kiểm H đỏ như cũ | thông báo nêu luật mới |
| Gate: đổi viền bằng màu thô | `outline: none; border-color: #2f5d63` | Kiểm H đỏ (và Kiểm B đỏ) | — |

</frozen-after-approval>

## Code Map

- `plan-vien-doi-mau-khi-focus-inventory.md` -- bảng kê từng khối theo nhóm 1–5, kèm biến thể (p) phải loại. Nguồn duy nhất cho danh sách selector.
- `scripts/check-tokens.mjs:1568-1653` -- Kiểm H. `isFocusRootSelector`, `OUTLINE_OFF_RE`, `exemptAt(p, d.index, 'outline-none')` (`:561`). Mỗi `block` có `prelude` và `decls` (`prop`, `value`, `index`). Kiểm B (`:796+`) đã cấm màu thô trong `border*`.
- `tests/frontend/checkTokensScan.test.ts` -- self-test của các hàm thuần trong cổng; thêm ca cho vị từ mới ở đây.
- `src/modes/LibraryMode.vue:1414-1444` -- khuôn khung bọc có sẵn (`.search-field:focus-within { border-color: var(--color-primary) }`); `.search-input` chưa tắt outline.
- `src/DropdownMenu.vue:126-139` -- một chỗ phủ 22 nơi dùng; `.menu-trigger` là `<summary>`.
- `src/SourceLangChips.vue:73`, `src/GlossaryManageOverlay.vue:1118` -- đang vẽ outline 2px primary qua `:has(input:focus-visible)`; đổi sang viền khung.
- Không đụng: `.mode/.panel/.dock:focus` (Kiểm H đã cho phép), `GridBlock.vue .cell-tgt:focus`, các miễn trừ `aura-allow-outline-none` sẵn có.

## Tasks & Acceptance

**Execution:**
- [ ] `scripts/check-tokens.mjs` -- Kiểm H: khối có `outline`/`outline-style` tắt được chấp nhận khi chính khối đó có khai báo `border-color` hoặc `border-(top|right|bottom|left|block|inline)(-start|-end)?-color` mà giá trị chứa `var(--color-`; tách thành hàm thuần export được; sửa câu `detail` và dòng `pass` tổng (đếm riêng số khối thay-bằng-viền) -- cổng phải nói đúng luật mới.
- [ ] `tests/frontend/checkTokensScan.test.ts` -- ca cho vị từ mới: có viền token ⇒ chấp nhận; không viền ⇒ từ chối; viền màu thô ⇒ từ chối; `border-bottom-color` ⇒ chấp nhận -- ba hàng gate của ma trận.
- [ ] Các `.vue` theo bảng kê nhóm 1, 2, 4, 5 -- mỗi tệp thêm (hoặc gộp) khối `X:not(<biến thể p>):focus-visible { outline: none; border-color: var(--color-primary) }` (nhóm 4: `border-bottom-color`; viền một phía: màu của đúng phía đó) -- hai hàng đầu của ma trận.
- [ ] Các `.vue` theo nhóm 3 -- khung: `:focus-within`/`:has(:focus-visible)` đổi `border-color`; control bên trong: `outline: none` kèm `/* aura-allow-outline-none: khung <tên> đổi viền khi focus */`; `SourceLangChips`/`GlossaryManage` thay outline 2px bằng viền khung -- hàng "ô nhập trong khung".
- [ ] Ô dùng viền gốc WebKit (`.gm-cell-input`, ô trong `label.bip-field`, `.btn`/`.mark-affordance` của ReadingMode) -- không thêm viền mới; giữ vòng focus -- ngoài định nghĩa "có viền của tác giả".

**Acceptance Criteria:**
- Given app chạy ở theme sáng và tối, when Tab qua một lớp phủ (ví dụ Glossary), then mọi nút/ô có khung đổi viền sang primary và không vòng xanh nào chồng lên viền.
- Given `npm run check:tokens`, when chạy trên cây đã sửa, then Kiểm H xanh và số miễn trừ `aura-allow-outline-none` chỉ tăng đúng bằng số control bên trong khung bọc của nhóm 3.
- Given gỡ khai báo `border-color` khỏi một khối focus vừa thêm (gỡ thật), when chạy lại `check:tokens`, then Kiểm H đỏ đúng tại khối đó.

## Implementation Notes

- Kiểm H: vị từ `blockRecoloursBorderWithToken` ở `scripts/lib/tokens-scan.mjs` (selector mang lớp/thuộc tính, không `*`; đúng một token ở `border-*-color`). Đếm sau vá: 4 gốc · 141 khối viền · 7 miễn trừ (4 mới, đúng bốn control trong khung bọc).
- Bốn gốc lớp phủ ngoài bảng kê (`aip-panel`, `attr-panel`, `bip-panel`, `ip-panel`) không có viền nên để nguyên. `.aip-efix` viền dưới vốn primary ⇒ giữ vòng gốc.
- Biến thể viền lỗi (`danger`, chip `needs-review`, `.bip-mismatch-focus`) chuyển sang primary khi focus — rule focus thắng selector biến thể.
- Radio HIỆN trong `.pl-pill`/`.tm-pill`/`.gq-chip`/`.ip-candidate`/`.bip-candidate` giữ vòng gốc (radio gốc ngoài phạm vi), nên các khung này có cả viền đổi màu lẫn vòng trên radio.
- `.gm-seg`: cả khung đổi viền khi một lựa chọn có focus; lựa chọn đang có focus là lựa chọn đang bật (nhóm radio), nên kiểu `gm-seg-on` chỉ ra nó.
- Chưa nghiệm thu bằng mắt trong app thật (theme sáng/tối) — Ice làm.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 2 medium · 3 low · 0 false · 0 maybe-false.

- medium, patch: chip ngôn ngữ đang chọn mất chỉ báo focus — `:not(.lang-chip--on)` loại nó, viền đã primary, `.lang-chip-input` `opacity: 0` nên vòng gốc vô hình; vá bằng outline 2px primary cho biến thể on (đúng quyết định "biến thể primary giữ outline").
- medium, patch: Kiểm H chấp nhận mọi selector (`*:focus-visible`, `button:focus`) qua đường viền token — vá: selector phải mang lớp/thuộc tính, không `*`.
- low, patch: giá trị chỉ cần CHỨA `var(--color-` (trộn màu thô vẫn qua) — vá: đúng một token.
- low, patch: chú thích đầu Kiểm H và dòng pass nói sai luật mới; bốn lý do miễn trừ viết tiếng Việt trái AGENTS.md — sửa tại chỗ.
- low, reject: test chỉ gọi hàm thuần, không chạm vòng Kiểm H — chỗ nối là một `if`; đối chứng gỡ `border-color` trên cây thật làm Kiểm H đỏ đúng khối (`.gm-act` bởi agent, `.lookup-pin` bởi phiên chính); dựng tự-kiểm trong cổng là thêm cơ chế.
- phiên chính, patch: `.lookup-pin` của LookupPanel (nhóm 4) bị sót — đã thêm.

## Verification

**Commands:**
- `npx vitest run tests/frontend/checkTokensScan.test.ts` -- expected: xanh.
- `npm run check:tokens` -- expected: exit 0; đối chứng đỏ theo AC thứ ba.
- Sửa `scripts/check-*` ⇒ chạy full suite một lần theo AGENTS.md.

**Manual checks (if no CLI):**
- Ice Tab qua Library (ô tìm, nhóm Chính xác dấu/Không dấu, thẻ), một lớp phủ có nút chính, Cài đặt › Phím tắt và panel Tra cứu, ở cả hai theme.

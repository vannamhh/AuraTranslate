---
type: epic
title: "Lớp phủ một bề rộng, một vùng cuộn, menu Công cụ theo nhóm"
parent: initiative-auratranslate
covers: []
sources: [_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md, _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md]
companions: [_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/lop-phu-mot-be-rong.html, _bmad-output/initiative-auratranslate/ux-auratranslate/.working/bang-ke-lop-phu-2026-11-17.md]
after: []
refined: true
risk: high
---

# Lớp phủ một bề rộng, một vùng cuộn, menu Công cụ theo nhóm

## Description

Ice chốt ngày 2026-11-17 ba luật bố cục cho mọi lớp phủ cài đặt và chức năng: một bề rộng tối đa 1120px, đúng một vùng cuộn dọc với đầu và chân khung dính, và menu `Công cụ ▾` xếp theo loại dữ liệu. Bảng kê ở master `528dec8a` đếm được tám bề rộng khung khác nhau, tám lớp phủ nghi cuộn lồng, và 16 mục menu phẳng. Epic chuyển 18 lớp phủ về một khung dùng chung, dựng một cổng nguồn giữ hai luật đầu, và sắp lại menu. Luật nằm ở `DESIGN.md` §Lớp phủ — một bề rộng, một vùng cuộn và `EXPERIENCE.md` §Thứ tự trong `Công cụ ▾`.

## Outcome

Ice mở bất kỳ màn cài đặt hay màn chức năng nào cũng gặp cùng một bề rộng, nút chính luôn trong tầm nhìn và chỉ một thanh cuộn, và tìm được công cụ trong menu theo nhóm dữ liệu.

## Requirements

- R1 (DESIGN.md §Lớp phủ — một bề rộng, một vùng cuộn; token `spacing.overlay-max-width`): mọi lớp phủ nhóm Màn có khung rộng tối đa 1120px, canh giữa ngang, sát đỉnh vùng dưới thanh tiêu đề, nền che `background` đặc, lề `panel-inline`; cửa sổ hẹp hơn 1120px cộng hai lề thì khung co theo cửa sổ.
- R2 (DESIGN.md §Lớp phủ, bảng nhóm): năm màn nhiều cột (Xuất, Xem trước nhập, Xem trước nhập song ngữ, Nguồn ảnh, Prompt inspector) tràn cửa sổ trừ lề `panel-inline`; Bảng lệnh và bộ chọn phạm vi Proofreader là popup ôm nội dung, không theo bề rộng 1120px.
- R3 (DESIGN.md §Lớp phủ — Bên trong khung; §Điều khiển › Chiều rộng 🔵 2026-11-17): trong lớp phủ, ô chữ và danh sách giãn hết bề ngang khung trừ đệm; ô số (ngưỡng, nhiệt độ, số token, số Chương) giữ bề rộng `ch` làm tròn lên bội 7; nút ôm chữ, cặp nút chân chung `min-width` 112px. Ô ngoài lớp phủ giữ luật `ch`.
- R4 (DESIGN.md §Lớp phủ — Một vùng cuộn; EXPERIENCE.md Component Patterns › Lớp phủ): đầu khung và chân khung dính, không cuộn; giữa chúng đúng một vùng cuộn dọc; không vùng cuộn dọc nào nằm trong vùng cuộn dọc khác; màn nhiều cột được mỗi cột tự cuộn; ô nhập nhiều dòng tự cuộn là ngoại lệ duy nhất; con trỏ bàn phím đi trong danh sách thì vùng cuộn duy nhất cuộn theo.
- R5 (EXPERIENCE.md §Thứ tự trong `Công cụ ▾`): menu xếp Glossary (Quản lý Glossary · Duyệt thu hoạch thuật ngữ) · TM (Quản lý bộ nhớ dịch) · Quy ước (Quét chính tả · Chỗ đã bỏ qua · Quản lý quy ước chữ Việt · Thay cả Chương theo quy ước) · Reviewer (Nhập bản reviewer · Nối tay bản reviewer) · Bố cục (Sắp xếp panel · Đối chiếu · Tra cứu · Đề xuất AI, công tắc giữ như hiện tại) · Mở bảng lệnh · Cài đặt; nhóm chia bằng `.ui-menu-sep`, không in nhãn nhóm.
- R6 (EXPERIENCE.md §Thứ tự trong `Công cụ ▾`, bảng lối vào): Ngưỡng quét Glossary và Thêm Chương… rời menu, mỗi lệnh vẫn có đường chuột (Thêm Chương… ở Library và trang Tác phẩm; `glossary.settings.open` khai `pointer` kiểu `via` với đích `settings.open`); mục Quét chính tả dispatch `ai.proofread.scope.open`, không mang phím.

## Constraints

- Chỉ đổi bố cục lớp phủ và thứ tự menu; nội dung, chức năng và phím của từng lớp phủ không đổi (Ice 2026-11-17, phạm vi đợt UX).
- Lưới 7px (DESIGN.md §Lưới 7px) vẫn giữ; 1120px là 160 × 7.
- AD-62: mọi command giữ đường bấm chuột; `check:commands` xanh với `POINTER_DEBT` rỗng.

## Done when

1. Trên WKWebView, mọi lớp phủ nhóm Màn đo được rộng `min(1120px, cửa sổ − 28px)` ở cửa sổ 1280px và 800px, và trong mỗi khung chỉ một phần tử cuộn dọc (trừ ô nhập nhiều dòng và cột kề nhau).
2. Một cổng nguồn canh R1 và R4 có ca tự kiểm đỏ, đối chứng gỡ thật cho đỏ đúng lý do, và danh sách nợ lớp phủ của nó rỗng.
3. Menu `Công cụ ▾` đúng thứ tự và nhóm của R5; `check:commands` xanh với `POINTER_DEBT` rỗng.
4. CI xanh cả nửa Windows; phần chỉ kiểm được trên Windows nằm trong mục `B7`.
5. Mục nợ `Chủ: Epic 17` đã đóng: lượt dùng thật của Ice trên các màn trong `mockups/lop-phu-mot-be-rong.html` và trên ba lớp phủ khác do Ice chọn.

## Boundaries

- Ngoài epic: panel Workspace, Library, Chế độ đọc; dải mọc dưới câu và popup neo tại câu (TM, Proofreader, chốt Glossary); chiều cao lớp phủ; nội dung từng lớp phủ.
- Ngoài epic: rút `focusableWithin` + `trapTab` thành một chỗ dùng chung (AI-11 retro Epic 3) — khung chung của entry 1 có thể đón nó, nhưng chỉ khi Ice nói.
- Touch point: `src/tokens/controls.css` và `tokens.json` — thêm token và lớp khung chung; Epic 13 đã đặt nền lưới 7px ở đây; owner: epic-lop-phu-mot-be-rong.
- Touch point: `scripts/check-commands.mjs` — chỉ đọc khai `pointer` kiểu `via`; owner: epic-san-chuot.
- Touch point: `package.json`, `.github/workflows/ci.yml`, `.githooks/pre-push` — chỉ khi cổng mới là cổng riêng thay vì một Kiểm trong `check:tokens` (ba danh sách, `check:gates`); owner: epic-lop-phu-mot-be-rong.
- Must not change: tiêu điểm trả về nút mở khi đóng lớp phủ (UX-DR17), thứ tự Tab trong từng lớp phủ, phím tắt hiện có, Esc đóng lớp phủ.

## References

- parent — _bmad-output/initiative-auratranslate/initiative-auratranslate.md, section Requirements
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md, §Lớp phủ — một bề rộng, một vùng cuộn · §Điều khiển › Chiều rộng · §Lưới 7px
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, §Thanh tiêu đề và lối vào · §Thứ tự trong `Công cụ ▾` · Component Patterns › Lớp phủ
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/lop-phu-mot-be-rong.html
- inventory — _bmad-output/initiative-auratranslate/ux-auratranslate/.working/bang-ke-lop-phu-2026-11-17.md
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-62

## Notes

- Decision: 2026-11-17 — `covers` để trống như Epic 13 và 14: đây là quyết định thiết kế, không phải một FR của PRD.
- Decision: 2026-11-17 — epic đứng cuối thứ tự xây, sau Epic 16.
- Decision: 2026-11-17, Ice — Epic 17 chờ Epic 13: Ice làm 13.9 trước rồi mới triển khai epic này; entry 1 mang `after = ["13.9"]`, cách khác (không chờ, chỉ cấm 13.9 đụng khung lớp phủ) bị bỏ.
- Waits on epic-luoi-7px-va-canh-baseline because: khung mới đứng trên lưới 7px đã nghiệm thu (13.1, 13.8), và Refactor sweep 13.9 không được chạy cùng lúc trên cùng các tệp lớp phủ.
- Decision: 2026-11-17 — đường xuyên suốt là entry 1: token, khung dùng chung, cổng với danh sách nợ đủ 18 lớp phủ, và một lớp phủ thật (Xem trước lượt nhập bộ prompt, màn có trong mockup) chuyển xong, đo trên WKWebView.
- Decision: 2026-11-17 — entry 2 (Cài đặt) đi ngay sau vì luật ô chữ giãn, ô số giữ `ch` trên mười mục Cài đặt là phần chưa chắc nhất.
- Decision: 2026-11-17 — bốn lô chuyển lớp phủ chia theo miền dữ liệu, mỗi lô xoá dòng của mình khỏi danh sách nợ của cổng; lô 2–5 cùng chạm danh sách đó, cùng dùng khung, token và helper e2e của entry 1 mà không sửa chúng, nên chạy song song được. Danh sách rỗng được kiểm ở Refactor sweep, entry duy nhất chờ cả bốn lô.
- Decision: 2026-11-17 — luật cổng: bề rộng khung lớp phủ chỉ lấy từ `--overlay-max-width`; không khai vùng cuộn dọc nằm trong vùng cuộn dọc khác; miễn trừ có tên cho popup (`PaletteOverlay.vue`, `ProofreadScopeOverlay.vue`) và năm màn nhiều cột ở luật bề rộng, cho ô nhập nhiều dòng ở luật cuộn; danh sách nợ ghi từng tệp chưa chuyển kèm lý do và chết khi rỗng.
- Decision: 2026-11-17 — entry 6 (menu) là làn riêng, không chờ khung lớp phủ; nó chỉ dùng lớp `.ui-menu-sep` có sẵn, không sửa `controls.css`.
- Decision: 2026-11-17 — mục nợ lượt dùng thật (Done when 5) do Sofia ghi lúc mở epic, `Chủ: Epic 17`.
- Open question: hình dạng khung dùng chung (lớp CSS chung ở `controls.css` hay component Vue có ba khe đầu/thân/chân) và chỗ đặt cổng (một Kiểm mới trong `check:tokens` hay một cổng riêng ở ba danh sách) — entry 1 trình Ice hai phương án có số đo trong plan; Ice chờ.

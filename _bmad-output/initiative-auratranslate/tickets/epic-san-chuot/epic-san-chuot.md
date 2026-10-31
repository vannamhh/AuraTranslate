---
type: epic
title: "Sàn chuột — mọi chức năng độc lập bấm được bằng chuột"
parent: initiative-auratranslate
covers: [NFR20]
sources: [_bmad-output/initiative-auratranslate/ad-brief-san-chuot/ad-brief-san-chuot.md]
companions: [_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, _bmad-output/initiative-auratranslate/tickets/epic-san-chuot/ux-san-chuot/EXPERIENCE.md, _bmad-output/initiative-auratranslate/tickets/epic-san-chuot/ux-san-chuot/DESIGN.md]
after: []
refined: true
risk: high
---

# Sàn chuột — mọi chức năng độc lập bấm được bằng chuột

## Description

Ice chốt 2026-10-30: ngoài phím tắt, mọi chức năng phải có đường bấm chuột. AD-62 biến câu đó thành bất biến có cổng. Epic hạ cổng Kiểm M trước, kèm sổ nợ chỉ được co lại, rồi các story vá theo bề mặt rút dần sổ nợ cho tới khi rỗng. Đo trên cây hiện tại: 329 command, 259 có điểm chuột, 70 không. 🔵 2026-10-30 (Story 14.1): cổng thật đọc 258 và 71, vì số 259 đếm nhầm một comment ở `src/PromptLibraryOverlay.vue:18`.

## Outcome

Người dùng không nhớ phím vẫn tìm thấy và bấm được mọi chức năng độc lập, và không command mới nào vào được kho mà thiếu đường chuột, thể hiện bằng `check:commands` xanh với `POINTER_DEBT` rỗng.

## Requirements

- R1 (AD-62 mục 2): mỗi command đã đăng ký thuộc đúng một trong ba nhóm: có điểm chuột, khai `pointer`, ở sổ nợ; không thuộc nhóm nào hoặc thuộc hai nhóm thì `check:commands` đỏ.
- R2 (AD-62 mục 1): chỉ `dispatch('<id>')` literal trong `@click`, `@dblclick`, `@submit`, `@contextmenu` mới là điểm chuột; `@mousedown`, listener trên `document` và lời gọi qua hàm không được tính.
- R3 (AD-62 mục 3): `CommandSpec` có trường `pointer` với đúng năm loại `focus`, `aim`, `gesture`, `auto`, `via`, khai ngay ở lượt `register()` và còn nguyên sau `frozen()`; mỗi loại đạt đúng điều kiện máy đọc được của nó.
- R4 (AD-62 mục 4): Kiểm M nằm trong `check:commands`, đọc bộ command thật và thuộc tính từ `scanVueAttrs`, kèm một sàn quần thể qua `judgeFloor`; không cổng thứ mười lăm.
- R5 (AD-62 mục 5): `POINTER_DEBT` là tập con không trùng của `POINTER_DEBT_BASELINE`, chỉ gồm id còn được đăng ký; chủ của nợ nằm ở `deferred-work.md`, mỗi bề mặt một mục, `Chủ:` là một story vá của epic này.
- R6 (AD-62 mục 6 (a)–(h)): mỗi đối chứng đỏ là một phép gỡ thật và đỏ đúng vì lý do đang đo.
- R7 (AD-62 mục 6 (i)): mỗi id rời `POINTER_DEBT` bằng điểm chuột kèm một ca Vitest mount bề mặt, bấm nút, khẳng định `dispatch('<id>')` đã chạy; gỡ thuộc tính khỏi nút làm ca đỏ. Id rời sổ bằng khai `pointer` kèm một ca Vitest bắn sự kiện chuột thật vào bề mặt và khẳng định con trỏ hoặc vùng chọn đổi đúng; gỡ handler chuột làm ca đỏ.
- R8 (AD-62 Prevents 1–2, mục 3): mỗi id trong sổ nợ kết thúc bằng một điểm chuột, một khai `pointer` đúng loại, hoặc bị xoá vì không ai cần; `POINTER_DEBT` rỗng khi epic đóng.
- R9 (AD-62 Prevents 1; AGENTS.md §Known pitfalls, silent emptiness): một nút dispatch command đọc caret hoặc vùng chọn thì chạy đúng trên đoạn người dùng đang sửa; không có đích thì nút tắt hoặc nói lý do, không bao giờ im lặng không làm gì.
- R10 (AD-62 mục 8): `src/commands/README.md` §*Thêm một command* và `src/AGENTS.md` §Conventions mỗi chỗ có một dòng luật về sàn chuột.

## Constraints

- AD-62 mục 8: Kiểm A, AD-34 §1, `unbound()` và màn hình phím tắt không đổi.
- AD-62 mục 1: `@mousedown` vẫn là cửa nhắm mục và đặt caret, không phải cửa thao tác.

## Done when

1. `npm run check:commands` xanh trên cây thật với `POINTER_DEBT` rỗng; các đối chứng (a)–(h) đã chạy và đỏ đúng id.
2. Mỗi id đã rời sổ nợ có ca theo R7 đang xanh, và CI xanh cả nửa Windows.
3. Mục nợ `Chủ: Epic 14` đã đóng: lượt dùng thật của Ice bấm các nút mới trên bản chạy thật ở mọi bề mặt của epic.
4. Phần chỉ kiểm được trên Windows (hành vi focus của nút trên WebView2) nằm trong một mục `B7`.
5. PRD có NFR sàn chuột do John thêm qua correct-course, và `covers` của epic trỏ vào NFR đó.

## Boundaries

- Ngoài epic: rà lại 258 điểm chuột đã có (AD-62 mục 9); kiểm tầm với của một nút; loại miễn trừ thứ sáu, thứ chỉ có thể là một lần sửa AD-62.
- Ngoài epic: hỗ trợ trình đọc màn hình và mọi việc của NFR17 phía bàn phím.
- Must not change: mọi đường bàn phím đang có; hiệu năng gõ trong lưới; luật kéo cửa sổ của AD-58.
- Touch point: `src/TitlebarToolsMenu.vue` — chỗ đặt nút mở lệnh; owner: epic-san-chuot.

## References

- parent — _bmad-output/initiative-auratranslate/initiative-auratranslate.md, section Requirements
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, §AD-62 · §AD-34
- brief — _bmad-output/initiative-auratranslate/ad-brief-san-chuot/ad-brief-san-chuot.md, §2 Đo
- ux — tickets/epic-san-chuot/ux-san-chuot/EXPERIENCE.md, §Bảng chỗ đặt · §Interaction Primitives
- ux — tickets/epic-san-chuot/ux-san-chuot/DESIGN.md, §Components

## Notes

- Decision: 2026-10-30 tracer bullet là entry 1: cổng hạ trước theo AD-62 §Phương án bị loại, entry 2 là chỗ ít chắc nhất vì đụng đường mất caret.
- Decision: 2026-10-30 Ice chọn một epic mới thay vì mở lại Epic 2, 3, 6: chủ của các bề mặt này phần lớn đã xong và sổ nợ cần một chủ duy nhất.
- Decision: 2026-10-30 Ice tách lưới Editor thành entry 2 (10 id) và entry 3 (4 id): vùng chọn nhiều đoạn chưa có đường chuột nào, phần caret được chứng minh độc lập.
- Decision: 2026-10-30 Ice chọn một lượt Sally cho cả epic thay vì từng story, để nút không rải rác.
- Decision: 2026-10-30 Ice giữ Refactor sweep, như Epic 13.
- Decision: 2026-10-30 Sally xong một lượt cho cả epic ở `tickets/epic-san-chuot/ux-san-chuot/`; Sofia nhận làm companion và gỡ `unknown` của entry 2, 4–9.
- Decision: 2026-10-30 Ice chọn menu chuột phải trên câu cho 12 id của Lưới, bỏ hàng nút; entry 2 đổi tên theo đó; khối nhập URL giữ cụm nút trên khối.
- Decision: 2026-10-30 ca (i) nằm ở từng story vá, không ở story cổng, vì nút mới chỉ có ở đó; mỗi story tự viết ca của mình, không dựng helper chung.
- Decision: 2026-10-30 entry 2–9 build lần lượt trên `master`; phần chung là từng dòng id trong `POINTER_DEBT` và mục menu, nên không thêm `after` chỉ để xếp thứ tự.
- Decision: 2026-10-30 CI nửa Windows đọc ở từng story theo AGENTS.md, không có entry riêng.
- Decision: 2026-10-30 Ice chọn `pointer: aim` cho hai id `extend`, `at: src/panels/GridPanel.vue`, không nới `gesture`: AD-62 mục 3 không đổi và cổng còn canh được đường kéo.
- Decision: 2026-10-30 Ice ký dòng 🔵 ở AD-62 mục 6 (i), khớp hai đường của R7.
- Decision: 2026-11-01 ba nợ mã Ice đưa ra khỏi Story 14.10 thành entry 11, 12, 13, đứng sau Refactor sweep như một đợt nối tiếp; sweep không dời xuống cuối, không thêm sweep thứ hai; `covers` trống vì ba entry bám Done when 3; build lần lượt trên `master` như entry 2–9, nên `GridPanel.vue` và `editorPanelState.ts` dùng chung không cần thêm `after`.
- Decision: 2026-11-01 Ice nới "mọi đường bàn phím đang có không đổi" cho đúng một hành vi: đóng lớp phủ trả caret về chỗ cũ, cả khi mở bằng phím (entry 11).
- Decision: 2026-11-01 Ice chọn đóng phiên Rà soát khi Chương khác đã mở xong, không chặn, không chuyển phiên theo Chương (entry 12).
- Open question: NFR sàn chuột chưa có số; John thêm qua correct-course ngoài cây story, rồi `covers` của epic mới trỏ được vào nó. 🔵 2026-11-01: đã thêm NFR20 (commit `37641584`), `covers` trỏ vào đó, Done when 5 đạt.

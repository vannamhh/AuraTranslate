---
title: 'Gõ có dấu ở Song song chậm vì mỗi lần sửa chữ bố cục lại cả lưới'
type: 'bugfix'
ticket: 19
created: '2026-11-12'
status: 'built'
baseline_revision: '6ec62f9f02e843142d16a961e3fc4414ea2a3cbc'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/chromium-bench/handoff-b.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Trên Blink, mỗi lần sửa chữ trong lưới bị bố cục lại từ đầu, không dùng lại kết quả cũ. Lý do: ô cuộn `.grid-scroll` (`flex: 1`, tức `1 1 0%`, kèm `overflow: auto`) nằm dưới một chuỗi `height: 100%` (panel và dockview), và thanh cuộn 10px của nó chiếm chỗ. Ở Song song, mỗi lần bố cục lại từ đầu tốn 67 ms trên máy ảo Windows.

**Approach:** Cho ô cuộn một bề rộng ổn định giữa các lượt bố cục, để Blink dùng lại kết quả cũ, bằng `scrollbar-gutter: stable` trên `.grid-scroll`.

**Decisions:**
- 2026-11-12, Ice: chọn hướng (c), tức tìm đúng chỗ gây bố cục lại rồi sửa ở đó. Không chọn (a) hay (b).
- 2026-11-12, Ice: điều tra bằng (A). Kết quả cho thấy lỗi nằm trong app, không nằm ở Blink trên Windows, nên đi tiếp bằng (B).
- 2026-11-12, Ice: chưa mở đường đổi cấu trúc lưới. Chờ số đo rồi mới quyết.
- 2026-11-12, Ice: sửa bằng `scrollbar-gutter: stable` (1,56 ms trên bản chụp, mốc 39 ms). Không chọn `flex: 1 1 0px` (dựa vào chi tiết riêng của Blink, dễ bị gỡ nhầm) hay `overflow-y: scroll` (thanh cuộn luôn hiện). Chương ngắn mất 10px mép phải là chấp nhận được.

## Boundaries & Constraints

**Always:** Giữ các Boundaries của vé 19: hàng thẳng, ruby đúng, vùng chọn và Auto-Lookup giữ nguyên, AD-35. Lưới vẫn cuộn được, và Chương dài vẫn không bị cắt chữ.

**Never:** Không đổi subgrid hay cách chia khối 200 hàng. Không gỡ thanh cuộn 10px dùng chung của app (`src/tokens/controls.css:729`). Không sửa các ô cuộn khác ngoài lưới trong vé này.

</frozen-after-approval>

## Code Map

- `src/panels/GridPanel.vue:2177-2183` -- `.grid-scroll`. Đây là chỗ sửa. Khối comment ngay trên nó giải thích vì sao lưới cần `flex: 1` và `min-height: 0` để cuộn được mà không cắt chữ, nên đọc trước khi sửa.
- `src/panels/PanelFrame.vue:127` -- `.panel{height:100%}`, một mắt xích của chuỗi chiều cao. Không sửa.
- `src/modes/LibraryMode.vue:943-946` và `src/modes/ReadingMode.vue:1054-1057` -- tiền lệ dùng `scrollbar-gutter: stable`, kèm `padding-right: 11px` để mép phải vẫn cách 21px.
- `e2e/specs/hanviet-parallel-row-height.e2e.mjs` -- spec ép thanh cuộn luôn hiện. Phải chạy lại.
- `_bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/chromium-bench/` -- bàn đo của lượt điều tra: `bench-snap.mjs`, `variants.mjs`, `bench-synth.mjs` và `handoff-b.md` (số đo đầy đủ, lệnh chạy lại). Bản chụp DOM nằm ngoài repo.

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/GridPanel.vue` -- thêm `scrollbar-gutter: stable` vào `.grid-scroll` -- để bề rộng ô cuộn không đổi giữa các lượt bố cục.
- [x] `e2e/measure/grid-scroll-relayout.mjs` -- chuyển `bench-synth.mjs` (chuỗi chiều cao đầy đủ, thanh cuộn 10px) thành bàn đo đọc `PW_PATH` như `2-15-ban-do/thu-engine/run-pw.mjs`. Bàn đo in trung vị và p90 bố cục mỗi lần sửa, ở 90 và 200 hàng, cho hai trường hợp: có và không có phép sửa -- đây là tiêu chí 3.
- [ ] một spec lưới hiện có trong `e2e/specs/` -- assert kiểu tính được của `.grid-scroll` mang đúng thuộc tính đã sửa -- để giữ phép sửa không bị gỡ.
- [x] `_bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/research-go-cham-luoi-subgrid-webkit.md` -- thêm D13: nguyên nhân, bảng số, sửa D12, và ghi rằng `totalObjects` là cỡ cả cây.

**Acceptance Criteria:**
- Given bản chụp Song song 90 hàng, Chrome 157, thanh cuộn hiện, when chạy `bench-snap.mjs` trên bản đã sửa, then trung vị một lần bố cục ≤ 3 ms. Mốc trước khi sửa là 39 ms.
- Given bàn đo `e2e/measure/grid-scroll-relayout.mjs`, when chạy, then trường hợp chưa sửa tăng theo cỡ lưới, còn trường hợp đã sửa thì không.
- Given `hanviet-parallel-row-height.e2e.mjs` và các ca vùng chọn qua ranh giới khối, when chạy, then cả hai đều xanh.
- Given Chương ngắn không cần cuộn và Chương dài, when mở ở cả ba kiểu xem, then lưới cuộn được, không cắt chữ, và mép phải như Ice đã duyệt.
- Tiêu chí 1, 2 và 4 của vé đo trên bản `v1.0.3`, theo quyết định của Ice. Bước đó ghi thành mục nợ `Chủ: Epic 2`, và phần Boot Camp ghi `Chủ: B7`.

## Design Notes

Đo ngày 2026-11-12, Chrome 157.0.8103.52, Mac, dpr 1, headless nhưng bỏ `--hide-scrollbars`. Bản chụp DOM lấy từ app thật qua wdio. Mỗi trường hợp gồm 40 lần sửa sau 5 lần khởi động, chạy hai lượt.

- Song song 90 hàng: không có thanh cuộn 1,4 ms; có thanh cuộn 38,7 ms rồi 38,9 ms (orchestrator chạy lại được 39,0 ms). Song song 200 hàng có thanh cuộn: 92–99 ms. Kiểu thứ ba: 14 ms. Nguyên văn: 5 ms. Tỉ lệ giữa ba kiểu khớp với vệt Windows (67 / 20 / 7 ms).
- Trang tổng hợp mặc định 1,1 ms. Khi bọc thêm chuỗi `height: 100%` của dockview và thanh cuộn 10px, nó lên 42 ms. ⇒ Đã xác nhận theo hai chiều.
- Mọi bàn đo trên Mac trước đó đều nhanh, vì Playwright headless tự thêm `--hide-scrollbars`. Trang `do-tren-windows.html` mà Ice chạy trên máy ảo thì không có chuỗi chiều cao này, nên cũng nhanh.
- Cơ chế (suy luận): ở lượt flex đầu, ô cuộn chưa có thanh dọc (rộng 638px); ở lượt cuối thì có (628px). Hai bề rộng đổi qua lại, nên bộ đệm bố cục trượt ở mọi lần sửa.

## Implementation Notes

- Bản đo ở Mac trước đó nhanh vì Playwright headless tự thêm `--hide-scrollbars`; bàn đo mới bỏ cờ đó (`ignoreDefaultArgs`).
- Đo trên bản đã sửa (Chrome 157, thanh cuộn hiện, n=40, hai lượt): `bench-snap.mjs` Song song 90 hàng 41,1 / 38,9 ms → 1,59 / 1,54 ms; `grid-scroll-relayout.mjs` 90 hàng 41,6 / 44,0 → 1,31 / 1,32 ms, 200 hàng 93,0 / 94,6 → 2,33 / 2,38 ms.
- `grid-blocks.e2e.mjs` từng đỏ một lần ở lượt của agent cài đặt, ca Mod+Shift+ArrowDown (nhận hàng 221–225 thay vì 199–203), và vẫn đỏ khi gỡ phép sửa. Chạy lại trên cùng binary có phép sửa: 5/5 xanh, `hanviet-parallel-row-height` 5/5 xanh. Nightly 2026-11-11 cũng xanh ca này.
- Kiểm chéo bằng phép gỡ thật (xoá dòng `scrollbar-gutter` khỏi `GridPanel.vue`): bàn đo in `WARNING`, ca sản phẩm lại 42,7 ms ở 90 hàng; ca e2e đỏ với `Received "auto"`. Trả dòng sửa lại: `hanviet-parallel-row-height` và `grid-blocks` cùng xanh 5/5.
- Hai mục nợ của tiêu chí đo trên `v1.0.3` đã ghi vào `deferred-work.md` (`Chủ: Epic 2`, `Chủ: B7`).

## Review Triage Log

Lượt 1 (quick), 2026-11-12: medium 1 · low 9 · false 1 · maybe-false 1 · thiếu bằng chứng 1. Patch 6, defer 2, bác 5.

- medium, patch: `grid-scroll-relayout.mjs` tự thêm `scrollbar-gutter`, nên gỡ phép sửa trong `GridPanel.vue` không đổi số đo (e2e/AGENTS.md cấm bàn đo chép mã sản phẩm). Sửa bằng cách đọc khối `.grid-scroll` từ `GridPanel.vue`.
- low, patch: D12 và dòng tóm tắt ~38 còn đọc `totalObjects` là số đối tượng được bố cục. Thêm câu 🔵 tại chỗ.
- low, patch: D13 mở đầu bằng 🔴, ký hiệu dành cho điều bất biến. Gỡ ký hiệu.
- low, patch: các script trong `chromium-bench/` có đường dẫn cứng của máy Ice và của scratchpad. Đổi sang `PW_PATH`, `CHROME_PATH` và `os.tmpdir()`.
- low, patch: ca e2e gọi `gutter.split` nên ném TypeError, gây đỏ sai, nếu WebKit thiếu thuộc tính đó. Assert bằng `String(...)`.
- low, patch: comment trong `GridPanel.vue` nêu cơ chế suy luận như sự thật. Viết lại theo điều đã đo.
- low, bác: comment tiếng Việt kèm số đo trong các script research. Đây là bằng chứng nằm ở `_bmad-output/`, và AGENTS.md cấm dọn hàng loạt.
- low, bác: `variants.mjs` gắn với hash scoped và chiều cao px của một bản chụp. Phép chia đôi vốn chỉ đúng cho bản chụp đó.
- low, bác: ca đã sửa vẫn tăng theo số hàng, từ 1,3 lên 2,3 ms. Đây là chi phí riêng của lưới, khoảng 10 µs mỗi hàng, có từ trước thay đổi này, và đã báo cho Ice.
- low, bác: describe chứa ca mới có tiêu đề về chiều cao hàng. Chỉ là vị trí đặt ca.
- false: con trỏ "mục ngay trên" trong `deferred-work.md` đúng khuôn các mục hiện có, và `check:debt-owner` đạt.
- maybe-false, defer: `grid-blocks` ca Mod+Shift+ArrowDown đỏ một lần, ở cả hai phía của phép sửa. Chạy lại thì xanh 5/5. Nếu đúng là ca chập chờn thì ở mức medium. Muốn chốt phải chạy ca đó nhiều lượt trên `master`.
- thiếu bằng chứng, defer: AC4 (Chương ngắn và dài, ba kiểu xem, mép phải) cần mắt Ice trong app thật. Ghi nợ `Chủ: Epic 2`.

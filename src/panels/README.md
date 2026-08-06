Bốn panel: `Source` · `Lookup` · `AiTranslation` · `Editor`.

Component đặt tên `PascalCase.vue` (Consistency Conventions). Panel Lookup → `LookupPanel`.

---

## Ranh giới sở hữu

| | Story | Trạng thái |
|---|---|---|
| `PanelFrame.vue` — vỏ panel + **hợp đồng thị giác tiêu điểm** (AC5, UX-DR8, UX-DR17) | **1.6** | ✅ đã dựng |
| Bốn panel thật trong `dockview`, lưới 2×2, preset bố cục, ngưỡng màn hình hẹp | **1.14** | ⬜ |
| Nội dung panel Source + tab Hán Việt | **1.16** | ✅ đã dựng |
| Nội dung panel Lookup — bản ghi từ điển có cấu trúc | **1.17** | ✅ đã dựng |
| Panel AiTranslation | **Epic 4** | ⬜ |
| Panel Editor | **Epic 2** | ⬜ |

**Story sở hữu nội dung: 1.14.** `PanelFrame.vue` hôm nay là **vỏ**, không phải panel: thanh tiêu đề, tiêu đề `ui-md`, và thân **để trống**. `WorkspaceMode.vue` dựng **hai** `PanelFrame` — `panel.source` và `panel.editor`, đúng cặp *Nguyên văn | Bản dịch* mà UX-DR15 nói *"không bao giờ nhường"*. Hai chứ không bốn: một cái không đủ để nhìn thấy tương phản có/không tiêu điểm, bốn cái là dựng trước Story 1.14. **Story 1.14 thay chỗ hai cái này bằng bốn panel trong `dockview`.**

---

## Hợp đồng thị giác tiêu điểm — đã đo, đừng sửa lại mà không đo lại

Panel có tiêu điểm: **vạch dọc 2px `primary` ở mép trái** + **tiêu đề chuyển `primary` in đậm**. ⛔ **Không dùng viền bao quanh để báo tiêu điểm** (AC5, UX-DR8, `DESIGN.md §Components`).

- ⛔ Vạch **không** được làm bằng `box-shadow`: Kiểm F của `check-tokens.mjs` cấm `box-shadow`/`text-shadow` **không có đường miễn trừ** (AC7 Story 1.4 — không elevation). Cách đúng là một `::before` `position:absolute; left:0; width:2px`.
- ⚠️ Trạng thái tiêu điểm đọc từ **DOM thật** (`focusin`/`focusout` với kiểm `relatedTarget`), không từ một cờ do ứng dụng tự giữ. Một cờ tự giữ sẽ vẫn sáng đúng một panel trong khi focus thật đã rơi ra ngoài — vạch dọc nói dối, và đúng nửa NFR17 mà AC5 tồn tại để giữ thì mất.
- ⚠️ `outline: none` **chỉ** ở gốc `tabindex="-1"` của panel, kèm lý do ngay cạnh dòng CSS. ⛔ Một `*:focus { outline: none }` phá NFR17 mà **không cổng nào bắt được**.
- ⚠️ `font-weight` đang mượn `var(--weight-read-title)` (600) vì bộ token **không có** biến trọng lượng cho nhãn giao diện đậm (`ui-md` là 400, `ui-label` là 700). Viết thẳng `600` thì Kiểm B2 của `check-tokens.mjs` đỏ, và ⛔ khai một biến CSS cục bộ để lách cổng là đúng thứ AD-34 tồn tại để chặn. Story 1.14 quyết token thật — xem `deferred-work.md`.

## Phân tách panel — hai cơ chế, không phải một

Bốn biến do `applyTheme()` ghi (`--panel-border-width` · `--panel-border-color` · `--panel-gap` · `--panel-radius`) mang cơ chế của theme đang chạy: **sáng phân tách bằng NÉT 1px `outline`, tối bằng KHE 2px lộ `background` cộng bo 3px** (AC6 Story 1.4). Component **không bao giờ** phải biết mình đang ở theme nào. ⛔ Đừng thay `gap: var(--panel-gap)` bằng một khoảng cách viết thẳng — làm vậy là thống nhất hai cơ chế về một cách làm, đúng thứ AC6 cấm.

*Đo thật 2026-08-04 trên panel thật (lần đầu — Story 1.4 mới chỉ nghiệm thu ở tầng token): sáng `border 1px #e2dccf`, khe `0px`; tối `border 0px`, khe `2px` lộ `#201e1b`, bo `3px`.*

## Chữ trong thân panel

⚠️ Hai panel còn lại (AiTranslation, Editor) hôm nay **để trống**. Ngày chúng đổ chữ vào, bề mặt đó **phải khai token `read-*` / `source-*` / `lookup-*` của chính nó**. Mặc định kế thừa từ `body` là `ui-md` ở giãn dòng **1.5** — dưới sàn 1.66 của AC5 Story 1.4 — và Kiểm E của `check-tokens.mjs` chỉ đọc `tokens.json` nên hoàn toàn mù với việc component nào đang kế thừa gì. Xem mục tương ứng ở `deferred-work.md`.

**Panel Source (Story 1.16) đã đóng nửa của nó**: nguyên văn khai `source-cjk` (tiếng Trung) hoặc `source-latin` (tiếng Anh — token thứ 16, Quyết định #6) tuỳ `work.source_lang`; tab Hán Việt khai `source-hanviet`. State (Chương đã nạp, tab/kiểu xem đang chọn, âm Hán Việt đã tra) sống ở `src/panels/sourcePanelState.ts` — module-level, sống sót qua một lượt đổi preset (AC9). Xem doc-comment đầu `SourcePanel.vue`/`SourceHanViet.vue`.

**Panel Lookup (Story 1.17) đã dựng xong bản ghi có cấu trúc**: đầu mục khai `lookup-headword`, nghĩa khai `lookup-gloss`, ví dụ/trích dẫn/ghi chú/từ loại khai `lookup-example`, nhãn nguồn + nhãn ngoại ngữ khai `ui-label`. Khối một nguồn sống ở `src/panels/LookupRecord.vue` (nhận `group`/`senses` đã lọc sẵn); vùng đầu mục cố định + thanh nhịp + bốn trạng thái rỗng + hai banner sống ở `LookupPanel.vue`. State (truy vấn, kết quả pha một+hai, năm vị từ trạng thái) sống ở `src/panels/lookupPanelState.ts` — module-level, sống sót qua đổi preset, reset khi đổi Tác phẩm (AC10). Đường kích hoạt là phím `Mod+Alt+L` (`lookup.lookup_selection`), lấy vùng chọn qua `window.getSelection()` — dep TỐI THIỂU, Story 1.18 sẽ thay bằng hợp đồng vùng chọn dùng chung cho bốn panel. ⚠️ **Món nợ chưa đóng**: hình dạng hiển thị cho mục từ TIẾNG ANH (`deferred-work.md:317`) dùng tạm cấu trúc khối giống tiếng Trung — chủ sở hữu vẫn là Sally (UX), ⛔ chưa phải chữ ký chính thức.

## Điểm vào focus

`PanelFrame` nhận `owner` qua **prop** (`owner="panel.source"`) rồi tự `declareFocus(props.owner, …)`. Owner phải có mặt trong `FOCUS_OWNERS` ở `src/commands/index.ts`; cổng đối chiếu hai chiều. Xem `src/commands/README.md`.

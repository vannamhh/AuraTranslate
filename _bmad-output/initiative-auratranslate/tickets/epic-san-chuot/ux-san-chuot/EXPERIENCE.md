---
name: Sàn chuột
status: final
created: 2026-10-30
updated: 2026-10-30
design: DESIGN.md
sources:
  - _bmad-output/initiative-auratranslate/tickets/epic-san-chuot/epic-san-chuot.md
  - _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md
  - _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md
---

# Sàn chuột — Experience

## Foundation

Bản này chỉ ghi phần khác `ux-auratranslate/EXPERIENCE.md`. Ứng dụng desktop, chuột là đường thứ hai bên cạnh bàn phím: không phím nào đổi, nút mới không đè lên việc gõ. Dáng nút ở `DESIGN.md`.

Kết quả cho 43 id: 37 id có điểm chuột (34 id nhận nút hoặc đích bấm mới, 3 id giữ nguyên chỗ và chỉ viết lại thành `dispatch` literal), 6 id khai `pointer: aim`.

## Information Architecture

Một thao tác nằm ở chỗ người dùng đang nhìn vào đối tượng của nó. Đối tượng là câu thì ở menu chuột phải trên chính câu đó. Đối tượng là Chương đang mở thì ở thanh trạng thái. Đối tượng là bố cục hoặc cả ứng dụng thì ở thanh tiêu đề.

| Bề mặt | Thêm gì |
|---|---|
| Lưới | Menu câu mở bằng chuột phải, kéo trên cột số câu |
| Thanh trạng thái | Hai nút chuyển Chương |
| Thanh tiêu đề | Nút Review Mode thứ ba trong công tắc bố cục |
| `Công cụ ▾` | Ngưỡng quét Glossary, ba công tắc panel |
| Dải Rà soát | Hai nút dời diff |
| Xem trước nhập URL | Nhắm khối bằng chuột, cụm nút trên khối đang nhắm, nút Chương, nút tới luật làm sạch |
| Xem trước song ngữ | Điểm cắt bấm được, hai nút cặp lệch |
| Bề mặt của Story 14.9 | Một nút tại chỗ, xem bảng dưới |

### Bảng chỗ đặt

Nhãn là chữ hiện trên nút; `—` là không có nút, "biểu tượng …" là nút chỉ có biểu tượng, "Không đổi" là giữ nút hiện có. Khoá `vi.json` mới do story đặt. Cột "Kết thúc" nói id rời sổ nợ theo cách nào.

**Story 14.2 — Lưới**

| Id | Chỗ | Nhãn | Kết thúc |
|---|---|---|---|
| `editor.confirm_segment` | Menu câu, nhóm 1 | Xác nhận | điểm chuột |
| `editor.merge_segments` | Menu câu, nhóm 1 | Gộp lên | điểm chuột |
| `editor.split_segment` | Menu câu, nhóm 1 | Tách câu | điểm chuột |
| `editor.next_untranslated` | Menu câu, nhóm 1 | Câu chưa dịch kế | điểm chuột |
| `editor.clear_source_cuts` | Menu câu, nhóm 1, ngay dưới Tách câu, chỉ khi có điểm cắt | Bỏ điểm cắt | điểm chuột |
| `editor.omit_segment` | Menu câu, nhóm 3 | Cắt bỏ câu | điểm chuột |
| `editor.restore_segment` | Menu câu, cùng chỗ với dòng trên, đổi theo trạng thái | Đưa câu trở lại | điểm chuột |
| `editor.end_target_paragraph` | Menu câu, nhóm 3 | Kết thúc đoạn | điểm chuột |
| `editor.join_target_paragraph` | Menu câu, cùng chỗ với dòng trên, đổi theo trạng thái | Nối với đoạn tiếp theo | điểm chuột |
| `editor.split_chapter` | Menu câu, nhóm 3 | Tách Chương tại đây | điểm chuột |

**Story 14.3 — vùng chọn và Chương**

| Id | Chỗ | Nhãn | Kết thúc |
|---|---|---|---|
| `segment.selection.extend_down` | Kéo hoặc `Shift`+bấm trên cột số câu | — | `pointer: aim` |
| `segment.selection.extend_up` | Như trên | — | `pointer: aim` |
| `editor.prev_chapter` | Thanh trạng thái, trước dòng vị trí | ‹ | điểm chuột |
| `editor.next_chapter` | Thanh trạng thái, ngay sau nút trên | › | điểm chuột |

**Story 14.4 — Rà soát và Lịch sử**

| Id | Chỗ | Nhãn | Kết thúc |
|---|---|---|---|
| `review.open` | Công tắc bố cục, ô thứ ba | biểu tượng Review Mode | điểm chuột |
| `review.diff_prev` | `.review-bar`, ngay sau số thay đổi | biểu tượng lên | điểm chuột |
| `review.diff_next` | `.review-bar`, ngay sau nút trên | biểu tượng xuống | điểm chuột |
| `history.open` | Menu câu, nhóm 2 | Lịch sử phiên bản | điểm chuột |

**Story 14.5 — Glossary**

| Id | Chỗ | Nhãn | Kết thúc |
|---|---|---|---|
| `glossary.add_term` | Menu câu, nhóm 2 | Thêm thuật ngữ | điểm chuột |
| `glossary.settings.open` | `Công cụ ▾`, ngay dưới Quản lý Glossary | Ngưỡng quét Glossary | điểm chuột |

**Story 14.6 — xem trước nhập URL**

| Id | Chỗ | Nhãn | Kết thúc |
|---|---|---|---|
| `import.preview.block_next` | Bấm vào khối để nhắm | — | `pointer: aim` |
| `import.preview.block_prev` | Như trên | — | `pointer: aim` |
| `import.preview.block_toggle_kept` | Cụm nút trên khối đang nhắm | Loại khối này / Giữ khối này | điểm chuột |
| `import.preview.block_mark_range_start` | Cụm nút trên khối đang nhắm | Đầu vùng giữ | điểm chuột |
| `import.preview.block_confirm_range` | Cụm nút, chỉ khi đã có đầu vùng ở khối khác | Cuối vùng giữ | điểm chuột |
| `import.preview.jump_to_cleanup_rules` | Dòng tiêu đề tầng 2, sát phải | Tới luật làm sạch | điểm chuột |
| `import.preview.chapter_prev` | Trước dòng "Chương n/N" của tầng 2 | ‹ | điểm chuột |
| `import.preview.chapter_next` | Ngay sau nút trên | › | điểm chuột |

**Story 14.7 — xem trước song ngữ**

| Id | Chỗ | Nhãn | Kết thúc |
|---|---|---|---|
| `import.preview.bilingual_caret_left` | Bấm vào điểm cắt để nhắm | — | `pointer: aim` |
| `import.preview.bilingual_caret_right` | Như trên | — | `pointer: aim` |
| `import.preview.bilingual_toggle_cut` | Mỗi điểm cắt trên dòng bản dịch | `¦` / `‖` | điểm chuột |
| `import.preview.bilingual_previous_mismatch` | Dòng tiêu đề cặp lệch | Cặp lệch trước | điểm chuột |
| `import.preview.bilingual_next_mismatch` | Ngay sau nút trên | Cặp lệch sau | điểm chuột |

**Story 14.8 — panel**

| Id | Chỗ | Nhãn | Kết thúc |
|---|---|---|---|
| `layout.toggle_grid` | `Công cụ ▾`, nhóm cuối, công tắc | Đối chiếu | điểm chuột |
| `layout.toggle_lookup` | Như trên | Tra cứu | điểm chuột |
| `layout.toggle_ai_translation` | Như trên | Đề xuất AI | điểm chuột |

**Story 14.9 — lệnh lẻ**

| Id | Chỗ | Nhãn | Kết thúc |
|---|---|---|---|
| `ai.proofread.hide` | `.pfs-head`, sát phải | Ẩn | điểm chuột |
| `library.chapters.clear_selection` | `.wp-foot`, ngay sau số đã chọn, chỉ khi có chọn | Bỏ chọn | điểm chuột |
| `reading.marks_clear_cancel` | `.toc-actions`, ngay sau nút bỏ hết, chỉ khi đang chờ xác nhận | Huỷ | điểm chuột |
| `export.alignment.prev` | `.al-actions`, ngay trước nút xem tiếp, ẩn ở mục đầu | Mục trước | điểm chuột |
| `prompt.import.open` | Giữ nguyên nút hiện có | Không đổi | điểm chuột, viết lại `@submit` |
| `tm.manage.edit` | Giữ nguyên nút trên hàng | Không đổi | điểm chuột, viết lại `@submit` |
| `tm.manage.delete` | Giữ nguyên nút trên hàng | Không đổi | điểm chuột, viết lại `@submit` |

## Voice and Tone

Nhãn nút là một động từ hoặc một cụm ngắn, theo `ux-auratranslate/EXPERIENCE.md` §Voice and Tone. Tên đầy đủ và phím nằm ở `title`, lấy từ nhãn command và `shortcutRows`, nên phím người dùng tự gán cũng hiện đúng. Mỗi mục menu câu ghi kèm phím, như `Công cụ ▾`.

⚠️ Thông báo `panel.grid.regroup_no_cut` hôm nay viết "bấm vào cột nguyên văn", nhưng bấm trơn không đánh dấu gì; cử chỉ thật là `Mod`+bấm. Story 14.2 sửa thông báo này và in phím theo nền tảng, vì mục Tách câu đưa người dùng chuột tới đúng thông báo đó.

`panel.ai_translation.batch_select_hint` thêm cách kéo trên cột số câu. `status.regroup_hint` ở thanh trạng thái đổi thành "Chuột phải vào câu để xem thao tác"; đây là dấu hiệu nhìn thấy duy nhất của menu câu, và phím đã ghi ngay trong menu.

## Component Patterns

**Menu câu.** Chuột phải vào bất kỳ ô nào của một câu mở menu ngay tại con trỏ, tiêu đề là `Câu N`. Ba nhóm, ngăn bằng nét kẻ: Xác nhận · Gộp lên · Tách câu · Bỏ điểm cắt · Câu chưa dịch kế; Thêm thuật ngữ · Lịch sử phiên bản; Cắt bỏ câu · Kết thúc đoạn · Tách Chương tại đây. Menu không có nút mở nào trên màn hình, nên Lưới không thêm chiều cao hay bề rộng.

`.grid` mang `@contextmenu.prevent="dispatch('editor.sentence_menu.open')"`. Command này mới, không hợp âm mặc định, có điểm chuột ngay từ lúc đăng ký nên không vào sổ nợ. Không có câu dưới con trỏ thì không mở. Đóng khi chọn mục, khi bấm ra ngoài, khi `Esc`, khi Lưới cuộn. Xem trước nhập URL không dùng khuôn này: Ice giữ cụm nút trên khối.

Ice chọn cách này sau khi so các phương án ở `mockups/noi-dat-thao-tac-cau.html`. Đo 2026-10-30 trên Playwright WebKit 27.2 bằng font thật của app; Lưới rộng 640px là giả định dockview chia đôi cửa sổ 1280, chưa đo trong app.

| Phương án | Cần | Lưới 900 | Lưới 640 | Lưới 550 |
|---|---|---|---|---|
| Hàng đầy đủ | 716px | 1 hàng, cao 42px | 2 hàng, cao 77px | 2 hàng, cao 77px |
| Hàng gọn + `Câu ▾` | 507px | chung hàng tab | chung hàng tab | chung hàng tab |
| Thanh bám hàng | 234px | thanh nổi | thanh nổi | thanh nổi |
| Menu chuột phải (Ice chọn) | 0px | không chiếm chỗ | không chiếm chỗ | không chiếm chỗ |

Cái giá của lựa chọn này: mỗi lệnh tốn hai cú bấm, và không nút nào nhìn thấy trên màn hình.

**Ô Review Mode.** Ô thứ ba của công tắc bố cục. Khi Review Mode đóng, ô dispatch `review.open`. Khi mở, ô đang chọn dispatch `review.close`, là một `<button v-if>` riêng ở cùng chỗ. ⚠️ Hôm nay `review.close` chỉ có nút trong dải thông báo; mở bằng chuột mà không có đường đóng bằng chuột là một ngõ cụt, nên hai nút đi cùng nhau.

**Công tắc panel.** Ba mục `menuitemcheckbox` cuối `Công cụ ▾`, như `layout.toggle_panel_headers`, chỉ hiện ở Workspace, khoá khi Review Mode mở.

## State Patterns

Một mục đọc caret hoặc vùng chọn phải có đích hoặc nói lý do (R9 của epic). Chuột phải tự cấp đích là câu dưới con trỏ, nên gần như không mục nào cần tắt.

| Mục | Tắt khi | Khi chọn mà không làm được |
|---|---|---|
| Xác nhận | Không bao giờ | Thông báo từ chối đã có ở thanh trạng thái |
| Gộp lên | Câu đầu của Chương | Thông báo từ chối đã có ở thanh trạng thái |
| Tách câu | Điểm cắt đang ở câu khác; `title` nói câu nào | Chưa có điểm cắt thì thông báo `regroup_no_cut` nói cách đặt |
| Bỏ điểm cắt | Ẩn khi không có điểm cắt | — |
| Câu chưa dịch kế | Không bao giờ | Thông báo "không còn câu nào" đã có |
| Thêm thuật ngữ | Không bao giờ | Không bôi đen thì mở ô trống |
| Lịch sử · Cắt bỏ · Đưa trở lại · Kết đoạn · Nối đoạn · Tách Chương | Không bao giờ | Xem đoạn cảnh báo về đường từ chối ngay dưới bảng |
| ‹ › Chương | Không bao giờ, khi Chương đã nạp | Thông báo ở biên Chương đã có |
| Cuối vùng giữ | Ẩn khi chưa có đầu vùng | — |

⚠️ `Tách câu` đọc `sourceCut`, không đọc caret. Điểm cắt nằm ở câu 5 mà chuột phải vào câu 20 rồi chọn Tách câu thì câu 5 bị tách, và AD-5 không cho hoàn tác. Vì vậy mục này tắt khi điểm cắt không thuộc câu đang nhắm.

⚠️ Hai đường từ chối hôm nay im lặng: `editorOmitError` chưa có component nào đọc, và `setCurrentSegmentParagraphEnd` từ chối chỉ ra `console.warn`. Với phím, người dùng ít thấy; với mục menu vừa bấm, đó là một cú bấm không có gì xảy ra. Story 14.2 đưa cả hai ra thanh trạng thái.

`omit`/`restore` và `end`/`join` mỗi cặp chiếm một dòng, hai `<button v-if>` mỗi nút một `dispatch` literal, quyết định theo `is_omitted` và `is_paragraph_end` của câu có caret. Đây là khuôn "một chỗ, đổi theo trạng thái" ở `ux-auratranslate/DESIGN.md` §Nút.

🔵 2026-10-30 (Story 14.2): cặp Kết đoạn · Nối đoạn quyết định theo `is_target_paragraph_end`, không theo `is_paragraph_end` như dòng trên ghi. Hai lệnh đổi cờ của bản dịch (AD-46), và đọc cờ nguồn sẽ hiện sai mục.

🔵 2026-10-31 (Story 14.6): ở tầng khối nhập URL, ba nút của cụm (Loại/Giữ khối này, Đầu vùng giữ, Cuối vùng giữ) tắt khi con trỏ Chương k > 0 (`title` nói chỉ sửa được ở Chương đầu), khi một lượt ghi đang bay hoặc đang xác nhận nhập; ‹ › tắt ở hai đầu và khi Chương đang dựng lại. Bấm ‹ › giữ nguyên tầm nhìn và không kéo tiêu điểm sang danh sách Chương như `⌥←` `⌥→`, vì danh sách nằm dưới tầng khối và nút sẽ rời khỏi con trỏ.

🔵 2026-10-30 (Story 14.3): ‹ › Chương ẩn khi Review Mode mở, thay cho "Không bao giờ" ở bảng trên. Hợp âm chuyển Chương vẫn chạy trong Review Mode; việc đó thuộc Story 14.4.

🔵 2026-10-31 (Story 14.4): câu trên không còn đúng: hợp âm chuyển Chương không đổi Chương khi Review Mode mở, `.diff-status` nói phải đóng Review Mode trước, vì một phiên Rà soát gắn với một Chương.

🔵 2026-11-01 (Story 14.12): mở một Chương khác từ Thư viện, Chế độ đọc hay Workspace khi Review Mode mở thì đóng phiên sau khi Chương mới đã mở xong; mở lại đúng Chương đang rà soát thì giữ nguyên. Lượt mở bị chặn giữ phiên và hiện lý do ở thanh trạng thái, cả ở Thư viện.

🔵 2026-10-31 (Story 14.5): Thêm thuật ngữ tắt khi dải đang mở, thay cho "Không bao giờ" ở bảng trên. `openGlossaryQuickAdd` bỏ qua lượt mở thứ hai, nên một cú bấm khi dải đang mở sẽ không làm gì; `title` nói dải đang mở. Đóng dải đã mở từ menu câu, bằng `Esc` hay sau Lưu, trả caret, vị trí trong chữ và vùng bôi của người dùng như đóng menu không chọn. Nhóm 2 của menu về giữa nhóm 1 và nhóm 3 đúng như §Component Patterns, và mục Lịch sử đứng sau Thêm thuật ngữ.

Dòng vị trí ở thanh trạng thái biến mất khi caret `null`, nên hai nút ‹ › neo theo `editorHasLoaded()` và đứng trước dòng đó để không xê dịch.

## Interaction Primitives

**Nhắm câu bằng chuột phải.** Đo 2026-10-30 trên Playwright WebKit 27.2 và Chromium 156 bằng một trang tái tạo `onCellMouseDown` và `onColumnFocusOut`, chưa đo trên WKWebView thật. Ca: caret đang ở câu 1, chuột phải vào câu 2.

| Bấm vào | Caret sau cú bấm | Bôi đen | Lệnh chạy trên |
|---|---|---|---|
| Ô bản dịch câu 2 | câu 2 | WebKit tự bôi đen từ dưới con trỏ | câu 2, đúng |
| Nguyên văn hoặc số câu 2 | `null` | WebKit tự bôi đen từ; Chromium không | không câu nào |
| Như trên, có chặn `mousedown` | vẫn câu 1 | WebKit vẫn tự bôi đen từ | câu 1, **nhầm câu, không báo gì** |
| Trong vùng đã bôi đen ở nguyên văn | giữ nguyên | giữ nguyên ở cả hai | theo vùng bôi đen |

Từ bảng đo rút ra các luật:
- `mousedown` nút phải trên ô nguyên văn và ô số câu gọi `preventDefault`, rồi nhắm chính câu đó bằng `setEditorCaret`. Chỉ chặn mà không nhắm là đường nhầm câu ở dòng thứ ba.
- Không gọi `focus()` lên ô bản dịch khi đang có vùng bôi đen ở nguyên văn, vì nó sẽ cướp vùng bôi đen của Thêm thuật ngữ.
- Đóng menu mà không chọn gì thì caret trở về giá trị trước lúc mở, và từ bị bôi đen trong ô bản dịch thu về caret, nếu không phím gõ tiếp theo sẽ đè lên từ đó. Hai vế này và việc phối hợp `preventDefault` với `setEditorCaret` chưa được đo; Story 14.2 đo trong app.

🔵 2026-10-30: cả hai vế "chưa đo" ở mục này đã được đo trên WKWebView thật trong app, xem `_bmad-output/implementation-artifacts/14-2-ban-do/ket-qua.md`.
- Mục menu chặn mặc định của `mousedown` (`@mousedown.prevent`, không giá trị, ở phần tử bọc). Đo trên cả hai engine: có chặn thì caret, tiêu điểm và vùng bôi đen còn nguyên và `click` bắn; không chặn thì `focusout` đặt caret `null` và `click` có thể không bắn.
- Không chép `focusOnPointerDown` của `TitlebarToolsMenu.vue`, vì nó cố ý lấy tiêu điểm. `Công cụ ▾` cũng không chứa được lệnh đọc caret vì lý do đó.

Chuột phải lên một từ ở nguyên văn trên macOS chạy luôn Auto-Lookup cho từ đó, vì WebKit bôi đen rồi `mouseup` tới `attachSelectionWatcher`, và Thêm thuật ngữ điền sẵn từ đó. Trên Windows không có hai hiệu ứng này, ô thuật ngữ mở trống. Đây là khác biệt theo nền tảng được chấp nhận: kiểm trong lượt dùng thật của Epic 14, phần Windows vào mục `B7`, cùng hành vi phím `Menu` của Windows (mở theo câu có caret, không có caret thì không mở).

**Kéo trên cột số câu.** `mousedown` trên một số đặt `anchor` và `focus` vào câu đó; kéo qua số khác dời `focus`; `Shift`+bấm dời `focus` từ `anchor`, hoặc từ caret khi chưa có vùng chọn. Bỏ vùng chọn vẫn ở nút hiện có trong panel Đề xuất AI. Cột số chặn mặc định của `mousedown` nên caret còn và không bôi đen chữ. `@mousedown` của đường này đặt ủy quyền ở `.grid` trong `GridPanel.vue`, bỏ qua ngay nếu không phải nút trái trên ô số câu, nên không chạy gì khi gõ. Tệp này đang có 0 thuộc tính `@mousedown`, nên `at` của `pointer: aim` có đối chứng thật; nhắm chuột phải vì thế đặt ở `GridBlock.vue`, không gộp vào đây. Kéo không tự cuộn ở bản đầu: chọn xa thì cuộn rồi `Shift`+bấm.

**Nhắm khối nhập URL.** `<li class="ip-block">` nhận `@mousedown` nhắm khối, cùng khuôn `GlossaryHarvestOverlay.vue`. Chỉ khối đang nhắm mới hiện cụm nút, ở mép phải khối: danh sách dài không có hàng trăm nút lặp lại, và nút luôn ở sát thứ nó tác động. Nhãn `Loại khối này` / `Giữ khối này` đổi theo `block.kept`, cùng một id. `J`/`K` chạy tiếp từ khối vừa bấm.

**Bấm điểm cắt song ngữ.** `annotatedTargetLine()` thành chuỗi mảnh chữ xen đích bấm tại mỗi `candidate_positions`; không vẽ hai đầu dòng vì `toggleBilingualCutAtCaret` từ chối chúng. Mỗi đích có `@mousedown.prevent` nhắm caret và `@click="dispatch('import.preview.bilingual_toggle_cut')"`, nên một cú bấm là cắt hoặc bỏ cắt. `.prevent` giữ tiêu điểm trên `.bip-mismatch-focus`, vì `onScrimKeydown` bỏ qua phím khi tiêu điểm ở `<button>`, và `←` `→` phải chạy tiếp.

🔵 2026-10-31 (Story 14.7): `.bip-mismatch-focus` không giữ được tiêu điểm vì `v-if` huỷ nó khi hàng cuối hết lệch; đích và hai nút Cặp lệch trước/sau mang `@mousedown.left.prevent` rồi đặt tiêu điểm vào `panel`. Hàng vừa khớp bằng cú bấm chuột ở lại trên màn (`‖`, tiêu đề "Đã khớp") tới khi người dùng rời đi, nên bấm lại là bỏ cắt; `Enter` vẫn tự sang hàng kế. Đích rộng 21px: tâm cách tâm đo 23,6px.

## Accessibility Floor

Mọi phím giữ nguyên (Epic 14 §Constraints). Nút chỉ có biểu tượng mang `aria-label` và `title`. Trạng thái công tắc và ô Review Mode đọc qua `aria-checked`, `aria-pressed`, không chỉ qua màu. Trình đọc màn hình nằm ngoài epic.

## Key Flows

### KF-5 · Ice dịch một đoạn chỉ bằng chuột

Ice vừa đổi máy, chưa gõ quen các phím, và đang sửa Chương 12.

1. Ice bấm vào ô bản dịch câu 12 và gõ xong câu. Thanh trạng thái ghi "Chuột phải vào câu để xem thao tác".
2. Ice chuột phải vào câu 12 và chọn `Xác nhận`. Caret sang câu 13, ô vẫn gõ được ngay.
3. Câu 13 thực ra là nửa sau của câu 12. Ice chuột phải vào câu 13, chọn `Gộp lên` và đọc thông báo ở thanh trạng thái.
4. Câu 20 quá dài. Ice giữ `⌘` bấm vào chỗ muốn tách ở nguyên văn, rồi chuột phải vào câu 20 và chọn `Tách câu`. Đỉnh điểm: lệnh chạy đúng câu vừa chuột phải, dù Ice chưa từng đặt caret ở đó.
5. Ice kéo trên cột số từ 21 tới 30, panel Đề xuất AI hiện nút dịch 10 câu đã chọn.
6. Ice bấm › ở thanh trạng thái để sang Chương 13.

### KF-6 · Ice tách một cặp lệch trong tệp song ngữ

1. Ice nhập một tệp hai cột, màn xem trước báo 6 cặp lệch.
2. Ice bấm `Cặp lệch sau` tới cặp cần sửa.
3. Trên dòng bản dịch, Ice bấm dấu `¦` giữa hai câu. Đỉnh điểm: dấu thành `‖` và hai mảnh tách ngay ở cột trên.
4. Bấm nhầm, Ice bấm lại dấu đó để bỏ cắt.

🔵 2026-10-31 (Story 14.7): ở bước 3, nếu cú bấm làm cặp vừa khớp thì cặp ở lại trên màn với tiêu đề "Đã khớp" để bước 4 còn bấm lại được; `Cặp lệch sau` rời cặp đó.

### KF-7 · Ice sửa ranh giới bóc của một Chương nhập từ URL

1. Trong xem trước nhập URL, Ice thấy phần đầu Chương 3 lẫn dòng quảng cáo. Ice bấm › cạnh dòng "Chương n/N" để tới đúng Chương.
2. Ice bấm vào khối quảng cáo. Khối được nhắm và hiện cụm nút ở mép phải.
3. Ice bấm `Loại khối này`. Khối chìm xuống và bộ đếm giữ/loại đổi.
4. Nội dung thật bắt đầu muộn hơn. Ice bấm khối đầu rồi `Đầu vùng giữ`, bấm khối cuối rồi `Cuối vùng giữ`. Đỉnh điểm: vùng giữ đổi một lượt mà tay không rời chuột.

🔵 2026-10-31 (Story 14.6): sửa khối diễn ra ở Chương đầu; ở Chương k > 0 ba nút hiện nhưng tắt (phiếu quyết #16).

### KF-8 · Ice vào và ra Review Mode bằng chuột

1. Ice bấm ô Review Mode ở công tắc bố cục, dải Rà soát hiện ra.
2. Ice bấm mũi tên xuống để tới chỗ khác biệt kế tiếp, xử lý xong rồi bấm tiếp.
3. Ice bấm lại chính ô đó. Đỉnh điểm: lối ra nằm đúng chỗ lối vào, và Workspace trở về như cũ.

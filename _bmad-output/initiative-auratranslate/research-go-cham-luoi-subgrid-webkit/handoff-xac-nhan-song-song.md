# Handoff: gõ chậm dần sau nhiều lượt Mod+Enter ở kiểu Song song Hán Việt

Working notes giữa các pha. Không chép vào spec.

## Triệu chứng (Ice, 2026-11-12, v1.0.2 trên Windows)

- Điền bản dịch AI vào mọi câu của Chương, rồi sửa và `Mod+Enter` từng câu.
- Lúc đầu gõ nhanh. Sau hơn 100 lượt xác nhận, gõ cực kỳ chậm.
- Chỉ ở kiểu xem **Song song Hán Việt**. Chữ thường thì không.
- Gần như không có gạch chân quy ước chữ hay soát lỗi.

## Đã loại (pha 0, Winston)

- Đường mỗi phím `reportEdit` → `noteEditorEdit` (`src/panels/GridPanel.vue` ~1766, `src/panels/editorPanelState.ts`) là O(1).
- AI không giữ khoá `OpenWorkState` qua lượt chờ mạng. Hai lệnh ghi theo hẹn giờ đã là `(async)` trong v1.0.2.
- Trạng thái Hán Việt dùng chung (`src/panels/sourcePanelState.ts`) dựng một lần mỗi Chương. `hanVietSurfaces.ts` không rò.
- `repaintProofreadHost` (`src/panels/proofreadHighlight.ts`) tăng siêu tuyến tính theo số ô có dấu: Chromium 156, 667 câu, 100 ô 72–112 ms, 667 ô 0,7–1,4 s. Nhưng Ice gần như không có dấu, nên đây là nghi can phụ. Bàn đo nằm ở scratchpad của phiên Winston.
- `editedText` (`editorPanelState.ts:738`) giữ mọi câu đã sửa trong phiên. `restoreEditedText` chạy một `querySelector` toàn lưới cho mỗi câu, mỗi lần `editorSegments` đổi: ≤ 63 ms ở 667 câu trên Chromium. Nhỏ, nhưng lớn dần theo phiên.

## Nghi can chưa đo

Mỗi ô nguồn ở kiểu Song song là một `SourceHanViet`. Nó nhận `:cuts="cutOffsetsOf(s.id)"` và `:glossary-terms="glossaryTermsOf(s.id).spans"`, hai giá trị được tính lại mỗi lần render (`src/panels/GridBlock.vue` ~210). Thêm vào đó là `v-memo` của bốn cột.

## Pha 1: kết quả (agent đo ghi vào đây)

Bàn đo: `e2e/measure/confirm-typing-parallel.e2e.mjs` (ngoài `e2e/specs/`; chạy `AURA_BENCH_VIEW=plain|parallel`, mỗi kiểu một cửa sổ và một tiến trình app mới).

**Điều kiện.** WKWebView thật qua wdio, binary debug `--features wdio`, Mac 8 lõi, `uptime` load 2,6–2,9 lúc bắt đầu mỗi lượt. Chương 320 câu Hán, 320 ô `[data-col="tgt"]` đã dựng cùng lúc (không cắt khối ở bàn đo này). Kiểu song song: 320 `.hv-surface`, 5440 `<rt>`. Mỗi câu đã điền sẵn bản dịch, rồi gõ 3 chữ và `Mod+Enter`.

**Đường điền.** Gọi thẳng `promoteAiTranslationBatchToEditor` của sản phẩm (đường "AI dịch → đưa sang Editor", chỉ thiếu lượt gọi mạng) cho cả 320 câu. Rust trả `needs-confirmation` cho 320/320 câu (ô đích đã có chữ chưa ký), nên bàn đo gọi `confirmPendingPromote()` của sản phẩm để ghi đè. Không dùng `save_segment_targets`.

**Đường gõ.** ⚠️ `browser.keys()` chỉ phát `keydown`, không sinh chữ (đã ghi ở `specs/editor-typing-flush.e2e.mjs`). Mỗi phím ở đây là `keydown` tổng hợp rồi `execCommand('insertText')` (cùng `beforeinput` → `input` như phím thật), đo từ đầu `keydown` tới khung hình kế. `Mod+Enter` là sự kiện tổng hợp (driver đánh rơi `Meta` ở `Enter`), đo tới khi con trỏ sang hàng kế. Không có IME, không có hàng đợi phím của hệ điều hành; số đo chạm sàn một khung hình (~17 ms).

**Đường cong** (20 phím mỗi mốc, ms; "đã xác nhận" là số câu đã `Mod+Enter` trước khi đo):

| đã xác nhận | plain: phím trung vị / p90 | plain: Mod+Enter | song song: phím trung vị / p90 | song song: Mod+Enter (trung vị 10 lượt gần nhất) |
|---|---|---|---|---|
| 0 | 21 / 24 | 22 | 24 / 26 | 18 |
| 25 | 20 / 31 | 28 (12) | 25 / 40 | 32 (17) |
| 50 | 20 / 31 | 30 (14) | 24 / 33 | 38 (22) |
| 100 | 20 / 34 | 31 (13) | 23 / 36 | 50 (37) |
| 150 | 20 / 30 | 34 (17) | 22 / 32 | 63 (47) |
| 200 | 15 / 24 | 32 (18) | 17 / 27 | 79 (59) |
| 250 | 15 / 22 | 35 (20) | 17 / 24 | 96 (83) |
| 300 | 17 / 30 | 37 (24) | 17 / 26 | 120 (98) |

Lô 20 phím chạm chữ: plain 327–463 ms, song song 351–548 ms, không tăng theo số câu.

**Kết luận pha 1.**
- ⇒ Độ trễ MỖI PHÍM không tăng theo số câu đã xác nhận ở cả hai kiểu (giảm nhẹ về cuối, nhiều khả năng do máy nguội dần). Triệu chứng "gõ cực chậm sau 100 lượt" KHÔNG tái hiện ở đường gõ này.
- ⇒ `Mod+Enter` chậm dần tuyến tính: song song 18 → 120 ms (≈ +0,33 ms mỗi câu đã xác nhận), plain 22 → 37 ms (≈ +0,05 ms mỗi câu). Tức có một chi phí theo số câu đã xác nhận, chỉ nặng ở kiểu song song, và nó nằm ở lượt xác nhận chứ không ở phím gõ.
- ⚠️ Chưa biết chi phí đó nằm ở bước nào của lượt xác nhận (`confirmCurrentSegment`, lượt cập nhật `editorSegments`/`editedText`, hay lượt render lại 320 `SourceHanViet`); đây là điểm để pha sau cắt. Cũng chưa loại được khả năng bàn đo không có đường nhập thật của hệ điều hành (IME, WebView2 trên Windows, bản release): Ice báo gõ từng phím chậm, bàn đo này chỉ chứng minh lượt xác nhận chậm dần.
- Không thấy bước nhảy ở biên khối 200 hàng: 320 ô được dựng cùng lúc trong bàn đo này, nên chưa đo kịch bản có cắt khối.

## Pha 2: kết quả (agent đo)

Bàn đo: `e2e/measure/confirm-trace.e2e.mjs` (knob `AURA_BENCH_VIEW|SAMPLES|CHECKPOINTS|TYPE_MS|FLUSH=0`). Bọc `fetch` của webview để lấy thời điểm yêu cầu/phản hồi từng lệnh IPC (gán đè `__TAURI_INTERNALS__.invoke` im lặng không có tác dụng: log rỗng ở lượt đầu); `MutationObserver` có callback thật cộng `takeRecords`; một bộ `setInterval(1ms)` để thấy khối bận của luồng chính.

**Điều kiện.** WKWebView thật, binary debug `--features wdio`, Vite dev, Mac 8 lõi, `uptime` load 2,9–4,2. Chương 320 câu, tất cả đã dựng; mỗi câu đã điền bằng `promoteAiTranslationBatchToEditor`, gõ 3 chữ rồi `Mod+Enter`. 3 lượt liên tiếp mỗi mốc.

**A1. Tách một lượt xác nhận** (ms từ lúc phát `keydown` Mod+Enter; song song / plain):

| đã xác nhận | `confirm_segment` Rust (req→resp) | khối bận luồng chính liền sau phản hồi | con trỏ sang hàng kế |
|---|---|---|---|
| 0 | 3 | 11–27 / 28 | 17–25 / 19 |
| 100 | 3 | 50–52 / 29–34 | 55–57 / 34–38 |
| 300 | 2–3 | 108–126 / 35–38 | 113–131 / 40–43 |

- ⇒ Rust KHÔNG đổi (2–3 ms ở mọi mốc, cả hai kiểu). Phần tăng nằm hoàn toàn ở webview: một khối bận LIÊN TỤC của luồng chính bắt đầu ngay sau khi `confirm_segment` trả về (~5 ms) và kết thúc ngay trước lô DOM đầu tiên rồi con trỏ mới sang hàng kế. `convention_scan_segments`, `tm_count_pairs`, `glossary_count_entries` chỉ "chậm" vì phản hồi của chúng chờ khối bận này.
- Phần đồng bộ của `keydown` Mod+Enter: 0–2 ms.

**A2. Nhóm thay đổi DOM mỗi lượt** (kiểu song song, mọi mốc): 33–34 bản ghi, 0 bản ghi trong ô nguồn (`data-col="src"`), 0 trong `.hv-surface`, 0 trong `<ruby>`. Chỉ 2 hàng `tgt` (hàng vừa xác nhận + hàng kế) và các cột vạch/số/quy tắc của 2 hàng đó. Số bản ghi không đổi theo số câu đã xác nhận.
- ⇒ Số ô nguồn / `.hv-surface` bị viết lại mỗi lượt KHÔNG tăng theo số hàng đã xác nhận: nó bằng 0. Nghi can `v-memo` cột nguồn, `cutOffsetsOf`/`glossaryTermsOf` và `SourceHanViet` bị loại ở cấp DOM (không hàng đã xác nhận nào được dựng lại). Chưa loại phần tính `v-memo` (chạy mà không ra thay đổi DOM), nhưng C bên dưới cho thấy khối bận biến mất khi bỏ một seam khác.

**B. Flush hẹn giờ khi gõ liên tục** (~8 phím/s, 15 s, đã xác nhận 303; 120 phím mỗi ca; lệnh flush mỗi ~5 s do `EDITOR_HARD_CAP_MS`):

| ca | song song: phím trung vị / max | plain: phím trung vị / max |
|---|---|---|
| B1 câu chưa xác nhận (hàng 304) | 12 / 18 | 12 / 17 |
| B2 câu đã xác nhận (hàng 152) | 16 / 18 | 16 / 20 |

- ⇒ KHÔNG có đỉnh quanh phản hồi flush ở cả hai kiểu, cả hai loại câu: `save_segment_targets` + `convention_scan_segments` mất 2–5 ms, phím quanh đó 11–17 ms (bằng nền). Không có lô DOM > 5 bản ghi trong 15 s. Tức cảnh gõ-liên-tục + flush không tái hiện triệu chứng; chi phí nằm ở lượt xác nhận.

**C. Phép gỡ đối chứng (một lần).** Gỡ vòng `for (const [id, text] of edited) { targetCellById… }` khỏi `restoreEditedText` (`src/panels/GridPanel.vue` ~1255, giữ chữ ký và `repaintProofread()`), chạy lại kiểu song song, cùng điều kiện (debug+wdio, Vite, load 2,96):

| đã xác nhận | khối bận trước → sau gỡ | con trỏ sang hàng kế trước → sau |
|---|---|---|
| 100 | 50–52 → 28–37 | 55–57 → 30–33 |
| 300 | 108–126 → 19–35 | 113–131 → 23–40 |

- ⇒ Độ dốc theo số câu đã xác nhận biến mất (đường 0 → 300 gần phẳng, ~20–35 ms). Seam: `restoreEditedText` chạy một `querySelector` toàn lưới (`targetCellById`) cho MỖI câu trong `editorEditedText` (mọi câu đã gõ trong phiên) sau mỗi lần `editorSegments` đổi; lượt xác nhận đổi `editorSegments` nên trả giá O(số câu đã sửa × kích thước DOM). Ở kiểu song song DOM nặng hơn (320 `.hv-surface`, 5440 `<rt>`) nên hệ số ≈ 6× plain, khớp tỉ lệ 0,33 so với 0,05 ms/câu của pha 1.
- ⚠️ Bàn đo gõ 3 chữ vào MỌI câu nên `editedText` = số câu đã xác nhận; ngoài đời cũng vậy khi Ice sửa từng câu rồi Mod+Enter. Chưa tách riêng chi phí `querySelector` khỏi phép so sánh `textContent`; chưa thử bản vá (ngoài phạm vi: chỉ định vị). `src/` và `src-tauri/` đã khôi phục (`git diff --stat` rỗng).
- ⚠️ Vẫn chưa loại: Windows/WebView2, bản release, IME. Mốc gõ từng phím của Ice chậm ở đó vẫn không tái hiện ở đây (phím 12–17 ms).

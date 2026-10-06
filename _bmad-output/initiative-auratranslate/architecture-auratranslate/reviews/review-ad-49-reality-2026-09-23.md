# Review AD-49 — reality-check, 2026-09-23

Lens: mọi khẳng định thực tế trong AD-49 (spine dòng 775-789) phải khớp mã HIỆN TẠI, không chỉ khớp ý đồ. Đọc bằng `grep`/`Read` có `offset`, không đọc trọn spine.

## Verdict

AD-49 khớp mã ở phần lõi (Rule 1's cơ chế nhường vùng gõ, cặp lệnh (i), hai lần bấm xoá lớp (iii), FR101) nhưng **KHÔNG khớp ở đúng chỗ nó tự nhận là đã đóng**: `promote_ai_translation` là một phản-ví-dụ ĐANG CHẠY cho tuyên bố "mỗi thao tác ghi rời rạc thuộc đúng một lớp", và tuyên bố "`⌘Z` gốc sửa văn bản trong đúng một editing host" có một đường vỡ thật (Vue ghi đè text node của **chính ô đang giữ tiêu điểm**), do cùng lệnh đó gây ra. Không có mục nào trong `deferred-work.md` gắn hai lỗ hổng này vào AD-49.

## Findings

### 1. [CAO] `promote_ai_translation` không thuộc lớp nào trong ba lớp Rule 2 — và không phải nợ có chủ

- Bằng chứng: `src-tauri/src/commands/segment.rs:2107-2132` (`promote_ai_translation`) — một câu `UPDATE segment SET target_text = ?1, translation_origin = ?2 ...` chạy thẳng, không đọc `segment_version`, không `needs_confirmation`/`force`, không phép thử "bản sao ở đâu không".
- Không phải lớp (i): không có lệnh nghịch đảo nào đăng ký cho nó (`grep -rn "promote" src/commands/index.ts` không ra một cặp undo).
- Không phải lớp (ii): nội dung cũ (bản người dùng gõ, kể cả bản CHƯA flush vào `editorEditedText`) bị mất, không "vẫn tra lại được".
- Không phải lớp (iii) hợp lệ: chính phép thử Rule 2(iii) — "cái sắp mất có bản sao ở đâu không" — được cài ĐÚNG ở `restore_segment_version` (`segment.rs:824-829`, `SELECT EXISTS(... segment_version WHERE segment_id=? AND target_text=?)`) nhưng KHÔNG được gọi ở đường promote.
- `_bmad-output/implementation-artifacts/deferred-work.md:2500-2502` có nhắc `promote_ai_translation không ghi segment_version` nhưng chỉ để giải thích UX cuộn-theo-lô (Story 4.9, Decision 2) — không đóng dấu đây là một khoảng hở của AD-49, không có mục nợ nào mang `Chủ:` trỏ vào AD-49 Rule 2/3 cho lệnh này. AD-49 tự nhận nó "sẽ là một mục nợ" (theo bối cảnh giao việc) nhưng mục đó chưa tồn tại trong sổ nợ.
- Hệ quả: một prompt AI có thể ghi đè bản người dùng tự gõ (chưa `confirm`, chưa có `segment_version`) mà không hỏi gì — đúng lớp lỗi "rỗng im lặng" mà AGENTS.md liệt là trung tâm.

### 2. [CAO] "`⌘Z` gốc sửa văn bản trong đúng một editing host" vỡ chính bởi lệnh promote đó

- `src/panels/GridPanel.vue:1022-1036` tự đặt luật "DOM SỞ HỮU VĂN BẢN BẢN DỊCH. VUE KHÔNG." và `restoreEditedText` (`GridPanel.vue:1051-1065`) tuân luật đó bằng cách bỏ qua đúng phần tử đang giữ tiêu điểm (`document.activeElement === el` ⇒ `continue`) trước khi ghi `el.textContent`.
- Nhưng đường promote đi đường KHÁC: `promoteAiTranslationToEditor` (`src/panels/editorPanelState.ts:297-310`) gọi `replaceEditorSegment` (dòng 259-268) — hàm này dựng mảng `segments` MỚI với `target_text` mới, **không kiểm activeElement**.
- Template bind trực tiếp `{{ s.target_text }}` lên chính node `contenteditable` (`GridPanel.vue:1831-1848`, có `:key="s.id"`) — nên khi `target_text` đổi, Vue vá lại text node của ĐÚNG ô đó, kể cả khi ô đang giữ tiêu điểm.
- Lệnh kích hoạt promote là `ai.translate.promote`, `keys: ['Mod+Shift+Enter']` (`src/commands/index.ts:3615`) — một hợp âm có phím mod chính nên **không nhường vùng gõ**: `lacksPrimaryMod(entry.mods) && isTypingZone(...)` ở `src/commands/keys.ts:510` trả `false` cho nó, đúng như AD-49 Prevents(2) mô tả — nghĩa là promote bắn được ngay cả khi con trỏ đang ở trong ô đó.
- Kết hợp ba điểm trên: bấm `⌘⇧↵` trong khi con trỏ đang ở đúng ô vừa dịch ⇒ Vue ghi `textContent` vào node đang có tiêu điểm ⇒ WebKit coi đây là một lượt sửa DOM ngoài luồng gõ, xoá sạch ngăn xếp hoàn tác native của editing host đó. Sau đó `⌘Z` không còn gì để lùi — kể cả những ký tự người dùng gõ TRƯỚC lượt promote mà chưa kịp `flush`. Đây là phản ví dụ trực tiếp cho khung "một editing host duy nhất" của Rule 1: editing host đó bị một lệnh KHÔNG-gõ ghi đè trong khi vẫn giữ tiêu điểm.

### 3. [THẤP] Đóng nợ "`⌘Z` bấm vào không phản hồi gì" bằng AD-49 chỉ đúng một nửa

- `deferred-work.md:4289` đóng mục nợ SCP 2026-08-18b bằng câu: "AD-49 Rule ① bắt mọi binding Mod+Z... nhường vùng gõ". Nhưng mục nợ gốc (`deferred-work.md` dưới `## Deferred from: SCP 2026-08-18b`) hỏi về ca **⌘Z bấm khi con trỏ KHÔNG ở trong vùng gõ nào** (sau gộp/tách) — im lặng hoàn toàn, không phản hồi.
- AD-49 Rule 1 chỉ quy định hành vi của một binding `Mod+Z` NẾU nó tồn tại trong vùng gõ; nó không nói gì về việc có nên phát tín hiệu khi `⌘Z` không trúng vùng gõ nào. Xác nhận hiện trạng: `grep -rn "KeyZ" src/` = **0** kết quả trên toàn `src/`, tức không command nào đăng ký `Mod+Z`/`Mod+Shift+Z` — đúng thực tế nhưng đây là "không có gì để nhường", không phải bằng chứng cho câu hỏi UX gốc.
- Mức độ thấp vì đây là khoảng hở về ghi chép (đóng nợ hơi rộng tay so với câu hỏi gốc), không phải một khẳng định sai của bản thân AD-49.

## Xác nhận đúng (để không đọc review này thành "AD sai toàn bộ")

- Bốn cặp lệnh Rule 2(i) đều tồn tại dưới dạng command id đã đăng ký: `editor.omit_segment`/`editor.restore_segment` (`Mod+Alt+X`/`Mod+Alt+R`, `src/commands/index.ts:2850-2851`); `editor.end_target_paragraph`/`editor.join_target_paragraph` (`Mod+Alt+P`/`Mod+Alt+U`, `index.ts:2883-2884`); `library.chapter_move_up`/`library.chapter_move_down` (`index.ts:1859,1870`); `library.chapter_merge_up`/`editor.split_chapter` (`index.ts:1881`, `index.ts:2963`).
- Ba đường DELETE lớp (iii) đều có cơ chế bấm-hai-lần thật (không chỉ copy chữ): `GlossaryManageOverlay.vue` (`manageDeletePending`, dùng `vi.json:876`), `PromptLibraryOverlay.vue` (`deletePending`, `vi.json:147`), `ImportPreviewOverlay.vue` (`cleanupDeleteLabelKey`/`isDeletePending`, `vi.json:439`) — cả ba đều hiện đúng cụm "không hoàn tác được".
- `restore_segment_version` (`src-tauri/src/commands/segment.rs:733-850`) cài đúng phép thử "bản sao ở đâu không" bằng `EXISTS` trên `segment_version` (dòng 824-829), có `needs_confirmation`/`force` đúng khuôn, và tự ghi rõ giới hạn thật của nó (chuỗi rỗng bỏ qua phép kiểm, dòng 807-822) — mức minh bạch đúng luật "measurement states its build/population".
- Không có binding `Mod+Z`/`Mod+Shift+Z` nào tồn tại trong toàn bộ `src/` hôm nay (`grep -rn "KeyZ" src/` = 0 dòng), nên Rule 1 hiện không bị vi phạm bởi bất kỳ command nào — đúng như AD-49 mô tả cho THỜI ĐIỂM VIẾT.

File: `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/reviews/review-ad-49-reality-2026-09-23.md`

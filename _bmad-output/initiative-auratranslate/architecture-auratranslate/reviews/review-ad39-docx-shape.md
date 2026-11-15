# Review: AD-39 amendment 2026-11-15 (docx shape), adversarial lens

Verdict: the amendment fixes who decides the shape (Rust) but leaves open where the answer lives, so stories 6.25 and 6.29 can each obey it and still collide.

## Findings

1. Two wire shapes, no rule which one carries "shape". Today plain preview (`ImportEncodingPreview`, `importPreviewState.ts`) and bilingual preview (`BilingualImportEncodingPreview`, `bilingualImportPreviewState.ts`, `preview_bilingual_import`) are separate commands, wires and webview state modules. 6.25 (overlay) routes `.csv/.tsv` to the bilingual command by extension and, for `.docx`, must call something before the shape is known; 6.29 returns "shape in the wire". 6.25 can legitimately call the plain command for every `.docx`; 6.29 then has no wire to put BilingualTables in. Also 6.25's tab Tệp shows ".docx, dò khi xem trước" (no detection at list time), while the 6.25 mixed-kind lock needs the kind at list time: a `.docx` is neither kind.
2. "Same held byte copy" does not exist for the file path. Re-run is path-based (`import_bilingual_docx` and plain import both `std::fs::read(path)`; webview keeps `pendingFiles` paths). The override "on the same held bytes" is therefore either a re-read (file may change between toggles) or a new Rust holder; the two stories can pick differently, and a second holder duplicates `PendingImportSourceState` (AD-1).
3. Detection threshold and error contract disagree. `import_bilingual_docx` returns `BilingualNoTwoColumnTable` when no table is wide; "Blob when none" needs a different entry. Per-turn parameters (`chapter_pattern`, source/target column, `table_index`, regroupings) are lost or kept when the toggle flips: unspecified.
4. Override scope is unstated for multi-file turns and for the mixed-kind rule: a toggled `.docx` changes the kind after the lock evaluated it.
5. Reality-check: shapes (`Blob`, `BilingualTables`, `read_docx`, `top_level_tables`) were checked and match; the shared-holder and wire claims are asserted, not checked. 6.29's own `unknown` ticket field still asks Winston whether AD is needed, so the amendment is not yet reflected in tickets.

## Proposed Rule wording

- R1: "Một lệnh dò duy nhất `detect_docx_shape(path)` trả `{shape: Text|Bilingual}` cho webview; xem trước văn bản và song ngữ nhận `shape` làm tham số tường minh, và mọi `.docx` trong danh sách tệp mang kết quả dò trước khi luật một-loại chạy. Webview không gọi lệnh xem trước nào khi shape chưa biết."
- R2: "Bản byte của lượt nhập được giữ trong `PendingImportSourceState` (một chỗ); ghi đè và đổi bảng mã đọc từ đó, không đọc lại đường dẫn."
- R3: "Công tắc ghi đè giữ Tên, Ngôn ngữ, Thể loại, luật làm sạch; xoá mẫu phân tách, vai cột và quy nhóm câu; hàm dò trả Blob khi không có bảng rộng, không ném `BilingualNoTwoColumnTable`."
- R4: "Kind của `.docx` trong luật một-loại là giá trị sau ghi đè."

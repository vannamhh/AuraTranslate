# PRD Amendment Review — FR159 Danh sách quy ước chữ Việt

Scope: uncommitted diff of `prd-auratranslate.md` only (FR159 in §6.4 lines ~646-667; §5.1 C4 row and "Tổng" line; §5.2 term row; file-sharing boundary ~602; FR103 table ~932), judged against FR46, FR49, FR69-FR72, FR77, FR80-FR86, FR100-FR103, FR117, FR150, NFR2, NFR8, NFR9, §3.2, §11.1 and the memlog tail. Checked: VietPhrase line count (679,311 lines with '=' - confirmed), FR id continuity (FR1-FR159, only gap FR20 = struck, 158 live definitions - the new "Tổng" is correct).

## Overall verdict
FR159 is a well-argued, honest amendment: the "why not Glossary / Proofreader / prompt" reasoning is explicit, the syllable-vs-word trap is named with a worked example, and the confirm-all-matches design follows from it. Its weaknesses are at the edges the stories will hit first: the acceptance criterion borrows a threshold from an unrelated NFR, three PM-set assumptions sit outside the PRD's own Assumptions Index, and several user-observable behaviours (replacing in already-confirmed segments, overlapping rules, tier conflicts, export format, interaction with finalized Glossary entries) are left unspecified.

## Decision-readiness — adequate
Decisions are stated as decisions and the trade-off against FR69 (prompt-only) is recorded ("Vì sao không đủ bằng prompt"). But the memlog's own open items ("luật đụng bản dịch Glossary đã chốt, định dạng file xuất") are not surfaced anywhere in the PRD - no `[NOTE FOR PM]`, no Open Question - so a reader of the PRD alone believes the FR is closed. The three assumptions John set himself (case shape, single-pass, NFR2 threshold) are likewise invisible in the PRD text.

### Findings
- **high** PM-set assumptions are not tagged or indexed (§6.4 FR159 bullets "Cụm thay thế lấy kiểu hoa...", "một lượt trên văn bản trước khi thay", acceptance line "giữ ngưỡng của NFR2"; §11.1) - §11.1 declares "Mọi giả định của PRD nằm trong bảng này... không có giả định nào tồn tại ngoài bảng", yet A1-A11 contain nothing for FR159 and the FR text presents all three as settled fact. Ice signed the FR content, but the memlog still records them as John's own, awaiting confirmation. *Fix:* add A12 (case shape: first-letter / ALL-CAPS only), A13 (single-pass replace, no re-match of inserted text), A14 (50 ms scan budget) to §11.1 with "Nếu sai thì sao", and tag the three places `[A12]`..`[A14]`; or, if Ice's sign-off already covered them, record that in the memlog and drop the "assumption" label so the two documents agree.
- **medium** Unresolved items live only in the memlog (§6.4 FR159; memlog last line) - "luật đụng bản dịch Glossary đã chốt" and "định dạng file xuất" are real open questions; a story slicer reading only the PRD will not see them. *Fix:* add a `[NOTE FOR PM]` under FR159 listing both, with owner (Sally/Winston/Ice).

## Substance over theater — strong
No furniture. The dimension is mostly n/a to a one-FR amendment; the FR carries a real, product-specific rationale (target-side rule has no source anchor, so FR46-style Glossary cannot catch it). The hedge on the example ("ví dụ dựng, tần suất chưa đo") is the right kind of honesty.

### Findings
- **low** VietPhrase evidence is vague about what was measured (§6.4 ⚠️ note: "`VietPhrase.txt` (679.311 dòng) cho 0 chỗ") - the 679,311 count is correct (lines containing '='), but "0 chỗ" is only true for the exact string "bảo đảm bảo" on the Vietnamese side; the plain rule phrase "đảm bảo" appears in 18 Vietnamese-side lines. A reader may take "0" to mean the rule never fires. *Fix:* write "cụm 'bảo đảm bảo' xuất hiện 0 lần ở vế tiếng Việt của `VietPhrase.txt`...", or drop the corpus claim since the note already says the frequency is unmeasured.

## Strategic coherence — strong
Placing FR159 in C4 rather than C7 with the stated reason (FR77: must work without AI) is coherent with the product's offline pillar. The C4 row, §5.2 term and FR103 table edits are consistent with each other.

### Findings
- **low** Overlap with Proofreader/spell-check not reconciled (FR80 "Quét chính tả... trên bản dịch", FR86 in-place marking vs FR159 "đánh dấu ngay tại chỗ, như phát hiện của Proofreader") - two features paint marks on the same spans, and a Proofreader finding could suggest the opposite of a convention rule (e.g. FR83 "đề xuất sửa"). Precedence/visual distinction is unspecified. *Fix:* one sentence: convention marks are distinct from Proofreader marks and win/lose where spans coincide (say which); defer visual treatment to Sally explicitly.

## Done-ness clarity — thin
The acceptance note is a genuine strength (0 AI calls, determinism, bỏ chọn giữ nguyên từng ký tự are all testable). But it is incomplete and one criterion is mis-sourced.

### Findings
- **high** NFR2 is cited for something it does not govern (§6.4 acceptance: "quét chữ đang gõ giữ ngưỡng của NFR2, không frame nào vượt 50 ms"; NFR2 row ~976) - NFR2 is "Auto-save không làm gián đoạn thao tác gõ"; it measures frames during auto-save, not a text scanner. The 50 ms number is borrowed, and the workload is missing: scan of what text (one segment? whole Chương?), how many rules (10? 5,000 community-shared?), on which hardware. As written it can neither be failed meaningfully nor traced to a risk. *Fix:* give FR159 its own NFR (or an inline threshold) "quét trong lúc gõ không làm frame nào vượt 50 ms với segment ≤ N ký tự và danh sách ≤ M luật", with N and M stated (and mark the number `[A14]`); reference NFR2 as the origin of the number, not as the governing requirement.
- **medium** Matching rules are testable only for the two examples given (§6.4 bullet "Khớp theo ranh giới âm tiết") - "ranh giới âm tiết" is not defined for punctuation, hyphens, digits, apostrophes, or a syllable-adjacent non-Vietnamese token; "không phân biệt hoa thường" is not defined for Vietnamese diacritic capitals (Đ/đ, Ả/ả). Two testers can reasonably disagree. *Fix:* add a one-line definition ("âm tiết = chuỗi chữ cái liên tiếp; mọi ký tự không phải chữ cái là ranh giới") plus 3 acceptance examples (match, no-match, punctuation-adjacent).
- **medium** Case shape is under-specified (§6.4 "viết hoa chữ đầu... viết hoa toàn bộ") - mixed-case matches (e.g. "ĐẢm bảo", Title Case across a multi-syllable phrase "Đảm Bảo"), single-letter phrases (is "A" first-letter or all-caps?) and a replacement that is shorter/longer in syllables are not covered. *Fix:* state the fallback explicitly ("mọi dạng khác: dùng cụm thay nguyên văn như người dùng đã gõ vào luật") and add "Đảm Bảo", "ĐẢM BẢO" examples to acceptance.
- **medium** Acceptance omits most FR159 behaviours - nothing covers tier override/disable (FR103 pattern), skip-memory persistence, no-replacement rule shows only a mark, export/import round-trip, or the FR117/FR101 consequences (origin becomes "tôi dịch", a version row is written). *Fix:* extend the acceptance line with one testable clause per bullet.

## Scope honesty — adequate
The FR states what it is not (no auto-fix, no AI, no suggestion for replacement-less rules) and where it differs from Glossary. Boundaries of the sweep are the weak spot.

### Findings
- **high** Bulk replace on already-confirmed segments is unspecified (§6.4 "thay cả Chương"; FR117; Translation Memory FR56 family; memlog decision "nhận thay = lần sửa của người dùng") - a Chương contains confirmed segments, "người khác dịch" segments, FR133 cut segments, and AI suggestions that FR72 says are not written to the Editor. FR159 says replacing is "một lần sửa... đi đúng đường của chữ gõ tay" but does not say: does a replaced confirmed segment stay confirmed? Does its TM pair update (TM accumulates on confirm)? Does replacing inside an AI suggestion (panel, not segment) count as a user edit and change origin? Are cut segments (FR133) scanned? These are exactly the "user-observable behaviour" a story will need, and the silent default may corrupt TM or origin data. *Fix:* add a bullet "Phạm vi thay": which segment states are included/excluded in "cả Chương", and a statement that replacing a confirmed segment follows the same path as editing it by hand (including TM and status), with the AI-panel text replaced only in the suggestion buffer and not changing origin until the user transfers it (FR72).
- **medium** Interaction with finalized Glossary translations is left open (memlog says "chưa chốt"; PRD silent) - a Glossary entry may legitimately contain a phrase a rule flags (a character named "Đảm Bảo"), and Glossary-injected AI output will trip the rule on every run. FR84-style skip is per place, not per Glossary term. *Fix:* decide (e.g. "chỗ khớp nằm trọn trong bản dịch Glossary đã chốt của câu thì không bị đánh dấu") or record it as an open question per the Decision-readiness finding.
- **low** §3.2 / non-goals not touched - there is no statement whether sharing a rule list implies any trust/validation of imported lists (a malicious/huge import). Boundary line only says "file văn bản". *Fix:* add "không có xác thực/kiểm duyệt danh sách nhập" or cap list size (links to the NFR gap above).

## Downstream usability — adequate
IDs and cross-refs resolve; FR159 is cited correctly from C4, §5.2, the sharing boundary and FR103. The new term "Quy ước chữ Việt" is used consistently.

### Findings
- **medium** Export format and NFR9 not reconciled (boundary line ~602 adds "quy ước chữ Việt (file văn bản, FR159)"; NFR9 row ~990 "Glossary và prompt xuất được định dạng văn bản mở") - NFR9 is the portability NFR and does not list the new list; FR159 says only "file văn bản mở". Round-trip fidelity (does an import carry the Global/Tác phẩm tier and the disabled-rule state?) is undefined. *Fix:* add "quy ước chữ Việt" to NFR9; in FR159 state which fields are exported (cụm tránh, cụm thay, tầng; not disabled flags/skip memory) so Sofia can cut the import/export story.
- **medium** Tier conflict semantics missing (§6.4 "Hai tầng... thêm được luật riêng và tắt được một luật Global") - FR46 and FR69 both say "tầng Tác phẩm thắng" for duplicates; FR159 does not say what happens when both tiers define the same avoid-phrase with different replacements, or when a Tác phẩm-level rule collides with an enabled Global rule on overlapping phrases. *Fix:* one line: "cùng cụm cần tránh ở hai tầng: tầng Tác phẩm thắng (FR46)"; and for overlapping/nested phrases (e.g. "đảm bảo" inside "đảm bảo an toàn"): longest match wins, one mark per span.
- **low** Panel naming drift (§6.4 "panel AI Translation" vs FR145 "panel Đề xuất AI", FR72 "panel AI Translation") - pre-existing drift, but FR159 reuses the variant. *Fix:* use the glossary term in §5.2 if one exists, else leave and fix in a later sweep.
- **low** NFC handling leaks into user-observable territory (§6.4 "Văn bản được chuẩn hoá Unicode NFC trước khi so") - it is not said whether the stored text is also normalized by a replacement (the acceptance line promises unchecked spans stay "nguyên từng ký tự"; an NFD segment would violate that if normalization is written back, and the highlight offsets differ). *Fix:* state "chuẩn hoá chỉ để so khớp, văn bản lưu không bị đổi ngoài chỗ được thay".

## Shape fit — strong
A capability FR added to a mature FR list, with rationale, acceptance and a table/index trail, is the right size. No UJ or SM is needed for one FR; n/a.

## Mechanical notes
- ID continuity: FR1-FR159 contiguous except FR20 (struck through); 158 live `**FRn.**` definitions confirmed - the corrected "Tổng" line is right. The 🔵 correction of the old "132" is properly marked.
- FR159 appears only in the C4 row; C-table ranges for other epics unchanged - fine.
- Assumptions Index roundtrip: broken for FR159 (see high finding); no `[An]` tags added.
- Cross-refs checked and resolving: FR84, FR85, FR86, FR101, FR103, FR117, FR69, FR77, NFR2 (resolves, mis-scoped).
- NFR9 not updated for the new exportable list (see Downstream usability).
- Vietnamese diacritics/glossary: "Quy ước chữ Việt" is used consistently in the §5.2 row, FR103 table and boundary note; §5.2 row contains no ~~synonym~~ column entry ("—"), acceptable.

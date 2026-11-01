# PRD Amendment Review — FR153–FR158 against AD-64

Scope: only the 2026-11-01 diff of `prd-auratranslate.md` (FR153–FR158, §5.2 term `Dịch nhanh`, C6 range, notes at FR71/FR104/NFR12/§3 network row). Compared with `### AD-64` (13 rules) and ad-brief §1.

## Overall verdict
Mostly sound: FR153–FR158 are numbered contiguously after FR152, every one is a capability, and most are testable. Three things keep it from clean: the "user-initiated" invariant is patched with appended notes while its bold headline stays literally false; FR155 cites a "Giữ/Loại khối" tier that FR123 does not contain; and several user-observable AD-64 rules (single run and `busy`, cancel on leaving, paid blocks kept on cancel, store-open failure, Dọn cancelling a run) have no FR.

## Decision-readiness — adequate
The amendment records the decision (Ice signed 2026-11-01) and its reason. The block at FR71 makes the FR71/FR76 split explicit, which is good. The open numbers (batch cap, wait time, N days) are correctly left out of the PRD; FR158 says "một số ngày tự chọn", which is acceptable.

### Findings
- **high** Headline invariant left false (§3 row "Kết nối mạng"; NFR12; FR104) — the bold text still says "cả ba đều do người dùng chủ động kích hoạt" and NFR12 says "ba luồng do người dùng chủ động kích hoạt". The appended 🔵 redefines "kích hoạt" as "bật tuỳ chọn". AD-64 rule 1 itself says it "nới AD-15": dịch nhanh is the only path that calls the network without a click. A reader of the bold sentence is misled, and FR75 ("thử lại do người dùng chủ động") and the line 774 rationale ("quét nền ... tính phí ngoài ý muốn") now sit beside an auto path. *Fix:* reword the headline and NFR12 to "ba luồng; hai luồng theo thao tác, riêng lời gọi AI của dịch nhanh chạy tự động khi người dùng đã bật tuỳ chọn (FR156)", and state NFR12 as a testable bound: with the option off or the quick config empty, 0 AI calls without a click.
- **medium** FR72 contradicted without a note (FR72 vs FR154/FR155) — FR72 says "Kết quả AI hiện ở panel AI Translation". Quick results show in the Glossary queue and the URL preview. The note under FR71 exempts FR71 only; the FR157 note cites FR72 only for "không ghi vào Editor". *Fix:* extend the 🔵 note at FR71 to say FR72's location rule does not apply to dịch nhanh.
- **medium** FR103 not reconciled (FR103 table vs FR153) — FR103 says "Mọi cấu hình tồn tại ở hai tầng, tầng dự án ghi đè". The quick config is Global only. FR153 says "khác FR68" but not FR103. *Fix:* add FR103 to the FR153 pointer, or add the quick config to the Global row.

## Substance over theater — strong
No furniture. The FR154–FR155 rationale (understand zh/en context to accept or reject) is concrete.

## Strategic coherence — adequate
Fits trụ #1 ("máy đề xuất, người duyệt"): FR155 says the translation never changes the keep/drop decision, and FR157's note keeps results out of Editor, TM and Glossary. FR77 is not reconciled, though. FR154 sits in the Glossary queue and FR155 in URL import, both outside C6 by function, and both depend on an AI config. FR77 only reads "ngoài C6 và C7" by range. *Low:* add one sentence that without any AI config these surfaces show no quick-translate action but still show the translation already in the Work (FR154) and the cache (FR157).

## Done-ness clarity — adequate
Testable: FR156 (default off, five negative conditions, off stops at once), FR157 (reuse with no second call, only complete results kept), FR158 (disk size actually drops, no auto-clean, no cap). No implementation detail leaks, apart from the minor ones listed below.

### Findings
- **high** FR155 cites a tier that FR123 does not define (FR155 -> FR123) — FR155 says "tầng chọn Giữ/Loại khối (FR123)". FR123 describes a preview of each Chương with manual boundary editing; "Giữ/Loại", "khối" and a per-block tier appear nowhere else in the PRD (grep). The capability FR155 hangs on is undefined, so "các khối đang thấy" is not testable from the PRD. *Fix:* either state the block-level Keep/Drop capability in FR123 (via correct-course), or restate FR155 in terms FR123 does define. Record whichever is chosen.
- **medium** AD-64 observable rules with no FR:
  - Rule 8: one quick-translate run at a time across the app; a second surface gets "bận" with no network call; leaving the row or screen cancels; scrolling does not cancel a running batch. FR154 says "huỷ được" and FR156 says "không chạy khi màn đã đóng", but "bận" and "cuộn không huỷ" are not stated.
  - Rule 5: when a batch is cancelled, the closed blocks already paid for are kept. Only "chỉ kết quả trọn vẹn được lưu" (FR157) is stated, and read alone it could imply the opposite.
  - Rule 3: if `quick-translation.db` cannot be opened or is corrupt, the app keeps running and quick translate reports an error.
  - Rule 10: Dọn cancels a running run, and a late run cannot re-write rows after Dọn.
  - Rule 4: "một chuỗi nguồn một hàng" is not defined (NFC and trim), so FR157 "mỗi chuỗi nguồn lưu một bản" is ambiguous. Also, `context_example` is cut at 200 characters, so its translation is valid only for the cut string.
  - Rule 7: when the option is off, URL preview has a manual button for the visible blocks. FR155 says "dịch nhanh" without saying it is a click when auto is off.
  *Fix:* add one or two sentences to FR154/FR156/FR157/FR158 (rule 8 and rule 5 matter most); rules 3 and 10 can go as a sentence at FR158/FR157.
- **medium** FR153 half-declared config is vague — "Để trống thì ... toàn cục. Khai thiếu thì báo chưa cấu hình" does not say what "trống" and "thiếu" mean (AD-64 rule 2: both endpoint and model empty = use the full main config at Global; one of them = not configured). Also unsaid: the manual click ignores a Work-level override. *Fix:* use those exact conditions.
- **low** FR76 is asserted only in the note under FR71 (token and cost shown per surface); no FR text carries it, so there is nothing to test for it. *Fix:* one clause in FR154/FR155.
- **low** Implementation leakage is slight: "endpoint" and "API key" in FR153 are user terms; "VACUUM" is not leaked. The FR153 key location (keychain) falls to FR67/NFR11, which should be cited.

## Scope honesty — adequate
The note says the manual part of FR154 exists (`abfd6026`) and the rest is not built; honest. Not recorded: that the two promises of `plan-cau-dich-tham-khao-bang-cho-glossary.md` are reversed (AD-64 rule 13); a code-side matter, so no PRD change needed.

## Downstream usability — adequate
IDs FR153–FR158 are unique and contiguous (max was FR152); cross-references FR53, FR72, FR74, FR75, FR123 resolve. C6 range updated; FR154 (Glossary) and FR155 (import, C1) are not added to the C4/C1 ranges, so a ticket that cites C1 or C4 ranges will miss them. `addendum.md` has no mention of FR153–FR158 or AD-64 (0 matches).

## Shape fit — strong
Capability spec, no UJs, consistent with the PRD's declared shape.

## Mechanical notes (term use, §5.2)
- New row `Dịch nhanh` is good. Missing aliases: the shipped button is `Dịch thử` (AD-64 rule 13, commit `abfd6026`) and the PRD uses `Dịch lại`, but the row's "Không dùng" column does not map `Dịch thử`. *Low:* add `Dịch thử` as an alias or a "không dùng", whichever Ice wants.
- The new row says the banned words "bản dịch AI/đề xuất AI" mean "kết quả ở panel Đề xuất AI, FR72", but FR72 calls that panel "panel AI Translation" (FR145 says "panel Đề xuất AI"). Pre-existing drift, repeated by the new row. *Low.*
- FR154 uses "Câu dịch tham khảo" and the note says "ghi rõ là tham khảo"; the term is not in §5.2 (the `Dịch nhanh` row says "chỉ để tham khảo"). Acceptable.
- FR154 "ví dụ ngữ cảnh" and "bảng chờ Glossary" match FR53 and the existing queue wording. "Cài đặt → Dữ liệu" is used nowhere else in the PRD; check it against the real Settings section name.
- Other sentences found false or stale after this amendment:
  1. §3 row network "đều do người dùng chủ động" (line ~137), NFR12 and FR104 — see the high finding above.
  2. FR72 location rule — see above.
  3. FR103 "Mọi cấu hình ... hai tầng" — see above.
  4. Not stale (checked): "ba luồng"/"ba chỗ" counts (still three, per AD-64 "không có điểm thứ tư"); FR75 and FR77 wording; NFR19 (URL import only); line 128/344 ("không bao giờ tự quyết định tải cái gì" is about page fetching); FR count: no sentence in the PRD states a total number of FRs (grep for "tổng số FR" and "N FR" found none).
